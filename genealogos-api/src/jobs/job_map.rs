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
            JobStatus::Error(e) => format!("Error: {}", e),
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

#[derive(rocket::serde::Deserialize, Debug)]
#[serde(crate = "rocket::serde")]
pub struct GCInterval(u64);

#[derive(rocket::serde::Deserialize, Debug)]
#[serde(crate = "rocket::serde")]
pub struct GCStaleAfter(u64);

impl Default for GCInterval {
    fn default() -> Self {
        // By default, run the garbage collector once every ten seconds
        Self(10)
    }
}

impl Default for GCStaleAfter {
    fn default() -> Self {
        // By default, remove a stale job after 1 hour
        Self(60 * 60)
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
/// * `gc_config` - The configuration for the garbage collector
pub async fn garbage_collector(job_map: JobMap, gc_config: crate::config::GCConfig) {
    let stale_after = time::Duration::from_secs(gc_config.stale_after.0);
    let mut interval = time::interval(time::Duration::from_secs(gc_config.interval.0));

    log::info!("Started the garbage collector");

    loop {
        interval.tick().await;

        let mut count: u16 = 0;
        let mut job_map = job_map.lock().await;
        log::info!("Current job count: {}", job_map.0.len());

        // Retain allo jobs that are not stale
        job_map.0.retain(|_, entry| {
            if entry.last_updated.elapsed() < stale_after {
                true
            } else {
                count += 1;
                false
            }
        });

        if count > 0 {
            log::info!("Removed {} stale jobs", count);
        }
    }
}
