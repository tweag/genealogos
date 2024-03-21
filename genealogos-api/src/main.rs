use std::sync::{atomic, Arc};

use genealogos::backend::Backend;
use genealogos::bom::cyclonedx::FileFormat;
use genealogos::bom::Bom;
use rocket::http::Status;
use rocket::response::status;
use rocket::serde::json::Json;
use rocket::tokio::sync::Mutex;
use rocket::Request;

mod jobs;
mod messages;

use messages::Result;

#[rocket::catch(default)]
fn handle_errors(req: &Request) -> status::Custom<String> {
    status::Custom(
        Status::InternalServerError,
        format!("An error occurred: {:?}", req),
    )
}

#[rocket::get("/analyze?<flake_ref>&<attribute_path>&<cyclonedx_version>")]
fn analyze(
    flake_ref: &str,
    attribute_path: Option<&str>,
    cyclonedx_version: Option<&str>,
) -> Result<messages::AnalyzeResponse> {
    let start_time = std::time::Instant::now();

    // Construct the Source from the flake reference and attribute path
    let source = genealogos::backend::Source::Flake {
        flake_ref: flake_ref.to_string(),
        attribute_path: attribute_path.map(str::to_string),
    };

    let backend = genealogos::backend::nixtract_backend::Nixtract::new_without_handle();

    let model = backend
        .to_model_from_source(source)
        .map_err(|err| messages::ErrResponse {
            metadata: messages::Metadata::new(None),
            message: err.to_string(),
        })?;

    let mut buf = String::new();
    let bom = match cyclonedx_version {
        Some(cyclonedx_version) => genealogos::bom::cyclonedx::CycloneDX::parse_version(
            cyclonedx_version,
            FileFormat::JSON,
        )
        .map_err(|err| messages::ErrResponse {
            metadata: messages::Metadata::new(None),
            message: err.to_string(),
        })?,
        None => genealogos::bom::cyclonedx::CycloneDX::default(),
    };

    bom.write_to_fmt_writer(model, &mut buf)
        .map_err(|err| messages::ErrResponse {
            metadata: messages::Metadata::new(None),
            message: err.to_string(),
        })?;

    let json = Json(messages::OkResponse {
        metadata: messages::Metadata {
            time_taken: Some(start_time.elapsed()),
            ..Default::default()
        },
        data: messages::AnalyzeResponse { sbom: buf },
    });

    Ok(json)
}

#[rocket::launch]
fn rocket() -> _ {
    rocket::build()
        .attach(rocket::fairing::AdHoc::on_response("cors", |_req, resp| {
            Box::pin(async move {
                resp.set_header(rocket::http::Header::new(
                    "Access-Control-Allow-Origin",
                    "*",
                ));
            })
        }))
        .mount("/api", rocket::routes![analyze])
        .register("/api", rocket::catchers![handle_errors])
        .mount(
            "/api/jobs/",
            rocket::routes![jobs::create, jobs::status, jobs::result],
        )
        .register("/api/jobs/", rocket::catchers![handle_errors])
        .manage(Arc::new(Mutex::new(std::collections::HashMap::<
            jobs::JobId,
            jobs::JobStatus,
        >::new())))
        .manage(atomic::AtomicU16::new(0))
}

#[cfg(test)]
mod tests {
    use super::rocket;
    use log::info;
    use pretty_assertions::assert_eq;
    use rocket::http::Status;
    use rocket::local::blocking::Client;
    use serde::Deserialize;

    #[derive(Deserialize, Debug)]
    pub struct FlakeArgs {
        flake_ref: String,
        attribute_path: Option<String>,
    }

    #[test]
    #[cfg_attr(feature = "nix", ignore)]
    fn test_flakes() {
        // Set the GENEALOGOS_DETERMINISTIC environment variable to ensure that the output is deterministic
        std::env::set_var("GENEALOGOS_DETERMINISTIC", "1");

        let client = Client::tracked(rocket()).unwrap();

        let input_dir = std::fs::read_dir("../genealogos/tests/fixtures/nixtract/flakes/").unwrap();

        for input_file in input_dir {
            let input_file = input_file.unwrap();
            let input_path = input_file.path();

            if input_path.extension().unwrap().to_string_lossy() == "in" {
                info!("testing: {}", input_path.to_string_lossy());

                let input = std::fs::read_to_string(input_path.clone()).unwrap();
                let flake_args: FlakeArgs = serde_json::from_str(&input).unwrap();

                // Escape non url-compatible character in flake ref and attribute path
                let flake_ref = urlencoding::encode(&flake_args.flake_ref);

                let attribute_path = flake_args.attribute_path.unwrap();
                let attribute_path = urlencoding::encode(&attribute_path);

                let response = client
                    .get(format!(
                        "/api/analyze?flake_ref={}&attribute_path={}",
                        flake_ref, attribute_path
                    ))
                    .dispatch();

                assert_eq!(response.status(), Status::Ok);

                // Extract the somb from the response
                let response_json: serde_json::Value = response.into_json().unwrap();
                let response_sbom = match response_json.get("sbom").unwrap() {
                    serde_json::Value::String(response_sbom) => response_sbom,
                    _ => panic!("Not a string"),
                };
                let response_sbom: serde_json::Value = serde_json::from_str(response_sbom).unwrap();

                // 1.4
                let mut expected_path_1_4 = input_path.clone();
                expected_path_1_4.set_extension("1_4.out");
                // Read expected_path_1_4 to a string
                let expected_string_1_4 = std::fs::read_to_string(expected_path_1_4).unwrap();
                let expected_output_1_4: serde_json::Value =
                    serde_json::from_str(&expected_string_1_4).unwrap();

                // Convert from and to json to remove the pretty printed stuff
                // let expected_json_1_4: serde_json::Value =
                //     serde_json::from_str(&expected_output_1_4).unwrap();
                // let expected_output_1_4 = serde_json::to_string(&expected_json_1_4).unwrap();

                assert_eq!(response_sbom, expected_output_1_4);
            }
        }
    }
}
