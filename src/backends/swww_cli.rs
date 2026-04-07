use std::process::Command;

use which::which;

use crate::log::Log;
use crate::log_debug;
use crate::log_error;
use crate::wallpaper::Wallpaper;

use super::{WallpaperBackend, WallpaperBackendError};

/// TEMP cli powered backend because I cannot get the socket to connect
pub struct SwwCliBackend {
    pub bin_name: &'static str,
}

impl SwwCliBackend {
    pub fn new() -> SwwCliBackend {
        let bin_name = which("swww").map_or("awww", |_| "swww");
        tracing::info!("using {bin_name}-cli backend");
        SwwCliBackend { bin_name }
    }
}

impl Log for SwwCliBackend {
    fn log_prefix(&self) -> String {
        format!("{}-cli", self.bin_name)
    }
}

impl WallpaperBackend for SwwCliBackend {
    fn set_wallpaper(&self, wallpaper: &Wallpaper) -> Result<(), WallpaperBackendError> {
        let wallpaper_path = &wallpaper
            .get_wallpaper_path()
            .map_err(|_| WallpaperBackendError::ChangeFailure)?;
        let wallpaper_path = wallpaper_path.to_str().unwrap();
        log_debug!(&self, "setting wallpaper {}", wallpaper_path);
        let output = Command::new(self.bin_name)
            // TODO: setup config option for resize
            .args(["img", "--resize=fit", wallpaper_path])
            .output()
            .map_err(|_| WallpaperBackendError::ChangeFailure)?;

        if output.status.success() {
            log_debug!(&self, "set wallpaper successfully");
            Ok(())
        } else {
            tracing::error!("[swww-cli] ");
            log_error!(&self, "failed to set wallpaper");
            Err(WallpaperBackendError::ChangeFailure)
        }
    }

    fn is_ready(&self) -> bool {
        let result = Command::new(self.bin_name).args(["query"]).output();
        match result {
            Ok(cmd) => cmd.status.success(),
            Err(_) => false,
        }
    }
}
