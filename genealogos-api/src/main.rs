use genealogos::genealogos;
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

#[rocket::get("/api/analyze/<flake_ref>/<attribute_path>")]
fn analyze(flake_ref: &str, attribute_path: &str) -> Result<String, status::Custom<String>> {
    // Construct the Source from the flake reference and attribute path
    let source = genealogos::Source::Flake {
        flake_ref: flake_ref.to_string(),
        attribute_path: Some(attribute_path.to_string()),
    };

    let output = genealogos(genealogos::backend::Backend::Nixtract, source)
        .map_err(|err| status::Custom(Status::InternalServerError, err.to_string()))?;

    Ok(output)
}

#[rocket::launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", rocket::routes![analyze])
        .register("/", rocket::catchers![handle_errors])
}
