use crate::{
    client::{Client, Response, Result},
    Oid,
};
use serde::Serialize;

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
        let response = self.inner.patch(&url).json(&request).send().await?;
        //TODO actually return the ref here
        self.inner.empty(response).await
    }
}
