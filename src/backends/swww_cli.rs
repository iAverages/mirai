use std::process::Command;

use crate::wallpaper::Wallpaper;

use super::{WallpaperBackend, WallpaperBackendError};

/// TEMP cli powered backend because I cannot get the socket to connect
pub struct SwwCliBackend;

impl SwwCliBackend {
    pub fn new() -> SwwCliBackend {
        tracing::info!("using swww-cli backend");
        SwwCliBackend {}
    }
}

impl WallpaperBackend for SwwCliBackend {
    fn set_wallpaper(&self, wallpaper: &Wallpaper) -> Result<(), WallpaperBackendError> {
        let wallpaper_path = &wallpaper
            .get_wallpaper_path()
            .map_err(|_| WallpaperBackendError::ChangeFailure)?;
        let wallpaper_path = wallpaper_path.to_str().unwrap();
        tracing::debug!("[swww-cli] setting wallpaper {}", wallpaper_path);
        let output = Command::new("swww")
            // TODO: setup config option for resize
            .args(["img", "--resize=fit", wallpaper_path])
            .output()
            .map_err(|_| WallpaperBackendError::ChangeFailure)?;

        if output.status.success() {
            tracing::debug!("[swww-cli] set wallpaper successfully");
            Ok(())
        } else {
            tracing::error!("[swww-cli] failed to set wallpaper");
            Err(WallpaperBackendError::ChangeFailure)
        }
    }
}
