#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

mod backends;
mod config;
mod content_managers;
mod store;
mod wallpaper;

#[cfg(not(target_os = "windows"))]
use self::backends::swww_cli::SwwCliBackend;

use self::backends::WallpaperBackend;
use self::config::{Config, LogLevel};
use self::content_managers::ContentManagerTypes;
use self::content_managers::git::GitContentManager;
use self::content_managers::local::LocalContentManager;
use self::store::Store;
use self::wallpaper::{ContentManager, WallpapersManager};
use chrono::{DateTime, Datelike, Local, TimeZone};
use once_cell::sync::OnceCell;
use std::fs;
use std::path::PathBuf;
use std::thread::sleep;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tracing::Level;

static CONFIG: OnceCell<Config> = OnceCell::new();
pub fn get_config() -> &'static Config {
    CONFIG.get().expect("config is not yet initizlised")
}

// main function for windows which has the cli for
// managing the service that is used on windows
#[cfg(target_os = "windows")]
fn main() -> Result<(), String> {
    use auto_launch::AutoLaunchBuilder;
    use clap::Parser;
    use std::env;

    #[derive(Parser, Debug)]
    #[command(version, about, long_about = None)]
    struct Args {
        /// Autostart mirai on boot
        #[arg(long, action)]
        autostart: Option<bool>,
    }

    let args = Args::parse();

    let auto = AutoLaunchBuilder::new()
        .set_app_name("mirai")
        .set_app_path(env::current_exe().unwrap().to_str().unwrap())
        .build()
        .unwrap();

    if let Some(autostart) = args.autostart {
        if autostart {
            auto.enable().map_err(|err| err.to_string())?;
            return Ok(());
        } else {
            auto.disable().map_err(|err| err.to_string())?;
            return Ok(());
        }
    }

    shared_main()
}

// main function for linux, does not have CLI as cli is used
// only to manage the windows service that is registered
#[cfg(not(target_os = "windows"))]
fn main() -> Result<(), String> {
    shared_main()
}

fn shared_main() -> Result<(), String> {
    let config = Config::create_config();
    let _ = CONFIG.set(config);

    let log_level = get_config().file_config.log_level;
    let log_level = log_level.unwrap_or(LogLevel(Level::INFO));
    tracing_subscriber::fmt()
        .with_max_level(log_level.inner())
        .init();

    tracing::info!("starting mirai");
    tracing::info!("using config at {}", get_config().data_dir);

    let data_dir_path: PathBuf = get_config().data_dir.clone().into();
    let data_dir_str = data_dir_path.to_str().unwrap_or("N/A");
    tracing::debug!("creating data directory {}", data_dir_str);
    fs::create_dir_all(data_dir_path).map_err(|err| err.to_string())?;

    let backend = get_backend();
    let store = Store::new().map_err(|err| err.to_string())?;
    let content_manager = get_content_manager();
    let wallpaper_manager = WallpapersManager::new(&store, backend);
    wallpaper_manager
        .store_wallpapers(&content_manager)
        .map_err(|err| err.to_string())?;

    // if wallpaper needs changing, dont set current wallpaper, handle next change in loop
    let last_update = store.get_last_update();
    if !should_update_wallpaper(get_config().file_config.local.update_interval, last_update) {
        wallpaper_manager.set_last_wallpaper();
    }

    loop {
        let last_update = store.get_last_update();
        if should_update_wallpaper(get_config().file_config.local.update_interval, last_update) {
            wallpaper_manager.set_next_wallpaper();
        }
        sleep(Duration::from_secs(get_seconds_till_minute()));
    }
}

fn get_content_manager() -> ContentManager {
    match get_config().file_config.content_manager_type {
        ContentManagerTypes::Git => ContentManager::Git(GitContentManager::new()),
        ContentManagerTypes::Local => ContentManager::Local(LocalContentManager::new()),
    }
}

#[cfg(target_os = "windows")]
fn get_backend() -> impl WallpaperBackend {
    use backends::windows::Windows;
    Windows::new()
}

#[cfg(not(target_os = "windows"))]
fn get_backend() -> impl WallpaperBackend {
    SwwCliBackend::new()
}

fn get_seconds_till_minute() -> u64 {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("failed to get duration_since epoch")
        .as_secs();
    let seconds = now % 60;
    60 - seconds
}

fn should_update_wallpaper(interval: u32, last_run_time: Option<DateTime<Local>>) -> bool {
    let current_time = Local::now();
    let today = Local
        .with_ymd_and_hms(
            current_time.year(),
            current_time.month(),
            current_time.day(),
            0,
            0,
            0,
        )
        .single()
        .expect("failed to get start of day");

    let total_mins_today = (current_time - today).num_minutes() as f64;
    let group = (total_mins_today / interval as f64).floor() as u32;

    let mut should_run = true;

    if let Some(last_run) = last_run_time {
        if last_run.date_naive() == current_time.date_naive() {
            let last_group_mins = (last_run - today).num_minutes() as f64;
            let last_group = (last_group_mins / interval as f64).floor() as u32;
            should_run = group != last_group
        }
    }

    should_run
}
