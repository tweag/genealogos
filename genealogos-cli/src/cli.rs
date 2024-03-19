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

#[derive(Clone, Copy, Debug, Default)]
#[non_exhaustive]
pub enum BomArg {
    CycloneDX1_3,
    #[default]
    CycloneDX1_4,
}

impl BomArg {
    pub fn get_bom(&self) -> Result<Box<impl Bom>> {
        match self {
            BomArg::CycloneDX1_3 => Ok(Box::new(genealogos::bom::cyclonedx::CycloneDX::new())),
            BomArg::CycloneDX1_4 => Ok(Box::new(genealogos::bom::cyclonedx::CycloneDX::new())),
        }
    }
}

impl std::fmt::Display for BomArg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BomArg::CycloneDX1_3 => write!(f, "cyclonedx1.3"),
            BomArg::CycloneDX1_4 => write!(f, "cyclonedx1.4"),
        }
    }
}

impl clap::ValueEnum for BomArg {
    fn value_variants<'a>() -> &'a [Self] {
        &[BomArg::CycloneDX1_3, BomArg::CycloneDX1_4]
    }

    fn to_possible_value(&self) -> Option<clap::builder::PossibleValue> {
        Some(
            match self {
                BomArg::CycloneDX1_3 => "cyclonedx1.3",
                BomArg::CycloneDX1_4 => "cyclonedx1.4",
            }
            .into(),
        )
    }
}
