use crate::{Error, Result};

pub enum Version {
    V1_4,
    V1_5,
}

impl std::str::FromStr for Version {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "1.4" => Ok(Version::V1_4),
            "1.5" => Ok(Version::V1_5),
            _ => Err(Error::InvalidCycloneDXVersion(s.to_owned())),
        }
    }
}

impl Default for Version {
    fn default() -> Self {
        Version::V1_5
    }
}
