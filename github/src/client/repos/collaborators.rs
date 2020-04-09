use super::RepositoryClient;
use crate::{
    client::{PaginationOptions, Response, Result},
    User,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize)]
pub struct ListCollaboratorsOptions {
    pub affiliation: String, //TODO type

    #[serde(flatten)]
    pub pagination_options: PaginationOptions,
}

// Implementation from the collaborators endpoint
// https://developer.github.com/v3/repos/collaborators/
impl RepositoryClient<'_> {
    /// List Collaborators
    ///
    /// GitHub API docs: https://developer.github.com/v3/repos/collaborators/#list-collaborators
    pub async fn list_collaborators(
        &self,
        owner: &str,
        repo: &str,
        options: ListCollaboratorsOptions,
    ) -> Result<Response<Vec<User>>> {
        let url = format!("repos/{}/{}/collaborators", owner, repo);
        let response = self.inner.get(&url).query(&options).send().await?;

        self.inner.json(response).await
    }

    /// Check if a user is a collaborator
    ///
    /// GitHub API docs: https://developer.github.com/v3/repos/collaborators/#check-if-a-user-is-a-collaborator
    pub async fn is_collaborator(
        &self,
        owner: &str,
        repo: &str,
        user: &str,
    ) -> Result<Response<bool>> {
        let url = format!("repos/{}/{}/collaborators/{}", owner, repo, user);
        let response = self.inner.get(&url).send().await?;

        self.inner.boolean(response).await
    }

    //TODO maybe use a type for the return value
    /// Checks the repository permission of a collaborator. The possible repository permissions are
    /// admin, write, read, and none.
    ///
    /// GitHub API docs: https://developer.github.com/v3/repos/collaborators/#review-a-users-permission-level
    pub async fn get_collaborator_permission_level(
        &self,
        owner: &str,
        repo: &str,
        user: &str,
    ) -> Result<Response<String>> {
        #[derive(Debug, Deserialize)]
        struct PermissionLevelResponse {
            permission: String,
        }

        let url = format!("repos/{}/{}/collaborators/{}/permission", owner, repo, user);
        let response = self.inner.get(&url).send().await?;

        let (pagination, rate, permission_level_response) = self
            .inner
            .json::<PermissionLevelResponse>(response)
            .await?
            .into_parts();

        Ok(Response::new(
            pagination,
            rate,
            permission_level_response.permission,
        ))
    }

    /// Add a user as a collaborator
    ///
    /// GitHub API docs: https://developer.github.com/v3/repos/collaborators/#add-user-as-a-collaborator
    pub async fn add_collaborator(
        &self,
        owner: &str,
        repo: &str,
        user: &str,
        permission: Option<&str>,
    ) -> Result<Response<()>> {
        #[derive(Debug, Serialize)]
        struct AddCollaboratorRequest<'a> {
            permission: Option<&'a str>,
        }

        let request = AddCollaboratorRequest { permission };
        let url = format!("repos/{}/{}/collaborators/{}", owner, repo, user);
        let response = self.inner.put(&url).json(&request).send().await?;

        self.inner.empty(response).await
    }

    /// Remove a collaborator
    ///
    /// GitHub API docs: https://developer.github.com/v3/repos/collaborators/#remove-user-as-a-collaborator
    pub async fn remove_collaborator(
        &self,
        owner: &str,
        repo: &str,
        user: &str,
    ) -> Result<Response<()>> {
        let url = format!("repos/{}/{}/collaborators/{}", owner, repo, user);
        let response = self.inner.delete(&url).send().await?;

        self.inner.empty(response).await
    }
}
