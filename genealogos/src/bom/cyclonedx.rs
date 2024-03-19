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

// TODO: Include target version
pub struct CycloneDX {}

impl CycloneDX {
    pub fn new() -> Self {
        CycloneDX {}
    }
}

impl Default for CycloneDX {
    fn default() -> Self {
        Self::new()
    }
}

impl super::Bom for CycloneDX {
    fn write_to_io_writer<W: std::io::Write>(
        &self,
        model: crate::model::Model,
        writer: &mut W,
    ) -> crate::Result<()> {
        //TODO: Include target version

        // Convert the model into a CycloneDX BOM
        let bom = Bom::try_from(model)?;

        // Convert the bom into JSON 1.4
        Ok(bom.output_as_json_v1_4(writer)?)
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
