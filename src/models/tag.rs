use rusqlite::{NO_PARAMS, params, Connection};

use crate::errors::{AppError, AppResult};

pub struct Tags(pub Vec<Tag>);

impl Tags {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn ids(&self) -> AppResult<Vec<i32>> {
        let ids = self.0
            .iter()
            .map(|t| t.id)
            .collect::<Vec<i32>>();

        Ok(ids)
    }

    pub fn names(self) -> Vec<String> {
        self.0
            .into_iter()
            .map(|t| t.name)
            .collect()
    }

    pub fn new(tags: Vec<Tag>) -> Self {
        Tags(tags)
    }

    pub fn default() -> Self {
        Tags(vec![])
    }

    pub fn all(conn: &Connection) -> AppResult<Self> {
        let mut stmt = conn.prepare("SELECT * FROM tags ORDER BY name")?;
        let tag_iter = stmt.query_map(NO_PARAMS, |row| {
            Ok(Tag {
                id: row.get(0)?,
                name: row.get(1)?,
            })
        })?;

        let mut tags = Tags::default();
        for tag in tag_iter {
            tags.0.push(tag?);
        }

        Ok(tags)
    }

    pub fn load(conn: &Connection, names: Vec<String>) -> AppResult<Self> {
        let joined = names
            .iter()
            .map(|n| format!("\"{}\"", n))
            .collect::<Vec<String>>()
            .join(",");

        let mut stmt = conn.prepare(&format!("select * from tags where name in ({})", joined))?;
        let tag_iter = stmt.query_map(NO_PARAMS, |row| {
            Ok(Tag {
                id: row.get(0)?,
                name: row.get(1)?,
            })
        })?;

        let mut tags = Vec::new();
        for tag in tag_iter {
            tags.push(tag?);
        }

        Ok(Tags::new(tags))
    }
}

pub struct Tag {
    pub id: i32,
    pub name: String,
}

impl Tag {
    pub fn batch_insert(conn: &mut Connection, names: Vec<String>) -> AppResult<Vec<i32>> {
        let tx = conn.transaction()?;
        for name in &names {
            tx.execute("INSERT OR IGNORE INTO tags (name) VALUES (?1)", &[name])?;
        }
        tx.commit()?;

        Tags::load(&conn, names)?.ids()
    }

    pub fn find_by_name(conn: &Connection, name: &str) -> AppResult<Self> {
        let mut stmt = conn.prepare("SELECT * FROM tags WHERE name = ?1")?;
        stmt.query_row(&[name], |row| {
            Ok(Tag {
                id: row.get(0)?,
                name: row.get(1)?,
            })
        }).map_err(|e| AppError::from(e))
    }


    #[allow(dead_code)]
    pub fn insert(conn: &Connection, name: &str) -> AppResult<usize> {
        conn.execute(
            "INSERT OR IGNORE INTO tags (name) VALUES (?1)",
            &[name]
        ).map_err(|e| AppError::from(e))
    }

    pub fn update(&self, conn: &Connection, new_name: &str) -> AppResult<()> {
        let sql = "UPDATE tags SET name=?2 WHERE name = ?1";
        conn.execute(&sql, params![self.name, new_name])?;

        Ok(())
    }
}
