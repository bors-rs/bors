use std::{borrow::Cow, io, str};
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
    #[error("toml parsing error")]
    Toml(#[from] toml::de::Error),
    #[error("json error")]
    Json(#[from] serde_json::Error),
    #[error("utf8 error")]
    Utf8(#[from] str::Utf8Error),
    #[error("channel error")]
    Channel(#[from] futures::channel::mpsc::SendError),
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
