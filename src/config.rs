use serde::Deserialize;
use std::fs;
use std::io;
use std::path::Path;
use thiserror::Error;
use toml;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("io error")]
    Io(#[from] io::Error),
    #[error("toml parsing error")]
    De(#[from] toml::de::Error),
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub database: String,
    pub secret: Option<String>,
}

impl Config {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let contents = fs::read_to_string(path)?;
        Ok(toml::from_str(&contents)?)
    }

    pub fn secret(&self) -> Option<&[u8]> {
        self.secret.as_ref().map(String::as_bytes)
    }
}
