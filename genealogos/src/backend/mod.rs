//! The `backend` module contains the definition of the `Backend` trait, which is used to
//! define a common interface for generating a `Model` from a given source, and the `Source`
//! enum, which represents the source of the data.
//! It also exports the actual implementations of the backends, in case you need to use just one.

use std::path;

use crate::{error::Result, model::Model};

// We have an crate dependency that is already called nixtract, to avoid conflict, this module is called nixtract_backend.
// TODO: Rename module to `nixtract`, crate to `nixtract-crate`.
pub mod nixtract_backend;

/// `Source` is an enum representing the source of data.
///
/// It can be one of the following:
/// - `Flake`: This variant is used when the source is a flake. It contains a `flake_ref` which is a string
///   representing the reference to the flake, and an `attribute_path` which is an optional string representing
///   the attribute path within the flake.
/// - `TraceFile`: This variant is used when the source is a trace file. It contains a `PathBuf` representing
///   the path to the trace file.
#[derive(Debug)]
pub enum Source {
    /// Represents a nix installable with a reference and an optional attribute path.
    Installable {
        flake_ref: String,
        attribute_path: Option<String>,
    },

    /// Represents a trace file source with a path.
    TraceFile(std::path::PathBuf),
}

/// This trait represents a backend that can be used to generate a `Model` from.
/// Every backend must be able to generate a `Model` from a flake reference and
/// an optional attribute path, or from a trace file.
///
/// Generating from a trace file or lines is preferred, as it is much faster. Generating
/// from a flake reference is slower, as it requires a full Nix evaluation.
pub trait Backend {
    /// Converts a given source into a Model.
    ///
    /// This function takes a `Source` enum as an argument, which can be either
    /// a `Flake` or a `TraceFile`. Depending on the variant of the `Source`, it
    /// calls the appropriate function to convert the source into a `Model`.
    ///
    /// # Arguments
    ///
    /// * `source` - A `Source` enum that represents the source to be converted into a `Model`.
    ///
    /// # Returns
    ///
    /// * `Result<Model>` - A `Result` type that returns a `Model` if the conversion is
    ///   successful, or an error if it fails.
    ///
    /// # Errors
    ///
    /// This function will return an error if the conversion from the source to
    /// the `Model` fails.
    fn to_model_from_source(&self, source: Source) -> Result<Model> {
        match source {
            Source::Installable {
                flake_ref,
                attribute_path,
            } => self.to_model_from_flake_ref(flake_ref, attribute_path),
            Source::TraceFile(file_path) => self.to_model_from_trace_file(file_path),
        }
    }

    /// Converts a trace file into a `Model`.
    ///
    /// This function reads a trace file from the provided path and converts it into a `Model`.
    /// By default, it does this by reading the file into a string, splitting
    /// it into lines, and then passing those lines to the `to_model_from_lines`
    /// function.
    ///
    /// # Arguments
    ///
    /// * `file_path` - A path that implements `AsRef<path::Path>`. This is the
    ///   path to the trace file.
    ///
    /// # Returns
    ///
    /// * `Result<Model>` - The `Model` generated from the trace file, or an
    ///   error if one occurred.
    ///
    /// # Errors
    ///
    /// This function will return an error if the file cannot be read.
    fn to_model_from_trace_file(&self, file_path: impl AsRef<path::Path>) -> Result<Model> {
        let file_path = file_path.as_ref();
        let lines = std::fs::read_to_string(file_path)?;
        self.to_model_from_lines(lines.lines())
    }

    /// Converts a flake reference to a model.
    ///
    /// This function takes a flake reference and an optional attribute path
    /// as input. The flake reference is a string that identifies a specific
    /// version of a package or project. The attribute path is an optional
    /// string that can be used to specify a particular attribute within the
    /// flake. If no attribute_path is provided, all of the flake's outputs
    /// are extracted.
    ///
    /// # Arguments
    ///
    /// * `flake_ref` - A string that represents the flake reference.
    /// * `attribute_path` - An optional string that represents the attribute path.
    ///
    /// # Returns
    ///
    /// This function returns a `Result` that contains the `Model` if the
    /// conversion was successful, or an error if it was not.
    fn to_model_from_flake_ref(
        &self,
        flake_ref: impl AsRef<str>,
        attribute_path: Option<impl AsRef<str>>,
    ) -> Result<Model>;

    /// Converts lines of text into a `Model`.
    ///
    /// This function takes an iterator over items that can be referenced as strings,
    /// and attempts to convert them into a `Model`. If the conversion is successful,
    /// the function returns `Ok(Model)`. If the conversion fails, it returns an `Err`.
    ///
    /// # Arguments
    ///
    /// * `lines` - An iterator over items that can be referenced as strings.
    ///
    /// # Returns
    ///
    /// A `Result` which is `Ok` if the conversion succeeded, and
    ///`crate:error::Error` otherwise.
    fn to_model_from_lines(&self, lines: impl Iterator<Item = impl AsRef<str>>) -> Result<Model>;

    /// Informs the backend to (from now on) attempt to fetch the narinfo for each derivation.
    ///
    /// # Arguments
    ///
    /// * `fetch` - A boolean indicating whether to fetch the narinfo for each derivation or not.
    ///
    /// # Examples
    ///
    /// ```
    /// use genealogos::backend::Backend;
    /// let mut backend = genealogos::backend::nixtract_backend::Nixtract::new_without_handle();
    /// backend.include_narinfo(true);
    /// ```
    fn include_narinfo(&mut self, include: bool);

    /// Informs the backend to (from now on) only include runtime dependencies in the model.
    ///
    /// # Arguments
    ///
    /// * `runtime_only` - A boolean indicating whether to include only runtime dependencies or not.
    ///
    /// # Examples
    ///
    /// ```
    /// use genealogos::backend::Backend;
    /// let mut backend = genealogos::backend::nixtract_backend::Nixtract::new_without_handle();
    /// backend.runtime_only(true);
    /// ```
    fn runtime_only(&mut self, runtime_only: bool);
}

/// If a backend was created with a `BackendHandle`, it can be queried for status updates.
/// In such case, this struct is returned.
/// Purpose of these messages is to communicate progress to the user.
#[derive(Debug, Clone, serde::Serialize)]
pub struct Message {
    /// Messages can have an index associated with them.
    /// This is not inteneded to reflect the message itself, but rather the sender.
    /// In the nixtract backend, this field is used to communicate which thread produced the message, but this index could be used for other purposes.
    /// The index fields are required to be unique and in the range of 0 to `BackendHandle::max_index()`.
    pub index: usize,

    /// The content of the message.
    /// The content is intended to be human-readable and can be any string.
    pub content: String,
}

/// `BackendHandle` is a trait that defines the behavior of a backend handle.
///
/// This trait should be implemented by any backend handle that wants to provide
/// a consistent interface for interacting with the backend.
pub trait BackendHandle {
    /// Gets all messages that were produced since the previous call to this function
    fn new_messages(&self) -> Result<Vec<Message>>;

    /// Gets an iterator over all messages
    #[cfg(feature = "backend_handle_messages")]
    fn messages(&self) -> Result<impl Iterator<Item = Message>>;

    /// Gets an upper bound to the number of different ids to expect in the messages
    fn max_index(&self) -> usize;
}
