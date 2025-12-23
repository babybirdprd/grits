pub mod advisor;
pub mod analysis;
pub mod context;
pub mod workflow;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StrategicResult<T> {
    pub data: T,
    pub message: Option<String>,
}

impl<T> StrategicResult<T> {
    pub fn new(data: T, message: Option<String>) -> Self {
        Self { data, message }
    }
}
