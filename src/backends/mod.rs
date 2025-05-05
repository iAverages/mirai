pub mod swww_cli;

use thiserror::Error;

use crate::wallpaper::Wallpaper;

#[derive(Error, Debug)]
pub enum WallpaperBackendError {
    #[error("failed to change wallpaper")]
    ChangeFailure,
}

pub trait WallpaperBackend {
    fn set_wallpaper(&self, wallpaper: &Wallpaper) -> Result<(), WallpaperBackendError>;
}
