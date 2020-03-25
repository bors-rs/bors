use crate::client::{Client, Response, Result};
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
struct MarkdownRequest {
    text: String,
    mode: Mode,
    context: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
enum Mode {
    /// Plain Markdown
    Markdown,
    /// Github Flavored Markdown
    Gfm,
}

/// `MarkdownClient` handles communication with the markdown related methods of the GitHub API.
///
/// GitHub API docs: https://developer.github.com/v3/markdown/
pub struct MarkdownClient<'a> {
    inner: &'a Client,
}

impl<'a> MarkdownClient<'a> {
    pub(super) fn new(client: &'a Client) -> Self {
        Self { inner: client }
    }

    /// Render a Markdown document
    ///
    /// https://developer.github.com/v3/markdown/#render-an-arbitrary-markdown-document
    pub async fn render(&self, text: String) -> Result<Response<String>> {
        let request = MarkdownRequest {
            text,
            mode: Mode::Markdown,
            context: None,
        };

        let response = self.inner.post("markdown").json(&request).send().await?;
        self.inner.text(response).await
    }

    /// Render a Github Flavored Markdown document
    /// Optional repository context can be provided: `Some("owner", "repo")`
    ///
    /// https://developer.github.com/v3/markdown/#render-an-arbitrary-markdown-document
    pub async fn render_gfm(
        &self,
        text: String,
        context: Option<(&str, &str)>,
    ) -> Result<Response<String>> {
        let context = context.map(|ctx| format!("{}/{}", ctx.0, ctx.1));
        let request = MarkdownRequest {
            text,
            mode: Mode::Gfm,
            context,
        };

        let response = self.inner.post("markdown").json(&request).send().await?;
        self.inner.text(response).await
    }
}
