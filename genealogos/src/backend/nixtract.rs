//! This module contains the code related to nixtract
//! it is responsible for:
//!   - Parsing the incoming output of Nixtract
//!   - Converting that input into the internal representation of Genealogos

// In this module, one might see that we do deserialize unused fields. This is
// to ensure we stay complient with nixtract output.

use crate::model::{
    Model, ModelComponent, ModelDependency, ModelExternalReference, ModelExternalReferenceType,
    ModelLicense, ModelSource, ModelType, ModelProperties,
};

pub struct Nixtract {}

use nixtract::{nixtract, DerivationDescription, License};

impl crate::backend::BackendTrait for Nixtract {
    fn from_flake_ref(
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
        )?;

        // Convert the nixtract output into a Genealogos model
        let model = Model::from(nixtract_output);

        Ok(model)
    }

    fn from_trace_file(file_path: impl AsRef<std::path::Path>) -> crate::Result<Model> {
        // Read the file contents and split them into individual lines
        let file_contents = std::fs::read_to_string(file_path)?;
        let lines = file_contents.lines();
        Self::from_lines(lines)
    }

    fn from_lines(lines: impl Iterator<Item = impl AsRef<str>>) -> crate::Result<Model> {
        // Parse each line as a Nixtract entry
        let entries: Vec<DerivationDescription> = lines
            .map(|line| serde_json::from_str(line.as_ref()))
            .collect::<Result<Vec<DerivationDescription>, _>>()?;

        // Convert the Nixtract to a Genealogos model
        let model = Model::from(entries);

        Ok(model)
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
                    let mut acc = std::collections::HashMap::new();
                    if let Some(narinfo) = &entry.nar_info {
                        // These macros add the provided fields to the acc map. The first macro works with basic types. The second works with Options. The final one works with Vec<impl Display>.
                        // TODO: Consider if this should be a Trait instead
                        macro_rules! insert_into_map {
                            ($($key:ident),+) => {
                                $(
                                    acc.insert(Some(format!("nix:narinfo:{}", stringify!($key))), Some(narinfo.$key.clone().to_string()));
                                )+
                            }
                        }

                        macro_rules! insert_into_map_option {
                            ($($key:ident),+) => {
                                $(
                                    acc.insert(Some(format!("nix:narinfo:{}", stringify!($key))), narinfo.$key.clone().map(|v| v.to_string()));
                                )+
                            }
                        }

                        macro_rules! insert_into_map_vec {
                            ($($key:ident),+) => {
                                $(
                                    acc.insert(Some(format!("nix:narinfo:{}", stringify!($key))), narinfo.$key.clone().map(|v| v.join(" ")));
                                )+
                            }
                        }

                        insert_into_map!(
                            store_path,
                            url,
                            nar_hash,
                            nar_size,
                            compression
                        );

                        insert_into_map_option!(
                            file_hash,
                            file_size,
                            deriver,
                            system,
                            sig,
                            ca
                        );

                        insert_into_map_vec!(
                            references
                        );

                        
                    }
                    ModelProperties { properties: acc }
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
