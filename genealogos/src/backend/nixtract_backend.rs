//! This module provides a backend for nixtract, a tool for extracting information from Nix derivations.
//! It implements the `Backend` trait for the `Nixtract` struct, providing methods to convert
//! nixtract output into a Genealogos model. It also implements the BackendHandle trait for the equivalently named NixtractHandle.
//!
//! The `Nixtract` struct hold the configuration and any other state state required by the nixtract backend.
//! Similarly, the `NixtractHandle` struct holds the state required to communicate with the nixtract backend.
//!
//! The `From<T>` trait is implemented for `Model` where `T` is an iterator over `DerivationDescription`.
//! This implementation converts a collection of `DerivationDescription` into a `Model`.
//!
//! The `From<&License>` trait is implemented for `ModelLicense`, converting a nixtract `License` into a `ModelLicense`.

use std::sync::mpsc::Receiver;

use crate::model::{
    Model, ModelComponent, ModelDependency, ModelExternalReference, ModelExternalReferenceType,
    ModelLicense, ModelProperties, ModelSource, ModelType,
};

use nixtract::{nixtract, DerivationDescription, License, NixtractConfig};

/// `NixtractHandle` is a structure all that is needed to communicate to the
/// Nixtract backend
#[derive(Debug)]
pub struct NixtractHandle {
    receiver: Receiver<nixtract::message::Message>,
}

#[derive(Debug, Clone)]
pub struct Nixtract {
    config: NixtractConfig,
}

impl Nixtract {
    pub fn new() -> (Self, NixtractHandle) {
        let (sender, receiver) = std::sync::mpsc::channel();
        let config = NixtractConfig {
            message_tx: Some(sender),
            ..NixtractConfig::default()
        };

        (Self { config }, NixtractHandle { receiver })
    }

    pub fn new_without_handle() -> Self {
        Self {
            config: NixtractConfig::default(),
        }
    }
}

impl crate::backend::Backend for Nixtract {
    fn to_model_from_flake_ref(
        &self,
        flake_ref: impl AsRef<str>,
        attribute_path: Option<impl AsRef<str>>,
    ) -> crate::Result<Model> {
        // Call nixtract to get the derivation descriptions
        let nixtract_output = nixtract(
            flake_ref.as_ref(),
            None::<String>,
            attribute_path.as_ref().map(AsRef::as_ref),
            self.config.clone(),
        )?;

        // Convert the nixtract output into a Genealogos model
        let model = Model::from(nixtract_output);

        Ok(model)
    }

    fn to_model_from_lines(
        &self,
        lines: impl Iterator<Item = impl AsRef<str>>,
    ) -> crate::Result<Model> {
        // Parse each line as a Nixtract entry
        let entries: Vec<DerivationDescription> = lines
            .map(|line| serde_json::from_str(line.as_ref()))
            .collect::<Result<Vec<DerivationDescription>, _>>()?;

        // Convert the Nixtract to a Genealogos model
        let model = Model::from(entries);

        Ok(model)
    }
}

impl super::BackendHandle for NixtractHandle {
    fn new_messages(&self) -> crate::Result<Vec<super::Message>> {
        // Get all current messages from the receiver
        let messages: Vec<super::Message> = self
            .receiver
            .try_iter()
            .map(|m| super::Message {
                index: m.id,
                content: m.to_string(),
            })
            .collect();

        Ok(messages)
    }

    #[cfg(feature = "backend_handle_messages")]
    fn messages(&self) -> crate::Result<impl Iterator<Item = super::Message>> {
        Ok(self.receiver.iter().map(|m| super::Message {
            index: m.id,
            content: m.to_string(),
        }))
    }

    fn max_index(&self) -> usize {
        rayon::current_num_threads()
    }
}

impl<T> From<T> for Model
where
    T: IntoIterator<Item = DerivationDescription>,
{
    fn from(nixtract: T) -> Self {
        let entries: Vec<DerivationDescription> = nixtract.into_iter().collect();
        let components: Vec<ModelComponent> = entries
            .iter()
            .filter_map(|entry| {
                if entry.output_path.is_none() {
                    log::warn!("Skipping component entry with no output_path: {:?}", entry);
                    return None;
                }

                let external_references = {
                    let mut acc = vec![];
                    if !entry.nixpkgs_metadata.homepage.is_empty() {
                        acc.push(ModelExternalReference {
                            url: entry.nixpkgs_metadata.homepage.clone(),
                            r#type: ModelExternalReferenceType::Website,
                        })
                    }
                    acc
                };
                let licenses = entry
                    .nixpkgs_metadata
                    .licenses
                    .as_ref()
                    .map(|v| v.iter().map(Into::into).collect());

                let src = entry.src.as_ref().map(|src| ModelSource {
                    git_repo_url: src.git_repo_url.clone(),
                    rev: src.rev.clone(),
                });

                // Convert the narinfo field of the DerivationDescription into a properties hashmap
                let properties = {
                    let map = if let Some(narinfo) = &entry.nar_info {
                        fn insert_if_some<T: ToString>(
                            map: &mut std::collections::HashMap<String, String>,
                            key: &str,
                            value: Option<T>,
                        ) {
                            if let Some(v) = value {
                                map.insert(format!("nix:narinfo:{}", key), v.to_string());
                            }
                        }

                        let mut res = std::collections::HashMap::new();
                        insert_if_some(&mut res, "store_path", Some(narinfo.store_path.clone()));
                        insert_if_some(&mut res, "url", Some(narinfo.url.clone()));
                        insert_if_some(&mut res, "nar_hash", Some(narinfo.nar_hash.clone()));
                        insert_if_some(&mut res, "nar_size", Some(narinfo.nar_size));
                        insert_if_some(&mut res, "compression", Some(narinfo.compression.clone()));
                        insert_if_some(&mut res, "file_hash", narinfo.file_hash.clone());
                        insert_if_some(&mut res, "file_size", narinfo.file_size);
                        insert_if_some(&mut res, "deriver", narinfo.deriver.clone());
                        insert_if_some(&mut res, "system", narinfo.system.clone());
                        insert_if_some(&mut res, "sig", narinfo.sig.clone());
                        insert_if_some(&mut res, "ca", narinfo.ca.clone());

                        if let Some(references) = narinfo.references.clone() {
                            res.insert("nix:narinfo:references".to_owned(), references.join(" "));
                        }

                        res
                    } else {
                        std::collections::HashMap::new()
                    };
                    ModelProperties { properties: map }
                };

                Some(ModelComponent {
                    r#type: ModelType::Application,
                    name: entry.parsed_name.name.clone(),
                    r#ref: entry.output_path.clone().expect("output_path is None"),
                    version: entry.nixpkgs_metadata.version.clone(),
                    description: entry.nixpkgs_metadata.description.clone(),
                    external_references,
                    licenses,
                    src,
                    properties,
                })
            })
            .collect();

        let dependencies: Vec<ModelDependency> = entries
            .into_iter()
            .filter_map(|entry| {
                if entry.output_path.is_none() {
                    log::warn!(
                        "Skipping dependencies entry with no output_path: {:?}",
                        entry
                    );
                    return None;
                }
                Some(ModelDependency {
                    r#ref: entry.output_path.expect("output_path is None"),
                    depends_on: entry
                        .build_inputs
                        .into_iter()
                        .filter_map(|bi| bi.output_path)
                        .collect(),
                })
            })
            .collect();

        Model {
            components,
            dependencies,
        }
    }
}

impl From<&License> for ModelLicense {
    fn from(nixtract_license: &License) -> Self {
        let id = nixtract_license.spdx_id.clone();
        let name = if id.is_some() {
            None
        } else {
            Some(nixtract_license.full_name.clone())
        };
        ModelLicense { id, name }
    }
}
