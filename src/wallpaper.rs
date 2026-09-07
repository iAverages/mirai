use rand::RngExt;
use std::fs;
use std::path::{Path, PathBuf};
use std::thread::sleep;
use std::time::Duration;

use thiserror::Error;

use crate::backends::WallpaperBackend;
use crate::content_managers::ContentManagerTypes;
use crate::content_managers::git::GitContentManager;
use crate::content_managers::local::LocalContentManager;
use crate::get_config;
use crate::store::Store;

#[derive(Debug, thiserror::Error)]
pub enum WallpaperContentManagerError {
    #[error("failed to get wallpapers")]
    Failure,
}

pub enum ContentManager {
    Git(GitContentManager),
    Local(LocalContentManager),
}

impl WallpaperContentManager for ContentManager {
    fn get_wallpapers(&self) -> Result<Vec<Wallpaper>, WallpaperContentManagerError> {
        match self {
            ContentManager::Git(manager) => manager.get_wallpapers(),
            ContentManager::Local(manager) => manager.get_wallpapers(),
        }
    }

    fn cleanup_wallpaper(&self, wallpaper: Wallpaper) -> bool {
        match self {
            ContentManager::Git(manager) => manager.cleanup_wallpaper(wallpaper),
            ContentManager::Local(manager) => manager.cleanup_wallpaper(wallpaper),
        }
    }
}

pub struct WallpapersManager<'a> {
    store: &'a Store,
    backend: Box<dyn WallpaperBackend>,
}

pub trait WallpaperContentManager {
    fn get_wallpapers(&self) -> Result<Vec<Wallpaper>, WallpaperContentManagerError>;
    // optional function to cleanup wallpapers when no longer needed
    fn cleanup_wallpaper(&self, wallpaper: Wallpaper) -> bool;
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
        let wallpapers = content_manager
            .get_wallpapers()
            .map_err(|_| WallpapersMangerError::GetWallpaperError)?;
        for wallpaper in wallpapers {
            tracing::trace!("inserting wallpaper {} to store", wallpaper.id,);
            self.store
                .insert_wallpaper(&wallpaper)
                .map_err(|_| WallpapersMangerError::DatabaseInsertError)?;
        }

        Ok(())
    }

    // [h:a] Materialize each selected wallpaper at <data_dir>/wallpaper without an extension, remove Git temporary repositories after transfer, pass that fixed path to both backends, restore it directly on startup, remove obsolete delayed cleanup across the content managers and main caller, and add a focused filesystem test for contents and source cleanup.
    pub fn set_next_wallpaper(&mut self, content_manager: &impl WallpaperContentManager) {
        tracing::info!("setting next wallpaper");
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

        let _ = self.set_wallpaper(&next_wallpaper);
        let _ = self.store.mark_as_seen(&next_wallpaper).inspect_err(|err| {
            tracing::error!("failed to mark wallpaper as seen: {}", err);
        });
        let current_wallpaper = self.get_current_wallpaper();
        self.store.set_last_used(&next_wallpaper);
        self.store.update_last_run();
        if let Some(wallpaper) = current_wallpaper {
            tracing::info!("cleaning up last used wallpaper");
            content_manager.cleanup_wallpaper(wallpaper);
        }
    }

    fn set_wallpaper(&self, wallpaper: &Wallpaper) -> Result<(), WallpaperContentManagerError> {
        let active_wallpaper_path = get_active_wallpaper_path();
        let temp_path = wallpaper
            .get_wallpaper_path()
            .map_err(|_| WallpaperContentManagerError::Failure)?;
        // we use copy here since we do not want to remove content from the local content manager
        let use_active_path = fs::copy(&temp_path, &active_wallpaper_path)
            .inspect_err(|err| tracing::error!("failed to move active wallpaper path: {}", err))
            .is_ok();

        self.backend
            // only use active if wallpaper was copied, system continues to work
            // if this copy fails, wallpaper is read from the true source
            .set_wallpaper(if use_active_path {
                None
            } else {
                Some(wallpaper)
            })
            .inspect_err(|err| {
                tracing::error!("failed to set wallpaper: {}", err);
            })
            .map_err(|_| WallpaperContentManagerError::Failure)?;

        Ok(())
    }

    pub fn set_last_wallpaper(&self) {
        let wallpaper = self.get_current_wallpaper();
        if wallpaper.is_none() {
            return;
        }
        let wallpaper = wallpaper.unwrap();

        let mut times = 0;
        loop {
            if self.set_wallpaper(&wallpaper).is_ok() {
                tracing::info!("set last wallpaper");
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

    pub fn get_current_wallpaper(&self) -> Option<Wallpaper> {
        let meta = self.store.get_meta()?;
        let db_wallpaper = self.store.get_wallpaper(&meta.last_used)?;

        let wallpaper: Wallpaper = db_wallpaper
            .try_into()
            .expect("failed to get wallpaper from db wallpaper");

        Some(wallpaper)
    }
}

#[derive(Debug, Error)]
pub enum WallpapersMangerError {
    #[error("failed to add wallpaper to internal database")]
    DatabaseInsertError,
    #[error("failed to get list of wallpapers")]
    GetWallpaperError,
}

#[derive(Clone)]
pub struct Wallpaper {
    pub id: String,
    pub type_id: ContentManagerTypes,
}

impl Wallpaper {
    pub fn new(id: String, type_id: ContentManagerTypes) -> Wallpaper {
        Wallpaper { id, type_id }
    }

    pub fn get_wallpaper_path(&self) -> Result<PathBuf, ()> {
        match self.type_id {
            ContentManagerTypes::Local => {
                let config = get_config();
                let wallpaper_path = PathBuf::from(config.file_config.local.path.clone());
                let meta = fs::metadata(&wallpaper_path).map_err(|_| ())?;
                if meta.len() == 0 {
                    tracing::error!("file has no bytes");
                    return Err(());
                }
                Ok(wallpaper_path.join(self.id.clone()))
            }
            ContentManagerTypes::Git => GitContentManager::get_temp_file(&self.id),
        }
    }
}

pub fn get_active_wallpaper_path() -> PathBuf {
    Path::new(&get_config().data_dir).join("wallpaper")
}
