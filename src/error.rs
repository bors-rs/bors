use crate::config;
use sled;
use std::{borrow::Cow, io};
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Io error")]
    Io(#[from] io::Error),
    #[error("http error")]
    Http(#[from] hyper::http::Error),
    #[error("hyper error")]
    Hyper(#[from] hyper::Error),
    #[error("config error")]
    Config(#[from] config::ConfigError),
    #[error("database error")]
    Database(#[from] sled::Error),
    #[error("json error")]
    Json(#[from] serde_json::Error),
    #[error("`{0}`")]
    Message(Cow<'static, str>),
}

impl From<&'static str> for Error {
    fn from(error: &'static str) -> Self {
        Error::Message(error.into())
    }
}

impl From<String> for Error {
    fn from(error: String) -> Self {
        Error::Message(error.into())
    }
}
