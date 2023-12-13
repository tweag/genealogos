//! This module contains the code related to nixtract
//! it is responsible for:
//!   - Parsing the incoming output of Nixtract
//!   - Converting that input into the internal representation of Genealogos

use serde::Deserialize;

use crate::model::{Model, ModelComponent, ModelType};

#[derive(Deserialize, Debug)]
pub(crate) struct Nixtract {
    pub(crate) entries: Vec<NixtractEntry>,
}

#[derive(Deserialize, Debug)]
pub(crate) struct NixtractEntry {
    pub(crate) attribute_path: String,
    pub(crate) derivation_path: String,
    pub(crate) output_path: String,
    pub(crate) outputs: Vec<NixtractOutput>,
    pub(crate) name: String,
    pub(crate) parsed_name: NixtractParsedName,
    pub(crate) nixpkgs_metadata: NixtractNixpkgsMetadata,
    pub(crate) build_inputs: Vec<NixtractBuiltInput>,
}

#[derive(Deserialize, Debug)]
pub(crate) struct NixtractOutput {
    pub(crate) name: String,
    pub(crate) output_path: String,
}

#[derive(Deserialize, Debug)]
pub(crate) struct NixtractParsedName {
    pub(crate) name: String,
    pub(crate) version: String,
}

#[derive(Deserialize, Debug)]
pub(crate) struct NixtractNixpkgsMetadata {
    pub(crate) description: String,
    pub(crate) pname: String,
    pub(crate) version: String,
    pub(crate) broken: bool,
    pub(crate) license: String,
}

#[derive(Deserialize, Debug)]
pub(crate) struct NixtractBuiltInput {
    pub(crate) attribute_path: String,
    pub(crate) build_input_type: String,
    pub(crate) output_path: Option<String>,
}

impl From<Nixtract> for Model {
    fn from(nixtract: Nixtract) -> Self {
        let components: Vec<ModelComponent> = nixtract
            .entries
            .into_iter()
            .map(|entry| ModelComponent {
                r#type: ModelType::Application,
                name: entry.parsed_name.name,
                r#ref: entry.output_path,
                version: entry.nixpkgs_metadata.version,
                description: entry.nixpkgs_metadata.description,
            })
            .collect();

        Model { components }
    }
}
