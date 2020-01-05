use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, NO_PARAMS};
use serde::{Deserialize, Serialize};

use crate::{
    errors::{AppError, AppResult},
    models::config::Config,
    utils::{format_seconds, rand_string},
};

#[derive(Debug)]
#[allow(dead_code)]
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

    #[allow(dead_code)]
    pub fn default() -> Self {
        Timers(vec![])
    }

    pub fn batch_delete(self, conn: &mut Connection) -> AppResult<()> {
        let tx = conn.transaction()?;
        for timer in self.0 {
            tx.execute(
                "DELETE FROM tags_timers WHERE timer_id = ?1",
                &[&timer.id],
            )?;
            tx.execute("DELETE FROM timers WHERE id = ?1", &[&timer.id])?;
        }
        tx.commit().map_err(|e| AppError::from(e))
    }

    //todo actually make this work
    pub fn for_project(conn: &Connection, project_id: i32) -> AppResult<Self> {
        let sql = "SELECT t.* FROM projects_timers pt JOIN timers t ON \
                   pt.timer_id = t.id WHERE pt.project_id = ?1";
        let mut stmt = conn.prepare(&sql)?;
        let timer_iter = stmt.query_map(&[project_id], |row| {
            Ok(Timer {
                id:    row.get(0)?,
                rid:   row.get(1)?,
                start: row.get(2)?,
                end:   row.get(3)?,
                note:  row.get(4)?,
            })
        })?;

        let mut timers = vec![];
        for timer in timer_iter {
            timers.push(timer?)
        }

        Ok(Timers::new(timers))
    }

    pub fn for_tag(conn: &Connection, project_id: i32) -> AppResult<Self> {
        let sql = "SELECT t.* FROM tags_timers tt JOIN timers t ON \
                   tt.timer_id = t.id WHERE tt.tag_id = ?1";
        let mut stmt = conn.prepare(&sql)?;
        let timer_iter = stmt.query_map(&[project_id], |row| {
            Ok(Timer {
                id:    row.get(0)?,
                rid:   row.get(1)?,
                start: row.get(2)?,
                end:   row.get(3)?,
                note:  row.get(4)?,
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
                id:    row.get(0)?,
                rid:   row.get(1)?,
                start: row.get(2)?,
                end:   row.get(3)?,
                note:  row.get(4)?,
            })
        })?;

        let mut timers = vec![];
        for timer in timer_iter {
            timers.push(timer?)
        }

        Ok(Timers::new(timers))
    }

    #[allow(dead_code)]
    pub fn all(conn: &Connection) -> AppResult<Self> {
        Timers::load(&conn, "")
    }

    pub fn currently_running(conn: &Connection) -> AppResult<Self> {
        Timers::load(&conn, "WHERE end IS NULL")
    }

    #[allow(dead_code)]
    pub fn finished(conn: &Connection) -> AppResult<Self> {
        Timers::load(&conn, "WHERE end IS NOT NULL")
    }

    pub fn limit(conn: &Connection, limit: &str) -> AppResult<Self> {
        Timers::load(&conn, &format!("WHERE end is NOT NULL LIMIT {}", limit))
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Timer {
    pub id:    i32,
    pub rid:   String,
    pub start: DateTime<Utc>,
    pub end:   Option<DateTime<Utc>>,
    pub note:  Option<String>,
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
        let sql = "UPDATE timers SET start=?2,end=?3 WHERE id = ?1";
        conn.execute(&sql, params![self.id, self.start, self.end])?;

        Ok(())
    }

    pub fn stop(&mut self, conn: &Connection) -> AppResult<()> {
        self.end = Some(Utc::now());
        self.update(&conn)?;
        println!("Stopped timer {}.", self.rid);

        Ok(())
    }

    pub fn find_by(
        conn: &Connection, column: &str, val: &str,
    ) -> AppResult<Timer> {
        let sql = format!("SELECT * FROM timers WHERE {} = ?1", column);
        let mut stmt = conn.prepare(&sql)?;
        stmt.query_row(&[val], |row| {
            Ok(Timer {
                id:    row.get(0)?,
                rid:   row.get(1)?,
                start: row.get(2)?,
                end:   row.get(3)?,
                note:  row.get(4)?,
            })
        })
        .map_err(|e| AppError::from(e))
    }

    pub fn last(conn: &Connection) -> AppResult<Timer> {
        let sql = format!("SELECT * FROM timers ORDER BY id DESC");
        let mut stmt = conn.prepare(&sql)?;
        stmt.query_row(NO_PARAMS, |row| {
            Ok(Timer {
                id:    row.get(0)?,
                rid:   row.get(1)?,
                start: row.get(2)?,
                end:   row.get(3)?,
                note:  row.get(4)?,
            })
        })
        .map_err(|e| AppError::from(e))
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateTimer {
    pub rid:   String,
    pub start: DateTime<Utc>,
    pub end:   Option<DateTime<Utc>>,
    pub note:  Option<String>,
}

impl CreateTimer {
    pub fn new(
        start: DateTime<Utc>, end: Option<DateTime<Utc>>, note: Option<String>,
    ) -> Self {
        CreateTimer {
            rid: rand_string(12),
            start,
            end,
            note,
        }
    }

    pub fn default() -> Self {
        CreateTimer {
            rid:   rand_string(12),
            start: Utc::now(),
            end:   None,
            note:  None,
        }
    }

    pub fn insert_and_get_id(&self, conn: &Connection) -> AppResult<i32> {
        &self.insert(&conn)?;
        let timer = Timer::find_by(&conn, "rid", &self.rid)?;
        Ok(timer.id)
    }

    pub fn insert(&self, conn: &Connection) -> AppResult<usize> {
        conn.execute(
            "INSERT OR IGNORE INTO timers (rid, start, end, note) VALUES (?1, \
             ?2, ?3, ?4)",
            params![self.rid, self.start, self.end, self.note],
        )
        .map_err(|e| AppError::from(e))
    }
}
