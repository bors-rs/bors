//! Error type for Github Client

use serde::Deserialize;
use std::{borrow::Cow, io, str};
use thiserror::Error;

pub type Result<T, E = Error> = std::result::Result<T, E>;

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

    #[error("`{}` `{:?}`", 0, 1)]
    GithubClientError(reqwest::StatusCode, GithubClientError),

    #[error("RateLimit")]
    RateLimit,

    #[error("AbuseLimit")]
    AbuseLimit,

    #[error("GraphqlError: {}", 0)]
    GraphqlError(Vec<GraphqlError>),
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

// Github Error Responses
// https://developer.github.com/v3/#client-errors
#[derive(Debug, Deserialize)]
pub struct GithubClientError {
    message: Option<String>,
    errors: Option<Vec<GithubClientErrorType>>,
    documentation_url: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum GithubClientErrorType {
    Message(String),
    Code {
        resourse: String,
        field: String,
        code: String,
    },
}

#[derive(Debug, Deserialize)]
pub struct GraphqlErrorLocation {
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Deserialize)]
pub struct GraphqlError {
    pub message: String,
    pub locations: Option<Vec<GraphqlErrorLocation>>,
}
