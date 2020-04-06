//! GraphQL queries and mutations for Github's v4 GraphQL API
//!
//! `github-schema.graphql` is sourced from
//! [here](https://github.com/octokit/graphql-schema/blob/master/schema.graphql)
//!
//! [Github's v4 API Explorer](https://developer.github.com/v4/explorer/)
//! [Github's v4 API Docs](https://developer.github.com/v4/)

use github::Client;
use std::ops::Deref;

const USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

#[derive(Debug)]
pub struct GithubClient(Client);

impl GithubClient {
    pub fn new(github_api_token: &str) -> Self {
        let client = Client::builder()
            .github_api_token(github_api_token)
            .user_agent(USER_AGENT)
            .build()
            .unwrap();
        Self(client)
    }
}

impl Deref for GithubClient {
    type Target = Client;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
