use crate::errors::AppResult;

pub struct Project {
    id: String,
    name: String,
}

pub struct NewProject {
    name: String,
}

impl Project {
    pub fn insert(conn: &Connection, name: String) -> AppResult<usize> {
        conn.execute(
            "INSERT OR IGNORE INTO projects (name) VALUES (?1)",
            &[name]
        )
    }
}
