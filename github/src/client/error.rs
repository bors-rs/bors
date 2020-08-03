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

    #[error("`{0}` `{1:?}`")]
    GithubClientError(reqwest::StatusCode, GithubClientError),

    #[error("RateLimit")]
    RateLimit,

    #[error("AbuseLimit")]
    AbuseLimit,

    #[cfg(feature = "graphql")]
    #[error("GraphqlError: {0:?}")]
    GraphqlError(Vec<graphql_client::Error>),
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
        resource: String,
        field: String,
        code: String,
        message: Option<String>,
    },
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn error_type() {
        let json = r#"{
            "documentation_url": "https://developer.github.com/v3/projects/cards/#create-a-project-card",
            "errors": [
                {
                    "code": "unprocessable",
                    "field": "data",
                    "message": "Project already has the associated issue",
                    "resource": "ProjectCard"
                }
            ],
            "message": "Validation Failed"
        }"#;

        let _e: GithubClientError = serde_json::from_str(&json).unwrap();
    }
}
