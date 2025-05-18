use rand::Rng;
use rand::distr::Alphanumeric;
use std::fs;
use std::io::BufRead;
use std::path::PathBuf;
use std::process::{Command, Output};
use thiserror::Error;

use crate::content_managers::ContentManagerTypes;
use crate::get_config;
use crate::wallpaper::{Wallpaper, WallpaperContentManager, WallpaperContentManagerError};

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[derive(Debug)]
pub struct GitContentManager;

impl GitContentManager {
    pub fn new() -> GitContentManager {
        tracing::info!(
            "using git content manager {}",
            get_config().file_config.git.url.clone()
        );
        GitContentManager {}
    }
}

impl WallpaperContentManager for GitContentManager {
    fn get_wallpapers(&self) -> Result<Vec<Wallpaper>, WallpaperContentManagerError> {
        let temp_repo = GitTempRepo::new().map_err(|_| WallpaperContentManagerError::Failure)?;
        temp_repo
            .clone_repo()
            .map_err(|_| WallpaperContentManagerError::Failure)?;
        temp_repo
            .sparse_checkout_2()
            .map_err(|_| WallpaperContentManagerError::Failure)?;

        // TODO: improve config validation
        let wallpapers_path = get_config().file_config.git.path.clone().unwrap();
        let wallpapers = temp_repo
            .ls_tree(wallpapers_path.as_str())
            .unwrap_or_else(|err| {
                tracing::error!("error while settign wallpaper: {}", err);
                vec![]
            });

        tracing::info!("found {} wallpapers in git repo", wallpapers.len());

        Ok(wallpapers
            .iter()
            .map(|wallpaper| Wallpaper {
                id: wallpaper.to_string(),
                type_id: ContentManagerTypes::Git,
            })
            .collect::<Vec<_>>())
    }
}

impl GitContentManager {
    pub fn get_temp_file(id: &str) -> Result<PathBuf, ()> {
        let temp_repo = GitTempRepo::new().map_err(|_| ())?;
        let file_path = id;
        temp_repo.clone_repo().unwrap();
        temp_repo.sparse_checkout(file_path).unwrap();
        temp_repo.checkout().unwrap();
        let wallpaper_path: PathBuf = get_config().file_config.git.path.clone().unwrap().into();
        let mut path = temp_repo.path.clone();
        path.push(&wallpaper_path);
        path.push(id);
        Ok(path)
    }
}

#[derive(Debug, Error)]
pub enum GitTempRepoError {
    #[error("failed to checkout the repository")]
    CheckoutFailure,
    #[error("failed to clone the repository")]
    CloneFailure,
    #[error("failed to find head rev")]
    NoHeadRev,
    #[error("failed get files in head rev")]
    NoFiles,
    #[error("failed to create temp directory for repo: {0}")]
    IoError(String),
}

pub struct GitTempRepo {
    pub path: PathBuf,
    repo_url: String,
}

impl GitTempRepo {
    pub fn new() -> Result<GitTempRepo, GitTempRepoError> {
        let mut temp_repo_loc: PathBuf = get_config().data_dir.clone().into();
        temp_repo_loc.push("temp-repos");
        let temp_id: String = rand::rng()
            .sample_iter(&Alphanumeric)
            .take(10)
            .map(char::from)
            .collect();
        tracing::info!("creating new temp repo: {}", temp_id);
        temp_repo_loc.push(temp_id);
        fs::create_dir_all(&temp_repo_loc)
            .map_err(|err| GitTempRepoError::IoError(err.to_string()))?;
        let repo_url = get_config().file_config.git.url.clone();
        Ok(GitTempRepo {
            path: temp_repo_loc,
            repo_url,
        })
    }

    pub fn clone_repo(&self) -> Result<(), GitTempRepoError> {
        self.run(&format!(
            "git clone -n --depth=1 --filter=tree:0 {} .",
            self.repo_url
        ))
        .map_err(|_| GitTempRepoError::CloneFailure)?;

        Ok(())
    }

    pub fn sparse_checkout(&self, file_path: &str) -> Result<(), GitTempRepoError> {
        self.run(&format!("git sparse-checkout set --no-cone {}", file_path))
            .map_err(|_| GitTempRepoError::CheckoutFailure)?;
        Ok(())
    }

    pub fn sparse_checkout_2(&self) -> Result<(), GitTempRepoError> {
        self.run("git sparse-checkout init --cone")
            .map_err(|_| GitTempRepoError::CheckoutFailure)?;
        Ok(())
    }

    pub fn ls_tree(&self, path: &str) -> Result<Vec<String>, GitTempRepoError> {
        let rev_output = self
            .run("git rev-parse HEAD")
            .map_err(|_| GitTempRepoError::NoHeadRev)?;
        let rev_str =
            String::from_utf8(rev_output.stdout).map_err(|_| GitTempRepoError::NoHeadRev)?;
        let rev = rev_str.trim_end_matches("\n");
        tracing::debug!("using {} as HEAD rev", rev);

        let output = self
            .run(&format!("git ls-tree -r {} --name-only", &rev))
            .map_err(|_| GitTempRepoError::NoFiles)?;
        let files = output
            .stdout
            .lines()
            .filter_map(|file| {
                if let Ok(file) = file {
                    if file.starts_with(path) {
                        return Some(file.replace(path, "").trim_start_matches("/").to_string());
                    }
                }

                None
            })
            .collect::<Vec<_>>();

        Ok(files)
    }

    pub fn checkout(&self) -> Result<(), GitTempRepoError> {
        self.run("git checkout")
            .map_err(|_| GitTempRepoError::CheckoutFailure)?;
        Ok(())
    }

    fn run(&self, command: &str) -> Result<Output, ()> {
        let parts = command.split_whitespace().collect::<Vec<_>>();
        let cmd = parts[0];
        let args = &parts[1..parts.len()];

        let mut cmd = Command::new(cmd);
        cmd.args(args).current_dir(&self.path);

        #[cfg(windows)]
        {
            const CREATE_NO_WINDOW: u32 = 0x08000000;
            cmd.creation_flags(CREATE_NO_WINDOW);
        }

        let output = cmd.output().map_err(|_| ())?;

        let span = tracing::debug_span!("running command", command = command);
        tracing::trace!("status: {}", output.status);
        tracing::trace!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        tracing::trace!("stderr: {}", String::from_utf8_lossy(&output.stderr));
        drop(span);

        Ok(output)
    }
}
