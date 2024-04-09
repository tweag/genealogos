//! This module contains all possible errors the Genealogos library can produce
pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// The provided nixtract JSON trace could not be parsed
    #[error("The provided nixtract JSON trace could not be parsed: {0}")]
    NixtractParsing(#[from] serde_json::Error),

    /// Wraps the Nixtract library's error type
    #[error("Nixtract failed: {0}")]
    Nixtract(#[from] nixtract::error::Error),

    /// We encountered an IO error
    #[error("Genealogos encountered an IO error: {0}")]
    Io(#[from] std::io::Error),

    /// We encountered an error while parsing the provided CycloneDX version
    #[error("The provided CycloneDX version is invalid: {0}")]
    InvalidCycloneDXVersion(String),

    /// We encountered an error while parsing the provided CycloneDX file format
    #[error("The provided CycloneDX file format is invalid: {0}")]
    InvalidCycloneDXFileFormat(String),

    /// Holds errors encountered while constructing the CycloneDX BOM
    #[error("Errors constructing CycloneDX BOM: {0}")]
    CycloneDX(#[from] cyclonedx_bom::errors::BomError),

    /// Holds errors encountered while constructing the CycloneDX JSON output
    #[error("Errors constructing CycloneDX JSON output: {0}")]
    CycloneDXJSON(#[from] cyclonedx_bom::errors::JsonWriteError),

    /// Holds errors encountered while constructing the CycloneDX XML output
    #[error("Errors constructing CycloneDX XML output: {0}")]
    CycloneDXXML(#[from] cyclonedx_bom::errors::XmlWriteError),

    /// Holds errors encountered while constructing attempting to parse the provided SPDX expression
    #[error("Errors constructing CycloneDX SPDX expression: {0}")]
    CycloneDXSpdxExpression(#[from] cyclonedx_bom::external_models::spdx::SpdxExpressionError),

    /// String conversion error from UTF8 bytes
    #[error("Errors constructing Converting to String output: {0}")]
    FromUtf8Error(#[from] std::string::FromUtf8Error),

    /// Wraps the `std::fmt::Error`
    #[error("Errors constructing output: {0}")]
    Fmt(#[from] std::fmt::Error),

    /// Parsing the installable to a Source was impossible
    #[error("Error parsing the installable")]
    InstallableParsing(String),
}
