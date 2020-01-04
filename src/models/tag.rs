use rusqlite::{NO_PARAMS, named_params, Connection};

use crate::errors::{AppError, AppResult};

pub struct Tag {
    id: i32,
    name: String,
}

pub struct NewTag {
    name: String,
}

impl Tag {
    pub fn insert(conn: &Connection, name: &str) -> AppResult<usize> {
        conn.execute(
            "INSERT OR IGNORE INTO tags (name) VALUES (?1)",
            &[name]
        ).map_err(|e| AppError::from(e))
    }

    pub fn get_ids(conn: &Connection, names: Vec<&str>) -> AppResult<Vec<i32>> {
        let joined = names
            .iter()
            .map(|n| format!("\"{}\"", n))
            .collect::<Vec<String>>()
            .join(",");

        let mut stmt = conn.prepare(&format!("select id from tags where name in ({})", joined))?;
        let rows = stmt.query_map(NO_PARAMS, |row| row.get(0))?;

        let mut ids = Vec::new();
        for result in rows {
            ids.push(result?);
        }

        println!("stmt: {:?}", &mut stmt.expanded_sql());

        println!("ids: {:?}", &ids);
        Ok(ids)
    }
}
