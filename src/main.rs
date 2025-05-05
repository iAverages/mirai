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
use once_cell::sync::OnceCell;
use std::fs;
use std::path::PathBuf;
use std::thread::sleep;
use std::time::Duration;

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
        wallpaper_manager.set_next_wallpaper();
        // sleep(Duration::from_secs(60));
        sleep(Duration::from_secs(10));
    }
}
