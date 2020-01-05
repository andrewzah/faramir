use std::{path::{Path, PathBuf}, fs, env};

use serde::{Deserialize, Serialize};

use crate::errors::{AppResult, AppError};

#[derive(Deserialize, Serialize)]
pub struct Config {
    pub data_dir: PathBuf,
    pub time_format: String,
    pub full_time_format: String,
    pub timezone: String,
}

const FARAMIR_DIR: &str = "faramir-tt";

impl Default for Config {
    fn default() -> Self {
        let data_dir = Config::default_config_dir();
        let time_format = "%Y/%m/%d %H:%M:%S".into();
        let full_time_format = "%Y/%m/%d %H:%M:%.3f, Day %j, Week %U".into();
        let timezone = "America/New_York".into();

        Config {
            data_dir,
            time_format,
            full_time_format,
            timezone,
        }
    }
}

impl Config {
    pub fn default_config_dir() -> PathBuf {
        let config_path;

        if let Ok(xdg_path) = env::var("XDG_CONFIG_HOME") {
            config_path = PathBuf::from(&xdg_path);
        } else if let Ok(home_path) = env::var("HOME") {
            config_path = Path::new(&home_path).join(".config");
        } else {
            config_path = PathBuf::from(".");
        }

        return config_path.join(FARAMIR_DIR)
    }

    pub fn default_config_path() -> PathBuf {
        Config::default_config_dir().join("config.json")
    }

    pub fn make_default_config(path: &Path) -> AppResult<Config> {
        let parent = match path.parent() {
            Some(p) => p,
            None => return Err(AppError::from_str("Unable to get path parent"))
        };
        fs::create_dir_all(&parent)?;

        let config: Config = Default::default();
        let json = serde_json::to_string_pretty(&config)?;

        fs::write(path, &json)?;
        println!("File created at {}.", &path.display());

        Ok(config)
    }

    pub fn from_path(path: &Path) -> AppResult<Config> {
        let content = fs::read_to_string(path)?;
        match serde_json::from_str(&content) {
            Ok(config) => Ok(config),
            Err(e) => Err(AppError::from(e))
        }
    }
}
