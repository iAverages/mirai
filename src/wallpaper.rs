use rand::Rng;
use std::path::PathBuf;
use std::thread::sleep;
use std::time::Duration;

use thiserror::Error;

use crate::backends::WallpaperBackend;
use crate::content_managers::ContentManagerTypes;
use crate::content_managers::git::GitContentManager;
use crate::content_managers::local::LocalContentManager;
use crate::get_config;
use crate::store::Store;

pub enum ContentManager {
    Git(GitContentManager),
    Local(LocalContentManager),
}

impl WallpaperContentManager for ContentManager {
    fn get_wallpapers(&self) -> Vec<Wallpaper> {
        match self {
            ContentManager::Git(manager) => manager.get_wallpapers(),
            ContentManager::Local(manager) => manager.get_wallpapers(),
        }
    }
}

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
        self.store.set_last_used(&next_wallpaper);
        self.store.update_last_run();
    }

    pub fn set_last_wallpaper(&self) {
        let Some(meta) = self.store.get_meta() else {
            return;
        };
        let Some(db_wallpaper) = self.store.get_wallpaper(&meta.last_used) else {
            return;
        };

        let wallpaper = db_wallpaper
            .try_into()
            .expect("failed to get wallpaper from db wallpaper");
        let mut times = 0;
        loop {
            if self.backend.set_wallpaper(&wallpaper).is_ok() {
                return;
            };

            if times > 5 {
                panic!(
                    "failed to set wallpaper 5 times, please make sure your chosen backend is available",
                );
            }

            times += 1;
            tracing::warn!("failed to set wallpaper, retrying in 5 seconds");
            sleep(Duration::from_secs(5));
        }
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
        match self.type_id {
            ContentManagerTypes::Local => {
                let config = get_config();
                let wallpaper_path = PathBuf::from(config.file_config.local.location.clone());
                wallpaper_path.join(self.id.clone())
            }
            ContentManagerTypes::Git => GitContentManager::get_temp_file(&self.id),
        }
    }
}
