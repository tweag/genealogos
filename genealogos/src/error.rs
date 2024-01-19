//! This module contains all possible errors the Genealogos library can produce

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    NixtractParsing(serde_json::Error),
    /// The Cyclone library has a very large number of errors.
    /// We don't want to propagate all of those individually, so we use a single Box instead.
    CycloneDX(Box<dyn std::error::Error>),
}

impl From<serde_json::Error> for Error {
    fn from(json_error: serde_json::Error) -> Self {
        Self::NixtractParsing(json_error)
    }
}

impl From<serde_cyclonedx::cyclonedx::v_1_5::ComponentBuilderError> for Error {
    fn from(error: serde_cyclonedx::cyclonedx::v_1_5::ComponentBuilderError) -> Self {
        Self::CycloneDX(Box::new(error))
    }
}

impl From<serde_cyclonedx::cyclonedx::v_1_5::ExternalReferenceBuilderError> for Error {
    fn from(error: serde_cyclonedx::cyclonedx::v_1_5::ExternalReferenceBuilderError) -> Self {
        Self::CycloneDX(Box::new(error))
    }
}

impl From<serde_cyclonedx::cyclonedx::v_1_5::LicenseBuilderError> for Error {
    fn from(error: serde_cyclonedx::cyclonedx::v_1_5::LicenseBuilderError) -> Self {
        Self::CycloneDX(Box::new(error))
    }
}

impl From<serde_cyclonedx::cyclonedx::v_1_5::DependencyBuilderError> for Error {
    fn from(error: serde_cyclonedx::cyclonedx::v_1_5::DependencyBuilderError) -> Self {
        Self::CycloneDX(Box::new(error))
    }
}

impl From<serde_cyclonedx::cyclonedx::v_1_5::CycloneDxBuilderError> for Error {
    fn from(error: serde_cyclonedx::cyclonedx::v_1_5::CycloneDxBuilderError) -> Self {
        Self::CycloneDX(Box::new(error))
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::NixtractParsing(json_error) => write!(
                f,
                "Could not parse the provided Nixtract JSON: {}",
                json_error
            ),
            Error::CycloneDX(cyclone_error) => {
                write!(f, "Could not build the CycloneDX output: {}", cyclone_error)
            }
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::NixtractParsing(json_error) => json_error.source(),
            Error::CycloneDX(cyclone_error) => cyclone_error.source(),
        }
    }
}
