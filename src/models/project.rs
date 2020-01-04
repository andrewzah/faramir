use rusqlite::Connection;

use crate::errors::{AppError,AppResult};

pub struct Project {
    id: i32,
    name: String,
}

pub struct NewProject {
    name: String,
}

impl Project {
    pub fn insert(conn: &Connection, name: &str) -> AppResult<usize> {
        conn.execute(
            "INSERT OR IGNORE INTO projects (name) VALUES (?1)",
            &[name]
        ).map_err(|e| AppError::from(e))
    }

    pub fn load(conn: &Connection, name: &str) -> AppResult<i32> {
        let mut stmt = conn.prepare("SELECT * FROM projects WHERE name = ?1")?;
        let project = stmt.query_row(&[name], |row| {
            Ok(Project {
                id: row.get(0)?,
                name: row.get(1)?,
            })
        })?;

        Ok(project.id)
    }

    pub fn insert_and_get_id(conn: &Connection, name: &str) -> AppResult<i32> {
        Project::insert(&conn, &name)?;
        Project::load(&conn, &name)
    }
}
