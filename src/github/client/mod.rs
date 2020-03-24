#![allow(dead_code)]

use log::debug;
use reqwest::{header, Client as ReqwestClient, Method, RequestBuilder};

mod error;
mod license;

pub use error::{Error, Result};
pub use license::LicenseClient;

// Constants
const DEFAULT_BASE_URL: &str = "https://api.github.com/";
const UPLOAD_BASE_URL: &str = "https://uploads.github.com/";
const USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

const HEADER_RATE_LIMIT: &str = "X-RateLimit-Limit";
const HEADER_RATE_REMAINING: &str = "X-RateLimit-Remaining";
const HEADER_RATE_RESET: &str = "X-RateLimit-Reset";
const HEADER_OTP: &str = "X-GitHub-OTP";

#[derive(Debug)]
pub struct ClientBuilder {
    base_url: Option<String>,
    user_agent: Option<String>,
    github_api_token: Option<String>,
}

impl ClientBuilder {
    pub fn new() -> Self {
        Self {
            base_url: None,
            user_agent: None,
            github_api_token: None,
        }
    }

    pub fn base_url<S: Into<String>>(mut self, base_url: S) -> Self {
        self.base_url = Some(base_url.into());
        self
    }

    pub fn user_agent<S: Into<String>>(mut self, user_agent: S) -> Self {
        self.user_agent = Some(user_agent.into());
        self
    }

    pub fn github_api_token<S: Into<String>>(mut self, github_api_token: S) -> Self {
        self.github_api_token = Some(github_api_token.into());
        self
    }

    pub fn build(self) -> Result<Client> {
        let base_url = self.base_url.unwrap_or_else(|| DEFAULT_BASE_URL.to_owned());
        let user_agent = self.user_agent.unwrap_or_else(|| USER_AGENT.to_owned());

        let mut client_builder = ReqwestClient::builder().user_agent(&user_agent);

        if let Some(token) = &self.github_api_token {
            let mut headers = header::HeaderMap::new();
            headers.insert(
                header::AUTHORIZATION,
                header::HeaderValue::from_str(&format!("token {}", token))
                    .map_err(|e| e.to_string())?,
            );
            client_builder = client_builder.default_headers(headers);
        }

        let client = client_builder.build()?;

        Ok(Client {
            base_url,
            user_agent,
            github_api_token: self.github_api_token,
            client,
        })
    }
}

impl Default for ClientBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct Client {
    /// Base URL to use for API requests. Defaults to the public GitHub API,
    /// but can be overridden for use with GitHub Enterprise. Must always be
    /// terminated with a trailing slash.
    base_url: String,

    /// User agent string sent when communicating with GitHub APIs
    #[allow(unused)]
    user_agent: String,

    /// API token to use when issuing requests to GitHub
    #[allow(unused)]
    github_api_token: Option<String>,

    /// Client used to make http requests
    client: ReqwestClient,
}

impl Client {
    pub fn new() -> Self {
        ClientBuilder::new().build().unwrap()
    }

    pub fn builder() -> ClientBuilder {
        ClientBuilder::new()
    }

    fn get(&self, url: &str) -> RequestBuilder {
        self.request(Method::GET, url)
    }

    fn post(&self, url: &str) -> RequestBuilder {
        self.request(Method::POST, url)
    }

    fn request(&self, method: Method, url: &str) -> RequestBuilder {
        let url = format!("{}{}", self.base_url, url);
        self.client.request(method, &url)
    }

    fn check_response(&self, response: &reqwest::Response) -> Result<()> {
        debug!("Github Response: {:#?}", response);

        if !response.status().is_success() {
            //TODO Better handling of these failure payloads
            Err(format!("Request failed: {}", response.status()).into())
        } else {
            Ok(())
        }
    }

    // Process a response recieved from Github. This checks for things like hitting rate limits,
    // etc., and then deserializes the json response.
    async fn process_response<T: serde::de::DeserializeOwned + std::fmt::Debug>(
        &self,
        response: reqwest::Response,
    ) -> Result<T> {
        debug!("Github Response: {:#?}", response);

        if !response.status().is_success() {
            //TODO Better handling of these failure payloads
            return Err(format!(
                "Request failed: {}",
                serde_json::to_string_pretty(&response.json::<serde_json::Value>().await?)?
            )
            .into());
        }

        let payload = response.text().await?;

        //TODO fix this mess
        match serde_json::from_str(&payload) {
            Ok(t) => Ok(t),
            Err(_) => {
                let pretty_json = serde_json::to_string_pretty(&serde_json::from_str::<
                    serde_json::Value,
                >(&payload)?)?;
                Err(format!(
                    "Error deserializing: {}\nContent: {}",
                    serde_json::from_str::<T>(&pretty_json)
                        .unwrap_err()
                        .to_string(),
                    pretty_json,
                )
                .into())
            }
        }
    }

    pub fn licenses(&self) -> LicenseClient {
        LicenseClient::new(&self)
    }
}

impl Default for Client {
    fn default() -> Self {
        Self::new()
    }
}
