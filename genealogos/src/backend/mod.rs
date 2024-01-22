use std::path;

use crate::{error::Result, model::Model};
mod nixtract;

/// This trait represents a backend that can be used to generate a `Model` from.
/// Every backend must be able to generate a `Model` from a flake reference and
/// an optional attribute path, or from a trace file.
///
/// Generating from a trace file or lines is preferred, as it is much faster. Generating
/// from a flake reference is slower, as it requires a full Nix evaluation.
pub(crate) trait BackendTrait {
    fn from_flake_ref(
        flake_ref: impl AsRef<str>,
        attribute_path: Option<impl AsRef<str>>,
    ) -> Result<Model>;
    fn from_trace_file(file_path: impl AsRef<path::Path>) -> Result<Model>;
    fn from_lines(lines: impl Iterator<Item = impl AsRef<str>>) -> Result<Model>;
}

#[derive(Debug, Clone)]
/// Represents a backend that can be used to generate a `Model` from.
/// Used to specify which backend to use when generating a `Model`.
pub enum Backend {
    Nixtract,
}

/// Converts a string to a `Backend`. Used to parse the `--backend` CLI argument.
impl From<String> for Backend {
    fn from(backend: String) -> Self {
        match backend.as_ref() {
            "nixtract" => Backend::Nixtract,
            _ => Backend::Nixtract,
        }
    }
}

impl Backend {
    pub(crate) fn from_flake_ref(
        &self,
        flake_ref: impl AsRef<str>,
        attribute_path: Option<impl AsRef<str>>,
    ) -> Result<Model> {
        match self {
            Backend::Nixtract => nixtract::Nixtract::from_flake_ref(flake_ref, attribute_path),
        }
    }

    pub(crate) fn from_trace_file(&self, file_path: impl AsRef<path::Path>) -> Result<Model> {
        match self {
            Backend::Nixtract => nixtract::Nixtract::from_trace_file(file_path),
        }
    }
}

impl Default for Backend {
    fn default() -> Self {
        Backend::Nixtract
    }
}
