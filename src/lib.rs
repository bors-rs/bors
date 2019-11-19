mod error;
pub mod github;
mod graphql;
mod service;

pub use error::{Error, Result};
pub use service::Service;
