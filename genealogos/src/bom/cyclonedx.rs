//! CycloneDX BOM (Bill of Materials) support.
//! This module provides a BOM implementation for the CycloneDX format.
//! The CycloneDX format is a lightweight BOM format that is designed to be easy to create and consume.

use std::fmt::Display;
use std::str::FromStr;

use cyclonedx_bom::models::{
    component::Classification,
    dependency::{Dependencies, Dependency},
    external_reference::{self, ExternalReference, ExternalReferenceType, ExternalReferences},
    license::{License, LicenseChoice, LicenseIdentifier, Licenses},
    property::{Properties, Property},
};
use cyclonedx_bom::prelude::*;

use crate::error::*;
use crate::model::*;

/// Represents the different output formats CycloneDX supports
#[derive(Clone, Copy, Debug, Default)]
#[cfg_attr(feature = "rocket", derive(rocket::FromFormField))]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
pub enum FileFormat {
    #[default]
    JSON,
    XML,
}

impl Display for FileFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FileFormat::JSON => write!(f, "json"),
            FileFormat::XML => write!(f, "xml"),
        }
    }
}

impl FromStr for FileFormat {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "json" => Ok(FileFormat::JSON),
            "xml" => Ok(FileFormat::XML),
            _ => Err(Error::InvalidCycloneDXFileFormat(s.to_string())),
        }
    }
}

/// Defines the supported CycloneDX specification versions that Genealogos supports.
#[derive(Clone, Copy, Debug, Default)]
#[cfg_attr(feature = "rocket", derive(rocket::FromFormField))]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
#[non_exhaustive]
pub enum SpecVersion {
    V1_3,
    #[default]
    V1_4,
}

impl Display for SpecVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SpecVersion::V1_3 => write!(f, "1.3"),
            SpecVersion::V1_4 => write!(f, "1.4"),
        }
    }
}

impl FromStr for SpecVersion {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "1.3" => Ok(SpecVersion::V1_3),
            "1.4" => Ok(SpecVersion::V1_4),
            _ => Err(Error::InvalidCycloneDXVersion(s.to_string())),
        }
    }
}

/// Holds the configuration for the CycloneDX Bom implementation.
pub struct CycloneDX {
    file_format: FileFormat,
    spec_version: SpecVersion,
}

impl CycloneDX {
    /// Constructs a new `CycloneDX` instance.
    ///
    /// # Arguments
    ///
    /// * `spec_version` - A `SpecVersion` instance representing the specification version.
    /// * `file_format` - A `FileFormat` instance representing the file format.
    ///
    /// # Returns
    ///
    /// A new `CycloneDX` instance.
    pub fn new(spec_version: SpecVersion, file_format: FileFormat) -> Self {
        CycloneDX {
            file_format,
            spec_version,
        }
    }

    /// Parses the given specification version string into a `CycloneDX` instance.
    ///
    /// # Arguments
    ///
    /// * `spec_version` - A string slice that holds the specification version.
    ///
    /// # Returns
    ///
    /// * `Result<Self>` - Returns a `Result` which is an `Ok` variant that wraps a `CycloneDX` instance if the parsing is successful,
    /// or an `Err` variant that contains an error if the parsing fails.
    pub fn parse_version(spec_version: &str, file_format: FileFormat) -> Result<Self> {
        let spec_version = SpecVersion::from_str(spec_version)?;
        Ok(CycloneDX {
            spec_version,
            file_format,
        })
    }

    /// Parses the specification version and file format to create a new CycloneDX instance.
    ///
    /// # Arguments
    ///
    /// * `spec_version` - A string slice that holds the specification version.
    /// * `file_format` - A string slice that holds the file format.
    ///
    /// # Returns
    ///
    /// * `Result<Self>` - Returns a `Result` which is an `Ok` variant that wraps a `CycloneDX` instance if the parsing is successful,
    /// or an `Err` variant that wraps an error if the parsing fails.
    pub fn parse(spec_version: &str, file_format: &str) -> Result<Self> {
        let spec_version = SpecVersion::from_str(spec_version)?;
        let file_format = FileFormat::from_str(file_format)?;
        Ok(CycloneDX {
            spec_version,
            file_format,
        })
    }
}

impl Default for CycloneDX {
    fn default() -> Self {
        // TODO: Update to 1_5, or ideally Default (but that's not implemented)
        Self::new(SpecVersion::V1_4, FileFormat::JSON)
    }
}

impl super::Bom for CycloneDX {
    fn write_to_io_writer<W: std::io::Write>(
        &self,
        model: crate::model::Model,
        writer: &mut W,
    ) -> crate::Result<()> {
        // Convert the model into a CycloneDX BOM
        let bom = Bom::try_from(model)?;

        match self.spec_version {
            SpecVersion::V1_3 => match self.file_format {
                FileFormat::JSON => bom.output_as_json_v1_3(writer)?,
                FileFormat::XML => bom.output_as_xml_v1_3(writer)?,
            },
            SpecVersion::V1_4 => match self.file_format {
                FileFormat::JSON => bom.output_as_json_v1_4(writer)?,
                FileFormat::XML => bom.output_as_xml_v1_4(writer)?,
            },
        }

        Ok(())
    }
}

impl TryFrom<Model> for Bom {
    type Error = crate::Error;

    fn try_from(model: Model) -> Result<Self> {
        let mut model = model;
        if cfg!(test) || std::env::var("GENEALOGOS_DETERMINISTIC").is_ok() {
            model.components.sort_by_key(|c| c.r#ref.clone());

            // This only sort the dependencies by key, not the depends on
            // hashset, that is done later, when it is converted into a vec.
            model.dependencies.sort_by_key(|c| c.r#ref.clone());
        }

        let mut bom = Bom {
            components: Some(Components(
                model
                    .components
                    .into_iter()
                    .map(TryInto::try_into)
                    .collect::<Result<Vec<_>>>()?,
            )),
            dependencies: Some(Dependencies(
                model
                    .dependencies
                    .into_iter()
                    .map(Into::into)
                    .collect::<Vec<_>>(),
            )),
            // Constructs a BOM with a default version of 1 and serial_number with a random UUID
            ..Default::default()
        };

        if cfg!(test) || std::env::var("GENEALOGOS_DETERMINISTIC").is_ok() {
            bom.serial_number = Some(
                UrnUuid::new("urn:uuid:00000000-0000-0000-0000-000000000000".to_owned()).unwrap(),
            );
        }

        Ok(bom)
    }
}

impl TryFrom<ModelComponent> for Component {
    type Error = crate::Error;

    fn try_from(model: ModelComponent) -> Result<Self> {
        Ok(Component {
            component_type: model.r#type.into(),
            mime_type: None,
            bom_ref: Some(model.r#ref.clone()),
            supplier: None,
            author: None,
            publisher: None,
            group: None,
            name: NormalizedString::new(&model.name),
            version: Some(NormalizedString::new(&model.version)),
            description: Some(NormalizedString::new(&model.description)),
            scope: None,
            hashes: None,
            // Remove the match
            licenses: match model.licenses.clone() {
                None => None,
                Some(licenses) => {
                    let licenses = licenses
                        .into_iter()
                        .map(TryInto::try_into)
                        .collect::<Result<Vec<_>>>()?;

                    Some(Licenses(licenses))
                }
            },
            copyright: None,
            cpe: None,
            purl: (&model).into(),
            swid: None,
            modified: None,
            pedigree: None,
            external_references: Some(ExternalReferences(
                model
                    .external_references
                    .into_iter()
                    .map(Into::into)
                    .collect(),
            )),
            properties: Some(Properties(
                model
                    .properties
                    .properties
                    .into_iter()
                    .map(|(name, value)| Property {
                        name,
                        value: NormalizedString::new(&value),
                    })
                    .collect(),
            )),
            components: None,
            evidence: None,
            signature: None,
        })
    }
}

impl From<ModelType> for Classification {
    fn from(model: ModelType) -> Self {
        match model {
            ModelType::Application => Classification::Application,
        }
    }
}

impl TryFrom<ModelLicense> for LicenseChoice {
    type Error = Error;

    fn try_from(model: ModelLicense) -> Result<Self> {
        if let Some(id) = model.id {
            Ok(LicenseChoice::Expression(SpdxExpression::parse_lax(id)?))
        } else if let Some(name) = model.name {
            Ok(LicenseChoice::License(License {
                license_identifier: LicenseIdentifier::Name(NormalizedString::new(&name)),
                text: None,
                url: None,
            }))
        } else {
            unreachable!("We only construct ModelLicense with at least id or name")
        }
    }
}

impl From<ModelExternalReference> for ExternalReference {
    fn from(model: ModelExternalReference) -> Self {
        ExternalReference {
            external_reference_type: model.r#type.into(),
            url: external_reference::Uri::Url(Uri::try_from(model.url).expect("Invalid URL")),
            comment: None,
            hashes: None,
        }
    }
}

impl From<ModelExternalReferenceType> for ExternalReferenceType {
    fn from(model: ModelExternalReferenceType) -> Self {
        match model {
            ModelExternalReferenceType::Website => ExternalReferenceType::Website,
        }
    }
}

impl From<ModelDependency> for Dependency {
    fn from(model: ModelDependency) -> Self {
        let mut dependencies = model.depends_on.into_iter().collect::<Vec<_>>();

        if cfg!(test) || std::env::var("GENEALOGOS_DETERMINISTIC").is_ok() {
            dependencies.sort_unstable();
        };

        Dependency {
            dependency_ref: model.r#ref,
            dependencies,
        }
    }
}

impl From<&ModelComponent> for Option<Purl> {
    fn from(model: &ModelComponent) -> Self {
        // the cyclonedx-bom crate uses the packageurl crate under the hood,
        // but provides no way to contruct a Purl from a packageurl, so we use the FromStr
        // trait instead, which is not great.
        let purl_str: String = match &model.src {
            Some(src) => {
                format!(
                    "pkg:generic/{}?vcs_url=git+{}@{}",
                    model.name, src.git_repo_url, src.rev
                )
            }
            None => format!("pkg:generic/{}@{}", model.name, model.version),
        }
        .to_owned();

        Purl::from_str(&purl_str).ok()
    }
}
