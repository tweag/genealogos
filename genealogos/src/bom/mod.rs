//! Similarly to the `backend` module, this module defines the Traits for the "frontend", which we will call `bom` instead.
//! The `Bom` trait contains those functions that a Bom must be able to implement.
//! There is not such thing as a `BomHandle`, like there is a `BackendHandle`, because writing the Bom is not a slow process.
//! And does thus not need to report intermediate status updates.

use crate::{error::Result, model::Model};

pub mod cyclonedx;

/// `Bom` is a trait that provides methods for writing a model to a writer or a string.
pub trait Bom {
    /// Writes the model to a writer.
    ///
    /// # Parameters
    /// - `model`: The model to be written.
    /// - `writer`: The writer to which the model will be written.
    ///
    /// # Returns
    /// - `Result<()>`: Returns `Ok(())` if the model is successfully written, otherwise returns an `Err`.
    fn write_to_io_writer<W: std::io::Write>(&self, model: Model, writer: &mut W) -> Result<()>;

    /// Writes the model to a formatter writer.
    ///
    /// By default, this method first writes the model to a byte buffer using the `write_to_io_writer` method,
    /// then converts the byte buffer to a UTF-8 string and writes it to the formatter writer.
    ///
    /// # Parameters
    /// - `model`: The model to be written.
    /// - `writer`: The formatter writer to which the model will be written.
    ///
    /// # Returns
    /// - `Result<()>`: Returns `Ok(())` if the model is successfully written, otherwise returns an `Err`.
    fn write_to_fmt_writer<W: std::fmt::Write>(&self, model: Model, writer: &mut W) -> Result<()> {
        let mut buf = Vec::new();
        self.write_to_io_writer(model, &mut buf)?;
        writer.write_str(&String::from_utf8(buf)?)?;
        Ok(())
    }
}
