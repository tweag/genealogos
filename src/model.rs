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

impl ModelComponent {
    // TODO: Error
    pub(crate) fn to_cyclonedx(self) -> cyclonedx::Component {
        cyclonedx::ComponentBuilder::default()
            .type_(self.r#type)
            .name(self.name)
            .build()
            .unwrap()
    }
}

impl Model {
    // TODO: Error
    pub(crate) fn to_cyclonedx(self) -> cyclonedx::CycloneDx {
        let components: Vec<cyclonedx::Component> = self
            .components
            .into_iter()
            .map(|component| component.to_cyclonedx())
            .collect();

        cyclonedx::CycloneDxBuilder::default()
            .bom_format("CycloneDX")
            .spec_version("1.5")
            .version(1)
            .components(components)
            .build()
            .unwrap()
    }
}
