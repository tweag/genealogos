use std::sync::OnceLock;

use anyhow::Result;
use genealogos::backend::{Backend, BackendHandle};
use genealogos::bom::Bom;

#[derive(Clone, Copy, Debug, Default, clap::ValueEnum)]
#[non_exhaustive]
pub enum BackendArg {
    #[default]
    Nixtract,
}

impl BackendArg {
    pub fn get_backend(&self) -> Result<Box<(impl Backend, impl BackendHandle)>> {
        match self {
            BackendArg::Nixtract => Ok(Box::new(
                genealogos::backend::nixtract_backend::Nixtract::new(),
            )),
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

#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub enum BomArg {
    CycloneDX(
        genealogos::bom::cyclonedx::SpecVersion,
        genealogos::bom::cyclonedx::FileFormat,
    ),
}

impl Default for BomArg {
    fn default() -> Self {
        BomArg::CycloneDX(
            genealogos::bom::cyclonedx::SpecVersion::default(),
            genealogos::bom::cyclonedx::FileFormat::default(),
        )
    }
}

impl BomArg {
    pub fn get_bom(&self) -> Result<Box<impl Bom>> {
        match self {
            BomArg::CycloneDX(version, file_format) => Ok(Box::new(
                genealogos::bom::cyclonedx::CycloneDX::new(*version, *file_format),
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

impl clap::ValueEnum for BomArg {
    fn value_variants<'a>() -> &'a [Self] {
        static BOM_ARG_VARIANTS: OnceLock<Vec<BomArg>> = OnceLock::new();
        let versions = genealogos::bom::cyclonedx::SpecVersion::value_variants();
        let file_formats = genealogos::bom::cyclonedx::FileFormat::value_variants();

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
