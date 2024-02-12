use std::sync::{atomic, Arc};

use rocket::serde::json::Json;
use rocket::tokio;

use genealogos::{self, cyclonedx};

use crate::messages::{self, Result};

pub type JobId = u16;

/// This JobMap holds the status of all jobs that are currently running
pub type JobMap = Arc<rocket::tokio::sync::Mutex<std::collections::HashMap<JobId, JobStatus>>>;

pub enum JobStatus {
    Stopped,
    Running,
    Done(genealogos::cyclonedx::CycloneDX),
    Error(String),
}

impl ToString for JobStatus {
    fn to_string(&self) -> String {
        match self {
            JobStatus::Running => "running".to_string(),
            JobStatus::Done(_) => "done".to_string(),
            JobStatus::Stopped => "stopped".to_string(),
            JobStatus::Error(e) => format!("error: {}", e),
        }
    }
}

#[rocket::get("/create?<flake_ref>&<attribute_path>&<cyclonedx_version>")]
pub async fn create(
    flake_ref: &str,
    attribute_path: &str,
    cyclonedx_version: Option<cyclonedx::Version>,
    job_map: &rocket::State<JobMap>,
    job_counter: &rocket::State<atomic::AtomicU16>,
) -> Result<messages::CreateResponse> {
    // Create random jobID
    let job_id = job_counter.fetch_add(1, atomic::Ordering::SeqCst);

    job_map
        .try_lock()
        .map_err(|_| {
            // Return a Json(ErrorResponse)
            Json(messages::ErrResponse {
                metadata: messages::Metadata::new(Some(job_id)),
                message: "Could not lock job map".to_owned(),
            })
        })?
        .insert(job_id, JobStatus::Running);

    // Spawn a new thread to call `genealogos` and store the result in the job map
    let job_map_clone = Arc::clone(job_map);
    let flake_ref = flake_ref.to_string();
    let attribute_path = attribute_path.to_string();
    tokio::spawn(async move {
        let output = genealogos::cyclonedx(
            genealogos::backend::Backend::Nixtract,
            genealogos::Source::Flake {
                flake_ref,
                attribute_path: Some(attribute_path),
            },
            cyclonedx_version.unwrap_or_default(),
        );

        job_map_clone.try_lock().unwrap().insert(
            job_id,
            match output {
                Ok(c) => JobStatus::Done(c),
                Err(e) => JobStatus::Error(e.to_string()),
            },
        );
    });

    let json = Json(messages::OkResponse {
        metadata: messages::Metadata::new(Some(job_id)),
        data: messages::CreateResponse { job_id },
    });

    Ok(json)
}

#[rocket::get("/status/<job_id>")]
pub async fn status(
    job_id: JobId,
    job_map: &rocket::State<JobMap>,
) -> Result<messages::StatusResponse> {
    let locked_map = job_map.try_lock().map_err(|_| {
        Json(messages::ErrResponse {
            metadata: messages::Metadata::new(Some(job_id)),
            message: "Could not lock job map".to_owned(),
        })
    })?;

    let status = locked_map.get(&job_id).unwrap_or(&JobStatus::Stopped);

    let json = Json(messages::OkResponse {
        metadata: messages::Metadata::new(Some(job_id)),
        data: messages::StatusResponse {
            status: status.to_string(),
        },
    });

    Ok(json)
}

#[rocket::get("/result/<job_id>")]
pub fn result(job_id: JobId, job_map: &rocket::State<JobMap>) -> Result<messages::AnalyzeResponse> {
    let mut locked_map = job_map.try_lock().map_err(|_| {
        Json(messages::ErrResponse {
            metadata: messages::Metadata::new(Some(job_id)),
            message: "Could not lock job map".to_owned(),
        })
    })?;

    let status = locked_map.get(&job_id).ok_or(Json(messages::ErrResponse {
        metadata: messages::Metadata::new(Some(job_id)),
        message: "Job not found".to_owned(),
    }))?;

    let sbom = match status {
        JobStatus::Done(s) => Ok(s.clone()),
        _ => Err(messages::ErrResponse {
            metadata: messages::Metadata::new(Some(job_id)),
            message: "Job not yet done".to_owned(),
        }),
    }?;

    // Delete the entry from the job map
    // This prevents having a huge job map over time
    locked_map.remove(&job_id);

    let json = Json(messages::OkResponse {
        metadata: messages::Metadata::new(Some(job_id)),
        data: messages::AnalyzeResponse { sbomb: sbom },
    });

    Ok(json)
}
