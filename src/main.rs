mod backends;
mod config;
mod store;
mod wallpaper;

use self::config::Config;
use self::store::Store;
use self::wallpaper::WallpapersManger;
use rand::Rng;
use std::error::Error;
use std::fs;
use std::path::Path;

fn main() -> Result<(), String> {
    let config = Config::create_config();
    let store = Store::new(&config).map_err(|err| err.to_string())?;
    let wallpaper_manager = WallpapersManger::new();
    wallpaper_manager.store_wallpapers(&store);

    Ok(())
}
}
