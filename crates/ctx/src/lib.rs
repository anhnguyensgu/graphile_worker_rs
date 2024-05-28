use getset::Getters;
use graphile_worker_extensions::ReadOnlyExtensions;
use graphile_worker_job::Job;
use serde_json::Value;
use sqlx::PgPool;

#[derive(Getters, Clone, Debug)]
#[getset(get = "pub")]
pub struct WorkerContext {
    payload: Value,
    pg_pool: PgPool,
    job: Job,
    worker_id: String,
    extensions: ReadOnlyExtensions,
}

impl WorkerContext {
    pub fn new(
        payload: Value,
        pg_pool: PgPool,
        job: Job,
        worker_id: String,
        extensions: ReadOnlyExtensions,
    ) -> Self {
        WorkerContext {
            payload,
            pg_pool,
            job,
            worker_id,
            extensions,
        }
    }
}
