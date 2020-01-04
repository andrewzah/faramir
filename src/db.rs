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
            rid TEXT NOT NULL,
            start TEXT NOT NULL,
            end TEXT
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
    )?;

    // projects_timers
    let result = conn.execute(
        "CREATE TABLE IF NOT EXISTS projects_timers (
            project_id INTEGER NOT NULL,
            timer_id INTEGER NOT NULL,
            FOREIGN KEY(project_id) REFERENCES projects(id),
            FOREIGN KEY(timer_id) REFERENCES timers(id)
        );

        CREATE UNIQUE INDEX projects_timers_idx
        ON projects_timers (project_id, timer_id);
        ",
        params![]
    );

    result.map_err(|e| AppError::from(e))
}

pub fn demo_data(conn: &Connection) -> AppResult<usize> {
    conn.execute(
        "
        ",
        params![]
    ).map_err(|e| AppError::from(e))
}
