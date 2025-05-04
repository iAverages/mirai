mod backends;
mod config;
mod content_managers;
mod store;
mod wallpaper;

use self::config::Config;
use self::content_managers::local::{LocalWallpaper, LocalWallpaperManager};
use self::store::Store;
use self::wallpaper::{Wallpaper, WallpaperContentManager, WallpapersManger};
use once_cell::sync::OnceCell;
use rand::Rng;
use std::error::Error;
use std::fs;
use std::path::Path;

static CONFIG: OnceCell<Config> = OnceCell::new();
pub fn get_config() -> &'static Config {
    CONFIG.get().expect("config is not yet initizlised")
}

fn main() -> Result<(), String> {
    let config = Config::create_config();
    let _ = CONFIG.set(config);

    let store = Store::new().map_err(|err| err.to_string())?;

    let content_manager = LocalWallpaperManager::new();
    let wallpaper_manager = WallpapersManger::new(&store);
    wallpaper_manager
        .store_wallpapers(&content_manager)
        .map_err(|err| err.to_string())?;

    let wallpapers = store.get_inserted_wallpapers();
    println!("{}", wallpapers.len());
    for wallpaper in wallpapers {
        println!("aa: {:?}", wallpaper);
    }

    Ok(())
}
}
