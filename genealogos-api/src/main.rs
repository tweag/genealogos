use genealogos::{cyclonedx, genealogos};
use rocket::http::Status;
use rocket::response::status;
use rocket::Request;

#[rocket::catch(default)]
fn handle_errors(req: &Request) -> status::Custom<String> {
    status::Custom(
        Status::InternalServerError,
        format!("An error occurred: {:?}", req),
    )
}

#[rocket::get("/api/analyze?<flake_ref>&<attribute_path>&<cyclonedx_version>")]
fn analyze(
    flake_ref: &str,
    attribute_path: Option<&str>,
    cyclonedx_version: cyclonedx::Version,
) -> Result<String, status::Custom<String>> {
    // Construct the Source from the flake reference and attribute path
    let source = genealogos::Source::Flake {
        flake_ref: flake_ref.to_string(),
        attribute_path: attribute_path.map(str::to_string),
    };

    let output = genealogos(
        genealogos::backend::Backend::Nixtract,
        source,
        cyclonedx_version,
    )
    .map_err(|err| status::Custom(Status::InternalServerError, err.to_string()))?;

    Ok(output)
}

#[rocket::launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", rocket::routes![analyze])
        .register("/", rocket::catchers![handle_errors])
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

                let response_string = response.into_string();

                // 1.5
                let mut expected_path_1_5 = input_path.clone();
                expected_path_1_5.set_extension("1_5.out");
                let expected_output_1_5 = std::fs::read_to_string(expected_path_1_5).unwrap();
                assert_eq!(
                    response_string,
                    Some(expected_output_1_5.trim().to_string())
                );
            }
        }
    }
}
