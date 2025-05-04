use std::path::Path;
use std::{env, fs};

use directories::ProjectDirs;
use serde::Deserialize;

const CONFIG_NAME: &str = "mirai.toml";

#[derive(Debug)]
pub struct Config {
    data_dir: String,
    file_config: FileConfig,
}

#[derive(Debug, Deserialize)]
struct FileConfig {
    pub wallpapers_dir: String,
}

impl Default for FileConfig {
    fn default() -> Self {
        FileConfig {
            wallpapers_dir: "".to_string(),
        }
    }
}

pub enum ConfigError {}

impl Config {
    pub fn create_config() -> Config {
        match ProjectDirs::from("dev", "kirsi", "mirai") {
            Some(proj_dirs) => Config::load_config(proj_dirs.config_dir(), proj_dirs.data_dir()),
            None => panic!("failed to create config"),
        }
    }

    fn load_config(config_dir: &Path, data_dir: &Path) -> Config {
        let config_str = fs::read_to_string(config_dir.join(CONFIG_NAME));

        let file_config: FileConfig = match config_str {
            Ok(contents) => toml::from_str(&contents).unwrap(),
            Err(_) => FileConfig::default(),
        };

        Config {
            data_dir: data_dir.to_str().unwrap().to_string(),
            file_config,
        }
    }
}
