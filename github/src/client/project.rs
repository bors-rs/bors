use crate::{
    client::{Client, PaginationOptions, Response, Result, MEDIA_TYPE_PROJECTS_PREVIEW},
    Project, ProjectCard, ProjectColumn,
};
use serde::Serialize;

#[derive(Debug, Default, Serialize)]
pub struct ListProjectsOptions {
    pub state: Option<String>,

    #[serde(flatten)]
    pub pagination_options: PaginationOptions,
}

#[derive(Debug, Default, Serialize)]
pub struct ListProjectCardsOptions {
    pub archived_state: Option<String>,

    #[serde(flatten)]
    pub pagination_options: PaginationOptions,
}

#[derive(Debug, Serialize)]
struct CreateProjectRequest<'a> {
    name: &'a str,
    body: Option<&'a str>,
}

#[derive(Debug, Serialize)]
pub struct CreateProjectCardRequest {
    /// The card's note content. Only valid for cards without another type of content, so you must
    /// omit when specifying content_id and content_type.
    pub note: Option<String>,

    /// The issue or pull request id you want to associate with this card.
    /// Note: Depending on whether you use the issue id or pull request id,
    /// you will need to specify Issue or PullRequest as the content_type.
    pub content_id: Option<u64>,

    /// Required if you provide content_id. The type of content you want to associate with this
    /// card. Use Issue when content_id is an issue id and use PullRequest when content_id is a
    /// pull request id.
    pub content_type: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct UpdateProjectCardRequest {
    pub note: Option<String>,
    pub archived: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct MoveProjectCardRequest {
    /// Required. Can be one of top, bottom, or after:<card_id>, where <card_id> is the id value of
    /// a card in the same column, or in the new column specified by column_id.
    pub position: String,

    /// The id value of a column in the same project.
    pub column_id: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct UpdateProjectRequest {
    pub name: Option<String>,
    pub body: Option<String>,
    pub state: Option<String>,
    pub organization_permission: Option<String>,
    pub private: Option<bool>,
}

/// `ProjectClient` handles communication with the project related methods of the GitHub API.
///
/// GitHub API docs: https://developer.github.com/v3/projects
pub struct ProjectClient<'a> {
    inner: &'a Client,
}

impl<'a> ProjectClient<'a> {
    pub(super) fn new(client: &'a Client) -> Self {
        Self { inner: client }
    }

    /// https://developer.github.com/v3/projects/#list-repository-projects
    pub async fn list_for_repo(
        &self,
        owner: &str,
        repo: &str,
        options: Option<ListProjectsOptions>,
    ) -> Result<Response<Vec<Project>>> {
        let url = format!("repos/{}/{}/projects", owner, repo);
        let response = self
            .inner
            .get(&url)
            // For the enabling projects endpoint
            .header(reqwest::header::ACCEPT, MEDIA_TYPE_PROJECTS_PREVIEW)
            .query(&options)
            .send()
            .await?;

        self.inner.json(response).await
    }

    /// https://developer.github.com/v3/projects/#list-organization-projects
    pub async fn list_for_org(
        &self,
        org: &str,
        options: Option<ListProjectsOptions>,
    ) -> Result<Response<Vec<Project>>> {
        let url = format!("orgs/{}/projects", org);
        let response = self
            .inner
            .get(&url)
            // For the enabling projects endpoint
            .header(reqwest::header::ACCEPT, MEDIA_TYPE_PROJECTS_PREVIEW)
            .query(&options)
            .send()
            .await?;

        self.inner.json(response).await
    }

    /// https://developer.github.com/v3/projects/#list-user-projects
    pub async fn list_for_user(
        &self,
        user: &str,
        options: Option<ListProjectsOptions>,
    ) -> Result<Response<Vec<Project>>> {
        let url = format!("users/{}/projects", user);
        let response = self
            .inner
            .get(&url)
            // For the enabling projects endpoint
            .header(reqwest::header::ACCEPT, MEDIA_TYPE_PROJECTS_PREVIEW)
            .query(&options)
            .send()
            .await?;

        self.inner.json(response).await
    }

    /// https://developer.github.com/v3/projects/#get-a-project
    pub async fn get(&self, project_id: u64) -> Result<Response<Project>> {
        let url = format!("projects/{}", project_id);
        let response = self
            .inner
            .get(&url)
            // For the enabling projects endpoint
            .header(reqwest::header::ACCEPT, MEDIA_TYPE_PROJECTS_PREVIEW)
            .send()
            .await?;

        self.inner.json(response).await
    }

    /// https://developer.github.com/v3/projects/#create-a-repository-project
    pub async fn create_for_repo(
        &self,
        owner: &str,
        repo: &str,
        name: &str,
        body: Option<&str>,
    ) -> Result<Response<Project>> {
        let request = CreateProjectRequest { name, body };
        let url = format!("repos/{}/{}/projects", owner, repo);
        let response = self
            .inner
            .post(&url)
            // For the enabling projects endpoint
            .header(reqwest::header::ACCEPT, MEDIA_TYPE_PROJECTS_PREVIEW)
            .json(&request)
            .send()
            .await?;

        self.inner.json(response).await
    }

    /// https://developer.github.com/v3/projects/#create-an-organization-project
    pub async fn create_for_org(
        &self,
        org: &str,
        name: &str,
        body: Option<&str>,
    ) -> Result<Response<Project>> {
        let request = CreateProjectRequest { name, body };
        let url = format!("orgs/{}/projects", org);
        let response = self
            .inner
            .post(&url)
            // For the enabling projects endpoint
            .header(reqwest::header::ACCEPT, MEDIA_TYPE_PROJECTS_PREVIEW)
            .json(&request)
            .send()
            .await?;

        self.inner.json(response).await
    }

    /// https://developer.github.com/v3/projects/#create-a-user-project
    pub async fn create_for_user(
        &self,
        user: &str,
        name: &str,
        body: Option<&str>,
    ) -> Result<Response<Project>> {
        let request = CreateProjectRequest { name, body };
        let url = format!("users/{}/projects", user);
        let response = self
            .inner
            .post(&url)
            // For the enabling projects endpoint
            .header(reqwest::header::ACCEPT, MEDIA_TYPE_PROJECTS_PREVIEW)
            .json(&request)
            .send()
            .await?;

        self.inner.json(response).await
    }

    /// https://developer.github.com/v3/projects/#update-a-project
    pub async fn update(
        &self,
        project_id: u64,
        request: &UpdateProjectRequest,
    ) -> Result<Response<Project>> {
        let url = format!("projects/{}", project_id);
        let response = self
            .inner
            .patch(&url)
            // For the enabling projects endpoint
            .header(reqwest::header::ACCEPT, MEDIA_TYPE_PROJECTS_PREVIEW)
            .json(request)
            .send()
            .await?;

        self.inner.json(response).await
    }

    /// https://developer.github.com/v3/projects/#delete-a-project
    pub async fn delete(&self, project_id: u64) -> Result<Response<()>> {
        let url = format!("projects/{}", project_id);
        let response = self
            .inner
            .delete(&url)
            // For the enabling projects endpoint
            .header(reqwest::header::ACCEPT, MEDIA_TYPE_PROJECTS_PREVIEW)
            .send()
            .await?;

        self.inner.empty(response).await
    }

    // ProjectCard endpoints
    // https://developer.github.com/v3/projects/cards/

    /// https://developer.github.com/v3/projects/cards/#list-project-cards
    pub async fn list_cards(
        &self,
        column_id: u64,
        options: Option<ListProjectCardsOptions>,
    ) -> Result<Response<Vec<ProjectCard>>> {
        let url = format!("projects/columns/{}/cards", column_id);
        let response = self
            .inner
            .get(&url)
            // For the enabling projects endpoint
            .header(reqwest::header::ACCEPT, MEDIA_TYPE_PROJECTS_PREVIEW)
            .query(&options)
            .send()
            .await?;

        self.inner.json(response).await
    }

    /// https://developer.github.com/v3/projects/cards/#get-a-project-card
    pub async fn get_card(&self, card_id: u64) -> Result<Response<ProjectCard>> {
        let url = format!("projects/columns/cards/{}", card_id);
        let response = self
            .inner
            .get(&url)
            // For the enabling projects endpoint
            .header(reqwest::header::ACCEPT, MEDIA_TYPE_PROJECTS_PREVIEW)
            .send()
            .await?;

        self.inner.json(response).await
    }

    /// https://developer.github.com/v3/projects/cards/#create-a-project-card
    pub async fn create_card(
        &self,
        column_id: u64,
        request: &CreateProjectCardRequest,
    ) -> Result<Response<ProjectCard>> {
        let url = format!("projects/columns/{}/cards", column_id);
        let response = self
            .inner
            .post(&url)
            // For the enabling projects endpoint
            .header(reqwest::header::ACCEPT, MEDIA_TYPE_PROJECTS_PREVIEW)
            .json(&request)
            .send()
            .await?;

        self.inner.json(response).await
    }

    /// https://developer.github.com/v3/projects/cards/#update-a-project-card
    pub async fn update_card(
        &self,
        card_id: u64,
        request: &UpdateProjectCardRequest,
    ) -> Result<Response<ProjectCard>> {
        let url = format!("projects/columns/cards/{}", card_id);
        let response = self
            .inner
            .patch(&url)
            // For the enabling projects endpoint
            .header(reqwest::header::ACCEPT, MEDIA_TYPE_PROJECTS_PREVIEW)
            .json(request)
            .send()
            .await?;

        self.inner.json(response).await
    }

    /// https://developer.github.com/v3/projects/cards/#delete-a-project-card
    pub async fn delete_card(&self, card_id: u64) -> Result<Response<()>> {
        let url = format!("projects/columns/cards/{}", card_id);
        let response = self
            .inner
            .delete(&url)
            // For the enabling projects endpoint
            .header(reqwest::header::ACCEPT, MEDIA_TYPE_PROJECTS_PREVIEW)
            .send()
            .await?;

        self.inner.empty(response).await
    }

    /// https://developer.github.com/v3/projects/cards/#move-a-project-card
    pub async fn move_card(
        &self,
        card_id: u64,
        request: &MoveProjectCardRequest,
    ) -> Result<Response<()>> {
        let url = format!("projects/columns/cards/{}/moves", card_id);
        let response = self
            .inner
            .post(&url)
            // For the enabling projects endpoint
            .header(reqwest::header::ACCEPT, MEDIA_TYPE_PROJECTS_PREVIEW)
            .json(request)
            .send()
            .await?;

        self.inner.empty(response).await
    }

    // TODO Collaborators Endpoints
    // https://developer.github.com/v3/projects/collaborators/

    // Columns Endpoints
    // https://developer.github.com/v3/projects/columns/

    /// https://developer.github.com/v3/projects/columns/#list-project-columns
    pub async fn list_columns(
        &self,
        project_id: u64,
        options: Option<PaginationOptions>,
    ) -> Result<Response<Vec<ProjectColumn>>> {
        let url = format!("projects/{}/columns", project_id);
        let response = self
            .inner
            .get(&url)
            // For the enabling projects endpoint
            .header(reqwest::header::ACCEPT, MEDIA_TYPE_PROJECTS_PREVIEW)
            .query(&options)
            .send()
            .await?;

        self.inner.json(response).await
    }

    /// https://developer.github.com/v3/projects/columns/#get-a-project-column
    pub async fn get_column(&self, column_id: u64) -> Result<Response<ProjectColumn>> {
        let url = format!("projects/columns/{}", column_id);
        let response = self
            .inner
            .get(&url)
            // For the enabling projects endpoint
            .header(reqwest::header::ACCEPT, MEDIA_TYPE_PROJECTS_PREVIEW)
            .send()
            .await?;

        self.inner.json(response).await
    }

    /// https://developer.github.com/v3/projects/columns/#create-a-project-column
    pub async fn create_column(
        &self,
        project_id: u64,
        name: &str,
    ) -> Result<Response<ProjectColumn>> {
        #[derive(Debug, Serialize)]
        struct CreateColumnRequest<'a> {
            name: &'a str,
        }

        let request = CreateColumnRequest { name };
        let url = format!("projects/{}/columns", project_id);
        let response = self
            .inner
            .post(&url)
            // For the enabling projects endpoint
            .header(reqwest::header::ACCEPT, MEDIA_TYPE_PROJECTS_PREVIEW)
            .json(&request)
            .send()
            .await?;

        self.inner.json(response).await
    }

    /// https://developer.github.com/v3/projects/columns/#update-a-project-column
    pub async fn update_column(
        &self,
        column_id: u64,
        name: &str,
    ) -> Result<Response<ProjectColumn>> {
        #[derive(Debug, Serialize)]
        struct UpdateColumnRequest<'a> {
            name: &'a str,
        }

        let request = UpdateColumnRequest { name };
        let url = format!("projects/columns/{}", column_id);
        let response = self
            .inner
            .patch(&url)
            // For the enabling projects endpoint
            .header(reqwest::header::ACCEPT, MEDIA_TYPE_PROJECTS_PREVIEW)
            .json(&request)
            .send()
            .await?;

        self.inner.json(response).await
    }

    /// https://developer.github.com/v3/projects/columns/#delete-a-project-column
    pub async fn delete_column(&self, column_id: u64) -> Result<Response<()>> {
        let url = format!("projects/columns/{}", column_id);
        let response = self
            .inner
            .delete(&url)
            // For the enabling projects endpoint
            .header(reqwest::header::ACCEPT, MEDIA_TYPE_PROJECTS_PREVIEW)
            .send()
            .await?;

        self.inner.empty(response).await
    }

    /// Move a Column
    ///
    /// `position` can be one of `first`, `last`, or `after:<column_id>`,
    /// where <column_id> is the id value of a column in the same project.
    ///
    /// https://developer.github.com/v3/projects/columns/#move-a-project-column
    pub async fn move_column(&self, column_id: u64, position: &str) -> Result<Response<()>> {
        #[derive(Debug, Serialize)]
        struct MoveColumnRequest<'a> {
            position: &'a str,
        }

        let request = MoveColumnRequest { position };
        let url = format!("projects/columns/{}/moves", column_id);
        let response = self
            .inner
            .post(&url)
            // For the enabling projects endpoint
            .header(reqwest::header::ACCEPT, MEDIA_TYPE_PROJECTS_PREVIEW)
            .json(&request)
            .send()
            .await?;

        self.inner.empty(response).await
    }
}
