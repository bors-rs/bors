use crate::{
    client::{Client, Response, Result},
    Oid,
};
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

/// `GitClient` handles communication with the git related methods of the GitHub API.
///
/// GitHub API docs: https://developer.github.com/v3/git/
pub struct GitClient<'a> {
    inner: &'a Client,
}

impl<'a> GitClient<'a> {
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

    /// Update a Ref
    ///
    /// https://developer.github.com/v3/git/refs/#update-a-reference
    pub async fn update_ref(
        &self,
        owner: &str,
        repo: &str,
        ref_name: &str,
        sha: &Oid,
        force: bool,
    ) -> Result<Response<()>> {
        #[derive(Debug, Serialize)]
        struct UpdateRefRequest {
            sha: String,
            force: bool,
        }

        let request = UpdateRefRequest {
            sha: sha.to_string(),
            force,
        };

        let url = format!("repos/{}/{}/git/refs/{}", owner, repo, ref_name);
        log::info!("url: {}", url);
        let response = self.inner.patch(&url).json(&request).send().await?;
        //TODO actually return the ref here
        self.inner.empty(response).await
    }
}
