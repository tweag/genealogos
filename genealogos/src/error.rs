//! This module contains all possible errors the Genealogos library can produce
pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("The provided JSON could not be parsed: {0}")]
    NixtractParsing(#[from] serde_json::Error),

    #[error("Errors constructing CycloneDX output: {0}")]
    CycloneDX(#[source] Box<dyn std::error::Error + Send + Sync + 'static>),
}

macro_rules! impl_from_for_error {
    ($error_type:ty) => {
        impl From<$error_type> for Error {
            fn from(error: $error_type) -> Self {
                Self::CycloneDX(Box::new(error))
            }
        }
    };
}

impl_from_for_error!(serde_cyclonedx::cyclonedx::v_1_5::ComponentBuilderError);
impl_from_for_error!(serde_cyclonedx::cyclonedx::v_1_5::ExternalReferenceBuilderError);
impl_from_for_error!(serde_cyclonedx::cyclonedx::v_1_5::LicenseBuilderError);
impl_from_for_error!(serde_cyclonedx::cyclonedx::v_1_5::DependencyBuilderError);
impl_from_for_error!(serde_cyclonedx::cyclonedx::v_1_5::CycloneDxBuilderError);
