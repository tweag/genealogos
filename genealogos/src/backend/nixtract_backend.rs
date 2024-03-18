//! This module contains the code related to nixtract
//! it is responsible for:
//!   - Parsing the incoming output of Nixtract
//!   - Converting that input into the internal representation of Genealogos

// In this module, one might see that we do deserialize unused fields. This is
// to ensure we stay complient with nixtract output.

use std::sync::mpsc::{Sender, Receiver};

use crate::model::{
    Model, ModelComponent, ModelDependency, ModelExternalReference, ModelExternalReferenceType,
    ModelLicense, ModelProperties, ModelSource, ModelType,
};

#[derive(Debug)]
pub struct NixtractHandle {
    receiver: Receiver<nixtract::message::Message>,
}

#[derive(Debug, Clone)]
pub struct Nixtract {
    sender: Sender<nixtract::message::Message>,
}

use nixtract::{nixtract, DerivationDescription, License};

impl Nixtract {
    // Create a new Nixtract backend given the Sender
    pub fn new() -> (Self, NixtractHandle) {
        let (sender, receiver) = std::sync::mpsc::channel();
        (Self { sender }, NixtractHandle { receiver })
    }

}

impl crate::backend::BackendTrait for Nixtract {
    fn parse_flake_ref(
        &self,
        flake_ref: impl AsRef<str>,
        attribute_path: Option<impl AsRef<str>>,
    ) -> crate::Result<Model> {
        // Call nixtract to get the derivation descriptions
        let nixtract_output = nixtract(
            flake_ref.as_ref(),
            None::<String>,
            attribute_path.as_ref().map(AsRef::as_ref),
            false,
            true,
            None,
            Some(self.sender.clone()),
        )?;

        // Convert the nixtract output into a Genealogos model
        let model = Model::from(nixtract_output);

        Ok(model)
    }

    fn parse_trace_file(&self, file_path: impl AsRef<std::path::Path>) -> crate::Result<Model> {
        // Read the file contents and split them into individual lines
        let file_contents = std::fs::read_to_string(file_path)?;
        let lines = file_contents.lines();
        self.parse_lines(lines)
    }

    fn parse_lines(&self, lines: impl Iterator<Item = impl AsRef<str>>) -> crate::Result<Model> {
        // Parse each line as a Nixtract entry
        let entries: Vec<DerivationDescription> = lines
            .map(|line| serde_json::from_str(line.as_ref()))
            .collect::<Result<Vec<DerivationDescription>, _>>()?;

        // Convert the Nixtract to a Genealogos model
        let model = Model::from(entries);

        Ok(model)
    }
}

impl crate::BackendHandleTrait for NixtractHandle {
    fn get_new_messages(&self) -> crate::Result<Vec<nixtract::message::Message>> {
        // Get all current messages from the receiver
        let messages: Vec<nixtract::message::Message> = self
            .receiver
            .try_iter()
            .collect();

        Ok(messages)
    }

    fn get_messages(&self) -> crate::Result<impl Iterator<Item = nixtract::message::Message>> {
        Ok(self.receiver.iter())
    }

    fn get_num_ids(&self) -> usize {
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
                        std::collections::HashMap::from([
                            (
                                Some(concat!("nix:narinfo:", "store_path").to_owned()),
                                Some(narinfo.store_path.clone().to_string()),
                            ),
                            (
                                Some(concat!("nix:narinfo:", "url").to_owned()),
                                Some(narinfo.url.clone().to_string()),
                            ),
                            (
                                Some(concat!("nix:narinfo:", "nar_hash").to_owned()),
                                Some(narinfo.nar_hash.clone().to_string()),
                            ),
                            (
                                Some(concat!("nix:narinfo:", "nar_size").to_owned()),
                                Some(narinfo.nar_size.clone().to_string()),
                            ),
                            (
                                Some(concat!("nix:narinfo:", "compression").to_owned()),
                                Some(narinfo.compression.clone().to_string()),
                            ),
                            (
                                Some(concat!("nix:narinfo:", "file_hash").to_owned()),
                                narinfo.file_hash.clone().map(|v| v.to_string()),
                            ),
                            (
                                Some(concat!("nix:narinfo:", "file_size").to_owned()),
                                narinfo.file_size.map(|v| v.to_string()),
                            ),
                            (
                                Some(concat!("nix:narinfo:", "deriver").to_owned()),
                                narinfo.deriver.clone().map(|v| v.to_string()),
                            ),
                            (
                                Some(concat!("nix:narinfo:", "system").to_owned()),
                                narinfo.system.clone().map(|v| v.to_string()),
                            ),
                            (
                                Some(concat!("nix:narinfo:", "sig").to_owned()),
                                narinfo.sig.clone().map(|v| v.to_string()),
                            ),
                            (
                                Some(concat!("nix:narinfo:", "ca").to_owned()),
                                narinfo.ca.clone().map(|v| v.to_string()),
                            ),
                            (
                                Some(concat!("nix:narinfo:", "references").to_owned()),
                                narinfo.references.clone().map(|v| v.join(" ")),
                            ),
                        ])
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