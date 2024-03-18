use std::path;

use clap::ValueEnum;

use crate::{error::Result, model::Model};

/// We have an crate dependency that is already called nixtract, to avoid conflict, this module is called nixtract_backend. This is fine, since we do not export this module.
pub mod nixtract_backend;

/// This trait represents a backend that can be used to generate a `Model` from.
/// Every backend must be able to generate a `Model` from a flake reference and
/// an optional attribute path, or from a trace file.
///
/// Generating from a trace file or lines is preferred, as it is much faster. Generating
/// from a flake reference is slower, as it requires a full Nix evaluation.
pub trait BackendTrait {
    // Add self as an argument to all these functions
    fn parse_flake_ref(
        &self,
        flake_ref: impl AsRef<str>,
        attribute_path: Option<impl AsRef<str>>,
    ) -> Result<Model>;
    fn parse_trace_file(&self, file_path: impl AsRef<path::Path>) -> Result<Model>;
    fn parse_lines(&self, lines: impl Iterator<Item = impl AsRef<str>>) -> Result<Model>;
}

/// `BackendHandleTrait` is a trait that defines the behavior of a backend handle.
///
/// This trait should be implemented by any backend handle that wants to provide
/// a consistent interface for interacting with the backend.
pub trait BackendHandleTrait {
    /// Gets all messages that were produced since the previous call to this function
    fn get_new_messages(&self) -> Result<Vec<nixtract::message::Message>>;

    /// Gets an iterator over all messages
    fn get_messages(&self) -> Result<impl Iterator<Item = nixtract::message::Message>>;

    /// Gets an upper bound to the number of different ids to expect in the messages
    fn get_num_ids(&self) -> usize;
}

#[derive(Debug, Clone, Default)]
/// Represents a backend that can be used to generate a `Model` from.
/// Used to specify which backend to use when generating a `Model`.
/// This enum mostly just exists for `clap`.
pub enum BackendEnum {
    #[default]
    Nixtract,
}

/// Represents a handle to a backend. This is used to communicate with the backend.
/// An implements the get_messages and get_new_messages functions.
pub enum BackendHandle {
    Nixtract(nixtract_backend::NixtractHandle),
}

#[derive(Debug, Clone)]
pub enum Backend {
    Nixtract(nixtract_backend::Nixtract),
}

impl ValueEnum for BackendEnum {
    fn value_variants<'a>() -> &'a [Self] {
        &[BackendEnum::Nixtract]
    }

    fn to_possible_value(&self) -> Option<clap::builder::PossibleValue> {
        match self {
            BackendEnum::Nixtract => Some(clap::builder::PossibleValue::new("nixtract")),
        }
    }
}

impl BackendEnum {
    pub fn get_backend(&self) -> (Backend, BackendHandle) {
        match self {
            BackendEnum::Nixtract => {
                let (nixtract, nixtract_handle) = nixtract_backend::Nixtract::new();
                (
                    Backend::Nixtract(nixtract),
                    BackendHandle::Nixtract(nixtract_handle),
                )
            }
        }
    }
}

impl BackendTrait for Backend {
    fn parse_flake_ref(
        &self,
        flake_ref: impl AsRef<str>,
        attribute_path: Option<impl AsRef<str>>,
    ) -> Result<Model> {
        match self {
            Backend::Nixtract(backend) => backend.parse_flake_ref(flake_ref, attribute_path),
        }
    }

    fn parse_trace_file(&self, file_path: impl AsRef<path::Path>) -> Result<Model> {
        match self {
            Backend::Nixtract(backend) => backend.parse_trace_file(file_path),
        }
    }

    fn parse_lines(&self, lines: impl Iterator<Item = impl AsRef<str>>) -> Result<Model> {
        match self {
            Backend::Nixtract(backend) => backend.parse_lines(lines),
        }
    }
}

impl BackendHandleTrait for BackendHandle {
    fn get_new_messages(&self) -> Result<Vec<nixtract::message::Message>> {
        match self {
            BackendHandle::Nixtract(handle) => handle.get_new_messages(),
        }
    }

    fn get_messages(&self) -> Result<impl Iterator<Item = nixtract::message::Message>> {
        match self {
            BackendHandle::Nixtract(handle) => handle.get_messages(),
        }
    }

    fn get_num_ids(&self) -> usize {
        match self {
            BackendHandle::Nixtract(handle) => handle.get_num_ids(),
        }
    }
}
