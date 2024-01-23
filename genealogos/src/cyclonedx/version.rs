use clap::ValueEnum;

#[derive(Debug, Clone)]
pub enum Version {
    V1_4,
    V1_5,
}

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

impl Default for Version {
    fn default() -> Self {
        Version::V1_5
    }
}
