use std::{borrow::Cow, io, str};
use thiserror::Error;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Io error")]
    Io(#[from] io::Error),
    #[error("probot error")]
    Probot(#[from] probot::Error),
    #[error("toml parsing error")]
    Toml(#[from] toml::de::Error),
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
