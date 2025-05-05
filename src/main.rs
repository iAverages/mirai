mod backends;
mod config;
mod content_managers;
mod store;
mod wallpaper;

use self::backends::swww_cli::SwwCliBackend;
use self::config::Config;
use self::content_managers::local::LocalContentManager;
use self::store::Store;
use self::wallpaper::WallpapersManager;
use chrono::{DateTime, Datelike, Duration, Local, TimeZone};
use once_cell::sync::OnceCell;
use std::fs;
use std::path::PathBuf;
use std::thread::sleep;
use std::time::{Duration as StdDuration, SystemTime, UNIX_EPOCH};

static CONFIG: OnceCell<Config> = OnceCell::new();
pub fn get_config() -> &'static Config {
    CONFIG.get().expect("config is not yet initizlised")
}

fn main() -> Result<(), String> {
    let config = Config::create_config();
    let _ = CONFIG.set(config);

    let data_dir_path: PathBuf = get_config().data_dir.clone().into();
    fs::create_dir_all(data_dir_path).map_err(|err| err.to_string())?;

    let backend = SwwCliBackend::new();
    let store = Store::new().map_err(|err| err.to_string())?;
    let content_manager = LocalContentManager::new();
    let wallpaper_manager = WallpapersManager::new(&store, backend);
    wallpaper_manager
        .store_wallpapers(&content_manager)
        .map_err(|err| err.to_string())?;

    loop {
        let last_update = store.get_last_update();
        if should_update_wallpaper(get_config().file_config.local.update_interval, last_update) {
            wallpaper_manager.set_next_wallpaper();
        }
        sleep(StdDuration::from_secs(get_seconds_till_minute()));
    }
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
