use crate::{state::Repo, Result};
use serde::Deserialize;
use std::{collections::HashMap, fs, path::Path};

#[derive(Debug, Deserialize)]
pub struct Config {
    pub repo: RepoConfig,
    pub secret: Option<String>,
    pub github_api_token: String,
}

impl Config {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let contents = fs::read_to_string(path)?;
        Ok(toml::from_str(&contents)?)
    }

    pub fn secret(&self) -> Option<&[u8]> {
        self.secret.as_ref().map(String::as_bytes)
    }

    pub fn repo(&self) -> &RepoConfig {
        &self.repo
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct RepoConfig {
    /// The repo this config pertains to: (Owner, Name)
    #[serde(flatten)]
    repo: Repo,

    /// Indicates if authors are allowed to self-review PRs
    #[serde(default)]
    allow_self_review: bool,

    /// Indicates if bors should use maintainer_mode and push directly to the PR
    #[serde(default)]
    maintainer_mode: bool,

    /// Set of commit checks that must have succeeded in order to merge a PR
    #[serde(default)]
    checks: HashMap<String, ChecksConfig>,
}

impl RepoConfig {
    pub fn owner(&self) -> &str {
        self.repo.owner()
    }

    pub fn name(&self) -> &str {
        &self.repo.name()
    }

    pub fn allow_self_review(&self) -> bool {
        self.allow_self_review
    }
}

#[derive(Debug, Deserialize)]
pub struct ChecksConfig {
    name: String,
}
