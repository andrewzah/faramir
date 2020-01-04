use std::{io, io::Write, path::{Path, PathBuf}, fs, fs::File, env};

use chrono::{Duration, DateTime, Utc};
use rusqlite::{named_params, params, Connection};
use serde::{Deserialize, Serialize};

use crate::errors::{AppResult, AppError};
use crate::models::config::Config;
use crate::models::project::Project;
use crate::models::tag::Tag;
use crate::utils::{rand_string, serialize_ints, format_seconds};

#[derive(Debug)]
pub struct Timers(pub Vec<Timer>);
impl Timers {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn stop_all(&mut self, conn: &Connection) -> AppResult<()> {
        for timer in &mut self.0 {
            &timer.stop(&conn)?;
        }

        Ok(())
    }

    pub fn new(timers: Vec<Timer>) -> Self {
        Timers(timers)
    }

    pub fn default() -> Self {
        Timers(vec![])
    }

    pub fn for_project(conn: &Connection, project_id: i32) -> AppResult<Self> {
        let sql = "SELECT timer_id FROM projects_timers WHERE project_id = ?1";
        let mut stmt = conn.prepare(&sql)?;
        let timer_iter = stmt.query_map(&[project_id], |row| {
            Ok(Timer {
                id: row.get(0)?,
                rid: row.get(1)?,
                start: row.get(2)?,
                end: row.get(3)?,
            })
        })?;

        let mut timers = vec![];
        for timer in timer_iter {
            timers.push(timer?)
        }

        Ok(Timers::new(timers))
    }

    pub fn load(conn: &Connection, extra: &str) -> AppResult<Self> {
        let sql = format!("SELECT * FROM timers {}", extra);
        let mut stmt = conn.prepare(&sql)?;
        let timer_iter = stmt.query_map(params![], |row| {
            Ok(Timer {
                id: row.get(0)?,
                rid: row.get(1)?,
                start: row.get(2)?,
                end: row.get(3)?,
            })
        })?;

        let mut timers = vec![];
        for timer in timer_iter {
            timers.push(timer?)
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

    pub fn limit(conn: &Connection, limit: &str) -> AppResult<Self> {
        Timers::load(&conn, &format!("WHERE end is NOT NULL LIMIT {}", limit))
    }
}

#[derive(Debug)]
pub struct Timer {
    pub id: i32,
    pub rid: String,
    pub start: DateTime<Utc>,
    pub end: Option<DateTime<Utc>>,
}

impl Timer {
    pub fn pretty_print(&self, config: &Config, is_detailed: bool) {
        let duration = match self.end {
            Some(end) => end.signed_duration_since(self.start),
            None => Utc::now().signed_duration_since(self.start),
        };
        let format_elapsed = format_seconds(duration.num_seconds());
        let format_start = match is_detailed {
            true => self.start.format(&config.full_time_format),
            false => self.start.format(&config.time_format),
        };

        println!("  Elapsed Time: {}", format_elapsed);
        println!("  Start Time: {}", format_start);
    }

    pub fn update(&self, conn: &Connection) -> AppResult<()> {
        let sql = "UPDATE timers SET end=?2 WHERE id = ?1";
        conn.execute(&sql, params![self.id, self.end])?;

        Ok(())
    }

    pub fn stop(&mut self, conn: &Connection) -> AppResult<()> {
        self.end = Some(Utc::now());
        self.update(&conn)?;
        println!("Stopped timer {}.", self.rid);

        Ok(())
    }

    pub fn find_by(conn: &Connection, column: &str, val: &str) -> AppResult<Timer> {
        let sql = format!("SELECT * FROM timers WHERE {} = ?1", column);
        let mut stmt = conn.prepare(&sql)?;
        stmt.query_row(&[val], |row| {
            Ok(Timer {
                id: row.get(0)?,
                rid: row.get(1)?,
                start: row.get(2)?,
                end: row.get(3)?,
            })
        }).map_err(|e| AppError::from(e))
    }
}

#[derive(Debug)]
pub struct CreateTimer {
    pub rid: String,
    pub start: DateTime<Utc>,
    pub end: Option<DateTime<Utc>>,
}

impl CreateTimer {
    pub fn new() -> Self {
        CreateTimer {
            rid: rand_string(12),
            start: Utc::now(),
            end: None,
        }
    }

    pub fn insert_and_get_id(&self, conn: &Connection) -> AppResult<i32> {
        &self.insert(&conn)?;
        println!("rid: {}", &self.rid);
        let timer = Timer::find_by(&conn, "rid", &self.rid)?;
        Ok(timer.id)
    }

    pub fn insert(&self, conn: &Connection) -> AppResult<usize> {
        conn.execute(
            "INSERT OR IGNORE INTO timers (rid, start, end) VALUES (?1, ?2, ?3)",
            params![self.rid, self.start, self.end]
        ).map_err(|e| AppError::from(e))
    }
}
