use super::job_map::{GCInterval, GCStaleAfter};

#[derive(rocket::serde::Deserialize, Debug)]
#[serde(crate = "rocket::serde")]
pub struct Config {
    #[serde(flatten)]
    pub gc: GCConfig,
}

#[derive(rocket::serde::Deserialize, Debug)]
#[serde(crate = "rocket::serde")]
pub struct GCConfig {
    #[serde(default, rename = "gc_interval")]
    pub interval: GCInterval,
    #[serde(default, rename = "gc_stale_after")]
    pub stale_after: GCStaleAfter,
}
