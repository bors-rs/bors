use crate::{state::Repo, Result};
use serde::Deserialize;
use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Deserialize)]
pub struct Config {
    pub github: GithubConfig,
    pub git: GitConfig,
    pub repo: Vec<RepoConfig>,
}

impl Config {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let contents = fs::read_to_string(path)?;
        Ok(toml::from_str(&contents)?)
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct GitConfig {
    pub ssh_key_file: PathBuf,
    pub user: String,
    pub email: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct GithubConfig {
    pub github_api_token: String,
    pub webhook_secret: Option<String>,
    // app_id
    // client_id = ""
    // client_secret = ""
}

impl GithubConfig {
    pub fn webhook_secret(&self) -> Option<&str> {
        self.webhook_secret.as_deref()
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct RepoConfig {
    /// The repo this config pertains to: (Owner, Name)
    #[serde(flatten)]
    repo: Repo,

    /// Indicates if an approving Github review is required
    #[serde(default)]
    require_review: bool,

    /// Indicates if bors should use maintainer_mode and push directly to the PR
    #[serde(default)]
    maintainer_mode: bool,

    /// Set of checks, statuses, or workflows that must have succeeded in order to merge a PR
    #[serde(default)]
    checks: Vec<String>,

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

    pub fn require_review(&self) -> bool {
        self.require_review
    }

    pub fn maintainer_mode(&self) -> bool {
        self.maintainer_mode
    }

    pub fn checks(&self) -> impl Iterator<Item = &str> {
        self.checks.iter().map(AsRef::as_ref)
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

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Labels {
    squash: Option<String>,
    high_priority: Option<String>,
    low_priority: Option<String>,
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

    pub fn low_priority(&self) -> &str {
        self.low_priority.as_deref().unwrap_or("bors-low-priority")
    }

    pub fn all(&self) -> impl Iterator<Item = &str> {
        use std::iter::once;
        once(self.squash())
            .chain(once(self.high_priority()))
            .chain(once(self.low_priority()))
    }
}
