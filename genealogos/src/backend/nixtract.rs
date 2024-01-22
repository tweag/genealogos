//! This module contains the code related to nixtract
//! it is responsible for:
//!   - Parsing the incoming output of Nixtract
//!   - Converting that input into the internal representation of Genealogos

// In this module, one might see that we do deserialize unused fields. This is
// to ensure we stay complient with nixtract output.

use serde::Deserialize;

use crate::model::{
    Model, ModelComponent, ModelDependency, ModelExternalReference, ModelExternalReferenceType,
    ModelLicense, ModelSource, ModelType,
};

#[derive(Deserialize, Debug)]
pub(crate) struct Nixtract {
    pub(crate) entries: Vec<NixtractEntry>,
}

#[derive(Deserialize, Debug)]
pub(crate) struct NixtractEntry {
    #[serde(rename(deserialize = "attribute_path"))]
    pub(crate) _attribute_path: String,
    #[serde(rename(deserialize = "derivation_path"))]
    pub(crate) _derivation_path: String,
    pub(crate) output_path: String,
    #[serde(rename(deserialize = "outputs"))]
    pub(crate) _outputs: Vec<NixtractOutput>,
    #[serde(rename(deserialize = "name"))]
    pub(crate) _name: String,
    pub(crate) parsed_name: NixtractParsedName,
    pub(crate) nixpkgs_metadata: NixtractNixpkgsMetadata,
    pub(crate) src: Option<NixtractSource>,
    pub(crate) build_inputs: Vec<NixtractBuiltInput>,
}

#[derive(Deserialize, Debug)]
pub(crate) struct NixtractOutput {
    #[serde(rename(deserialize = "name"))]
    pub(crate) _name: String,
    #[serde(rename(deserialize = "output_path"))]
    pub(crate) _output_path: String,
}

#[derive(Deserialize, Debug)]
pub(crate) struct NixtractParsedName {
    pub(crate) name: String,
    #[serde(rename(deserialize = "version"))]
    pub(crate) _version: String,
}

#[derive(Deserialize, Debug)]
pub(crate) struct NixtractNixpkgsMetadata {
    pub(crate) description: String,
    #[serde(rename(deserialize = "pname"))]
    pub(crate) _pname: String,
    pub(crate) version: String,
    #[serde(rename(deserialize = "broken"))]
    pub(crate) _broken: bool,
    pub(crate) homepage: String,
    pub(crate) licenses: Option<Vec<NixtractLicense>>,
}

#[derive(Deserialize, Debug)]
pub(crate) struct NixtractSource {
    pub(crate) git_repo_url: String,
    // Revision or tag of the git repo
    pub(crate) rev: String,
}

#[derive(Deserialize, Debug)]
pub(crate) struct NixtractLicense {
    // Not all licenses in nixpkgs have an associated spdx id
    pub(crate) spdx_id: Option<String>,
    pub(crate) full_name: String,
}

#[derive(Deserialize, Debug)]
pub(crate) struct NixtractBuiltInput {
    #[serde(rename(deserialize = "attribute_path"))]
    pub(crate) _attribute_path: String,
    #[serde(rename(deserialize = "build_input_type"))]
    pub(crate) _build_input_type: String,
    pub(crate) output_path: Option<String>,
}

impl crate::backend::BackendTrait for Nixtract {
    fn from_flake_ref(
        flake_ref: impl AsRef<str>,
        attribute_path: Option<impl AsRef<str>>,
    ) -> crate::Result<Model> {
        // Call `nixtract` from $PATH, providing the `flake_ref` and `attribute_path` as arguments
        let mut command = std::process::Command::new("nixtract");
        command.arg("--target-flake-ref").arg(flake_ref.as_ref());
        if let Some(attr_path) = attribute_path {
            command
                .arg("--target-attribute-path")
                .arg(attr_path.as_ref());
        }
        command.arg("-");

        // Execute the command and capture the output
        let output = command.output()?;
        let stdout = String::from_utf8_lossy(&output.stdout);

        // Split stdout into lines
        let lines = stdout.lines();

        // Convert the lines into a `Model`
        Self::from_lines(lines)
    }

    fn from_trace_file(file_path: impl AsRef<std::path::Path>) -> crate::Result<Model> {
        // Read the file contents and split them into individual lines
        let file_contents = std::fs::read_to_string(file_path)?;
        let lines = file_contents.lines();
        Self::from_lines(lines)
    }

    fn from_lines(lines: impl Iterator<Item = impl AsRef<str>>) -> crate::Result<Model> {
        // Parse each line as a Nixtract entry
        let entries: Vec<NixtractEntry> = lines
            .map(|line| serde_json::from_str(line.as_ref()))
            .collect::<Result<Vec<NixtractEntry>, _>>()?;

        // Convert the Nixtract entries into Nitract
        let nixtract = Nixtract { entries };

        // Convert the Nixtract to a Genealogos model
        let model = Model::from(nixtract);

        Ok(model)
    }
}

impl From<Nixtract> for Model {
    fn from(nixtract: Nixtract) -> Self {
        let components: Vec<ModelComponent> = nixtract
            .entries
            .iter()
            .map(|entry| {
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

                ModelComponent {
                    r#type: ModelType::Application,
                    name: entry.parsed_name.name.clone(),
                    r#ref: entry.output_path.clone(),
                    version: entry.nixpkgs_metadata.version.clone(),
                    description: entry.nixpkgs_metadata.description.clone(),
                    external_references,
                    licenses,
                    src,
                }
            })
            .collect();

        let dependencies: Vec<ModelDependency> = nixtract
            .entries
            .into_iter()
            .map(|entry| ModelDependency {
                r#ref: entry.output_path,
                depends_on: entry
                    .build_inputs
                    .into_iter()
                    .filter_map(|bi| bi.output_path)
                    .collect(),
            })
            .collect();

        Model {
            components,
            dependencies,
        }
    }
}

impl From<&NixtractLicense> for ModelLicense {
    fn from(nixtract_license: &NixtractLicense) -> Self {
        let id = nixtract_license.spdx_id.clone();
        let name = if id.is_some() {
            None
        } else {
            Some(nixtract_license.full_name.clone())
        };
        ModelLicense { id, name }
    }
}
