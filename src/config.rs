use std::fmt::Display;
use std::fs;
use std::path::Path;

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

use crate::content_managers::ContentManagerTypes;

const CONFIG_NAME: &str = "mirai.toml";

#[derive(Debug)]
pub struct Config {
    pub data_dir: String,
    pub file_config: FileConfig,
}

impl Display for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // TODO: output internal config options not within the config file
        self.file_config.fmt(f)
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FileConfig {
    pub content_manager_type: ContentManagerTypes,
    pub local: LocalWallpaperConfig,
}

impl Default for FileConfig {
    fn default() -> Self {
        FileConfig {
            content_manager_type: ContentManagerTypes::Local,
            local: LocalWallpaperConfig::default(),
        }
    }
}

impl Display for FileConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            toml::to_string(self).expect("failed to convert default config to toml")
        )
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LocalWallpaperConfig {
    pub location: String,
}
impl Default for LocalWallpaperConfig {
    fn default() -> Self {
        LocalWallpaperConfig {
            location: "".to_string(),
        }
    }
}

#[cfg(test)]
impl Config {
    pub fn create_config() -> Config {
        Config {
            data_dir: "/tmp/mirai".into(),
            file_config: FileConfig::default(),
        }
    }
}

#[cfg(not(test))]
impl Config {
    pub fn create_config() -> Config {
        match ProjectDirs::from("dev", "kirsi", "mirai") {
            Some(proj_dirs) => Config::load_config(proj_dirs.config_dir(), proj_dirs.data_dir()),
            None => panic!("failed to create config"),
        }
    }
}

impl Config {
    fn load_config(config_dir: &Path, data_dir: &Path) -> Config {
        let config_path = config_dir.join(CONFIG_NAME);
        let config_str = fs::read_to_string(&config_path);

        let file_config: FileConfig = match config_str {
            Ok(contents) => toml::from_str(&contents).unwrap(),
            Err(_) => {
                let defaults = FileConfig::default();
                fs::create_dir_all(config_dir).unwrap_or_else(|_| {
                    panic!(
                        "failed to create config directory: {}",
                        config_path.to_str().unwrap()
                    )
                });
                fs::write(&config_path, defaults.to_string()).unwrap_or_else(|_| {
                    panic!(
                        "failed to write default config to {}",
                        config_path.to_str().unwrap()
                    )
                });
                defaults
            }
        };

        Config {
            data_dir: data_dir.to_str().unwrap().to_string(),
            file_config,
        }
    }
}
