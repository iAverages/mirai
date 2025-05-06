use rand::Rng;
use std::path::PathBuf;

use thiserror::Error;

use crate::backends::WallpaperBackend;
use crate::content_managers::ContentManagerTypes;
use crate::get_config;
use crate::store::Store;

pub struct WallpapersManager<'a> {
    store: &'a Store,
    backend: Box<dyn WallpaperBackend>,
}

pub trait WallpaperContentManager {
    fn get_wallpapers(&self) -> Vec<Wallpaper>;
}

impl<'a> WallpapersManager<'a> {
    pub fn new<T: WallpaperBackend + 'static>(
        store: &'a Store,
        backend: T,
    ) -> WallpapersManager<'a> {
        WallpapersManager {
            store,
            backend: Box::new(backend),
        }
    }

    pub fn store_wallpapers(
        &self,
        content_manager: &impl WallpaperContentManager,
    ) -> Result<(), WallpapersMangerError> {
        let wallpapers = content_manager.get_wallpapers();
        for wallpaper in wallpapers {
            tracing::trace!(
                "inserting wallpaper {} to store",
                wallpaper.get_wallpaper_path().display()
            );
            self.store
                .insert_wallpaper(&wallpaper)
                .map_err(|_| WallpapersMangerError::DatabaseInsertError)?;
        }

        Ok(())
    }

    pub fn set_next_wallpaper(&self) {
        tracing::debug!("setting next wallpaper");
        let mut unseen_wallpapers = self.store.get_unseen_wallpaperrs();
        tracing::debug!("{} unseen wallpapers", unseen_wallpapers.len());

        if unseen_wallpapers.is_empty() {
            tracing::info!("all wallpapers have been seen, resetting seen state");
            self.store.reset_seen_state();
            unseen_wallpapers = self.store.get_unseen_wallpaperrs();
        }

        if unseen_wallpapers.is_empty() {
            tracing::info!("no wallpapers found in internal store");
            return;
        }

        let mut rng = rand::rng();
        let random_index = rng.random_range(0..unseen_wallpapers.len());

        let next_wallpaper_db = &unseen_wallpapers[random_index];
        let next_wallpaper: Wallpaper = next_wallpaper_db
            .clone()
            .try_into()
            .expect("database has unsupported manager id. this is a bug");
        // TODO: handle errors
        self.backend.set_wallpaper(&next_wallpaper).unwrap();
        self.store.mark_as_seen(&next_wallpaper).unwrap();
        self.store.update_last_run();
    }
}

#[derive(Debug, Error)]
pub enum WallpapersMangerError {
    #[error("failed to add wallpaper to internal database")]
    DatabaseInsertError,
}

pub struct Wallpaper {
    pub id: String,
    pub type_id: ContentManagerTypes,
}

impl Wallpaper {
    pub fn new(id: String, type_id: ContentManagerTypes) -> Wallpaper {
        Wallpaper { id, type_id }
    }

    pub fn get_wallpaper_path(&self) -> PathBuf {
        let config = get_config();
        let wallpaper_path = PathBuf::from(config.file_config.local.location.clone());
        wallpaper_path.join(self.id.clone())
    }
}
