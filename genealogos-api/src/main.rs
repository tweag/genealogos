use std::sync::{atomic, Arc};

use genealogos::args::BomArg;
use genealogos::backend::Backend;
use genealogos::bom::Bom;
use jobs::job_map::{self, garbage_collector};
use rocket::http::Status;
use rocket::response::{content, status};
use rocket::serde::json::Json;
use rocket::tokio::sync::Mutex;
use rocket::Request;

mod config;
mod jobs;
mod messages;

use messages::{ErrResponse, Result};

#[rocket::catch(default)]
fn handle_errors(req: &Request) -> status::Custom<String> {
    status::Custom(
        Status::InternalServerError,
        format!("An error occurred: {:?}", req),
    )
}

#[rocket::get("/")]
#[cfg(feature = "frontend")]
fn index() -> status::Custom<content::RawHtml<&'static str>> {
    status::Custom(
        Status::Ok,
        content::RawHtml(include_str!("../../genealogos-frontend/index.html")),
    )
}

#[rocket::get("/")]
#[cfg(not(feature = "frontend"))]
fn index() -> status::Custom<content::RawHtml<&'static str>> {
    status::Custom(
        Status::Ok,
        content::RawHtml("<h1>Genealogos is running</h1>"),
    )
}

#[rocket::get("/analyze?<installable>&<bom_format>")]
fn analyze(installable: &str, bom_format: Option<BomArg>) -> Result<messages::AnalyzeResponse> {
    let start_time = std::time::Instant::now();

    // Construct the Source from the flake reference and attribute path
    let source = genealogos::backend::Source::parse_installable(installable)
        .map_err(Into::<ErrResponse>::into)?;

    let backend = genealogos::backend::nixtract_backend::Nixtract::new_without_handle();
    let model = backend
        .to_model_from_source(source)
        .map_err(Into::<ErrResponse>::into)?;
    let mut buf = String::new();
    let bom_arg = bom_format.unwrap_or_default();
    let bom = bom_arg.get_bom().map_err(Into::<ErrResponse>::into)?;

    bom.write_to_fmt_writer(model, &mut buf)
        .map_err(Into::<ErrResponse>::into)?;

    let json = Json(messages::OkResponse {
        metadata: messages::Metadata {
            time_taken: Some(start_time.elapsed()),
            ..Default::default()
        },
        data: messages::AnalyzeResponse { bom: buf },
    });

    Ok(json)
}

#[rocket::launch]
fn rocket() -> _ {
    let job_map = Arc::new(Mutex::new(job_map::JobHashMap::new()));

    let job_map_clone = job_map.clone();

    let rocket = rocket::build();
    let figment = rocket.figment();

    let config: config::Config = figment.extract().expect("Failed to load configuration");

    rocket
        .attach(rocket::fairing::AdHoc::on_response("cors", |_req, resp| {
            Box::pin(async move {
                resp.set_header(rocket::http::Header::new(
                    "Access-Control-Allow-Origin",
                    "*",
                ));
            })
        }))
        .attach(rocket::fairing::AdHoc::on_liftoff(
            "garbage_collector",
            |_| {
                Box::pin(async move {
                    rocket::tokio::spawn(async move {
                        garbage_collector(job_map_clone, config.gc).await;
                    });
                })
            },
        ))
        .mount("/", rocket::routes![index])
        .mount("/api", rocket::routes![analyze])
        .register("/api", rocket::catchers![handle_errors])
        .mount(
            "/api/jobs/",
            rocket::routes![jobs::create, jobs::status, jobs::result],
        )
        .register("/api/jobs/", rocket::catchers![handle_errors])
        .manage(job_map)
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
                        "/api/analyze?installable={}%23{}",
                        flake_ref, attribute_path
                    ))
                    .dispatch();

                assert_eq!(response.status(), Status::Ok);

                // Extract the somb from the response
                let response_json: serde_json::Value = response.into_json().unwrap();
                let response_bom = match response_json.get("bom").unwrap() {
                    serde_json::Value::String(response_bom) => response_bom,
                    _ => panic!("Not a string"),
                };
                let response_bom: serde_json::Value = serde_json::from_str(response_bom).unwrap();

                // 1.5
                let mut expected_path_1_5 = input_path.clone();
                expected_path_1_5.set_extension("1_5.out");
                // Read expected_path_1_5 to a string
                let expected_string_1_5 = std::fs::read_to_string(expected_path_1_5).unwrap();
                let expected_output_1_5: serde_json::Value =
                    serde_json::from_str(&expected_string_1_5).unwrap();

                assert_eq!(response_bom, expected_output_1_5);
            }
        }
    }
}
