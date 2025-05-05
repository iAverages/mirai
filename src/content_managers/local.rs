use std::fs;

use crate::get_config;
use crate::wallpaper::{Wallpaper, WallpaperContentManager};

use super::ContentManagerTypes;

#[derive(Debug)]
pub struct LocalContentManager;

impl LocalContentManager {
    pub fn new() -> LocalContentManager {
        LocalContentManager {}
    }
}

impl WallpaperContentManager for LocalContentManager {
    fn get_wallpapers(&self) -> Vec<Wallpaper> {
        fs::read_dir(get_config().file_config.local.location.clone())
            .unwrap()
            .filter_map(Result::ok)
            .filter(|entry| entry.file_type().map(|ft| ft.is_file()).unwrap_or(false))
            .map(|path| {
                Wallpaper::new(
                    path.file_name().to_string_lossy().to_string(),
                    ContentManagerTypes::Local,
                )
            })
            .collect::<Vec<_>>()
    }
}
