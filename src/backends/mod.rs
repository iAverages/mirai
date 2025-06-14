#[cfg(not(target_os = "windows"))]
pub mod swww_cli;
#[cfg(target_os = "windows")]
pub mod windows;

use thiserror::Error;

use crate::wallpaper::Wallpaper;

#[derive(Error, Debug)]
pub enum WallpaperBackendError {
    #[error("failed to change wallpaper")]
    ChangeFailure,
}

pub trait WallpaperBackend {
    fn set_wallpaper(&self, wallpaper: &Wallpaper) -> Result<(), WallpaperBackendError>;
    fn is_ready(&self) -> bool;
}
