#[cfg(not(test))]
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
#[cfg(not(test))]
use std::fs;
#[cfg(not(test))]
use std::path::Path;
use tracing::Level;

use crate::content_managers::ContentManagerTypes;

#[cfg(not(test))]
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
    pub log_level: Option<LogLevel>,
    pub content_manager_type: ContentManagerTypes,
    pub local: LocalWallpaperConfig,
    pub git: GitWallpaperConfig,
}

impl Default for FileConfig {
    fn default() -> Self {
        FileConfig {
            content_manager_type: ContentManagerTypes::Local,
            local: LocalWallpaperConfig::default(),
            log_level: None,
            git: GitWallpaperConfig::default(),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct LogLevel(pub Level);

impl LogLevel {
    pub fn inner(&self) -> Level {
        self.0
    }
}

impl Serialize for LogLevel {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let level_str = match self.0 {
            Level::TRACE => "trace",
            Level::DEBUG => "debug",
            Level::INFO => "info",
            Level::WARN => "warn",
            Level::ERROR => "error",
        };
        serializer.serialize_str(level_str)
    }
}

impl<'de> Deserialize<'de> for LogLevel {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.to_lowercase().as_str() {
            "trace" => Ok(LogLevel(Level::TRACE)),
            "debug" => Ok(LogLevel(Level::DEBUG)),
            "info" => Ok(LogLevel(Level::INFO)),
            "warn" => Ok(LogLevel(Level::WARN)),
            "error" => Ok(LogLevel(Level::ERROR)),
            _ => Err(serde::de::Error::custom(format!(
                "Unknown log level: {}",
                s
            ))),
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
    pub update_interval: u32,
}
impl Default for LocalWallpaperConfig {
    fn default() -> Self {
        LocalWallpaperConfig {
            location: "".to_string(),
            update_interval: 1440,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GitWallpaperConfig {
    pub url: String,
    pub path: Option<String>,
}

impl Default for GitWallpaperConfig {
    fn default() -> Self {
        GitWallpaperConfig {
            url: "".to_string(),
            path: Some("".to_string()),
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

#[cfg(not(test))]
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
