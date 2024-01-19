//! Genealogos is a tool that takes nixtract output and creates a (`CycloneDX`)[cyclonedx] compatible sbom file.
//! This output file can then be used by external tools for further analysis.
//!
//! [cyclonedx]: https://cyclonedx.org/
use serde_cyclonedx::cyclonedx::v_1_5 as cyclonedx;

use crate::model::Model;
use crate::nixtract::Nixtract;

// Export the Error type for external users
pub use self::error::{Error, Result};

mod error;
pub mod model;
pub mod nixtract;

/// Converts Nixtract entries to CycloneDX model and serializes it to JSON.
///
/// # Arguments
///
/// * `input_entries`: A collection of Nixtract entries, represented as `impl IntoIterator<Item = impl AsRef<str>>`
///   (or any iterator that contains things that can be referenced as `str`).
///
/// # Returns
///
/// A JSON-formatted string representation of the CycloneDX sbom.
///
/// # Panics
///
/// Panics if any of the input entries cannot be parsed as Nixtract entries.
pub fn genealogos(input_entries: impl IntoIterator<Item = impl AsRef<str>>) -> Result<String> {
    let mut entries = vec![];

    for input_entry in input_entries {
        let entry: nixtract::NixtractEntry = serde_json::from_str(input_entry.as_ref().trim())?;
        entries.push(entry);
    }
    let nixtract: Nixtract = Nixtract { entries };

    let model: Model = nixtract.into();
    let cyclonedx: cyclonedx::CycloneDx = model.try_into()?;

    Ok(serde_json::to_string(&cyclonedx)?)
}

#[cfg(test)]
mod tests {
    use log::info;
    use std::{fs, io::BufRead};
    use test_log::test;

    #[test]
    fn test_fixtures() {
        let input_dir = fs::read_dir("tests/fixtures/nixtract/success/").unwrap();

        for input_file in input_dir {
            let input_file = input_file.unwrap();
            let input_path = input_file.path();

            if input_path.extension().unwrap().to_string_lossy() == "in" {
                info!("testing: {}", input_path.to_string_lossy());

                let input_file = fs::File::open(&input_path).unwrap();

                let output =
                    crate::genealogos(std::io::BufReader::new(input_file).lines().flatten())
                        .unwrap();

                let mut expected_path = input_path.clone();
                expected_path.set_extension("out");

                let expected_output = fs::read_to_string(expected_path).unwrap();

                assert_eq!(output, expected_output.trim());
            }
        }
    }
}
