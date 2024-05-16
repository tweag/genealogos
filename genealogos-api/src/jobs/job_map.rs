use rocket::tokio::time;

pub type JobMap = std::sync::Arc<rocket::tokio::sync::Mutex<JobHashMap>>;

/// This JobMap holds the status of all jobs that are currently running
pub struct JobHashMap(std::collections::HashMap<JobId, JobMapEntry>);

/// A single entry in the job map, contains all data related to the job and some
/// metadata required for the garbage collector.
pub struct JobMapEntry {
    /// Stores the last time this job was accesed. Any job that is not accessed
    /// for a certain amount of time is considered stale and will be removed.
    last_updated: time::Instant,
    /// The status of the job
    status: JobStatus,
}

pub type JobId = u16;

/// The status of a single job
pub enum JobStatus {
    /// The job has been stopped and is not running anymore, or it has not been started yet
    Stopped,
    /// The job is still running, the receiver is used receive status messages from worker threads
    Running(Box<dyn genealogos::backend::BackendHandle + Send>),
    /// The job has finished, the string contains the output of the job
    /// and the duration contains how long it took to finish
    Done(String, time::Duration),
    /// The job has thrown an error, the string contains the error message
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

impl JobMapEntry {
    pub fn new(status: JobStatus) -> Self {
        Self {
            last_updated: time::Instant::now(),
            status,
        }
    }
}

impl JobHashMap {
    pub fn insert(&mut self, job_id: JobId, job_status: JobStatus) {
        self.0.insert(job_id, JobMapEntry::new(job_status));
    }

    pub fn get(&mut self, job_id: &JobId) -> Option<&JobStatus> {
        self.0.get(job_id).map(|entry| &entry.status)
    }

    pub fn remove(&mut self, job_id: &JobId) -> Option<JobStatus> {
        self.0.remove(job_id).map(|entry| entry.status)
    }

    pub(crate) fn new() -> Self {
        Self(std::collections::HashMap::new())
    }
}

/// The garbage collector will check for any stale jobs in the `JobMap` and remove them
/// after a certain amount of time. The interval is how often the garbage collector
/// will run, and the remove_after is when a job is considered stale.
/// The garbage collector will run in a loop forever.
/// This function will block the thread it is running in.
///
/// # Arguments
/// * `job_map` - A reference to the `JobMap` that contains all the jobs
/// * `interval` - How often the garbage collector will run
/// * `remove_after` - How long after a job is considered stale
pub async fn garbage_collector(
    job_map: JobMap,
    interval: time::Duration,
    remove_after: time::Duration,
) {
    let mut interval = time::interval(interval);

    log::info!("Started the garbage collector");

    loop {
        log::info!("Collecting garbage");
        interval.tick().await;

        for (job_id, job_entry) in job_map.lock().await.0.iter_mut() {
            if job_entry.last_updated.elapsed() > remove_after {
                log::info!("Removing a stale job");
                job_map.lock().await.remove(job_id);
            }
        }
    }
}
