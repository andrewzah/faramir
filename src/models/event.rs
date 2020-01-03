use std::path::Path;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::errors::{AppResult, AppError};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct NewEvent {
    pub project: String,
    pub tags: Option<Vec<String>>,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Event {
    pub id: i32,
    pub project: String,
    pub tags: Option<Vec<String>>,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Events(pub Vec<Event>);

impl Events {
    pub fn new() -> Self {
        Events(vec![])
    }
}
