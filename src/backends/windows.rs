use crate::wallpaper::Wallpaper;
use std::ffi::OsStr;
use std::iter;
use std::os::windows::ffi::OsStrExt;
use winapi::ctypes::c_void;
use winapi::um::winuser::SPI_SETDESKWALLPAPER;
use winapi::um::winuser::SPIF_SENDCHANGE;
use winapi::um::winuser::SPIF_UPDATEINIFILE;
use winapi::um::winuser::SystemParametersInfoW;
use winreg::RegKey;
use winreg::enums::*;

use super::{WallpaperBackend, WallpaperBackendError};

pub struct Windows;

impl Windows {
    pub fn new() -> Windows {
        tracing::info!("using windows backend");
        Windows {}
    }
}

impl WallpaperBackend for Windows {
    fn set_wallpaper(&self, wallpaper: &Wallpaper) -> Result<(), WallpaperBackendError> {
        let wallpaper_path = &wallpaper
            .get_wallpaper_path()
            .map_err(|_| WallpaperBackendError::ChangeFailure)?;
        let wallpaper_path = wallpaper_path.to_str().unwrap();
        tracing::debug!("[windows] setting wallpaper {}", wallpaper_path);

        unsafe {
            let path = OsStr::new(wallpaper_path)
                .encode_wide()
                // append null byte
                .chain(iter::once(0))
                .collect::<Vec<u16>>();
            let successful = SystemParametersInfoW(
                SPI_SETDESKWALLPAPER,
                0,
                path.as_ptr() as *mut c_void,
                SPIF_UPDATEINIFILE | SPIF_SENDCHANGE,
            ) == 1;

            if !successful {
                return Err(WallpaperBackendError::ChangeFailure);
            }

            // TODO:add config option for mode
            let hkcu = RegKey::predef(HKEY_CURRENT_USER);
            let (desktop, _) = hkcu
                .create_subkey(r"Control Panel\Desktop")
                .map_err(|_| WallpaperBackendError::ChangeFailure)?;

            desktop
                .set_value("WallpaperStyle", &"6".to_string())
                .map_err(|_| WallpaperBackendError::ChangeFailure)?;

            Ok(())
        }
    }
}
