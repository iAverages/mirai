use rand::Rng;
use rand::distr::Alphanumeric;
use std::fs;
use std::io::BufRead;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use crate::content_managers::ContentManagerTypes;
use crate::get_config;
use crate::wallpaper::{Wallpaper, WallpaperContentManager};

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
    fn get_wallpapers(&self) -> Vec<Wallpaper> {
        let temp_repo = GitTempRepo::new();
        temp_repo.clone_repo().unwrap();
        temp_repo.sparse_checkout_2().unwrap();

        let wallpapers_path = get_config().file_config.git.path.clone().unwrap();
        let wallpapers = temp_repo.ls_tree(wallpapers_path.as_str()).unwrap();

        tracing::info!("found {} wallpapers in git repo", wallpapers.len());

        wallpapers
            .iter()
            .map(|wallpaper| Wallpaper {
                id: wallpaper.to_string(),
                type_id: ContentManagerTypes::Git,
            })
            .collect::<Vec<_>>()
    }
}

impl GitContentManager {
    pub fn get_temp_file(id: &str) -> PathBuf {
        let temp_repo = GitTempRepo::new();
        let file_path = id;
        temp_repo.clone_repo().unwrap();
        temp_repo.sparse_checkout(file_path).unwrap();
        temp_repo.checkout().unwrap();
        let wallpaper_path: PathBuf = get_config().file_config.git.path.clone().unwrap().into();
        let mut path = temp_repo.path.clone();
        path.push(&wallpaper_path);
        path.push(id);
        path
    }
}

pub struct GitTempRepo {
    pub path: PathBuf,
    repo_url: String,
}

impl GitTempRepo {
    pub fn new() -> GitTempRepo {
        let mut temp_repo_loc: PathBuf = get_config().data_dir.clone().into();
        temp_repo_loc.push("temp-repos");
        let temp_id: String = rand::rng()
            .sample_iter(&Alphanumeric)
            .take(10)
            .map(char::from)
            .collect();
        temp_repo_loc.push(temp_id);
        fs::create_dir_all(&temp_repo_loc).unwrap();
        let repo_url = get_config().file_config.git.url.clone();
        GitTempRepo {
            path: temp_repo_loc,
            repo_url,
        }
    }

    pub fn clone_repo(&self) -> Result<(), ()> {
        self.run(&format!(
            "git clone -n --depth=1 --filter=tree:0 {} .",
            self.repo_url
        ))?;

        Ok(())
    }

    pub fn sparse_checkout(&self, file_path: &str) -> Result<(), ()> {
        self.run(&format!("git sparse-checkout set --no-cone {}", file_path))?;
        Ok(())
    }

    pub fn sparse_checkout_2(&self) -> Result<(), ()> {
        self.run("git sparse-checkout init --cone")?;
        Ok(())
    }

    pub fn ls_tree(&self, path: &str) -> Result<Vec<String>, ()> {
        let rev_output = self.run("git rev-parse HEAD")?;
        let rev_str = String::from_utf8(rev_output.stdout).unwrap();
        let rev = rev_str.trim_end_matches("\n");
        tracing::debug!("using {} as HEAD rev", rev);

        let output = self.run(&format!("git ls-tree -r {} --name-only", &rev))?;
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

    pub fn checkout(&self) -> Result<(), ()> {
        self.run("git checkout")?;
        Ok(())
    }

    fn run(&self, command: &str) -> Result<Output, ()> {
        let parts = command.split(" ").collect::<Vec<_>>();
        let cmd = parts[0];
        let args = &parts[1..parts.len()];
        let output = Command::new(cmd)
            .args(args)
            .current_dir(&self.path)
            .output()
            .map_err(|_| ())?;

        let span = tracing::debug_span!("running command", command = command);
        tracing::debug!("status: {}", output.status);
        tracing::debug!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        tracing::debug!("stderr: {}", String::from_utf8_lossy(&output.stderr));
        drop(span);
        Ok(output)
    }
}
