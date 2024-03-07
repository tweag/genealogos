use serde_cyclonedx::cyclonedx::v_1_4 as cyclonedx;

use super::*;
use crate::{Error, Result};

impl TryFrom<ModelComponent> for cyclonedx::Component {
    type Error = Error;

    fn try_from(model_component: ModelComponent) -> Result<Self> {
        let mut builder = cyclonedx::ComponentBuilder::default();
        let mut builder = builder
            .type_(model_component.r#type)
            .name(model_component.name.clone())
            .bom_ref(model_component.r#ref)
            .description(model_component.description)
            .properties(model_component.properties);

        if !model_component.version.is_empty() {
            builder = builder.version(model_component.version.clone());
        }

        let external_references: Vec<cyclonedx::ExternalReference> = model_component
            .external_references
            .into_iter()
            .map(TryInto::try_into)
            .collect::<Result<Vec<_>>>()?;

        if let Some(model_licenses) = model_component.licenses {
            let licenses = model_licenses
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>>>()?;

            builder.licenses(licenses);
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

        Ok(builder.build()?)
    }
}

impl TryFrom<ModelExternalReference> for cyclonedx::ExternalReference {
    type Error = Error;

    fn try_from(model_external_reference: ModelExternalReference) -> Result<Self> {
        Ok(cyclonedx::ExternalReferenceBuilder::default()
            .url(model_external_reference.url)
            .type_(model_external_reference.r#type)
            .build()?)
    }
}

impl TryFrom<ModelLicense> for cyclonedx::LicenseChoice {
    type Error = Error;

    fn try_from(model_license: ModelLicense) -> Result<Self> {
        let mut builder = cyclonedx::LicenseChoiceBuilder::default();

        let license: cyclonedx::License = model_license.try_into()?;

        builder.license(license);

        Ok(builder.build()?)
    }
}

impl TryFrom<ModelLicense> for cyclonedx::License {
    type Error = Error;

    fn try_from(model_license: ModelLicense) -> Result<Self> {
        let mut builder = cyclonedx::LicenseBuilder::default();

        if let Some(id) = model_license.id {
            builder.id(id);
        }
        if let Some(name) = model_license.name {
            builder.name(name);
        }

        Ok(builder.build()?)
    }
}

impl TryFrom<ModelDependency> for cyclonedx::Dependency {
    type Error = Error;

    fn try_from(model_dependency: ModelDependency) -> Result<Self> {
        let mut depends_on: Vec<String> = model_dependency.depends_on.into_iter().collect();

        // For testing, we need deterministic output, so we sort the strings before conversion
        if cfg!(test) || std::env::var("GENEALOGOS_DETERMINISTIC").is_ok() {
            depends_on.sort_unstable();
        }

        let depends_on: Vec<cyclonedx::RefType> = depends_on.into_iter().map(Into::into).collect();

        Ok(cyclonedx::DependencyBuilder::default()
            .ref_(model_dependency.r#ref)
            .depends_on(depends_on)
            .build()?)
    }
}

impl TryFrom<Model> for cyclonedx::CycloneDx {
    type Error = Error;

    fn try_from(model: Model) -> Result<Self> {
        let mut components: Vec<cyclonedx::Component> = model
            .components
            .into_iter()
            .map(TryInto::try_into)
            .collect::<Result<Vec<_>>>()?;

        // Sort components by bom_ref for deterministic output if testing or GENEALOGOS_DETERMINISTIC is set
        if cfg!(test) || std::env::var("GENEALOGOS_DETERMINISTIC").is_ok() {
            components.sort_by(|a, b| a.bom_ref.cmp(&b.bom_ref));
        }

        // Sort model dependencies by ref for deterministic output if testing or GENEALOGOS_DETERMINISTIC is set
        // We need to sort the dependencies before we convert them to cyclonedx::Dependency
        let mut dependencies: Vec<ModelDependency> = model.dependencies;
        if cfg!(test) || std::env::var("GENEALOGOS_DETERMINISTIC").is_ok() {
            dependencies.sort_by(|a, b| a.r#ref.cmp(&b.r#ref));
        }

        let dependencies: Vec<cyclonedx::Dependency> = dependencies
            .into_iter()
            .map(TryInto::try_into)
            .collect::<Result<Vec<_>>>()?;

        let mut cyclonedx = cyclonedx::CycloneDxBuilder::default();
        cyclonedx
            .bom_format("CycloneDX")
            .spec_version("1.4")
            .version(1);

        if cfg!(test) || std::env::var("GENEALOGOS_DETERMINISTIC").is_ok() {
            // Deterministic
            cyclonedx.serial_number("urn:uuid:00000000-0000-0000-0000-000000000000");
        } else {
            cyclonedx.serial_number(format!("urn:uuid:{}", uuid::Uuid::new_v4()));
        }

        Ok(cyclonedx
            .components(components)
            .dependencies(dependencies)
            .build()?)
    }
}

impl From<ModelProperties> for Vec<cyclonedx::Property> {
    fn from(model_properties: ModelProperties) -> Self {
        let mut properties: Vec<cyclonedx::Property> = model_properties
            .properties
            .into_iter()
            .map(|(key, value)| cyclonedx::Property { name: key, value })
            .collect();

        // For testing, we need deterministic output
        if cfg!(test) || std::env::var("GENEALOGOS_DETERMINISTIC").is_ok() {
            properties.sort_by_key(|v| v.name.clone());
        }

        properties
    }
}
