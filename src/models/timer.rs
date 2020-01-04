use std::{io, io::Write, path::{Path, PathBuf}, fs, fs::File, env};

use chrono::{Duration, DateTime, Utc};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

use crate::errors::{AppResult, AppError};
use crate::models::config::Config;
use crate::models::project::Project;
use crate::models::tag::Tag;
use crate::utils::{serialize_ints, format_seconds};

#[derive(Debug)]
pub struct Timers(pub Vec<Timer>);
impl Timers {
    pub fn new(timers: Vec<Timer>) -> Self {
        Timers(timers)
    }

    pub fn default() -> Self {
        Timers(vec![])
    }

    pub fn load(conn: &Connection, extra: &str) -> AppResult<Self> {
        let sql = format!("SELECT id, project_id, start, end FROM timers {}", extra);
        let mut stmt = conn.prepare(&sql)?;
        let timer_iter = stmt.query_map(params![], |row| {
            Ok(Timer {
                id: row.get(0)?,
                project_id: row.get(1)?,
                start: row.get(2)?,
                end: row.get(3)?,
            })
        })?;

        let mut timers = vec![];
        for timer in timer_iter {
            timers.push(timer.unwrap())
        }

        Ok(Timers::new(timers))
    }

    pub fn all(conn: &Connection) -> AppResult<Self> {
        Timers::load(&conn, "")
    }

    pub fn currently_running(conn: &Connection) -> AppResult<Self> {
        Timers::load(&conn, "WHERE end IS NULL")
    }

    pub fn finished(conn: &Connection) -> AppResult<Self> {
        Timers::load(&conn, "WHERE end IS NOT NULL")
    }
}

#[derive(Debug)]
pub struct Timer {
    pub id: i32,
    pub project_id: i32,
    pub start: DateTime<Utc>,
    pub end: Option<DateTime<Utc>>,
}

impl Timer {
    pub fn pretty_print(&self, config: &Config) {
        let duration = Utc::now().signed_duration_since(self.start);
        let formatted = format_seconds(duration.num_seconds());

        println!("  Timer for project {}", &self.project_id);
        println!("  Elapsed Time: {}", formatted);
        println!("  Start Time: {}", &self.start.format(&config.full_time_format));
    }
}

#[derive(Debug)]
pub struct CreateTimer {
    pub project_id: i32,
    pub start: DateTime<Utc>,
    pub end: Option<DateTime<Utc>>,
}

impl CreateTimer {
    pub fn new(project_id: i32) -> Self {
        CreateTimer {
            project_id,
            start: Utc::now(),
            end: None,
        }
    }

    pub fn insert(self, conn: &Connection) -> AppResult<usize> {
        conn.execute(
            "INSERT OR IGNORE INTO timers (project_id, start, end) VALUES (?1, ?2, ?3)",
            params![self.project_id, self.start, self.end]
        ).map_err(|e| AppError::from(e))
    }
}

// for ease of use with passing around
pub struct NewTimer {
    pub project: String,
    pub tags: Option<Vec<String>>,
    pub start: DateTime<Utc>,
    pub end: Option<DateTime<Utc>>,
}

impl NewTimer {
    pub fn new(project: String, tags: Option<Vec<String>>) -> Self {
        NewTimer {
            project,
            tags,
            start: Utc::now(),
            end: None
        }
    }
}
