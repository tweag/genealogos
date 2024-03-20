//! This module contains all possible errors the Genealogos library can produce
pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("The provided JSON could not be parsed: {0}")]
    NixtractParsing(#[from] serde_json::Error),

    #[error("Nixtract failed: {0}")]
    Nixtract(#[from] nixtract::error::Error),

    #[error("Genealogos encountered an IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("The provided CycloneDX version is invalid: {0}")]
    InvalidCycloneDXVersion(String),

    #[error("Errors constructing CycloneDX output: {0}")]
    CycloneDX(#[from] cyclonedx_bom::errors::BomError),

    #[error("Errors constructing CycloneDX JSON output: {0}")]
    CycloneDXJSON(#[from] cyclonedx_bom::errors::JsonWriteError),

    #[error("Errors constructing CycloneDX SPDX expression: {0}")]
    CycloneDXSpdxExpression(#[from] cyclonedx_bom::external_models::spdx::SpdxExpressionError),

    #[error("Genealogos cannot handle CycloneDX version {0} yet")]
    CycloneDXUnimplemented(String),

    #[error("Errors constructing Converting to String output: {0}")]
    FromUtf8Error(#[from] std::string::FromUtf8Error),

    #[error("Errors constructing output: {0}")]
    Fmt(#[from] std::fmt::Error),
}
