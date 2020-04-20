use crate::{state::Repo, Result};
use serde::Deserialize;
use std::{fs, path::Path};

#[derive(Debug, Deserialize)]
pub struct Config {
    pub repo: Repo,
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

    pub fn repo(&self) -> &Repo {
        &self.repo
    }
}
