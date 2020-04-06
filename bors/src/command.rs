//! Defines commands which can be asked to be performed
use std::str::FromStr;
use thiserror::Error;

#[derive(Debug)]
pub struct Command {
    cmd: String,
}

impl Command {
    pub fn from_comment(c: &str) -> Option<Result<Self, ParseCommnadError>> {
        c.lines()
            .find(|line| line.starts_with("/"))
            .map(FromStr::from_str)
    }
}

#[derive(Error, Debug)]
#[error("invalid command")]
pub struct ParseCommnadError;

impl FromStr for Command {
    type Err = ParseCommnadError;

    // TODO extend this impl to handle more commands and parameters
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "/lgtm" | "/land" => Ok(Command { cmd: s.to_owned() }),
            _ => Err(ParseCommnadError),
        }
    }
}
