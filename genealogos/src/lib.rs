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

    fn empty_model() -> crate::model::Model {
        crate::model::Model {
            components: Vec::new(),
            dependencies: Vec::new(),
        }
    }

    #[test]
    fn test_cyclonedx_1_5_json_output() {
        let bom = crate::bom::cyclonedx::CycloneDX::new(
            crate::bom::cyclonedx::SpecVersion::V1_5,
            crate::bom::cyclonedx::FileFormat::JSON,
        );

        let mut output = String::new();
        bom.write_to_fmt_writer(empty_model(), &mut output).unwrap();

        let output: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(output["specVersion"], "1.5");
    }

    #[test]
    fn test_cyclonedx_1_5_xml_output() {
        let bom = crate::bom::cyclonedx::CycloneDX::new(
            crate::bom::cyclonedx::SpecVersion::V1_5,
            crate::bom::cyclonedx::FileFormat::XML,
        );

        let mut output = String::new();
        bom.write_to_fmt_writer(empty_model(), &mut output).unwrap();

        assert!(output.contains("http://cyclonedx.org/schema/bom/1.5"));
    }

    #[test]
    fn test_cyclonedx_1_5_multiple_license_ids_output_as_license_choices() {
        let bom = crate::bom::cyclonedx::CycloneDX::new(
            crate::bom::cyclonedx::SpecVersion::V1_5,
            crate::bom::cyclonedx::FileFormat::JSON,
        );
        let model = crate::model::Model {
            components: vec![crate::model::ModelComponent {
                r#type: crate::model::ModelType::Application,
                name: "component".to_owned(),
                r#ref: "component".to_owned(),
                version: "1.0.0".to_owned(),
                description: String::new(),
                external_references: Vec::new(),
                licenses: Some(vec![
                    crate::model::ModelLicense {
                        id: Some("MIT".to_owned()),
                        name: None,
                    },
                    crate::model::ModelLicense {
                        id: Some("Apache-2.0".to_owned()),
                        name: None,
                    },
                ]),
                src: None,
                properties: crate::model::ModelProperties {
                    properties: Default::default(),
                },
            }],
            dependencies: Vec::new(),
        };

        let mut output = String::new();
        bom.write_to_fmt_writer(model, &mut output).unwrap();

        let output: serde_json::Value = serde_json::from_str(&output).unwrap();
        let licenses = output["components"][0]["licenses"].as_array().unwrap();
        assert_eq!(licenses[0]["license"]["id"], "MIT");
        assert_eq!(licenses[1]["license"]["id"], "Apache-2.0");
    }

    #[test]
    fn test_cyclonedx_1_5_spdx_expression_output_as_expression() {
        let bom = crate::bom::cyclonedx::CycloneDX::new(
            crate::bom::cyclonedx::SpecVersion::V1_5,
            crate::bom::cyclonedx::FileFormat::JSON,
        );
        let model = crate::model::Model {
            components: vec![crate::model::ModelComponent {
                r#type: crate::model::ModelType::Application,
                name: "component".to_owned(),
                r#ref: "component".to_owned(),
                version: "1.0.0".to_owned(),
                description: String::new(),
                external_references: Vec::new(),
                licenses: Some(vec![crate::model::ModelLicense {
                    id: Some("MIT OR Apache-2.0".to_owned()),
                    name: None,
                }]),
                src: None,
                properties: crate::model::ModelProperties {
                    properties: Default::default(),
                },
            }],
            dependencies: Vec::new(),
        };

        let mut output = String::new();
        bom.write_to_fmt_writer(model, &mut output).unwrap();

        let output: serde_json::Value = serde_json::from_str(&output).unwrap();
        let licenses = output["components"][0]["licenses"].as_array().unwrap();
        assert_eq!(licenses[0]["expression"], "MIT OR Apache-2.0");
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
                let bom_1_4 = crate::bom::cyclonedx::CycloneDX::new(
                    crate::bom::cyclonedx::SpecVersion::V1_4,
                    crate::bom::cyclonedx::FileFormat::JSON,
                );

                let mut output_1_4 = String::new();
                bom_1_4.write_to_fmt_writer(model, &mut output_1_4).unwrap();

                // 1.4
                let mut expected_path_1_4 = input_path.clone();
                expected_path_1_4.set_extension("1_4.out");
                debug!("testing against {}", expected_path_1_4.to_string_lossy());
                let expected_output_1_4 = fs::read_to_string(expected_path_1_4).unwrap();
                assert_eq!(output_1_4, expected_output_1_4.trim());

                let backend = crate::backend::nixtract_backend::Nixtract::new_without_handle();
                let model = backend.to_model_from_trace_file(&input_path).unwrap();
                let bom_1_5 = crate::bom::cyclonedx::CycloneDX::new(
                    crate::bom::cyclonedx::SpecVersion::V1_5,
                    crate::bom::cyclonedx::FileFormat::JSON,
                );

                let mut output_1_5 = String::new();
                bom_1_5.write_to_fmt_writer(model, &mut output_1_5).unwrap();

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

                let backend = crate::backend::nixtract_backend::Nixtract::new_without_handle();
                let model = backend
                    .to_model_from_flake_ref(flake_args.flake_ref, flake_args.attribute_path)
                    .unwrap();

                let bom_1_4 = crate::bom::cyclonedx::CycloneDX::new(
                    crate::bom::cyclonedx::SpecVersion::V1_4,
                    crate::bom::cyclonedx::FileFormat::JSON,
                );

                let mut output_1_4 = String::new();
                bom_1_4.write_to_fmt_writer(model, &mut output_1_4).unwrap();

                // 1.4
                let mut expected_path_1_4 = input_path.clone();
                expected_path_1_4.set_extension("1_4.out");
                debug!("testing against {}", expected_path_1_4.to_string_lossy());
                let expected_output_1_4 = fs::read_to_string(expected_path_1_4).unwrap();
                assert_eq!(output_1_4, expected_output_1_4.trim());

                let input = fs::read_to_string(input_path.clone()).unwrap();
                let flake_args: FlakeArgs = serde_json::from_str(&input).unwrap();

                let backend = crate::backend::nixtract_backend::Nixtract::new_without_handle();
                let model = backend
                    .to_model_from_flake_ref(flake_args.flake_ref, flake_args.attribute_path)
                    .unwrap();

                let bom_1_5 = crate::bom::cyclonedx::CycloneDX::new(
                    crate::bom::cyclonedx::SpecVersion::V1_5,
                    crate::bom::cyclonedx::FileFormat::JSON,
                );

                let mut output_1_5 = String::new();
                bom_1_5.write_to_fmt_writer(model, &mut output_1_5).unwrap();

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
