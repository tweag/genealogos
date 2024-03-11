use std::sync::{atomic, Arc};
use std::time;

use genealogos::backend::BackendHandleTrait;
use rocket::serde::json::Json;
use rocket::tokio;

use genealogos::{self, cyclonedx};

use crate::messages::{self, Result, StatusEnum, StatusResponse};

pub type JobId = u16;

/// This JobMap holds the status of all jobs that are currently running
pub type JobMap = Arc<rocket::tokio::sync::Mutex<std::collections::HashMap<JobId, JobStatus>>>;

pub enum JobStatus {
    Stopped,
    /// The job is still running, the receiver is used receive status messages from worker threads
    // Running(genealogos::backend::nixtract_backend::Nixtract),
    Running(genealogos::backend::BackendHandle),
    Done(genealogos::cyclonedx::CycloneDX, time::Duration),
    Error(String),
}

impl ToString for JobStatus {
    fn to_string(&self) -> String {
        match self {
            JobStatus::Running(_) => "running".to_string(),
            JobStatus::Done(_, _) => "done".to_string(),
            JobStatus::Stopped => "stopped".to_string(),
            JobStatus::Error(e) => e.to_owned(),
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
    let start_time = time::Instant::now();

    // Create backend
    let (backend, backend_handle) = genealogos::backend::BackendEnum::default().get_backend();

    job_map
        .try_lock()
        .map_err(|_| {
            // Return a Json(ErrorResponse)
            Json(messages::ErrResponse {
                metadata: messages::Metadata::new(Some(job_id)),
                message: "Could not lock job map".to_owned(),
            })
        })?
        .insert(job_id, JobStatus::Running(backend_handle));

    // Spawn a new thread to call `genealogos` and store the result in the job map
    let job_map_clone = Arc::clone(job_map);
    let flake_ref = flake_ref.to_string();
    let attribute_path = attribute_path.to_string();
    tokio::spawn(async move {
        let output = genealogos::cyclonedx(
            backend,
            genealogos::Source::Flake {
                flake_ref,
                attribute_path: Some(attribute_path),
            },
            cyclonedx_version.unwrap_or_default(),
        );

        job_map_clone.try_lock().unwrap().insert(
            job_id,
            match output {
                Ok(c) => JobStatus::Done(c, start_time.elapsed()),
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
    let mut locked_map = job_map.try_lock().map_err(|_| {
        Json(messages::ErrResponse {
            metadata: messages::Metadata::new(Some(job_id)),
            message: "Could not lock job map".to_owned(),
        })
    })?;

    let status = locked_map.get(&job_id).unwrap_or(&JobStatus::Stopped);

    let response = match status {
        JobStatus::Error(message) => Err(Json(messages::ErrResponse {
            metadata: messages::Metadata::new(Some(job_id)),
            message: message.to_owned(),
        })),
        JobStatus::Running(backend) => {
            let messages = backend.get_new_messages().map_err(|e| {
                Json(messages::ErrResponse {
                    metadata: messages::Metadata::new(Some(job_id)),
                    message: e.to_string(),
                })
            })?;

            // Show the last message if there are multiple with the same id
            let mut messages: Vec<nixtract::message::Message> = messages.into_iter().collect();
            messages.sort_by_key(|m| m.id);
            messages.dedup_by_key(|m| m.id);

            Ok(StatusResponse {
                status: StatusEnum::LogMessages(messages),
            })
        }
        JobStatus::Done(_, _) => Ok(StatusResponse {
            status: StatusEnum::Done,
        }),
        JobStatus::Stopped => Ok(StatusResponse {
            status: StatusEnum::Stopped,
        }),
    };

    if response.is_err() {
        // Remove the job if it was an error
        locked_map.remove(&job_id);
    }

    // Propagate errors
    let messages = response?;

    let json = Json(messages::OkResponse {
        metadata: messages::Metadata::new(Some(job_id)),
        data: messages,
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

    let (sbom, elapsed) = match status {
        JobStatus::Done(s, elapsed) => Ok((s.clone(), *elapsed)),
        _ => Err(messages::ErrResponse {
            metadata: messages::Metadata::new(Some(job_id)),
            message: "Job not yet done".to_owned(),
        }),
    }?;

    // Delete the entry from the job map
    // This prevents having a huge job map over time
    locked_map.remove(&job_id);

    let json = Json(messages::OkResponse {
        metadata: messages::Metadata {
            job_id: Some(job_id),
            time_taken: Some(elapsed),
            ..Default::default()
        },
        data: messages::AnalyzeResponse { sbom },
    });

    Ok(json)
}
