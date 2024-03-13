//! This module contains Genealogos' internal representation of incomming data.
//! Since the initial target of Genealogos is CycloneDX, this model is largely based on their representation.
use std::collections::{HashMap, HashSet};

pub mod v1_4;
pub mod v1_5;

#[derive(Debug)]
pub(crate) struct Model {
    pub(crate) components: Vec<ModelComponent>,
    pub(crate) dependencies: Vec<ModelDependency>,
}

#[derive(Debug)]
pub(crate) struct ModelComponent {
    pub(crate) r#type: ModelType,
    pub(crate) name: String,
    /// A unique identifier op the component to be used as a reference elsewhere in the sbom
    pub(crate) r#ref: String,
    pub(crate) version: String,
    pub(crate) description: String,
    pub(crate) external_references: Vec<ModelExternalReference>,
    pub(crate) licenses: Option<Vec<ModelLicense>>,
    // Not directly taken from the cycloneDX spec, but part of the purl
    pub(crate) src: Option<ModelSource>,
    /// We use the properties field of the component to store the narinfo data
    pub(crate) properties: ModelProperties,
}

#[derive(Debug)]
pub(crate) enum ModelType {
    /// (spec) For software components, classify as application if no more specific
    /// appropriate classification is available or cannot be determined for the
    /// component.
    Application,
}

#[derive(Debug)]
pub(crate) struct ModelExternalReference {
    pub(crate) url: String,
    pub(crate) r#type: ModelExternalReferenceType,
}

#[derive(Debug)]
pub(crate) enum ModelExternalReferenceType {
    Website,
}

// TODO: Consider if it is worth splitting this struct up into 2 different
// structs like the cyclone spec. For now, just make id and name both Options
#[derive(Debug)]
pub(crate) struct ModelLicense {
    // SPDX id
    pub(crate) id: Option<String>,
    pub(crate) name: Option<String>,
}

#[derive(Debug)]
pub(crate) struct ModelSource {
    pub(crate) git_repo_url: String,
    pub(crate) rev: String,
}

#[derive(Debug)]
pub(crate) struct ModelProperties {
    pub(crate) properties: HashMap<Option<String>, Option<String>>,
}

#[derive(Debug)]
pub(crate) struct ModelDependency {
    pub(crate) r#ref: String,
    pub(crate) depends_on: HashSet<String>,
}

impl From<ModelType> for String {
    fn from(val: ModelType) -> Self {
        match val {
            ModelType::Application => "application".to_owned(),
        }
    }
}

impl From<ModelExternalReferenceType> for String {
    fn from(val: ModelExternalReferenceType) -> Self {
        match val {
            ModelExternalReferenceType::Website => "website".to_owned(),
        }
    }
}
