use super::RepositoryClient;
use crate::{
    client::{PaginationOptions, Response, Result},
    DateTime, NodeId, StatusEventState, User,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct RepoStatus {
    pub id: u64,
    pub node_id: NodeId,
    pub url: String,

    pub state: StatusEventState,
    pub target_url: Option<String>,
    pub description: Option<String>,
    pub context: String,

    pub creator: User,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

/// CombinedStatus represents the combined status of a repository at a particular reference.
#[derive(Debug, Deserialize)]
pub struct CombinedStatus {
    pub state: StatusEventState,

    pub name: String,
    pub sha: String,
    pub total_count: u64,
    pub statuses: Vec<RepoStatus>,

    pub commit_url: String,
    pub repository_url: String,
}

#[derive(Debug, Serialize)]
pub struct CreateStatusRequest<'a> {
    pub state: StatusEventState,
    pub target_url: Option<&'a str>,
    pub description: Option<&'a str>,
    pub context: &'a str,
}

// Implementation for the status enpoint
// https://developer.github.com/v3/repos/statuses/
impl RepositoryClient<'_> {
    /// List Statuses
    ///
    /// GitHub API docs: https://developer.github.com/v3/repos/statuses/#list-statuses-for-a-specific-ref
    pub async fn list_statuses(
        &self,
        owner: &str,
        repo: &str,
        ref_name: &str,
        options: PaginationOptions,
    ) -> Result<Response<Vec<RepoStatus>>> {
        let url = format!("repos/{}/{}/commits/{}/statuses", owner, repo, ref_name);
        let response = self.inner.get(&url).query(&options).send().await?;

        self.inner.json(response).await
    }

    /// Get combined status for the specified reference.
    ///
    /// GitHub API docs: https://developer.github.com/v3/repos/statuses/#get-the-combined-status-for-a-specific-ref
    pub async fn get_combined_status(
        &self,
        owner: &str,
        repo: &str,
        ref_name: &str,
        options: PaginationOptions,
    ) -> Result<Response<CombinedStatus>> {
        let url = format!("repos/{}/{}/commits/{}/status", owner, repo, ref_name);
        let response = self.inner.get(&url).query(&options).send().await?;

        self.inner.json(response).await
    }

    /// Create a status for the specified reference.
    ///
    /// GitHub API docs: https://developer.github.com/v3/repos/statuses/#create-a-status
    pub async fn create_status(
        &self,
        owner: &str,
        repo: &str,
        ref_name: &str,
        request: &CreateStatusRequest<'_>,
    ) -> Result<Response<RepoStatus>> {
        let url = format!("repos/{}/{}/statuses/{}", owner, repo, ref_name);
        let response = self.inner.post(&url).json(request).send().await?;

        self.inner.json(response).await
    }
}
