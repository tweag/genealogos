use std::sync::{atomic, Arc};

use rocket::http::Status;
use rocket::{response, tokio};

use genealogos;

/// This JobMap holds the status of all jobs that are currently running
pub type JobMap = Arc<rocket::tokio::sync::Mutex<std::collections::HashMap<JobId, JobStatus>>>;

pub type JobId = u16;
pub enum JobStatus {
    Stopped,
    Running,
    Done(String),
}

impl ToString for JobStatus {
    fn to_string(&self) -> String {
        match self {
            JobStatus::Running => "running".to_string(),
            JobStatus::Done(_) => "done".to_string(),
            JobStatus::Stopped => "stopped".to_string(),
        }
    }
}

#[rocket::get("/create/<flake_ref>/<attribute_path>")]
pub async fn create(
    flake_ref: &str,
    attribute_path: &str,
    job_map: &rocket::State<JobMap>,
    job_counter: &rocket::State<atomic::AtomicU16>,
) -> Result<String, response::status::Custom<String>> {
    // Create random jobID
    let jobid = job_counter.fetch_add(1, atomic::Ordering::SeqCst);

    job_map
        .try_lock()
        .map_err(|_| {
            response::status::Custom(
                Status::InternalServerError,
                "Could not lock job map".to_string(),
            )
        })?
        .insert(jobid, JobStatus::Running);

    // Spawn a new thread to call `genealogos` and store the result in the job map
    let job_map_clone = Arc::clone(job_map);
    let flake_ref = flake_ref.to_string();
    let attribute_path = attribute_path.to_string();
    let jobid = jobid.clone();
    tokio::spawn(async move {
        let output = genealogos::genealogos(
            genealogos::backend::Backend::Nixtract,
            genealogos::Source::Flake {
                flake_ref,
                attribute_path: Some(attribute_path),
            },
        )
        .unwrap();

        job_map_clone
            .try_lock()
            .unwrap()
            .insert(jobid, JobStatus::Done(output));
    });

    Ok(jobid.to_string())
}

#[rocket::get("/status/<jobid>")]
pub async fn status(
    jobid: JobId,
    job_map: &rocket::State<JobMap>,
) -> Result<String, response::status::Custom<String>> {
    let locked_map = job_map.try_lock().map_err(|_| {
        response::status::Custom(
            Status::InternalServerError,
            "Could not lock job map".to_string(),
        )
    })?;

    let status = locked_map.get(&jobid).unwrap_or(&JobStatus::Stopped);

    Ok(status.to_string())
}

#[rocket::get("/result/<jobid>")]
pub fn result(
    jobid: JobId,
    job_map: &rocket::State<JobMap>,
) -> Result<String, response::status::Custom<String>> {
    let locked_map = job_map.try_lock().map_err(|_| {
        response::status::Custom(
            Status::InternalServerError,
            "Could not lock job map".to_string(),
        )
    })?;

    let status = locked_map.get(&jobid).unwrap_or(&JobStatus::Stopped);

    match status {
        JobStatus::Done(s) => Ok(s.clone()),
        _ => Err(response::status::Custom(
            Status::InternalServerError,
            "Job not done yet".to_string(),
        )),
    }
}
