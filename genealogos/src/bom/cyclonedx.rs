use std::str::FromStr;

use cyclonedx_bom::models::bom::SpecVersion;
use cyclonedx_bom::models::component::Classification;
use cyclonedx_bom::models::external_reference::ExternalReference;
use cyclonedx_bom::models::external_reference::ExternalReferenceType;
use cyclonedx_bom::models::external_reference::ExternalReferences;
use cyclonedx_bom::models::license::License;
use cyclonedx_bom::models::license::LicenseChoice;
use cyclonedx_bom::models::license::LicenseIdentifier;
use cyclonedx_bom::models::license::Licenses;
use cyclonedx_bom::models::property::Properties;
use cyclonedx_bom::models::property::Property;
use cyclonedx_bom::prelude::*;

use crate::error::*;
use crate::model::*;

// TODO: Include output filetype
pub struct CycloneDX {
    spec_version: SpecVersion,
}

impl CycloneDX {
    pub fn new(spec_version: SpecVersion) -> Self {
        CycloneDX { spec_version }
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
    pub fn parse_version(spec_version: &str) -> Result<Self> {
        let spec_version = SpecVersion::from_str(spec_version)?;
        Ok(CycloneDX { spec_version })
    }
}

impl Default for CycloneDX {
    fn default() -> Self {
        // TODO: Update to 1_5, or ideally Default (but that's not implemented)
        Self::new(SpecVersion::V1_4)
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
            SpecVersion::V1_3 => bom.output_as_json_v1_3(writer)?,
            SpecVersion::V1_4 => bom.output_as_json_v1_4(writer)?,
            _ => return Err(Error::CycloneDXUnimplemented(self.spec_version.to_string())),
        }

        Ok(())
    }
}

impl TryFrom<Model> for Bom {
    type Error = crate::Error;

    fn try_from(model: Model) -> Result<Self> {
        let bom = Bom {
            components: Some(Components(
                model
                    .components
                    .into_iter()
                    .map(TryInto::try_into)
                    .collect::<Result<Vec<_>>>()?,
            )),
            // Constructs a BOM with a default version of 1 and serial_number with a random UUID
            ..Default::default()
        };

        Ok(bom)
    }
}

impl TryFrom<ModelComponent> for Component {
    type Error = crate::Error;

    fn try_from(model: ModelComponent) -> Result<Self> {
        Ok(Component {
            component_type: model.r#type.into(),
            mime_type: None,
            bom_ref: Some(model.r#ref),
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
            licenses: match model.licenses {
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
            // TODO!
            purl: None,
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
            url: Uri::try_from(model.url).expect("Invalid URL"),
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
