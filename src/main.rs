#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_variables)]

use std::{io, path::{Path, PathBuf}, fs, fs::File, env};
use std::io::Write;

use chrono::{DateTime, Utc};
use clap::{Arg, ArgMatches, App, load_yaml};
use termcolor::{BufferWriter, Color, ColorChoice, ColorSpec, StandardStream, WriteColor};
use rusqlite::{params, Connection};

mod db;
mod errors;
mod models;
mod utils;

use db::*;
use models::timer::{Timers, Timer, CreateTimer};
use models::config::Config;
use models::project::{Projects, Project};
use models::tag::{Tag, Tags};
use errors::{AppResult, AppError, ErrorKind};

fn load_or_create_config(config_path: PathBuf) -> AppResult<Config> {
    match Config::from_path(&config_path) {
        Ok(config) => Ok(config),
        Err(e) => {
            match e.kind() {
                &ErrorKind::IO { .. } => {
                    println!("Unable to load config file at {}.", &config_path.display());
                    println!("Attempting to create default config file.");
                    Config::make_default_config(&config_path)
                }
                _ => Err(e)
            }
        }
    }
}

fn parse_tags(tags: Option<&str>) -> Option<Vec<String>> {
    match tags {
        Some(tags_string) => Some(
            tags_string
                .split(",")
                .map(|t| t.into())
                .collect()),
        None => None
    }
}

fn main() -> AppResult<()> {
    let yaml = load_yaml!("clap.yml");
    let matches = App::from_yaml(yaml).get_matches();

    let config_path = match matches.value_of("config") {
        Some(path) => PathBuf::from(path),
        None => Config::default_config_path()
    };
    let config = load_or_create_config(config_path)?;

    let mut conn = Connection::open(config.data_dir.join("faramir.db"))?;
    db::init_db(&conn)?;

    match matches.subcommand() {
        ("start", Some(sub_matches)) => {
            let project = sub_matches.value_of("project").unwrap().into();
            let tag_str = sub_matches.value_of("tags");

            timer_start(&mut conn, &config, project, tag_str)
        },
        ("status", Some(sub_matches)) => {
            timer_status(&conn, &config, sub_matches)
        },
        ("stop", Some(sub_matches)) => {
            timer_stop(&conn, sub_matches)
        },
        ("ls", Some(sub_matches)) => {
            timer_ls(&conn, sub_matches)
        },
        ("history", Some(sub_matches)) => {
            timer_history(&conn, sub_matches)
        },
        ("rename", Some(sub_matches)) => {
            timer_rename(&conn, sub_matches)
        },
        ("", None) => Err(AppError::from_str("A subcommand must be provided.")),
        _ => Err(AppError::from_str("A subcommand must be provided."))
    }
}

fn timer_status(conn: &Connection, config: &Config, sub_matches: &ArgMatches) -> AppResult<()> {
    let timers = Timers::currently_running(&conn)?;

    if timers.len() == 0 {
        println!("No timers are running.");
        return Ok(())
    }

    println!("{} timer(s) found.", timers.len());
    for timer in timers.0 {
        let project = Project::for_timer(&conn, timer.id)?;
        println!("timer for project {} - with id {}", project.name, timer.rid);
        timer.pretty_print(&config, sub_matches.is_present("detailed"));
    }

    Ok(())
}

fn timer_start(
    conn: &mut Connection,
    config: &Config,
    project: &str,
    tag_str: Option<&str>,
) -> AppResult<()> {

    //1. create project
    //2. create tags
    //3. create timer
    //4. create projects_timers associations
    //5. create tags_timers associations

    println!("inserting project");
    let project_id = Project::insert_and_get_id(&conn, project)?;
    let tags = parse_tags(tag_str);

    println!("inserting tags");
    let tag_ids = match tags {
        Some(tags) => Some(Tag::batch_insert(conn, tags)?),
        None => None
    };

    println!("inserting timer");
    let create_timer = CreateTimer::new();
    let timer_id = create_timer.insert_and_get_id(&conn)?;

    println!("inserting projects_timers");
    conn.execute(
        "INSERT OR IGNORE INTO projects_timers (project_id, timer_id) VALUES (?1, ?2)",
        params![project_id, timer_id]
    )?;

    println!("inserting tags_timers");
    if let Some(tag_ids) = tag_ids {
        let tx = conn.transaction()?;
        for tag_id in tag_ids {
            tx.execute("INSERT OR IGNORE INTO tags_timers (timer_id, tag_id) VALUES (?1, ?2)", &[timer_id, tag_id])?;
        }
        tx.commit()?;
    }

    Ok(())
}

fn timer_stop(conn: &Connection, sub_matches: &ArgMatches) -> AppResult<()> {
    let mut current_timers = Timers::currently_running(&conn)?;

    match current_timers.len() {
        0 => {
            println!("No timers are running.");
            Ok(())
        },
        1 => {
            current_timers.0[0].stop(&conn)
        },
        _ => {
            if sub_matches.is_present("all") {
                return current_timers.stop_all(&conn)
            }

            match sub_matches.value_of("id") {
                Some(rid) => {
                    current_timers.0.retain(|t| t.rid == rid);
                    match current_timers.0.first_mut() {
                        Some(timer) => {
                            timer.stop(&conn)
                        },
                        None => {
                            println!("No currently running timer has that id.");
                            Ok(())
                        }
                    }
                },
                None => {
                    println!("Multiple timers are running. Specify a timer with -i <id>.");
                    Ok(())
                }
            }
        }
    }

}

fn ls_projects(conn: &Connection, sub_matches: &ArgMatches) -> AppResult<()> {
    let projects = Projects::all(&conn)?;
    if projects.len() == 0 {
        println!("No projects found.");
        println!("Projects are automatically created when you start a timer:");
        println!("  faramir start project1 -t tag1,tag2");
        return Ok(())
    }

    match sub_matches.is_present("detailed") {
        true => projects.print_detailed(&conn)?,
        false => projects.print_basic(),
    }

    Ok(())
}

fn ls_tags(conn: &Connection, sub_matches: &ArgMatches) -> AppResult<()> {
    //TODO detailed
    let tags = Tags::all(&conn)?;
    if tags.len() == 0 {
        println!("No projects found.");
        println!("Tags are automatically created when you start a timer:");
        println!("  faramir start project1 -t tag1,tag2");
        return Ok(())
    }

    println!("{} Tag(s) found.", tags.len());
    println!("tags: {}", tags.names().join(", "));
    Ok(())
}

// type.unwrap() is fine because clap handles it if it's not provided
fn timer_ls(conn: &Connection, sub_matches: &ArgMatches) -> AppResult<()> {
    match sub_matches.value_of("type").unwrap() {
        "p" | "project" | "projects" => ls_projects(&conn, &sub_matches),
        "ta" | "tag" | "tags" => ls_tags(&conn, &sub_matches),
        "t" | "timer" | "timers" => Ok(()),
        _ => {
            println!("Type not recognized. Run `faramir ls --help` for possible values.");
            Err(AppError::from_str("Type not recognized for `ls` subcommand.".into()))
        }
    }
}

fn rename_project(conn: &Connection, old_name: &str, new_name: &str) -> AppResult<()> {
    if let Ok(project) = Project::find_by_name(&conn, &old_name) {
        project.update(&conn, new_name)?;
        println!("Successfully renamed project {} to {}.", &old_name, &new_name);
    } else {
        println!("Unable to find project with name {}.", &old_name);
    }

    Ok(())
}

fn rename_tag(conn: &Connection, old_name: &str, new_name: &str) -> AppResult<()> {
    if let Ok(tag) = Tag::find_by_name(&conn, &old_name) {
        tag.update(&conn, new_name)?;
        println!("Successfully renamed tag {} to {}.", &old_name, &new_name);
    } else {
        println!("Unable to find tag with name {}.", &old_name);
    }

    Ok(())
}

fn timer_rename(conn: &Connection, sub_matches: &ArgMatches) -> AppResult<()> {
    let old_name = sub_matches.value_of("old-name").unwrap();
    let new_name = sub_matches.value_of("new-name").unwrap();
    match sub_matches.value_of("type").unwrap() {
        "p" | "project" | "projects" => rename_project(&conn, old_name, new_name),
        "ta" | "tag" | "tags" => rename_tag(&conn, old_name, new_name),
        _ => {
            println!("Type not recognized. Run `faramir rename --help` for possible values.");
            Err(AppError::from_str("Type not recognized for `rename` subcommand.".into()))
        }
    }
}


fn timer_history(conn: &Connection, sub_matches: &ArgMatches) -> AppResult<()> {
    let limit = match sub_matches.value_of("limit") {
        Some(l) => l,
        None => "10"
    };

    let timers = Timers::limit(&conn, limit)?;

    println!("{} timer(s) retrieved.", timers.len());
    for timer in timers.0 {
        println!("{} - start: {}, end: {}", timer.rid, timer.start, timer.end.unwrap());
    }

    Ok(())
}
