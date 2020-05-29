use crate::{state::Repo, Result};
use serde::Deserialize;
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Deserialize)]
pub struct Config {
    pub git: GitConfig,
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

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct GitConfig {
    pub ssh_key_file: PathBuf,
    pub user: String,
    pub email: String,
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

    /// Set of commit statuses that must have succeeded in order to merge a PR
    #[serde(default)]
    status: HashMap<String, StatusConfig>,

    /// Timeout for tests in seconds
    timeout_seconds: Option<u64>,

    /// Labels
    #[serde(default)]
    labels: Labels,
}

impl RepoConfig {
    pub fn repo(&self) -> &Repo {
        &self.repo
    }

    pub fn owner(&self) -> &str {
        self.repo.owner()
    }

    pub fn name(&self) -> &str {
        &self.repo.name()
    }

    pub fn allow_self_review(&self) -> bool {
        self.allow_self_review
    }

    pub fn checks(&self) -> impl Iterator<Item = &str> {
        let checks = self.checks.iter().map(|(_app, check)| check.name.as_ref());
        let status = self
            .status
            .iter()
            .map(|(_app, status)| status.context.as_ref());

        checks.chain(status)
    }

    pub fn timeout(&self) -> ::std::time::Duration {
        const DEFAULT_TIMEOUT_SECONDS: u64 = 60 * 60 * 2; // 2 hours

        let seconds = self.timeout_seconds.unwrap_or(DEFAULT_TIMEOUT_SECONDS);
        ::std::time::Duration::from_secs(seconds)
    }

    pub fn labels(&self) -> &Labels {
        &self.labels
    }
}

#[derive(Debug, Deserialize)]
pub struct ChecksConfig {
    name: String,
}

#[derive(Debug, Deserialize)]
pub struct StatusConfig {
    context: String,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Labels {
    squash: Option<String>,
    high_priority: Option<String>,
}

impl Labels {
    pub fn squash(&self) -> &str {
        self.squash.as_deref().unwrap_or("bors-squash")
    }

    pub fn high_priority(&self) -> &str {
        self.high_priority
            .as_deref()
            .unwrap_or("bors-high-priority")
    }

    pub fn all(&self) -> impl Iterator<Item = &str> {
        use std::iter::once;
        once(self.squash()).chain(once(self.high_priority()))
    }
}
