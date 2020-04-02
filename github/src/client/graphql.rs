use crate::client::{Client, Error, GraphqlError, Response, Result};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

// TODO revisit using this type to ensure that all GraphQL Queries have the correct fields
// /// Struct to represent a request to a GraphQL server
// #[derive(Debug, Serialize)]
// pub struct Query<V> {
//     /// The values for the variables. They must match those declared in the provided query.
//     pub variables: V,
//     /// The GraphQL query
//     pub query: &'static str,
//     /// The GraphQL operation name
//     #[serde(rename = "operationName")]
//     pub operation_name: &'static str,
// }

#[derive(Debug, Deserialize)]
struct GraphqlResponse<T> {
    data: Option<T>,
    errors: Option<Vec<GraphqlError>>,
}

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
    pub async fn query<Q: Serialize, R: DeserializeOwned>(&self, query: &Q) -> Result<Response<R>> {
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
