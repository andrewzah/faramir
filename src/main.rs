use std::{env, fs, fs::File, io::BufWriter, path::PathBuf, process::Command};

use chrono::{offset::TimeZone, Utc};
use chrono_tz::Tz;
use clap::{load_yaml, App, ArgMatches, Shell};
use rusqlite::Connection;

mod db;
mod errors;
mod models;
mod utils;

use errors::{AppError, AppResult, ErrorKind};
use models::{
    config::Config,
    project::{Project, Projects},
    tag::{Tag, Tags},
    timer::{CreateTimer, Timer, Timers},
};

fn load_or_create_config(config_path: PathBuf) -> AppResult<Config> {
    match Config::from_path(&config_path) {
        Ok(config) => Ok(config),
        Err(e) => match e.kind() {
            &ErrorKind::IO { .. } => {
                println!(
                    "Unable to load config file at {}.",
                    &config_path.display()
                );
                println!("Attempting to create default config file.");
                Config::make_default_config(&config_path)
            },
            _ => Err(e),
        },
    }
}

fn main() -> AppResult<()> {
    let yaml = load_yaml!("clap.yml");
    let mut app = App::from_yaml(yaml);
    let matches = app.clone().get_matches();

    let config_path = match matches.value_of("config") {
        Some(path) => PathBuf::from(path),
        None => Config::default_config_path(),
    };
    let config = load_or_create_config(config_path)?;

    let mut conn = Connection::open(config.data_dir.join("faramir.db"))?;
    db::init_db(&conn)?;

    match matches.subcommand() {
        ("add", Some(sub_matches)) => {
            timer_add(&mut conn, &config, sub_matches)
        },
        ("completions", Some(sub_matches)) => {
            completions(&mut app, &config, sub_matches)
        },
        ("edit", Some(sub_matches)) => timer_edit(&conn, &config, sub_matches),
        ("log", Some(sub_matches)) => log(&conn, sub_matches),
        ("ls", Some(sub_matches)) => timer_ls(&conn, sub_matches),
        ("rename", Some(sub_matches)) => rename(&conn, sub_matches),
        ("rm", Some(sub_matches)) => rm(&mut conn, sub_matches),
        ("start", Some(sub_matches)) => timer_start(&mut conn, sub_matches),
        ("stats", Some(sub_matches)) => stats(&conn, sub_matches),
        ("status", Some(sub_matches)) => {
            timer_status(&conn, &config, sub_matches)
        },
        ("stop", Some(sub_matches)) => timer_stop(&conn, sub_matches),
        ("", None) => Err(AppError::from_str("A subcommand must be provided.")),
        _ => Err(AppError::from_str("A subcommand must be provided.")),
    }
}

fn stats(conn: &Connection, sub_matches: &ArgMatches) -> AppResult<()> {
    let projects = Projects::all(&conn)?;

    for project in projects.0 {
        let timers = Timers::for_project(&conn, project.id)?;
        println!("Project {} - {} timer(s) found.", project.name, timers.len());
        let total_seconds = timers.total_seconds();

        println!("total seconds: {}", total_seconds);
        println!("foramtted: {}", utils::format_seconds(total_seconds));
    }

    Ok(())
}

fn rm(conn: &mut Connection, sub_matches: &ArgMatches) -> AppResult<()> {
    let id = sub_matches.value_of("id").unwrap();
    let autoconfirm = sub_matches.is_present("yes");

    match sub_matches.value_of("type").unwrap() {
        "t" | "timer" | "timers" => db::delete_timer(&conn, id),
        "p" | "project" | "projects" => {
            db::delete_project(conn, id, autoconfirm)
        },
        "ta" | "tag" | "tags" => db::delete_tag(&conn, id, autoconfirm),
        _ => {
            println!(
                "Type not recognized. Run `faramir rename --help` for \
                 possible values."
            );
            Err(AppError::from_str(
                "Type not recognized for `rename` subcommand.".into(),
            ))
        },
    }
}

fn timer_add(
    conn: &mut Connection, config: &Config, sub_matches: &ArgMatches,
) -> AppResult<()> {
    let tz: Tz = config.timezone.parse()?;
    let project = sub_matches.value_of("project").unwrap();
    let tags = sub_matches.value_of("tags");
    let note = match sub_matches.value_of("note") {
        Some(note_str) => Some(note_str.into()),
        None => None,
    };

    if let Some(start_str) = sub_matches.value_of("start") {
        let end_str = match sub_matches.value_of("end") {
            Some(end_str) => end_str,
            None => {
                println!(
                    "If -s/--start is specified, -e/--end must also be \
                     specified."
                );
                return Ok(());
            },
        };

        let start_dt = tz.datetime_from_str(&start_str, &config.time_format)?;
        let start_utc = start_dt.with_timezone(&Utc);

        let end_dt = tz.datetime_from_str(&end_str, &config.time_format)?;
        let end_utc = end_dt.with_timezone(&Utc);

        let mut create_timer = CreateTimer::new(start_utc, Some(end_utc), note);

        if sub_matches.is_present("confirm") {
            let file_name =
                format!(".faramir-edit-{}.tmp.json", utils::rand_string(5));
            let tmp_file_path = &config.data_dir.join(file_name);

            let editor = match env::var("EDITOR") {
                Ok(editor) => editor,
                Err(_) => {
                    println!("Please set the EDITOR environment variable.");
                    return Ok(());
                },
            };

            let json = serde_json::to_string_pretty(&create_timer)?;
            fs::write(&tmp_file_path, &json)?;

            Command::new(editor)
                .arg(&tmp_file_path.to_str().unwrap())
                .status()?;

            let content = fs::read_to_string(&tmp_file_path)?;
            create_timer = match serde_json::from_str(&content) {
                Ok(timer) => timer,
                Err(e) => return Err(AppError::from(e)),
            };
        }

        db::handle_inserts(conn, project, tags, &create_timer)?;
        println!("Successfully added timer {}.", create_timer.rid);
    }

    Ok(())
}

fn timer_status(
    conn: &Connection, config: &Config, sub_matches: &ArgMatches,
) -> AppResult<()> {
    let timers = Timers::currently_running(&conn)?;

    if timers.len() == 0 {
        println!("No timers are running.");
        return Ok(());
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
    conn: &mut Connection, sub_matches: &ArgMatches,
) -> AppResult<()> {
    let note = match sub_matches.value_of("note") {
        Some(note_str) => Some(note_str.into()),
        None => None,
    };

    match sub_matches.is_present("keep") {
        true => {
            let last_timer = Timer::last(&conn)?;
            let project = Project::for_timer(&conn, last_timer.id)?;
            let tags = Tags::for_timer(&conn, last_timer.id)?;
            let tag_str = tags
                .0
                .into_iter()
                .map(|t| t.name)
                .collect::<Vec<String>>()
                .join(",");
            let mut create_timer = CreateTimer::default();
            create_timer.note = note;

            db::handle_inserts(
                conn,
                &project.name,
                Some(&tag_str),
                &create_timer,
            )?;
        },
        false => {
            let project = match sub_matches.value_of("project") {
                Some(p) => p,
                None => {
                    return Err(AppError::from_str(
                        "Please specify a project name.",
                    ));
                },
            };
            let tag_str = sub_matches.value_of("tags");

            let mut create_timer = CreateTimer::default();
            create_timer.note = note;

            db::handle_inserts(conn, project, tag_str, &create_timer)?;
            println!(
                "Successfully started timer {} for project {}.",
                create_timer.rid, project
            );
        },
    };

    Ok(())
}

fn timer_stop(conn: &Connection, sub_matches: &ArgMatches) -> AppResult<()> {
    let mut current_timers = Timers::currently_running(&conn)?;

    match current_timers.len() {
        0 => {
            println!("No timers are running.");
            Ok(())
        },
        1 => current_timers.0[0].stop(&conn),
        _ => {
            if sub_matches.is_present("all") {
                return current_timers.stop_all(&conn);
            }

            match sub_matches.value_of("id") {
                Some(rid) => {
                    current_timers.0.retain(|t| t.rid == rid);
                    match current_timers.0.first_mut() {
                        Some(timer) => timer.stop(&conn),
                        None => {
                            println!("No currently running timer has that id.");
                            Ok(())
                        },
                    }
                },
                None => {
                    println!(
                        "Multiple timers are running. Specify a timer with -i \
                         <id>."
                    );
                    Ok(())
                },
            }
        },
    }
}

fn ls_projects(conn: &Connection, sub_matches: &ArgMatches) -> AppResult<()> {
    let projects = Projects::all(&conn)?;
    if projects.len() == 0 {
        println!("No projects found.");
        println!("Projects are automatically created when you start a timer:");
        println!("  faramir start project1 -t tag1,tag2");
        return Ok(());
    }

    match sub_matches.is_present("detailed") {
        true => projects.print_detailed(&conn)?,
        false => projects.print_basic(),
    }

    Ok(())
}

fn ls_tags(conn: &Connection, _sub_matches: &ArgMatches) -> AppResult<()> {
    //TODO detailed
    let tags = Tags::all(&conn)?;
    if tags.len() == 0 {
        println!("No projects found.");
        println!("Tags are automatically created when you start a timer:");
        println!("  faramir start project1 -t tag1,tag2");
        return Ok(());
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
            println!(
                "Type not recognized. Run `faramir ls --help` for possible \
                 values."
            );
            Err(AppError::from_str(
                "Type not recognized for `ls` subcommand.".into(),
            ))
        },
    }
}

fn rename_project(
    conn: &Connection, old_name: &str, new_name: &str,
) -> AppResult<()> {
    if let Ok(project) = Project::find_by_name(&conn, &old_name) {
        project.update(&conn, new_name)?;
        println!(
            "Successfully renamed project {} to {}.",
            &old_name, &new_name
        );
    } else {
        println!("Unable to find project with name {}.", &old_name);
    }

    Ok(())
}

fn rename_tag(
    conn: &Connection, old_name: &str, new_name: &str,
) -> AppResult<()> {
    if let Ok(tag) = Tag::find_by_name(&conn, &old_name) {
        tag.update(&conn, new_name)?;
        println!("Successfully renamed tag {} to {}.", &old_name, &new_name);
    } else {
        println!("Unable to find tag with name {}.", &old_name);
    }

    Ok(())
}

fn rename(conn: &Connection, sub_matches: &ArgMatches) -> AppResult<()> {
    let old_name = sub_matches.value_of("old-name").unwrap();
    let new_name = sub_matches.value_of("new-name").unwrap();
    match sub_matches.value_of("type").unwrap() {
        "p" | "project" | "projects" => {
            rename_project(&conn, old_name, new_name)
        },
        "ta" | "tag" | "tags" => rename_tag(&conn, old_name, new_name),
        _ => {
            println!(
                "Type not recognized. Run `faramir rename --help` for \
                 possible values."
            );
            Err(AppError::from_str(
                "Type not recognized for `rename` subcommand.".into(),
            ))
        },
    }
}

fn log(conn: &Connection, sub_matches: &ArgMatches) -> AppResult<()> {
    let limit = match sub_matches.value_of("limit") {
        Some(l) => l,
        None => "10",
    };

    let timers = Timers::limit(&conn, limit)?;

    println!("{} timer(s) retrieved.", timers.len());
    for timer in timers.0 {
        println!(
            "{} - start: {}, end: {}",
            timer.rid,
            timer.start,
            timer.end.unwrap()
        );
    }

    Ok(())
}

fn completions(
    app: &mut App, config: &Config, sub_matches: &ArgMatches,
) -> AppResult<()> {
    let shell_name = sub_matches.value_of("shell").unwrap();
    let path = &config
        .data_dir
        .join(format!("faramir.{}-completion", &shell_name));

    let shell = match shell_name {
        "bash" => Shell::Bash,
        "fish" => Shell::Fish,
        "zsh" => Shell::Zsh,
        "powershell" => Shell::PowerShell,
        "elvish" => Shell::Elvish,
        _ => return Err(AppError::from_str("Unsupported shell.")),
    };

    let f = File::create(path)?;
    let mut f = BufWriter::new(f);

    app.gen_completions_to("faramir", shell, &mut f);

    Ok(())
}

fn timer_edit(
    conn: &Connection, config: &Config, sub_matches: &ArgMatches,
) -> AppResult<()> {
    let rid = sub_matches.value_of("id").unwrap();
    let file_name = format!(".faramir-edit-{}.tmp.json", utils::rand_string(5));
    let tmp_file_path = config.data_dir.join(file_name);

    let old_timer = match Timer::find_by(&conn, "rid", rid) {
        Ok(timer) => timer,
        Err(e) => return Err(e),
    };

    let json = serde_json::to_string_pretty(&old_timer)?;
    fs::write(&tmp_file_path, &json)?;

    let editor = match env::var("EDITOR") {
        Ok(editor) => editor,
        Err(_) => {
            println!("Please set the EDITOR environment variable.");
            return Ok(());
        },
    };

    Command::new(editor)
        .arg(&tmp_file_path.to_str().unwrap())
        .status()?;

    let content = fs::read_to_string(&tmp_file_path)?;
    let new_timer: Timer = match serde_json::from_str(&content) {
        Ok(timer) => timer,
        Err(e) => return Err(AppError::from(e)),
    };

    if old_timer.id != new_timer.id {
        println!("The IDs of the timers don't match.");
        return Ok(());
    }

    if old_timer.rid != new_timer.rid {
        println!("The RIDs of the timers don't match.");
        return Ok(());
    }

    new_timer.update(&conn)?;
    println!("Updated timer {}", new_timer.rid);
    fs::remove_file(tmp_file_path)?;
    Ok(())
}
