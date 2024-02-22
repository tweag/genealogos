//! CycloneDX version enum
//!
//! This module contains the `Version` enum, which represents the CycloneDX version to use
//! when generating CycloneDX output.

use clap::ValueEnum;

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "rocket", derive(rocket::FromFormField))]
pub enum Version {
    V1_4,
    #[default]
    V1_5,
}

/// This trait is used to convert a `Version` into a `PossibleValue` for clap
/// And ensures that clap can display the possible values for `Version`
impl ValueEnum for Version {
    fn value_variants<'a>() -> &'a [Self] {
        &[Version::V1_4, Version::V1_5]
    }

    fn to_possible_value(&self) -> Option<clap::builder::PossibleValue> {
        match self {
            Version::V1_4 => Some(clap::builder::PossibleValue::new("1.4")),
            Version::V1_5 => Some(clap::builder::PossibleValue::new("1.5")),
        }
    }
}

impl TryFrom<&str> for Version {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "1.4" => Ok(Version::V1_4),
            "1.5" => Ok(Version::V1_5),
            _ => Err(format!("Invalid CycloneDX version: {}", value)),
        }
    }
}
