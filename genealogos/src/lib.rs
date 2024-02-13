//! Genealogos is a tool that takes nixtract output and creates a (`CycloneDX`)[cyclonedx] compatible sbom file.
//! This output file can then be used by external tools for further analysis.
//!
//! [cyclonedx]: https://cyclonedx.org/

use cyclonedx::CycloneDX;

// Export the Error type for external users
pub use self::error::{Error, Result};

pub mod backend;
pub mod cyclonedx;
mod error;
pub mod model;

#[derive(Debug)]
pub enum Source {
    Flake {
        flake_ref: String,
        attribute_path: Option<String>,
    },
    TraceFile(std::path::PathBuf),
}

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
pub fn json_string(
    backend: crate::backend::Backend,
    source: Source,
    version: cyclonedx::Version,
) -> Result<String> {
    // Convert `Model` to `CycloneDx`
    let cyclonedx = cyclonedx(backend, source, version)?;

    // Serialize the `Model` to JSON
    let json = serde_json::to_string_pretty(&cyclonedx)?;

    Ok(json)
}

pub fn cyclonedx(
    backend: crate::backend::Backend,
    source: Source,
    version: cyclonedx::Version,
) -> Result<CycloneDX> {
    // Convert the input entries to a `Model`
    let model = match source {
        Source::Flake {
            flake_ref,
            attribute_path,
        } => backend.to_model_from_flake_ref(flake_ref, attribute_path)?,
        Source::TraceFile(file_path) => backend.to_model_from_trace_file(file_path)?,
    };

    // Convert `Model` to `CycloneDx`
    let cyclonedx = CycloneDX::from_model(model, version)?;
    Ok(cyclonedx)
}

#[cfg(test)]
mod tests {
    use log::{debug, info};
    use pretty_assertions::assert_eq;
    use serde::Deserialize;
    use std::fs;
    use test_log::test;

    #[derive(Deserialize, Debug)]
    pub struct FlakeArgs {
        flake_ref: String,
        attribute_path: Option<String>,
    }

    #[test]
    fn test_trace_files() {
        let input_dir = fs::read_dir("tests/fixtures/nixtract/trace-files/").unwrap();

        for input_file in input_dir {
            let input_file = input_file.unwrap();
            let input_path = input_file.path();

            if input_path.extension().unwrap().to_string_lossy() == "in" {
                info!("testing: {}", input_path.to_string_lossy());

                let output_1_4 = super::json_string(
                    crate::backend::Backend::default(),
                    super::Source::TraceFile(input_path.clone()),
                    super::cyclonedx::Version::V1_4,
                )
                .unwrap();

                let output_1_5 = super::json_string(
                    crate::backend::Backend::default(),
                    super::Source::TraceFile(input_path.clone()),
                    super::cyclonedx::Version::V1_5,
                )
                .unwrap();

                // 1.4
                let mut expected_path_1_4 = input_path.clone();
                expected_path_1_4.set_extension("1_4.out");
                debug!("testing against {}", expected_path_1_4.to_string_lossy());
                let expected_output_1_4 = fs::read_to_string(expected_path_1_4).unwrap();
                assert_eq!(output_1_4, expected_output_1_4.trim());

                // 1.5
                let mut expected_path_1_5 = input_path.clone();
                expected_path_1_5.set_extension("1_5.out");
                debug!("testing against {}", expected_path_1_5.to_string_lossy());
                let expected_output_1_5 = fs::read_to_string(expected_path_1_5).unwrap();
                assert_eq!(output_1_5, expected_output_1_5.trim());
            }
        }
    }

    #[test]
    #[cfg_attr(feature = "nix", ignore)]
    fn test_flakes() {
        let input_dir = fs::read_dir("tests/fixtures/nixtract/flakes/").unwrap();

        for input_file in input_dir {
            let input_file = input_file.unwrap();
            let input_path = input_file.path();

            if input_path.extension().unwrap().to_string_lossy() == "in" {
                info!("testing: {}", input_path.to_string_lossy());

                let input = fs::read_to_string(input_path.clone()).unwrap();
                let flake_args: FlakeArgs = serde_json::from_str(&input).unwrap();

                let output_1_4 = super::json_string(
                    crate::backend::Backend::default(),
                    super::Source::Flake {
                        flake_ref: flake_args.flake_ref.clone(),
                        attribute_path: flake_args.attribute_path.clone(),
                    },
                    super::cyclonedx::Version::V1_4,
                )
                .unwrap();

                let output_1_5 = super::json_string(
                    crate::backend::Backend::default(),
                    super::Source::Flake {
                        flake_ref: flake_args.flake_ref,
                        attribute_path: flake_args.attribute_path,
                    },
                    super::cyclonedx::Version::V1_5,
                )
                .unwrap();

                // 1.4
                let mut expected_path_1_4 = input_path.clone();
                expected_path_1_4.set_extension("1_4.out");
                debug!("testing against {}", expected_path_1_4.to_string_lossy());
                let expected_output_1_4 = fs::read_to_string(expected_path_1_4).unwrap();
                assert_eq!(output_1_4, expected_output_1_4.trim());

                // 1.5
                let mut expected_path_1_5 = input_path.clone();
                expected_path_1_5.set_extension("1_5.out");
                debug!("testing against {}", expected_path_1_5.to_string_lossy());
                let expected_output_1_5 = fs::read_to_string(expected_path_1_5).unwrap();
                assert_eq!(output_1_5, expected_output_1_5.trim());
            }
        }
    }
}
