use crate::client::{
    Client, Response, Result, HEADER_RATE_LIMIT, HEADER_RATE_REMAINING, HEADER_RATE_RESET,
};
use serde::Deserialize;

#[derive(Debug, Default, Deserialize)]
pub struct Rate {
    pub limit: usize,
    pub remaining: usize,
    pub reset: usize, //TODO fix this to be UTC epoch seconds
}

impl Rate {
    pub(super) fn from_headers(headers: &reqwest::header::HeaderMap) -> Self {
        let mut rate = Self::default();

        if let Some(limit) = headers
            .get(HEADER_RATE_LIMIT)
            .and_then(|h| h.to_str().ok())
            .and_then(|s| s.parse().ok())
        {
            rate.limit = limit;
        };

        if let Some(remaining) = headers
            .get(HEADER_RATE_REMAINING)
            .and_then(|h| h.to_str().ok())
            .and_then(|s| s.parse().ok())
        {
            rate.remaining = remaining;
        };

        if let Some(reset) = headers
            .get(HEADER_RATE_RESET)
            .and_then(|h| h.to_str().ok())
            .and_then(|s| s.parse().ok())
        {
            rate.reset = reset;
        };

        rate
    }
}

#[derive(Debug, Deserialize)]
pub struct RateLimits {
    // The rate limit for non-search API v3 requests. Unauthenticated
    // requests are limited to 60 per hour. Authenticated requests are
    // limited to 5,000 per hour.
    //
    // GitHub API docs: https://developer.github.com/v3/#rate-limiting
    core: Rate,

    // The rate limit for search API requests. Unauthenticated requests
    // are limited to 10 requests per minute. Authenticated requests are
    // limited to 30 per minute.
    //
    // GitHub API docs: https://developer.github.com/v3/search/#rate-limit
    search: Rate,

    // The rate limit for GraphQl API v4 requests. Authenticated requests
    // are limited to 5,000 points per hour. Note that 5,000 points per
    // hour is not the same as 5,000 calls per hour: the GraphQL API v4
    // and REST API v3 use different rate limits.
    //
    // GitHub API docs: https://developer.github.com/v4/guides/resource-limitations/
    graphql: Rate,

    integration_manifest: Rate,
}

#[derive(Debug, Deserialize)]
struct RateLimitResponse {
    resources: RateLimits,
}

/// `RateLimitClient` handles communication with the rate_limit related methods of the GitHub API.
///
/// GitHub API docs: https://developer.github.com/v3/rate_limit/
pub struct RateLimitClient<'a> {
    inner: &'a Client,
}

impl<'a> RateLimitClient<'a> {
    pub(super) fn new(client: &'a Client) -> Self {
        Self { inner: client }
    }

    /// Query this clients current rate limit status
    /// Note: Accessing this endpoint does not count against your REST API rate limit.
    ///
    /// GitHub API docs: https://developer.github.com/v3/rate_limit/
    pub async fn get(&self) -> Result<Response<RateLimits>> {
        let response = self.inner.get("rate_limit").send().await?;

        let (pagination, rate, rate_limit_response) = self
            .inner
            .json::<RateLimitResponse>(response)
            .await?
            .into_parts();

        Ok(Response::new(
            pagination,
            rate,
            rate_limit_response.resources,
        ))
    }
}

#[cfg(test)]
mod test {
    use super::{Rate, HEADER_RATE_LIMIT, HEADER_RATE_REMAINING, HEADER_RATE_RESET};
    use reqwest::header::HeaderMap;

    #[test]
    fn rate() {
        let mut headers = HeaderMap::new();
        headers.insert(HEADER_RATE_LIMIT, "60".parse().unwrap());
        headers.insert(HEADER_RATE_REMAINING, "56".parse().unwrap());
        headers.insert(HEADER_RATE_RESET, "1372700873".parse().unwrap());

        let r = Rate::from_headers(&headers);
        assert_eq!(r.limit, 60);
        assert_eq!(r.remaining, 56);
        assert_eq!(r.reset, 1372700873);
    }
}
