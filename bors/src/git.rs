use crate::{config::GitConfig, state::Repo, Config, Result};
use anyhow::{anyhow, Context};
use log::info;
use std::{
    path::{Path, PathBuf},
    process::Command,
};

const REPOS_DIR: &str = "repos";

#[derive(Debug)]
pub struct GitRepository {
    directory: PathBuf,
    github_repo: Repo,
    git_config: GitConfig,
}

impl GitRepository {
    pub fn from_config(config: &Config) -> Result<Self> {
        let github_repo = config.repo().repo().clone();
        let git_config = config.git.clone();
        let mut directory = std::env::current_dir()?;
        directory.push(REPOS_DIR);
        directory.push(github_repo.owner());
        directory.push(github_repo.name());

        if !Git::is_git_repo(&directory)? {
            info!(
                "cloning '{}' to '{}'",
                github_repo.to_github_ssh_url(),
                directory.display()
            );
            Git::clone(&directory, &github_repo, &git_config.ssh_key_file)?;
        } else {
            info!("using existing on-disk repo at {}", directory.display());
        }

        if !Git::remote_matches_github_repo(&directory, &github_repo)? {
            return Err(anyhow!(
                "on-disk repo's 'origin' remote doesn't match config"
            ));
        }

        Ok(Self {
            directory,
            github_repo,
            git_config,
        })
    }
}

struct Git {
    inner: Command,
}

impl Git {
    fn new() -> Self {
        let mut inner = Command::new("git");

        // Stop git looking at directories above the `REPOS_DIR`
        inner.env(
            "GIT_CEILING_DIRECTORIES",
            std::env::current_dir().unwrap().join(REPOS_DIR),
        );

        Self { inner }
    }

    fn run(&mut self) -> Result<String> {
        let output = self.inner.output()?;

        if !output.status.success() {
            return Err(anyhow!(
                "failed to run git command:\n{}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        Ok(String::from_utf8(output.stdout)?)
    }

    pub fn is_git_repo(path: &Path) -> Result<bool> {
        if !path.exists() {
            return Ok(false);
        }

        let output = Self::new()
            .current_dir(path)
            .args(&["rev-parse", "--git-dir"])
            .output()
            .context("checking if a directory is a git repo")?;

        Ok(output.status.success())
    }

    pub fn remote_matches_github_repo(path: &Path, github_repo: &Repo) -> Result<bool> {
        let mut cmd = Self::new();
        cmd.current_dir(path).args(&["remote", "get-url", "origin"]);
        let output = cmd.run()?;

        Ok(output.trim() == github_repo.to_github_ssh_url())
    }

    fn git_ssh_command(ssh_key_file: &Path) -> String {
        format!("ssh -i {} -S none", ssh_key_file.display())
    }

    pub fn clone(path: &Path, github_repo: &Repo, ssh_key_file: &Path) -> Result<()> {
        let mut cmd = Self::new();
        cmd.env("GIT_SSH_COMMAND", Self::git_ssh_command(ssh_key_file))
            .arg("clone")
            .arg(github_repo.to_github_ssh_url())
            .arg(path);
        cmd.run()
            .with_context(|| format!("cloning {}", github_repo.to_github_ssh_url()))?;
        Ok(())
    }
}

impl std::ops::Deref for Git {
    type Target = Command;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl std::ops::DerefMut for Git {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
