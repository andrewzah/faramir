#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_variables)]

use std::{io, path::{Path, PathBuf}, fs, fs::File, env};
use std::io::Write;

use clap::{Arg, App, load_yaml};
use termcolor::{BufferWriter, Color, ColorChoice, ColorSpec, StandardStream, WriteColor};
use rusqlite::{params, Connection};

mod db;
mod errors;
mod models;
mod utils;

use db::*;
use models::timer::{Timers, Timer};
use models::event::{Events, Event};
use models::config::Config;
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

fn timer_start(
    timers: Timers,
    project: String,
    tags: Option<Vec<String>>,
    config: &Config
) -> AppResult<()> {
    let timer = Timer::new(project, tags);

    //let msg = format!("Started timer at {}", timer.start.format(&config.full_time_format));
    //timers.append_timer(timer)?;
    //timers.write_file(&config.data_dir)?;
    //println!("{}", msg);

    Ok(())
}

fn timer_status(timers: Timers, config: &Config) -> AppResult<()> {
    let len = timers.0.len();
    let word = match len {
        1 => "timer",
        0 => {
            println!("No timers are running.");
            return Ok(())
        },
        _ => "timers"
    };
    //let mut stdout = StandardStream::stdout(ColorChoice::Always);
    //stdout.set_color(ColorSpec::new().set_fg(Some(Color::Green)))?;

    //let mut bufwtr = BufferWriter::stdout(ColorChoice::Always);
    //let mut buffer = bufwtr.buffer();

    //buffer.set_color(ColorSpec::new().set_fg(Some(Color::Green)))?;
    //writeln!(&mut buffer, "{}", format!("{} {} found:", len, word));

    let mut i = 1;
    for timer in timers.0 {
        timer.pretty_print(&config);
        i += 1;
    }

    //bufwtr.print(&buffer)?;

    Ok(())
}

fn main() -> AppResult<()> {
    let yaml = load_yaml!("clap.yml");
    let matches = App::from_yaml(yaml).get_matches();

    let config_path = match matches.value_of("config") {
        Some(path) => PathBuf::from(path),
        None => Config::default_config_path()
    };
    let config = load_or_create_config(config_path)?;

    let conn = Connection::open(config.data_dir.join("faramir.db"))?;
    db::init_db(conn)?;

    //let timers = Timers::load_or_create(&config.data_dir)?;
    //let events = Events::load_or_create(&config.data_dir)?;

    //match matches.subcommand() {
        //("start", Some(sub_matches)) => {
            //let project = sub_matches.value_of("project").unwrap();
            //timer_start(timers, project.into(), None, &config)
        //},
        //("status", Some(sub_matches)) => {
            //timer_status(timers, &config)
        //},
        //("", None) => Err(AppError::from_str("A subcommand must be provided.")),
        //_ => Err(AppError::from_str("A subcommand must be provided."))
    //}
    Ok(())
}
