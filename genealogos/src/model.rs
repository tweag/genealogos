//! This module contains Genealogos' internal representation of incomming data.
//! Since the initial target of Genealogos is CycloneDX, this model is largely based on their representation.
use std::collections::HashSet;

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
    pub(crate) licenses: Option<Vec<ModelLicense>>,
    // Not directly taken from the cycloneDX spec, but part of the purl
    pub(crate) src: Option<ModelSource>,
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

impl From<ModelComponent> for cyclonedx::Component {
    // TODO: Error
    fn from(model_component: ModelComponent) -> Self {
        let mut builder = cyclonedx::ComponentBuilder::default();
        let mut builder = builder
            .type_(model_component.r#type)
            .name(model_component.name.clone())
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

        if let Some(model_licenses) = model_component.licenses {
            let licenses = model_licenses.into_iter().map(Into::into).collect();
            builder.licenses(cyclonedx::LicenseChoiceUrl::Variant0(licenses));
        }

        builder.external_references(external_references);

        if !model_component.name.is_empty() && !model_component.version.is_empty() {
            let purl: String = if let Some(src) = model_component.src {
                format!(
                    "pkg:generic/{}?vcs_url=git+{}@{}",
                    model_component.name, src.git_repo_url, src.rev
                )
            } else {
                format!(
                    "pkg:generic/{}@{}",
                    model_component.name, model_component.version
                )
            }
            .to_owned();
            builder = builder.purl(purl);
        }

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

impl From<ModelLicense> for cyclonedx::LicenseChoiceUrlVariant0ItemUrl {
    fn from(model_license: ModelLicense) -> Self {
        let mut builder = cyclonedx::LicenseBuilder::default();

        if let Some(id) = model_license.id {
            builder.id(id);
        }
        if let Some(name) = model_license.name {
            builder.name(name);
        }
        cyclonedx::LicenseChoiceUrlVariant0ItemUrl {
            license: builder.build().unwrap(),
        }
    }
}

impl From<ModelDependency> for cyclonedx::Dependency {
    fn from(model_dependency: ModelDependency) -> Self {
        let mut depends_on: Vec<String> = model_dependency.depends_on.into_iter().collect();

        // For testing, we need deterministic output, so we sort the strings before conversion
        if cfg!(test) || std::env::var("GENEALOGOS_DETERMINISTIC").is_ok() {
            depends_on.sort_unstable();
        }

        let depends_on: Vec<serde_json::Value> = depends_on.into_iter().map(Into::into).collect();

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

        let mut cyclonedx = cyclonedx::CycloneDxBuilder::default();
        cyclonedx
            .bom_format("CycloneDX")
            .spec_version("1.5")
            .version(1);

        if cfg!(test) || std::env::var("GENEALOGOS_DETERMINISTIC").is_ok() {
            // Deterministic
            cyclonedx.serial_number("urn:uuid:00000000-0000-0000-0000-000000000000");
        } else {
            cyclonedx.serial_number(format!("urn:uuid:{}", uuid::Uuid::new_v4()));
        }

        cyclonedx
            .components(components)
            .dependencies(dependencies)
            .build()
            .unwrap()
    }
}
