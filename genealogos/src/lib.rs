//! Genealogos is a tool that takes nixtract output and creates an bom file.
//! This output file can then be used by external tools for further analysis.
//! Currently, this crate only supports [cyclonedx],
//!
//! Conceptually, this library consists of three domains.
//! The first is the `backend` domain, which is responsible for extracting information from a source.
//! The second is the `model` domain, which is responsible for representing the extracted information in a structured format.
//! The third is the `bom` domain, which is responsible for writing the structured information to an output format.
//!
//! Using the library will typically go through those three domains in sequence.
//!
//! [cyclonedx]: https://cyclonedx.org/
//!
//! # Examples
//! ```no_run
//! use genealogos::backend::Backend;
//! use genealogos::bom::Bom;
//!
//! fn main() -> Result<(), genealogos::Error> {
//!
//!   // Step 1: Construct the backend, we do not care about updates so we construct it without a communication handle
//!   let backend = genealogos::backend::nixtract_backend::Nixtract::new_without_handle();
//!
//!   // Step 2: Extract the information from the backend
//!   let model = backend.to_model_from_flake_ref("github:NixOS/nixpkgs/nixos-21.05", Some("hello"))?;
//!
//!   // Step 3: Write the model to a BOM file
//!   let bom = genealogos::bom::cyclonedx::CycloneDX::default();
//!   let mut output = String::new();
//!   bom.write_to_fmt_writer(model, &mut output)?;
//!   println!("{}", output);
//!   Ok(())
//! }
//! ```

// Export the Error type for external users
pub use self::error::{Error, Result};

#[cfg(feature = "args")]
pub mod args;
pub mod backend;
pub mod bom;
pub mod error;
pub mod model;

#[cfg(test)]
mod tests {
    use log::{debug, info};
    use pretty_assertions::assert_eq;
    use serde::Deserialize;
    use std::fs;
    use test_log::test;

    use crate::{backend::Backend, bom::Bom};

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

                let backend = crate::backend::nixtract_backend::Nixtract::new_without_handle();
                let model = backend.to_model_from_trace_file(&input_path).unwrap();
                let bom = crate::bom::cyclonedx::CycloneDX::default();

                let mut output_1_4 = String::new();
                bom.write_to_fmt_writer(model, &mut output_1_4).unwrap();

                // 1.4
                let mut expected_path_1_4 = input_path.clone();
                expected_path_1_4.set_extension("1_4.out");
                debug!("testing against {}", expected_path_1_4.to_string_lossy());
                let expected_output_1_4 = fs::read_to_string(expected_path_1_4).unwrap();
                assert_eq!(output_1_4, expected_output_1_4.trim());
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

                let backend = crate::backend::nixtract_backend::Nixtract::new_without_handle();
                let model = backend
                    .to_model_from_flake_ref(flake_args.flake_ref, flake_args.attribute_path)
                    .unwrap();

                let bom = crate::bom::cyclonedx::CycloneDX::default();

                let mut output_1_4 = String::new();
                bom.write_to_fmt_writer(model, &mut output_1_4).unwrap();

                // 1.4
                let mut expected_path_1_4 = input_path.clone();
                expected_path_1_4.set_extension("1_4.out");
                debug!("testing against {}", expected_path_1_4.to_string_lossy());
                let expected_output_1_4 = fs::read_to_string(expected_path_1_4).unwrap();
                assert_eq!(output_1_4, expected_output_1_4.trim());
            }
        }
    }
}
