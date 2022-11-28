use std::collections::HashMap;
use std::fmt::Debug;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use futures::FutureExt;
use serde::Deserialize;

pub mod context;
mod db;
pub mod errors;
pub mod migrate;
mod migrations;
mod utils;

#[derive(Clone)]
pub struct WorkerContext {
    pool: sqlx::PgPool,
}

type WorkerFn =
    Box<dyn Fn(WorkerContext, &str) -> Pin<Box<dyn Future<Output = Result<(), String>> + Send>>>;

pub struct Worker {
    concurrency: usize,
    poll_interval: u32,
    jobs: HashMap<String, WorkerFn>,
}

impl Worker {
    pub fn builder() -> WorkerBuilder {
        WorkerBuilder {
            concurrency: None,
            poll_interval: None,
            jobs: None,
        }
    }
}

#[derive(Default)]
pub struct WorkerBuilder {
    concurrency: Option<usize>,
    poll_interval: Option<u32>,
    jobs: Option<HashMap<String, WorkerFn>>,
}

impl WorkerBuilder {
    pub fn build(self) -> Worker {
        Worker {
            concurrency: self.concurrency.unwrap_or_else(num_cpus::get),
            poll_interval: self.poll_interval.unwrap_or(1000),
            jobs: self.jobs.unwrap_or_else(|| HashMap::new()),
        }
    }

    pub fn concurrency(&mut self, value: usize) -> &mut Self {
        self.concurrency = Some(value);
        self
    }

    pub fn poll_interval(&mut self, value: u32) -> &mut Self {
        self.poll_interval = Some(value);
        self
    }

    pub fn jobs<T, E, Fut, F>(&mut self, identifier: &str, job_fn: F) -> &mut Self
    where
        T: for<'de> Deserialize<'de> + Send,
        E: Debug,
        Fut: Future<Output = Result<(), E>> + Send,
        F: Fn(WorkerContext, T) -> Fut + Send + Sync + Clone + 'static,
    {
        let worker_fn = |ctx, payload| {
            async {
                let de_payload = serde_json::from_str(payload).cloned();

                match de_payload {
                    Err(e) => Err(format!("{:?}", e)),
                    Ok(p) => {
                        let job_result = job_fn.clone()(ctx, p).await;
                        match job_result {
                            Err(e) => Err(format!("{:?}", e)),
                            Ok(v) => Ok(v),
                        }
                    }
                }
            }
            .boxed()
        };

        let job_map = self.jobs.unwrap_or_else(|| HashMap::new());
        job_map.insert(identifier.to_string(), Box::new(worker_fn));
        self
    }
}
