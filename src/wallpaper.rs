use crate::store::Store;

pub struct WallpapersManger;

impl WallpapersManger {
    pub fn new() -> WallpapersManger {
        WallpapersManger {}
    }

    pub fn store_wallpapers(&self, store: &Store) {
        todo!()
    }
}

pub struct Wallpaper {
    pub path: String,
}
