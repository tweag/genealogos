//! This module is a unique one, it provides utility functions to parse command line arguments and rocket form fields.
//! If you want to use Genealogos as a library, you should probably try to do it without this module.
//! For that reason, the module is hidden behind the `args` feature flag.
//! Additionally, this module only exports relevant types and functions when the `clap` or `rocket` features are enabled.

#[cfg(feature = "rocket")]
use std::str::FromStr as _;
#[cfg(feature = "clap")]
use std::sync::OnceLock;

use crate::backend::{Backend, BackendHandle, BackendHandleMessages};
use crate::bom::Bom;
use crate::error::*;

/// This type represents all the possible backends from which Genealogos can extract the Nix graph.
/// Notably,  type does not construct the actual backends, instead just holding all information needed to construct it in the future.
/// This makes is possible to use the type in a `clap` or `rocket` context, without having to construct the actual backend type.
/// Use the `get_backend` method to get the actual backend type.
#[derive(Clone, Copy, Debug, Default)]
#[cfg_attr(feature = "rocket", derive(rocket::FromFormField))]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
#[non_exhaustive]
pub enum BackendArg {
    #[default]
    Nixtract,
}

impl BackendArg {
    /// Get a backend and handle that is object safe
    pub fn get_backend(&self) -> Result<Box<(impl Backend, impl BackendHandle)>> {
        self.get_backend_messages()
    }

    /// Get a backend and handle that is not object safe and implements the `messages` function
    pub fn get_backend_messages(&self) -> Result<Box<(impl Backend, impl BackendHandleMessages)>> {
        match self {
            BackendArg::Nixtract => Ok(Box::new(crate::backend::nixtract_backend::Nixtract::new())),
        }
    }
}

impl std::fmt::Display for BackendArg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BackendArg::Nixtract => write!(f, "nixtract"),
        }
    }
}

/// This type represents all the possible bom formats supported by Genealogos.
/// It is important to note, that this type does not construct the actual bom types, instead just holding all information needed to construct it in the future.
/// This makes is possible to use the type in a `clap` or `rocket` context, without having to construct the actual bom type.
/// This type is supposed to be lightweight.
/// Use the `get_bom` method to get the actual bom type.
#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub enum BomArg {
    /// A subset of the CycloneDX bom format, currently only supporting 1.3, 1.4, and 1.5, both xml and json output.
    CycloneDX(
        crate::bom::cyclonedx::SpecVersion,
        crate::bom::cyclonedx::FileFormat,
    ),
}

impl Default for BomArg {
    fn default() -> Self {
        BomArg::CycloneDX(
            crate::bom::cyclonedx::SpecVersion::default(),
            crate::bom::cyclonedx::FileFormat::default(),
        )
    }
}

impl BomArg {
    pub fn get_bom(&self) -> Result<Box<impl Bom>> {
        match self {
            BomArg::CycloneDX(version, file_format) => Ok(Box::new(
                crate::bom::cyclonedx::CycloneDX::new(*version, *file_format),
            )),
        }
    }
}

impl std::fmt::Display for BomArg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BomArg::CycloneDX(version, file_format) => {
                write!(f, "cyclonedx_{version}_{file_format}")
            }
        }
    }
}

#[cfg(feature = "rocket")]
impl<'v> rocket::form::FromFormField<'v> for BomArg {
    fn from_value(field: rocket::form::ValueField<'v>) -> rocket::form::Result<'v, Self> {
        let value = field.value;

        // The individual parameters are seperated by _
        let parts: Vec<&str> = value.split('_').collect();

        // The first string indicates the BOM format
        match parts[0] {
            "cyclonedx" => {
                // The second string indicates the cyclonedx version
                let version = crate::bom::cyclonedx::SpecVersion::from_str(parts[1])
                    .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send>)?;

                // The third string indicates the cyclonedx file format
                let file_format = crate::bom::cyclonedx::FileFormat::from_str(parts[2])
                    .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send>)?;
                Ok(BomArg::CycloneDX(version, file_format))
            }
            _ => Err(
                rocket::form::Error::validation(format!("Unknown BOM format: {}", parts[0])).into(),
            ),
        }
    }
}

#[cfg(feature = "clap")]
impl clap::ValueEnum for BomArg {
    fn value_variants<'a>() -> &'a [Self] {
        static BOM_ARG_VARIANTS: OnceLock<Vec<BomArg>> = OnceLock::new();
        let versions = crate::bom::cyclonedx::SpecVersion::value_variants();
        let file_formats = crate::bom::cyclonedx::FileFormat::value_variants();

        // Create the carthesian product of versions and file formats
        let variants = versions
            .iter()
            .flat_map(|v| file_formats.iter().map(move |f| BomArg::CycloneDX(*v, *f)))
            .collect();

        BOM_ARG_VARIANTS.get_or_init(|| variants)
    }

    fn to_possible_value(&self) -> Option<clap::builder::PossibleValue> {
        let value = match self {
            BomArg::CycloneDX(v, f) => format!("cyclonedx_{v}_{f}"),
        };
        Some(clap::builder::PossibleValue::new(value))
    }
}
