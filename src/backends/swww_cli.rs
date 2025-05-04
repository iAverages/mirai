use std::process::Command;

use crate::wallpaper::Wallpaper;

use super::{WallpaperBackend, WallpaperBackendError};

/// TEMP cli powered backend because I cannot get the socket to connect
pub struct SwwCliBackend;

impl WallpaperBackend for SwwCliBackend {
    fn set_wallpaper(&self, wallpaper: &Wallpaper) -> Result<(), WallpaperBackendError> {
        let output = Command::new("swww")
            .args(["img", "--resize=fit", wallpaper.path.as_ref()])
            .output()
            .map_err(|_| WallpaperBackendError::ChangeFailure)?;

        if output.status.success() {
            Ok(())
        } else {
            Err(WallpaperBackendError::ChangeFailure)
        }
    }
}
