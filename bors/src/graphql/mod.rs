//! GraphQL queries and mutations for Github's v4 GraphQL API
//!
//! `github-schema.graphql` is sourced from
//! [here](https://developer.github.com/v4/public_schema/)
//!
//! [Github's v4 API Explorer](https://developer.github.com/v4/explorer/)
//! [Github's v4 API Docs](https://developer.github.com/v4/)

use crate::{state::PullRequestState, Result};
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

    pub async fn open_pulls(&self, owner: &str, name: &str) -> Result<Vec<PullRequestState>> {
        use query::{
            list_pulls::{ResponseData, Variables},
            ListPulls,
        };

        let mut ret = Vec::new();
        let mut has_next_page = true;
        let mut cursor = None;

        while has_next_page {
            let q = ListPulls::build_query(Variables {
                owner: owner.to_owned(),
                name: name.to_owned(),
                cursor: cursor.clone(),
            });

            let response: ResponseData = self.0.graphql().query(&q).await?.into_inner();

            let pull_requests = if let Some(repo) = response.repository {
                repo.pull_requests
            } else {
                // Maybe error?
                break;
            };

            has_next_page = pull_requests.page_info.has_next_page;
            cursor = pull_requests.page_info.end_cursor;

            let pr_iter = pull_requests
                .nodes
                .into_iter()
                .flat_map(|nodes| nodes.into_iter().flat_map(|pr| pr.map(Into::into)));
            ret.extend(pr_iter);
        }

        Ok(ret)
    }
}

impl Deref for GithubClient {
    type Target = Client;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
