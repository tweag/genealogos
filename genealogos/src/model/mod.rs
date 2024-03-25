//! This module contains Genealogos' internal representation of incomming data.
//! Since the initial target of Genealogos is CycloneDX, this model is largely based on their representation.
use std::collections::{HashMap, HashSet};

#[derive(Debug)]
pub struct Model {
    pub components: Vec<ModelComponent>,
    pub dependencies: Vec<ModelDependency>,
}

#[derive(Debug)]
pub struct ModelComponent {
    pub r#type: ModelType,
    pub name: String,
    /// A unique identifier op the component to be used as a reference elsewhere in the sbom
    pub r#ref: String,
    pub version: String,
    pub description: String,
    pub external_references: Vec<ModelExternalReference>,
    pub licenses: Option<Vec<ModelLicense>>,
    // Not directly taken from the cycloneDX spec, but part of the purl
    pub src: Option<ModelSource>,
    /// We use the properties field of the component to store the narinfo data
    pub properties: ModelProperties,
}

#[derive(Debug, Copy, Clone)]
pub enum ModelType {
    /// (spec) For software components, classify as application if no more specific
    /// appropriate classification is available or cannot be determined for the
    /// component.
    Application,
}

#[derive(Debug)]
pub struct ModelExternalReference {
    pub url: String,
    pub r#type: ModelExternalReferenceType,
}

#[derive(Debug)]
pub enum ModelExternalReferenceType {
    Website,
}

// TODO: Consider if it is worth splitting this struct up into 2 different
// structs like the cyclone spec. For now, just make id and name both Options
#[derive(Debug, Clone)]
pub struct ModelLicense {
    // SPDX id
    pub id: Option<String>,
    pub name: Option<String>,
}

#[derive(Debug)]
pub struct ModelSource {
    pub git_repo_url: String,
    pub rev: String,
}

#[derive(Debug)]
pub struct ModelProperties {
    pub properties: HashMap<String, String>,
}

#[derive(Debug)]
pub struct ModelDependency {
    pub r#ref: String,
    pub depends_on: HashSet<String>,
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
