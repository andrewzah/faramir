use std::io;

use rusqlite::{params, Connection};

use crate::{
    errors::*,
    models::{project::*, tag::*, timer::*},
    utils,
};

pub fn init_db(conn: &Connection) -> AppResult<usize> {
    // projects
    conn.execute(
        "CREATE TABLE IF NOT EXISTS projects (
            id INTEGER PRIMARY KEY,
            name TEXT UNIQUE NOT NULL
        );",
        params![],
    )?;

    // tags
    conn.execute(
        "CREATE TABLE IF NOT EXISTS tags (
            id INTEGER PRIMARY KEY,
            name TEXT UNIQUE NOT NULL
        );",
        params![],
    )?;

    // timers
    conn.execute(
        "
        CREATE TABLE IF NOT EXISTS timers (
            id INTEGER PRIMARY KEY,
            rid TEXT NOT NULL,
            start TEXT NOT NULL,
            end TEXT,
            note TEXT
        );",
        params![],
    )?;

    // tags_timers
    conn.execute(
        "CREATE TABLE IF NOT EXISTS tags_timers (
            tag_id INTEGER NOT NULL,
            timer_id INTEGER NOT NULL,
            FOREIGN KEY(tag_id) REFERENCES tags(id),
            FOREIGN KEY(timer_id) REFERENCES timers(id)
        );",
        params![],
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
        params![],
    );

    result.map_err(|e| AppError::from(e))
}

pub fn delete_project(
    conn: &mut Connection, name: &str, autoconfirm: bool,
) -> AppResult<()> {
    let project = match Project::find_by_name(&conn, &name) {
        Ok(p) => p,
        Err(e) => {
            println!("Project not found.");
            return Err(AppError::from(e));
        },
    };
    let timers = Timers::for_project(&conn, project.id)?;

    if timers.len() > 0 && autoconfirm != true {
        println!(
            "Project {} has {} timers associated with it. Are you sure you \
             want to remove it?\nIf so, type 'y'.",
            &name,
            timers.len()
        );

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if &input != "y\n" {
            return Err(AppError::from_str(
                "A confirmation with 'y' is needed to delete.",
            ));
        }
    }

    println!("deleting pt");
    conn.execute(
        "DELETE FROM projects_timers WHERE project_id = ?1",
        params![&project.id],
    )?;

    println!("deleting projects");
    conn.execute("DELETE FROM projects WHERE id = ?1", params![&project.id])?;

    println!("batch deleting");
    timers.batch_delete(conn)?;

    println!("Successfully removed project {}.", project.name);
    Ok(())
}

pub fn delete_tag(
    conn: &Connection, name: &str, autoconfirm: bool,
) -> AppResult<()> {
    let tag = match Tag::find_by_name(&conn, &name) {
        Ok(tag) => tag,
        Err(e) => {
            println!("Tag not found.");
            return Err(AppError::from(e));
        },
    };

    let timers = Timers::for_tag(&conn, tag.id)?;

    if timers.len() > 0 && autoconfirm != true {
        println!(
            "Tag {} has {} timers associated with it. Are you sure you want \
             to remove it?\nIf so, type 'y'.",
            &name,
            timers.len()
        );

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if &input != "y\n" {
            return Err(AppError::from_str(
                "A confirmation with 'y' is needed to delete.",
            ));
        }
    }

    conn.execute(
        "DELETE FROM tags_timers WHERE tag_id = ?1",
        params![&tag.id],
    )?;

    conn.execute("DELETE FROM tags WHERE id = ?1", params![&tag.id])?;

    println!("Succesfully removed tag {}.", &name);
    Ok(())
}

pub fn delete_timer(conn: &Connection, rid: &str) -> AppResult<()> {
    let timer = Timer::find_by(&conn, "rid", rid)?;

    conn.execute(
        "DELETE FROM projects_timers where timer_id = ?1",
        params![&timer.id],
    )?;

    conn.execute(
        "DELETE FROM tags_timers where timer_id = ?1",
        params![&timer.id],
    )?;

    conn.execute("DELETE FROM timers where id = ?1", params![&timer.id])?;

    println!(
        "Successfully deleted timer {} - start: {}, end: {:?}",
        timer.rid, timer.start, timer.end
    );

    Ok(())
}

pub fn handle_inserts(
    conn: &mut Connection, project: &str, tag_str: Option<&str>,
    create_timer: &CreateTimer,
) -> AppResult<()> {
    let project_id = Project::insert_and_get_id(&conn, project)?;
    let tags = utils::parse_tags(tag_str);

    let tag_ids = match tags {
        Some(tags) => Some(Tag::batch_insert(conn, tags)?),
        None => None,
    };

    let timer_id = create_timer.insert_and_get_id(&conn)?;

    conn.execute(
        "INSERT OR IGNORE INTO projects_timers (project_id, timer_id) VALUES \
         (?1, ?2)",
        params![project_id, timer_id],
    )?;

    if let Some(tag_ids) = tag_ids {
        let tx = conn.transaction()?;
        for tag_id in tag_ids {
            tx.execute(
                "INSERT OR IGNORE INTO tags_timers (tag_id, timer_id) VALUES \
                 (?1, ?2)",
                &[tag_id, timer_id],
            )?;
        }
        tx.commit()?;
    }

    Ok(())
}
