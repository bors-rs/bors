#![allow(dead_code)]

use reqwest::{header, Client as ReqwestClient, Method, RequestBuilder};

mod error;
mod issues;
mod license;
mod markdown;
mod pagination;
mod pulls;
mod rate_limit;
mod reactions;

pub use error::{Error, Result};
pub use issues::IssuesClient;
pub use license::LicenseClient;
pub use markdown::MarkdownClient;
pub use pagination::{
    Pagination, PaginationCursorOptions, PaginationOptions, SortDirection, SortPages, StateFilter,
};
pub use pulls::PullsClient;
pub use rate_limit::{Rate, RateLimitClient, RateLimits};
pub use reactions::ReactionsClient;

// Constants
const DEFAULT_BASE_URL: &str = "https://api.github.com/";
const UPLOAD_BASE_URL: &str = "https://uploads.github.com/";
const USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

const HEADER_RATE_LIMIT: &str = "X-RateLimit-Limit";
const HEADER_RATE_REMAINING: &str = "X-RateLimit-Remaining";
const HEADER_RATE_RESET: &str = "X-RateLimit-Reset";
const HEADER_OTP: &str = "X-GitHub-OTP";
const HEADER_LINK: &str = "Link";

const MEDIA_TYPE_V3: &str = "application/vnd.github.v3+json";
const DEFAULT_MEDIA_TYPE: &str = "application/octet-stream";
const MEDIA_TYPE_V3_SHA: &str = "application/vnd.github.v3.sha";
const MEDIA_TYPE_V3_DIFF: &str = "application/vnd.github.v3.diff";
const MEDIA_TYPE_V3_PATCH: &str = "application/vnd.github.v3.patch";
const MEDIA_TYPE_ORG_PERMISSION_REPO: &str = "application/vnd.github.v3.repository+json";

// Media Type values to access preview APIs
//
// https://developer.github.com/v3/previews/#api-previews

// https://developer.github.com/changes/2020-01-10-revoke-installation-token/
const MEDIA_TYPE_REVOKE_TOKEN_PREVIEW: &str = "application/vnd.github.gambit-preview+json";

// https://developer.github.com/changes/2014-12-09-new-attributes-for-stars-api/
const MEDIA_TYPE_STARRING_PREVIEW: &str = "application/vnd.github.v3.star+json";

// https://help.github.com/enterprise/2.4/admin/guides/migrations/exporting-the-github-com-organization-s-repositories/
const MEDIA_TYPE_MIGRATIONS_PREVIEW: &str = "application/vnd.github.wyandotte-preview+json";

// https://developer.github.com/changes/2016-04-06-deployment-and-deployment-status-enhancements/
const MEDIA_TYPE_DEPLOYMENT_STATUS_PREVIEW: &str = "application/vnd.github.ant-man-preview+json";

// https://developer.github.com/changes/2018-10-16-deployments-environments-states-and-auto-inactive-updates/
const MEDIA_TYPE_EXPAND_DEPLOYMENT_STATUS_PREVIEW: &str =
    "application/vnd.github.flash-preview+json";

// https://developer.github.com/changes/2016-05-12-reactions-api-preview/
const MEDIA_TYPE_REACTIONS_PREVIEW: &str = "application/vnd.github.squirrel-girl-preview";

// https://developer.github.com/changes/2016-05-23-timeline-preview-api/
const MEDIA_TYPE_TIMELINE_PREVIEW: &str = "application/vnd.github.mockingbird-preview+json";

// https://developer.github.com/changes/2016-09-14-projects-api/
const MEDIA_TYPE_PROJECTS_PREVIEW: &str = "application/vnd.github.inertia-preview+json";

// https://developer.github.com/changes/2016-09-14-Integrations-Early-Access/
const MEDIA_TYPE_INTEGRATION_PREVIEW: &str = "application/vnd.github.machine-man-preview+json";

// https://developer.github.com/changes/2017-01-05-commit-search-api/
const MEDIA_TYPE_COMMIT_SEARCH_PREVIEW: &str = "application/vnd.github.cloak-preview+json";

// https://developer.github.com/changes/2017-02-28-user-blocking-apis-and-webhook/
const MEDIA_TYPE_BLOCK_USERS_PREVIEW: &str =
    "application/vnd.github.giant-sentry-fist-preview+json";

// https://developer.github.com/changes/2017-02-09-community-health/
const MEDIA_TYPE_REPOSITORY_COMMUNITY_HEALTH_METRICS_PREVIEW: &str =
    "application/vnd.github.black-panther-preview+json";

// https://developer.github.com/changes/2017-05-23-coc-api/
const MEDIA_TYPE_CODES_OF_CONDUCT_PREVIEW: &str =
    "application/vnd.github.scarlet-witch-preview+json";

// https://developer.github.com/changes/2017-07-17-update-topics-on-repositories/
const MEDIA_TYPE_TOPICS_PREVIEW: &str = "application/vnd.github.mercy-preview+json";

// https://developer.github.com/changes/2018-03-16-protected-branches-required-approving-reviews/
const MEDIA_TYPE_REQUIRED_APPROVING_REVIEWS_PREVIEW: &str =
    "application/vnd.github.luke-cage-preview+json";

// https://developer.github.com/changes/2018-01-10-lock-reason-api-preview/
const MEDIA_TYPE_LOCK_REASON_PREVIEW: &str = "application/vnd.github.sailor-v-preview+json";

// https://developer.github.com/changes/2018-05-07-new-checks-api-public-beta/
const MEDIA_TYPE_CHECK_RUNS_PREVIEW: &str = "application/vnd.github.antiope-preview+json";

// https://developer.github.com/enterprise/2.13/v3/repos/pre_receive_hooks/
const MEDIA_TYPE_PRE_RECEIVE_HOOKS_PREVIEW: &str = "application/vnd.github.eye-scream-preview";

// https://developer.github.com/changes/2018-02-22-protected-branches-required-signatures/
const MEDIA_TYPE_SIGNATURE_PREVIEW: &str = "application/vnd.github.zzzax-preview+json";

// https://developer.github.com/changes/2018-09-05-project-card-events/
const MEDIA_TYPE_PROJECT_CARD_DETAILS_PREVIEW: &str = "application/vnd.github.starfox-preview+json";

// https://developer.github.com/changes/2018-12-18-interactions-preview/
const MEDIA_TYPE_INTERACTION_RESTRICTIONS_PREVIEW: &str =
    "application/vnd.github.sombra-preview+json";

// https://developer.github.com/changes/2019-02-14-draft-pull-requests/
const MEDIA_TYPE_DRAFT_PREVIEW: &str = "application/vnd.github.shadow-cat-preview+json";

// https://developer.github.com/changes/2019-03-14-enabling-disabling-pages/
const MEDIA_TYPE_ENABLE_PAGES_API_PREVIEW: &str = "application/vnd.github.switcheroo-preview+json";

// https://developer.github.com/changes/2019-04-24-vulnerability-alerts/
const MEDIA_TYPE_REQUIRED_VULNERABILITY_ALERTS_PREVIEW: &str =
    "application/vnd.github.dorian-preview+json";

// https://developer.github.com/changes/2019-06-04-automated-security-fixes/
const MEDIA_TYPE_REQUIRED_AUTOMATED_SECURITY_FIXES_PREVIEW: &str =
    "application/vnd.github.london-preview+json";

// https://developer.github.com/changes/2019-05-29-update-branch-api/
const MEDIA_TYPE_UPDATE_PULL_REQUEST_BRANCH_PREVIEW: &str =
    "application/vnd.github.lydian-preview+json";

// https://developer.github.com/changes/2019-04-11-pulls-branches-for-commit/
const MEDIA_TYPE_LIST_PULLS_OR_BRANCHES_FOR_COMMIT_PREVIEW: &str =
    "application/vnd.github.groot-preview+json";

// https://developer.github.com/v3/previews/#repository-creation-permissions
const MEDIA_TYPE_MEMBER_ALLOWED_REPO_CREATION_TYPE_PREVIEW: &str =
    "application/vnd.github.surtur-preview+json";

// https://developer.github.com/v3/previews/#create-and-use-repository-templates
const MEDIA_TYPE_REPOSITORY_TEMPLATE_PREVIEW: &str = "application/vnd.github.baptiste-preview+json";

// https://developer.github.com/changes/2019-10-03-multi-line-comments/
const MEDIA_TYPE_MULTI_LINE_COMMENTS_PREVIEW: &str =
    "application/vnd.github.comfort-fade-preview+json";

// https://developer.github.com/changes/2019-11-05-deprecated-passwords-and-authorizations-api/
const MEDIA_TYPE_O_AUTH_APP_PREVIEW: &str = "application/vnd.github.doctor-strange-preview+json";

#[derive(Debug)]
pub struct Response<T> {
    pagination: Pagination,

    rate: Rate,

    inner: T,
}

impl<T> Response<T> {
    pub fn new(pagination: Pagination, rate: Rate, inner: T) -> Self {
        Self {
            pagination,
            rate,
            inner,
        }
    }

    pub fn pagination(&self) -> &Pagination {
        &self.pagination
    }

    pub fn rate(&self) -> &Rate {
        &self.rate
    }

    pub fn inner(&self) -> &T {
        &self.inner
    }

    pub fn into_inner(self) -> T {
        self.inner
    }

    pub fn into_parts(self) -> (Pagination, Rate, T) {
        (self.pagination, self.rate, self.inner)
    }
}

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

    fn delete(&self, url: &str) -> RequestBuilder {
        self.request(Method::DELETE, url)
    }

    fn get(&self, url: &str) -> RequestBuilder {
        self.request(Method::GET, url)
    }

    fn patch(&self, url: &str) -> RequestBuilder {
        self.request(Method::PATCH, url)
    }

    fn post(&self, url: &str) -> RequestBuilder {
        self.request(Method::POST, url)
    }

    fn put(&self, url: &str) -> RequestBuilder {
        self.request(Method::PUT, url)
    }

    fn request(&self, method: Method, url: &str) -> RequestBuilder {
        let url = format!("{}{}", self.base_url, url);
        self.client.request(method, &url)
    }

    //TODO explicitly check for and construct a RateLimit error when rate limits are hit
    //TODO explicitly check for an construct an AbuseLimit error
    async fn check_response(
        &self,
        response: reqwest::Response,
    ) -> Result<(reqwest::Response, Pagination, Rate)> {
        if !response.status().is_success() {
            let status = response.status();
            // BUG: Don't try to look for a payload for all response types
            // https://developer.github.com/v3/#client-errors
            let msg = response.json().await?;
            return Err(Error::GithubClientError(status, msg));
        }

        let pagination = Pagination::from_headers(response.headers());
        let rate = Rate::from_headers(response.headers());

        Ok((response, pagination, rate))
    }

    async fn empty(&self, response: reqwest::Response) -> Result<Response<()>> {
        let (_response, pagination, rate) = self.check_response(response).await?;
        Ok(Response::new(pagination, rate, ()))
    }

    async fn json<T: serde::de::DeserializeOwned>(
        &self,
        response: reqwest::Response,
    ) -> Result<Response<T>> {
        let (response, pagination, rate) = self.check_response(response).await?;
        let json = response.json().await?;
        Ok(Response::new(pagination, rate, json))
    }

    async fn text(&self, response: reqwest::Response) -> Result<Response<String>> {
        let (response, pagination, rate) = self.check_response(response).await?;
        let text = response.text().await?;
        Ok(Response::new(pagination, rate, text))
    }

    // TODO: actions endpoint
    // https://developer.github.com/v3/actions/

    // TODO: activity endpoint
    // https://developer.github.com/v3/activity/

    // TODO: apps endpoint
    // https://developer.github.com/v3/apps/

    // TODO checks endpoint
    // https://developer.github.com/v3/checks/

    // TODO code of conduct endpoint
    // https://developer.github.com/v3/codes_of_conduct/

    // TODO emojis endpoint
    // https://developer.github.com/v3/emojis/

    // TODO gists endpoint
    // https://developer.github.com/v3/gists/

    // TODO git endpoint
    // https://developer.github.com/v3/git/

    // TODO gitignore endpoint
    // https://developer.github.com/v3/gitignore/

    // TODO interactions endpoint
    // https://developer.github.com/v3/interactions/

    pub fn issues(&self) -> IssuesClient {
        IssuesClient::new(&self)
    }

    pub fn licenses(&self) -> LicenseClient {
        LicenseClient::new(&self)
    }

    pub fn markdown(&self) -> MarkdownClient {
        MarkdownClient::new(&self)
    }

    // TODO meta endpoint
    // https://developer.github.com/v3/meta/

    // TODO migrations endpoint
    // https://developer.github.com/v3/migrations/

    // TODO orgs endpoint
    // https://developer.github.com/v3/orgs/

    // TODO projects endpoint
    // https://developer.github.com/v3/projects/

    pub fn pulls(&self) -> PullsClient {
        PullsClient::new(&self)
    }

    pub fn rate_limit(&self) -> RateLimitClient {
        RateLimitClient::new(&self)
    }

    pub fn reactions(&self) -> ReactionsClient {
        ReactionsClient::new(&self)
    }

    // TODO repos endpoint
    // https://developer.github.com/v3/repos/

    // TODO search endpoint
    // https://developer.github.com/v3/search/

    // TODO teams endpoint
    // https://developer.github.com/v3/teams/

    // TODO users endpoint
    // https://developer.github.com/v3/users/
}

impl Default for Client {
    fn default() -> Self {
        Self::new()
    }
}
