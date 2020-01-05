use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};

use crate::errors::*;
use crate::models::timer::*;
use crate::models::project::*;
use crate::models::tag::*;
use crate::utils;

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

pub fn handle_inserts(
    conn: &mut Connection,
    project: &str,
    tag_str: Option<&str>,
    create_timer: &CreateTimer
) -> AppResult<()> {
    let project_id = Project::insert_and_get_id(&conn, project)?;
    let tags = utils::parse_tags(tag_str);

    let tag_ids = match tags {
        Some(tags) => Some(Tag::batch_insert(conn, tags)?),
        None => None
    };

    let timer_id = create_timer.insert_and_get_id(&conn)?;

    conn.execute(
        "INSERT OR IGNORE INTO projects_timers (project_id, timer_id) VALUES (?1, ?2)",
        params![project_id, timer_id]
    )?;

    if let Some(tag_ids) = tag_ids {
        let tx = conn.transaction()?;
        for tag_id in tag_ids {
            tx.execute("INSERT OR IGNORE INTO tags_timers (timer_id, tag_id) VALUES (?1, ?2)", &[timer_id, tag_id])?;
        }
        tx.commit()?;
    }

    Ok(())
}
