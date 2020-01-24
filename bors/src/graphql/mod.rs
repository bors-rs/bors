//! GraphQL queries and mutations for Github's v4 GraphQL API
//!
//! `github-schema.graphql` is sourced from
//! [here](https://developer.github.com/v4/public_schema/)
//!
//! [Github's v4 API Explorer](https://developer.github.com/v4/explorer/)
//! [Github's v4 API Docs](https://developer.github.com/v4/)

use crate::Result;
use github::{client::Response, Client, NodeId, ReactionType};
use graphql_client::GraphQLQuery;
use std::ops::Deref;

mod query;

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

    pub async fn add_reaction(&self, id: &NodeId, reaction: ReactionType) -> Result<()> {
        use query::{
            add_reaction::{ResponseData, Variables},
            AddReaction,
        };

        let q = AddReaction::build_query(Variables {
            id: id.id().to_owned(),
            reaction: reaction.into(),
        });

        let _: Response<ResponseData> = self.0.graphql().query(&q).await?;

        Ok(())
    }
}

impl Deref for GithubClient {
    type Target = Client;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
