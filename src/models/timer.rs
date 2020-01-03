use std::{io, io::Write, path::{Path, PathBuf}, fs, fs::File, env};

use chrono::{Duration, DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::errors::{AppResult, AppError};
use crate::config::Config;

const TIMERS_FILE: &str = "timers.json";

#[derive(Deserialize, Serialize)]
pub struct Timers(pub Vec<Timer>);

impl Timers {
    pub fn new() -> Self {
        Timers(vec![])
    }

    //TODO: check for duplicate timers
    pub fn append_timer(&mut self, timer: Timer) -> AppResult<()> {
        &self.0.push(timer);
        Ok(())
    }

    pub fn write_file(&self, data_path: &Path) -> AppResult<()> {
        let path = data_path.join(TIMERS_FILE);
        let json = serde_json::to_string_pretty(&self.0)?;
        fs::write(path, json)?;
        Ok(())
    }

    pub fn init_file(path: &Path) -> AppResult<Timers> {
        let timers = Timers::new();
        let json = serde_json::to_string_pretty(&timers)?;
        fs::write(path, json)?;
        Ok(timers)
    }

    pub fn from_path(path: &Path) -> AppResult<Timers> {
        let content = fs::read_to_string(path)?;
        match serde_json::from_str(&content) {
            Ok(timers) => Ok(timers),
            Err(e) => Err(AppError::from(e))
        }
    }

    pub fn load_or_create(data_path: &Path) -> AppResult<Timers> {
        let path = data_path.join(TIMERS_FILE);
        if !path.exists() {
            let timers = Timers::init_file(&path)?;
            return Ok(timers)
        }

        Timers::from_path(&path)
    }
}

#[derive(Deserialize, Serialize)]
pub struct Timer {
    pub project: String,
    pub tags: Option<Vec<String>>,
    pub start: DateTime<Utc>,
    pub end: Option<DateTime<Utc>>,
}

impl Timer {
    pub fn new(project: String, tags: Option<Vec<String>>) -> Self {
        Timer {
            project,
            start: Utc::now(),
            end: None,
            tags: None,
        }
    }

    pub fn format_seconds(secs: i64) -> String {
        let weeks = secs / 604800;
        let days = (secs % 604800) / 86400;
        let hours = ((secs % 604800) % 86400) / 3600;
        let minutes = (((secs % 604800) % 86400) % 3600) / 60;
        let seconds = (((secs % 604800) % 86400) % 3600) % 60;

        format!(
            "{}w, {}d, {}h, {}m, {}s",
            weeks, days, hours, minutes, seconds
        )
    }

    pub fn pretty_print(&self, config: &Config) {
        let duration = Utc::now().signed_duration_since(self.start);
        let formatted = Timer::format_seconds(duration.num_seconds());

        println!("  Timer for project {}", &self.project);
        println!("  Elapsed Time: {}", formatted);
        println!("  Start Time: {}", &self.start.format(&config.full_time_format));
    }
}
