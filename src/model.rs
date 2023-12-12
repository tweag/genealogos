//! This module contains Genealogos' internal representation of incomming data.
//! Since the initial target of Genealogos is CycloneDX, this model is largely based on their representation.

#[derive(Debug)]
pub(crate) struct Model {
    pub(crate) components: Vec<ModelComponent>,
}

#[derive(Debug)]
pub(crate) struct ModelComponent {
    pub(crate) r#type: ModelType,
    pub(crate) name: String,
}

#[derive(Debug)]
pub(crate) enum ModelType {
    /// (spec) For software components, classify as application if no more specific
    /// appropriate classification is available or cannot be determined for the
    /// component.
    Application,
}
