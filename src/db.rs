use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};

use crate::errors::*;
use crate::models::timer::*;

pub fn init_db(conn: &Connection) -> AppResult<usize> {
    // projects
    conn.execute(
        "CREATE TABLE IF NOT EXISTS projects (
            id INTEGER PRIMARY KEY,
            name TEXT UNIQUE NOT NULL
        );",
        params![]
    )?;

    // tags
    conn.execute(
        "CREATE TABLE IF NOT EXISTS tags (
            id INTEGER PRIMARY KEY,
            name TEXT UNIQUE NOT NULL
        );",
        params![]
    )?;

    // timers
    conn.execute("
        CREATE TABLE IF NOT EXISTS timers (
            id INTEGER PRIMARY KEY,
            project_id INTEGER,
            start TEXT NOT NULL,
            end TEXT,
            FOREIGN KEY(project_id) REFERENCES projects(id)
        );",
        params![]
    )?;

    // tags_timers
    conn.execute(
        "CREATE TABLE IF NOT EXISTS tags_timers (
            tag_id INTEGER NOT NULL,
            timer_id INTEGER NOT NULL,
            FOREIGN KEY(tag_id) REFERENCES tags(id),
            FOREIGN KEY(timer_id) REFERENCES timers(id)
        );",
        params![]
    ).map_err(|e| AppError::from(e))
}

pub fn demo_data(conn: &Connection) -> AppResult<usize> {
    let timers = vec![
        NewTimer {
            project: "project1".into(),
            start: Utc::now(),
            end: None,
            tags: None
        },
        NewTimer {
            project: "project2".into(),
            start: Utc::now(),
            tags: Some(vec!["tag1".into(), "tag2".into()]),
            end: None,
        }
    ];

    conn.execute(
        "
        ",
        params![]
    ).map_err(|e| AppError::from(e))
}
