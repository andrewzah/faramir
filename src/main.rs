use std::{io, path::{Path, PathBuf}, fs, fs::File, env};

use clap::{Arg, App, load_yaml};

mod config;
mod errors;
mod timer;

use timer::{Timers, Timer};
use config::Config;
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
    mut timers: Timers,
    project: String,
    tags: Option<Vec<String>>,
    config: &Config
) -> AppResult<()> {
    let timer = Timer::new(project, tags);

    let msg = format!("Started timer at {}", timer.start.format(&config.full_time_format));
    timers.append_timer(timer)?;
    timers.write_file(&config.data_dir)?;

    println!("{}", msg);

    Ok(())
}

fn timer_status(timers: Timers, config: &Config) -> AppResult<()> {
    timers.pretty_print(&config);

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

    let timers = Timers::load_or_create(&config.data_dir)?;

    match matches.subcommand() {
        ("start", Some(sub_matches)) => {
            let project = sub_matches.value_of("project").unwrap();
            timer_start(timers, project.into(), None, &config)
        },
        ("status", Some(sub_matches)) => {
            timer_status(timers, &config)
        },
        ("", None) => Err(AppError::from_str("A subcommand must be provided.")),
        _ => Err(AppError::from_str("A subcommand must be provided."))
    }
}
