use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};

use crate::errors::*;
use crate::event::*;
use crate::timer::*;

pub fn init_db(conn: Connection) -> AppResult<usize> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS projects (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL
        );",
        params![]
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS tags (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL
        );",
        params![]
    )?;

    conn.execute("
        CREATE TABLE IF NOT EXISTS timers (
            id INTEGER PRIMARY KEY,
            project_id INTEGER,
            start TEXT NOT NULL,
            end TEXT,
            tag_ids TEXT,
            FOREIGN KEY(project_id) REFERENCES projects(id)
        );",
        params![]
    )?;

    conn.execute("
        CREATE TABLE IF NOT EXISTS events (
            id INTEGER PRIMARY KEY,
            project_id INTEGER,
            start TEXT NOT NULL,
            end TEXT NOT NULL,
            tag_ids TEXT,
            FOREIGN KEY(project_id) REFERENCES projects(id)
        );",
        params![]
    ).map_err(|e| AppError::from(e))
}

pub fn demo_data(conn: Connection) -> AppResult<usize> {
    timers = vec![
        Timer {
            project: "project1",
            tags: None,
            start: Utc::now(),
            end: None
        },
        Timer {
            project: "project2",
            tags: vec!["tag1", "tag2"],
            start: Utc::now(),
            end: None
        }
    ];

    events = vec![
        Event {
            project: "project1",
            tags: None,
            start: Utc::now(),
            end: Utc::now(),
        },
        Timer {
            project: "project2",
            tags: vec!["tag1", "tag2"],
            start: Utc::now(),
            end: Utc::now()
        }
    ];

    conn.execute(
        "CREATE TABLE IF NOT EXISTS projects (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL
        );",
        params![]
    ).map_err(|e| AppError::from(e))
}
