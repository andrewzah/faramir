pub struct Tag {
    id: String,
    name: String,
}

pub struct NewTag {
    name: String,
}

impl Tag {
    pub fn insert(conn: Connection, name: String) -> AppResult<usize> {
        conn.execute(
            "INSERT OR IGNORE INTO tags (name) VALUES (?1)",
            &[name]
        );
    }

    pub fn get_ids(conn: Connection, names: Vec<String>) -> AppResult<Vec<i32>> {
        conn.execute(
            "select id from projects where name in ?1",
            &[names]
        );
    }
}
