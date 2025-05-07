use std::fs;
use std::io::BufRead;
use std::path::{Path, PathBuf};
use std::process::Command;

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
        let temp_repo_loc: PathBuf = get_config().data_dir.clone().into();
        let temp_repo_path = temp_repo_loc.join("temp-repo");
        fs::create_dir_all(temp_repo_path.clone()).unwrap();
        let repo_url = get_config().file_config.git.url.clone();
        let temp_repo = GitTempRepo {
            path: &temp_repo_path,
            repo_url: &repo_url,
        };
        temp_repo.clone_repo().unwrap();
        temp_repo.sparse_checkout_2().unwrap();

        let wallpapers_path = get_config().file_config.git.path.clone().unwrap();
        let a = temp_repo.ls_tree(wallpapers_path.as_str()).unwrap();
        println!("{:?}", a);
        todo!()
    }
}

struct GitTempRepo<'a> {
    path: &'a PathBuf,
    repo_url: &'a str,
}

impl GitTempRepo<'_> {
    pub fn clone_repo(&self) -> Result<(), ()> {
        let _ = Command::new("git")
            .args(["clone", "-n", "--depth=1", "--filter=tree:0", self.repo_url])
            .current_dir(self.path)
            .output()
            .map_err(|_| ());

        Ok(())
    }

    pub fn sparse_checkout(&self, file_path: &str) -> Result<(), ()> {
        Command::new("git")
            .args(["sparse-checkout", "set", "--no-cone", file_path])
            .current_dir(self.path)
            .output()
            .map_err(|_| ())?;

        Ok(())
    }

    pub fn sparse_checkout_2(&self) -> Result<(), ()> {
        Command::new("git")
            .args(["sparse-checkout", "init", "--cone"])
            .current_dir(self.path)
            .output()
            .map_err(|_| ())?;

        Ok(())
    }

    pub fn ls_tree(&self, path: &str) -> Result<Vec<String>, ()> {
        let rev_output = Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(self.path)
            .output()
            .map_err(|_| ())?
            .stdout;

        println!("{:?}", rev_output);

        let rev = String::from_utf8(rev_output).unwrap();
        println!("rev {:?}", rev);

        let output = Command::new("git")
            .args(["ls-tree", "-r", &rev, "--name-only"])
            .current_dir(self.path)
            .output()
            .map_err(|_| ())?;

        let out = output.stdout;
        println!("{:?}", out);
        Ok(out
            .lines()
            .map(|line| line.unwrap().replace(path, ""))
            .collect::<Vec<_>>())
    }

    pub fn checkout(&self) -> Result<(), ()> {
        Command::new("git")
            .args(["checkout"])
            .current_dir(self.path)
            .output()
            .map_err(|_| ())?;

        Ok(())
    }
}
