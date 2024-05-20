use std::sync::{atomic, Arc};
use std::time;

use genealogos::args::BomArg;
use genealogos::backend::Backend;
use genealogos::bom::Bom;
use rocket::serde::json::Json;
use rocket::tokio;

use crate::messages::{self, Result, StatusEnum, StatusResponse};

pub mod job_map;

use job_map::{JobId, JobMap, JobStatus};

#[rocket::get("/create?<installable>&<bom_format>")]
pub async fn create(
    installable: &str,
    bom_format: Option<BomArg>,
    job_map: &rocket::State<JobMap>,
    job_counter: &rocket::State<atomic::AtomicU16>,
) -> Result<messages::CreateResponse> {
    // Create random jobID
    let job_id = job_counter.fetch_add(1, atomic::Ordering::SeqCst);
    let start_time = time::Instant::now();

    // Create backend
    let (backend, backend_handle) = genealogos::backend::nixtract_backend::Nixtract::new();

    job_map
        .lock()
        .await
        .insert(job_id, JobStatus::Running(Box::new(backend_handle)));

    let bom_arg = bom_format.unwrap_or_default();

    let bom = bom_arg
        .get_bom()
        .map_err(|e| messages::ErrResponse::with_job_id(job_id, e))?;

    // Spawn a new thread to call `genealogos` and store the result in the job map
    let job_map_clone = Arc::clone(job_map);
    let installable = installable.to_string();

    tokio::spawn(async move {
        let source = match genealogos::backend::Source::parse_installable(installable) {
            Ok(m) => m,
            Err(e) => {
                job_map_clone
                    .lock()
                    .await
                    .insert(job_id, JobStatus::Error(e.to_string()));
                return;
            }
        };

        let model = match backend.to_model_from_source(source) {
            Ok(m) => m,
            Err(e) => {
                job_map_clone
                    .lock()
                    .await
                    .insert(job_id, JobStatus::Error(e.to_string()));
                return;
            }
        };

        let mut buf = String::new();
        let output = bom.write_to_fmt_writer(model, &mut buf);

        job_map_clone.lock().await.insert(
            job_id,
            match output {
                Ok(_) => JobStatus::Done(buf, start_time.elapsed()),
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
    let mut locked_map = job_map.lock().await;

    let status = locked_map.get(&job_id).unwrap_or(&JobStatus::Stopped);

    let response = match status {
        JobStatus::Error(message) => Err(Json(messages::ErrResponse {
            metadata: messages::Metadata::new(Some(job_id)),
            message: message.to_owned(),
        })),
        JobStatus::Running(backend) => {
            let messages = backend
                .new_messages()
                .map_err(|e| messages::ErrResponse::with_job_id(job_id, e))?;

            // Show the last message if there are multiple with the same id
            let mut messages: Vec<genealogos::backend::Message> = messages.into_iter().collect();
            messages.sort_by_key(|m| m.index);
            messages.dedup_by_key(|m| m.index);

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
    let mut locked_map = job_map
        .try_lock()
        .map_err(|e| messages::ErrResponse::with_job_id(job_id, e))?;

    let status = locked_map.get(&job_id).ok_or(Json(messages::ErrResponse {
        metadata: messages::Metadata::new(Some(job_id)),
        message: "Job not found".to_owned(),
    }))?;

    let (bom, elapsed) = match status {
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
        data: messages::AnalyzeResponse { bom },
    });

    Ok(json)
}
