//! Error type for Github Client

use std::{borrow::Cow, io, str};
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Io error")]
    Io(#[from] io::Error),

    #[error("reqwest error")]
    Reqwest(#[from] reqwest::Error),

    #[error("json error")]
    Json(#[from] serde_json::Error),

    #[error("`{0}`")]
    Message(Cow<'static, str>),

    #[error("RateLimit")]
    RateLimit,
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
