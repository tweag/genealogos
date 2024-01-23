mod version;

use crate::Result;
use serde_cyclonedx::cyclonedx;
pub use version::Version;

use crate::model::Model;

pub enum CycloneDX {
    V1_4(cyclonedx::v_1_4::CycloneDx),
    V1_5(cyclonedx::v_1_5::CycloneDx),
}

impl CycloneDX {
    pub(crate) fn from_model(model: Model, version: Version) -> Result<Self> {
        match version {
            Version::V1_4 => Ok(CycloneDX::V1_4(model.try_into()?)),
            Version::V1_5 => Ok(CycloneDX::V1_5(model.try_into()?)),
        }
    }
}

impl serde::Serialize for CycloneDX {
    fn serialize<S: serde::Serializer>(
        &self,
        serializer: S,
    ) -> std::result::Result<S::Ok, S::Error> {
        match self {
            CycloneDX::V1_4(cyclonedx) => cyclonedx.serialize(serializer),
            CycloneDX::V1_5(cyclonedx) => cyclonedx.serialize(serializer),
        }
    }
}
