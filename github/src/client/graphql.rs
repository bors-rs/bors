use crate::client::{Client, Error, Response, Result};
use graphql_client::{QueryBody, Response as GraphqlResponse};
use serde::{de::DeserializeOwned, Serialize};

/// `GraphqlClient` handles communication with the GitHub's GraphQL API.
///
/// GitHub API docs: https://developer.github.com/v4/
pub struct GraphqlClient<'a> {
    inner: &'a Client,
}

impl<'a> GraphqlClient<'a> {
    pub(super) fn new(client: &'a Client) -> Self {
        Self { inner: client }
    }

    /// Perform a Query against GitHub's GraphQL Endpoint
    pub async fn query<V: Serialize, R: DeserializeOwned>(
        &self,
        query: &QueryBody<V>,
    ) -> Result<Response<R>> {
        let response = self.inner.post("graphql").json(query).send().await?;
        let (pagination, rate_limit, response) = self
            .inner
            .json::<GraphqlResponse<R>>(response)
            .await?
            .into_parts();

        match (response.data, response.errors) {
            (Some(data), None) => Ok(Response::new(pagination, rate_limit, data)),
            (_, Some(errors)) => Err(Error::GraphqlError(errors)),
            // GraphQL endpoints should always retern something
            (None, None) => unreachable!(),
        }
    }
}
