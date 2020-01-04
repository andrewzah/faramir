use std::{io, io::Write, path::{Path, PathBuf}, fs, fs::File, env};

use chrono::{Duration, DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::errors::{AppResult, AppError};
use crate::models::config::Config;
use crate::models::project::Project;
use crate::models::tag::Tag;
use crate::utils::format_seconds;

#[derive(Debug)]
pub struct Timers(pub Vec<Timer>);

#[derive(Debug)]
pub struct Timer {
    pub id: i32,
    pub project: i32,
    pub tags: Option<Vec<i32>>,
    pub start: DateTime<Utc>,
    pub end: Option<DateTime<Utc>>,
}

#[derive(Debug)]
pub struct NewTimer {
    pub project: String,
    pub tags: Option<Vec<String>>,
    pub start: DateTime<Utc>,
    pub end: Option<DateTime<Utc>>,
}

impl Timer {
    pub fn new(project_id: i32, tags: Option<Vec<String>>) -> Self {
        Timer {
            project,
            start: Utc::now(),
            end: None,
            tags: None,
        }
    }

    pub fn pretty_print(&self, config: &Config) {
        let duration = Utc::now().signed_duration_since(self.start);
        let formatted = format_seconds(duration.num_seconds());

        println!("  Timer for project {}", &self.project);
        println!("  Elapsed Time: {}", formatted);
        println!("  Start Time: {}", &self.start.format(&config.full_time_format));
    }
}

impl NewTimer {
    pub fn insert(&self, conn: Connection) -> AppResult<usize> {
        Ok(0)
    }
}
