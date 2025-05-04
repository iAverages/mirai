use std::path::Path;

use thiserror::Error;

use crate::content_managers::ContentManagerTypes;
use crate::store::Store;
use crate::wallpaper;

pub struct WallpapersManger<'a> {
    store: &'a Store,
}

pub trait WallpaperContentManager {
    fn get_wallpapers(&self) -> Vec<impl Wallpaper>;
}

impl WallpapersManger<'_> {
    pub fn new(store: &Store) -> WallpapersManger {
        WallpapersManger { store }
    }

    pub fn store_wallpapers(
        &self,
        content_manager: &impl WallpaperContentManager,
    ) -> Result<(), WallpapersMangerError> {
        let wallpapers = content_manager.get_wallpapers();
        for wallpaper in wallpapers {
            println!("{:?}", wallpaper.get_type_id());
            self.store
                .insert_wallpaper(&wallpaper)
                .map_err(|_| WallpapersMangerError::DatabaseInsertError)?;
        }

        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum WallpapersMangerError {
    #[error("failed to add wallpaper to internal database")]
    DatabaseInsertError,
}

pub trait Wallpaper {
    fn get_id(&self) -> &str;
    fn get_wallpaper_on_disk(&self) -> &Path;
    fn get_type_id(&self) -> ContentManagerTypes;
}
