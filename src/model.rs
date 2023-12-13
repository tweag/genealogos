//! This module contains Genealogos' internal representation of incomming data.
//! Since the initial target of Genealogos is CycloneDX, this model is largely based on their representation.
use serde_cyclonedx::cyclonedx::v_1_5 as cyclonedx;

#[derive(Debug)]
pub(crate) struct Model {
    pub(crate) components: Vec<ModelComponent>,
}

#[derive(Debug)]
pub(crate) struct ModelComponent {
    pub(crate) r#type: ModelType,
    pub(crate) name: String,
    /// A unique identifier op the component to be used as a reference elsewhere in the sbom
    pub(crate) r#ref: String,
    pub(crate) version: String,
    pub(crate) description: String,
}

#[derive(Debug)]
pub(crate) enum ModelType {
    /// (spec) For software components, classify as application if no more specific
    /// appropriate classification is available or cannot be determined for the
    /// component.
    Application,
}

impl Into<String> for ModelType {
    fn into(self) -> String {
        match self {
            ModelType::Application => "application".to_owned(),
        }
    }
}

impl From<ModelComponent> for cyclonedx::Component {
    // TODO: Error
    fn from(model_component: ModelComponent) -> Self {
        cyclonedx::ComponentBuilder::default()
            .type_(model_component.r#type)
            .name(model_component.name)
            .bom_ref(model_component.r#ref)
            .version(model_component.version)
            .description(model_component.description)
            .build()
            .unwrap()
    }
}

impl From<Model> for cyclonedx::CycloneDx {
    // TODO: Error
    fn from(model: Model) -> Self {
        let components: Vec<cyclonedx::Component> = model
            .components
            .into_iter()
            .map(|component| component.into())
            .collect();

        cyclonedx::CycloneDxBuilder::default()
            .bom_format("CycloneDX")
            .spec_version("1.5")
            .version(1)
            .serial_number(format!("urn:uuid:{}", uuid::Uuid::new_v4()))
            .components(components)
            .build()
            .unwrap()
    }
}
