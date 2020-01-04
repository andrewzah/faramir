use std::collections::HashMap;

use rusqlite::{Connection, params, NO_PARAMS};

use crate::errors::{AppError,AppResult};
use crate::models::timer::Timers;

pub struct Projects(Vec<Project>);

impl Projects {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn print_basic(self) {
        println!("{} Project(s) found.", self.len());
        println!("{}", self.0.into_iter().map(|p| p.name).collect::<Vec<String>>().join(", "));
    }

    pub fn print_detailed(&self, conn: &Connection) -> AppResult<()> {
        println!("{} Project(s) found.", self.len());

        let mut project_timers = HashMap::new();

        for project in &self.0 {
            let timers = Timers::for_project(&conn, project.id)?;
            project_timers.insert(&project.name, timers);
        }

        for project in &self.0 {
            println!("{} - {} timer(s) found.", &project.name, project_timers.get(&project.name).unwrap().len())
        }

        Ok(())
    }

    pub fn default() -> Self {
        Projects(vec![])
    }

    pub fn push(&mut self, project: Project) {
        self.0.push(project)
    }

    pub fn all(conn: &Connection) -> AppResult<Self> {
        let mut stmt = conn.prepare("SELECT * FROM projects ORDER BY name")?;
        let project_iter = stmt.query_map(NO_PARAMS, |row| {
            Ok(Project {
                id: row.get(0)?,
                name: row.get(1)?,
            })
        })?;

        let mut projects = Projects::default();
        for project in project_iter {
            projects.push(project?);
        }

        Ok(projects)
    }
}

pub struct Project {
    pub id: i32,
    pub name: String,
}

pub struct NewProject {
    pub name: String,
}

impl Project {
    pub fn insert_and_get_id(conn: &Connection, name: &str) -> AppResult<i32> {
        Project::insert(&conn, &name)?;
        let project = Project::find_by_name(&conn, &name)?;
        Ok(project.id)
    }

    pub fn insert(conn: &Connection, name: &str) -> AppResult<usize> {
        conn.execute(
            "INSERT OR IGNORE INTO projects (name) VALUES (?1)",
            &[name]
        ).map_err(|e| AppError::from(e))
    }

    pub fn update(&self, conn: &Connection, new_name: &str) -> AppResult<()> {
        let sql = "UPDATE projects SET name=?2 WHERE name = ?1";
        conn.execute(&sql, params![self.name, new_name])?;

        Ok(())
    }

    pub fn find_by_name(conn: &Connection, name: &str) -> AppResult<Project> {
        let mut stmt = conn.prepare("SELECT * FROM projects WHERE name = ?1")?;
        stmt.query_row(&[name], |row| {
            Ok(Project {
                id: row.get(0)?,
                name: row.get(1)?,
            })
        }).map_err(|e| AppError::from(e))
    }

    pub fn find(conn: &Connection, project_id: i32) -> AppResult<Project> {
        let mut stmt = conn.prepare("SELECT * FROM projects WHERE id = ?1")?;
        stmt.query_row(&[project_id], |row| {
            Ok(Project {
                id: row.get(0)?,
                name: row.get(1)?,
            })
        }).map_err(|e| AppError::from(e))
    }

    pub fn for_timer(conn: &Connection, timer_id: i32) -> AppResult<Project> {
        let mut stmt = conn.prepare("SELECT project_id FROM projects_timers WHERE timer_id = ?1")?;
        let project_id = stmt.query_row(&[timer_id], |row| row.get(0))?;
        Project::find(&conn, project_id)
    }
}
