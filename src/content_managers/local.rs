use std::fs;
use std::path::Path;

use crate::get_config;
use crate::wallpaper::{Wallpaper, WallpaperContentManager};

use super::ContentManagerTypes;

pub struct LocalWallpaperManager;

impl LocalWallpaperManager {
    pub fn new() -> LocalWallpaperManager {
        LocalWallpaperManager {}
    }
}

impl WallpaperContentManager for LocalWallpaperManager {
    fn get_wallpapers(&self) -> Vec<impl Wallpaper> {
        fs::read_dir(get_config().file_config.wallpapers_dir.clone())
            .unwrap()
            .filter_map(Result::ok)
            .filter(|entry| entry.file_type().map(|ft| ft.is_file()).unwrap_or(false))
            .map(|path| LocalWallpaper::new(path.path()))
            .collect::<Vec<_>>()
    }
}

pub struct LocalWallpaper<P: AsRef<Path>> {
    path: P,
}

impl<P: AsRef<Path>> LocalWallpaper<P> {
    pub fn new(path: P) -> LocalWallpaper<P> {
        LocalWallpaper { path }
    }
}

impl<P: AsRef<Path>> Wallpaper for LocalWallpaper<P> {
    fn get_wallpaper_on_disk(&self) -> &Path {
        self.path.as_ref()
    }

    fn get_id(&self) -> &str {
        // TODO: how can i handle this without failing? OsString? sounds annoying
        self.path.as_ref().to_str().expect("invalid utf-8")
    }

    fn get_type_id(&self) -> ContentManagerTypes {
        ContentManagerTypes::Local
    }
}
