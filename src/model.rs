//! This module contains Genealogos' internal representation of incomming data.
//! Since the initial target of Genealogos is CycloneDX, this model is largely based on their representation.
use serde_cyclonedx::cyclonedx::v_1_5 as cyclonedx;

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

#[derive(Debug)]
pub(crate) struct ModelDependency {
    pub(crate) r#ref: String,
    pub(crate) depends_on: Vec<String>,
}

impl Into<String> for ModelType {
    fn into(self) -> String {
        match self {
            ModelType::Application => "application".to_owned(),
        }
    }
}

impl Into<String> for ModelExternalReferenceType {
    fn into(self) -> String {
        match self {
            ModelExternalReferenceType::Website => "website".to_owned(),
        }
    }
}

impl From<ModelComponent> for cyclonedx::Component {
    // TODO: Error
    fn from(model_component: ModelComponent) -> Self {
        let mut builder = cyclonedx::ComponentBuilder::default();
        let mut builder = builder
            .type_(model_component.r#type)
            .name(model_component.name)
            .bom_ref(model_component.r#ref)
            .description(model_component.description);

        if !model_component.version.is_empty() {
            builder = builder.version(model_component.version.clone());
        }

        let external_references: Vec<cyclonedx::ExternalReference> = model_component
            .external_references
            .into_iter()
            .map(Into::into)
            .collect();

        builder.external_references(external_references);

        builder.build().unwrap()
    }
}

impl From<ModelExternalReference> for cyclonedx::ExternalReference {
    fn from(model_external_reference: ModelExternalReference) -> Self {
        cyclonedx::ExternalReferenceBuilder::default()
            .url(model_external_reference.url)
            .type_(model_external_reference.r#type)
            .build()
            .unwrap()
    }
}

impl From<ModelDependency> for cyclonedx::Dependency {
    fn from(model_dependency: ModelDependency) -> Self {
        let depends_on: Vec<serde_json::Value> = model_dependency
            .depends_on
            .into_iter()
            .map(Into::into)
            .collect();

        cyclonedx::DependencyBuilder::default()
            .ref_(model_dependency.r#ref)
            .depends_on(depends_on)
            .build()
            .unwrap()
    }
}

impl From<Model> for cyclonedx::CycloneDx {
    // TODO: Error
    fn from(model: Model) -> Self {
        let components: Vec<cyclonedx::Component> =
            model.components.into_iter().map(Into::into).collect();

        let dependencies: Vec<cyclonedx::Dependency> =
            model.dependencies.into_iter().map(Into::into).collect();

        cyclonedx::CycloneDxBuilder::default()
            .bom_format("CycloneDX")
            .spec_version("1.5")
            .version(1)
            .serial_number(format!("urn:uuid:{}", uuid::Uuid::new_v4()))
            .components(components)
            .dependencies(dependencies)
            .build()
            .unwrap()
    }
}
