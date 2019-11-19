use sled;
use std::{borrow::Cow, io};
use thiserror::Error;
use crate::config;

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
