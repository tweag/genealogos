//! This module contains the message types used by the API.
//! The `GenealogosResponder` enum is used to define the possible responses from the server.
//! Responses are created from the `OkResponse` and `ErrorResponse`, but need to be manually
//! converted into a json value.

use rocket::serde::json::Json;

pub type Result<T> = std::result::Result<Json<OkResponse<T>>, Json<ErrResponse>>;

#[derive(serde::Serialize)]
pub struct AnalyzeResponse {
    pub sbom: String,
}

#[derive(serde::Serialize)]
pub struct CreateResponse {
    pub job_id: u16,
}

#[derive(serde::Serialize)]
pub struct StatusResponse {
    pub status: StatusEnum,
}

#[derive(serde::Serialize)]
pub enum StatusEnum {
    LogMessages(Vec<genealogos::backend::Message>),
    Done,
    Stopped,
}

#[derive(serde::Serialize)]
pub struct OkResponse<T> {
    pub metadata: Metadata,
    #[serde(flatten)]
    pub data: T,
}

#[derive(serde::Serialize)]
pub struct ErrResponse {
    pub metadata: Metadata,
    pub message: String,
}

#[derive(serde::Serialize)]
pub struct Metadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub job_id: Option<u16>,
    pub timestamp: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_taken: Option<std::time::Duration>,
}

impl Default for Metadata {
    fn default() -> Self {
        Metadata {
            job_id: None,
            timestamp: chrono::Utc::now().to_rfc3339(),
            time_taken: None,
        }
    }
}

impl Metadata {
    pub fn new(job_id: Option<u16>) -> Self {
        Metadata {
            job_id,
            timestamp: chrono::Utc::now().to_rfc3339(),
            time_taken: None,
        }
    }
}
