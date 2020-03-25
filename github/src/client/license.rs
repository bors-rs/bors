use crate::{
    client::{Client, Response, Result},
    License, RepositoryLicense,
};

/// `LicenseClient` handles communication with the license related methods of the GitHub API.
///
/// GitHub API docs: https://developer.github.com/v3/licenses/
pub struct LicenseClient<'a> {
    inner: &'a Client,
}

impl<'a> LicenseClient<'a> {
    pub(super) fn new(client: &'a Client) -> Self {
        Self { inner: client }
    }

    /// List popular open source licenses
    ///
    /// GitHub API docs: https://developer.github.com/v3/licenses/#list-all-licenses
    pub async fn list(&self) -> Result<Response<Vec<License>>> {
        let response = self.inner.get("licenses").send().await?;

        self.inner.json(response).await
    }

    /// Get metadata for an individual license
    ///
    /// GitHub API docs: https://developer.github.com/v3/licenses/#get-an-individual-license
    pub async fn get(&self, license_name: &str) -> Result<Response<License>> {
        let url = format!("licenses/{}", license_name);
        let response = self.inner.get(&url).send().await?;

        self.inner.json(response).await
    }

    /// Get the contents of a repository's license
    ///
    /// GitHub API docs: https://developer.github.com/v3/licenses/#get-the-contents-of-a-repositorys-license
    pub async fn get_for_repo(
        &self,
        owner: &str,
        repo: &str,
    ) -> Result<Response<RepositoryLicense>> {
        let url = format!("repos/{}/{}/license", owner, repo);
        let response = self.inner.get(&url).send().await?;

        self.inner.json(response).await
    }
}
