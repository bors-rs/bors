use crate::{
    client::{
        Client, PaginationOptions, Response, Result, SortDirection, SortPages, StateFilter,
        MEDIA_TYPE_INTEGRATION_PREVIEW, MEDIA_TYPE_LOCK_REASON_PREVIEW,
        MEDIA_TYPE_REACTIONS_PREVIEW,
    },
    Comment, DateTime, Issue, Label, State, User,
};
use serde::Serialize;

#[derive(Debug, Default, Serialize)]
pub struct ListIssuesOptions {
    /// Indicates which sorts of issues to return. Can be one of:
    /// * assigned: Issues assigned to you
    /// * created: Issues created by you
    /// * mentioned: Issues mentioning you
    /// * subscribed: Issues you're subscribed to updates for
    /// * all: All issues the authenticated user can see, regardless of participation or creation
    /// Default: assigned
    pub filter: ListIssuesFilter,

    /// Indicates the state of the issues to return. Default: open
    pub state: StateFilter,

    /// A list of comma separated label names. Example: bug,ui,@high
    pub labels: Vec<String>,

    /// What to sort results by. Default: created
    pub sort: SortPages,

    /// The direction of the sort. Default: desc
    pub direction: SortDirection,

    /// Only issues updated at or after this time are returned. This is a timestamp in ISO 8601 format: YYYY-MM-DDTHH:MM:SSZ.
    pub since: Option<DateTime>,

    #[serde(flatten)]
    pub pagination_options: PaginationOptions,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ListIssuesFilter {
    Assigned,
    Created,
    Mentioned,
    Subscribed,
    All,
}

impl Default for ListIssuesFilter {
    fn default() -> Self {
        ListIssuesFilter::Assigned
    }
}

#[derive(Debug, Default, Serialize)]
pub struct ListIssuesForRepoOptions {
    // If an integer is passed, it should refer to a milestone by its number field. If the string *
    // is passed, issues with any milestone are accepted. If the string none is passed, issues
    // without milestones are returned.
    pub milestone: Option<MilestoneFilter>,

    /// Indicates the state of the issues to return. Default: open
    pub state: StateFilter,

    /// Can be the name of a user. Pass in none for issues with no assigned user, and * for issues
    /// assigned to any user.
    pub assignee: String, //TODO type

    /// The user that created the issue.
    pub creator: String, //TODO type

    /// A user that's mentioned in the issue.
    pub mentioned: String,

    /// A list of comma separated label names. Example: bug,ui,@high
    pub labels: Vec<String>,

    /// What to sort results by. Default: created
    pub sort: SortPages,

    /// The direction of the sort. Default: desc
    pub direction: SortDirection,

    /// Only issues updated at or after this time are returned. This is a timestamp in ISO 8601 format: YYYY-MM-DDTHH:MM:SSZ.
    pub since: Option<DateTime>,

    #[serde(flatten)]
    pub pagination_options: PaginationOptions,
}

#[derive(Debug)]
pub enum MilestoneFilter {
    Number(u64),
    Any,
    None,
}

impl Serialize for MilestoneFilter {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        match self {
            MilestoneFilter::Number(v) => serializer.serialize_u64(*v),
            MilestoneFilter::Any => serializer.serialize_str("*"),
            MilestoneFilter::None => serializer.serialize_str("none"),
        }
    }
}

#[derive(Debug, Default, Serialize)]
pub struct ListIssueCommentsOptions {
    /// What to sort results by. Default: created
    pub sort: SortPages,

    /// The direction of the sort. Default: desc
    pub direction: SortDirection,

    /// Only issues updated at or after this time are returned. This is a timestamp in ISO 8601 format: YYYY-MM-DDTHH:MM:SSZ.
    pub since: Option<DateTime>,

    #[serde(flatten)]
    pub pagination_options: PaginationOptions,
}

#[derive(Debug, Default, Serialize)]
pub struct IssueRequest {
    /// The title of the issue
    pub title: Option<String>,
    /// The contents of the issue
    pub body: Option<String>,
    // State of the issue
    pub state: Option<State>,
    /// Labels to associate with this issue. Send an empty array ([]) to clear all Labels from the
    /// Issue. NOTE: Only users with push access can set labels for new issues. Labels are silently
    /// dropped otherwise.
    pub labels: Option<Vec<String>>,
    /// The number of the milestone to associate this issue with. The number of the milestone to
    /// associate this issue with or null to remove current. NOTE: Only users with push access can
    /// set the milestone for new issues. The milestone is silently dropped otherwise.
    pub milestone: Option<u64>,
    /// Logins for Users to assign to this issue. Send an empty array ([]) to clear all assignees
    /// from the Issue. NOTE: Only users with push access can set assignees for new issues.
    /// Assignees are silently dropped otherwise.
    pub assignees: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
pub enum LockReason {
    #[serde(rename = "off-topic")]
    OffTopic,
    #[serde(rename = "too heated")]
    TooHeated,
    #[serde(rename = "resolved")]
    Resolved,
    #[serde(rename = "spam")]
    Spam,
}

/// `IssuesClient` handles communication with the issues related methods of the GitHub API.
///
/// GitHub API docs: https://developer.github.com/v3/issues/
pub struct IssuesClient<'a> {
    inner: &'a Client,
}

impl<'a> IssuesClient<'a> {
    pub(super) fn new(client: &'a Client) -> Self {
        Self { inner: client }
    }

    async fn list_issues<O: Serialize>(
        &self,
        url: &str,
        options: Option<O>,
    ) -> Result<Response<Vec<Issue>>> {
        let response = self
            .inner
            .get(url)
            // TODO: remove custom Accept headers when APIs fully launch.
            .header(reqwest::header::ACCEPT, MEDIA_TYPE_INTEGRATION_PREVIEW)
            // For the 'reactions' object in an Issue
            .header(reqwest::header::ACCEPT, MEDIA_TYPE_REACTIONS_PREVIEW)
            .query(&options)
            .send()
            .await?;

        self.inner.json(response).await
    }

    /// List issues assigned to the authenticated user across all visible
    /// repositories including owned repositories, member repositories,
    /// and organization repositories.
    ///
    /// GitHub API docs: https://developer.github.com/v3/issues/#list-issues-assigned-to-the-authenticated-user
    pub async fn list(&self, options: Option<ListIssuesOptions>) -> Result<Response<Vec<Issue>>> {
        let url = "issues";
        self.list_issues(url, options).await
    }

    /// List issues across owned and member repositories assigned to the authenticated user
    ///
    /// GitHub API docs: https://developer.github.com/v3/issues/#list-user-account-issues-assigned-to-the-authenticated-user
    pub async fn list_for_authenticated_user(
        &self,
        options: Option<ListIssuesOptions>,
    ) -> Result<Response<Vec<Issue>>> {
        let url = "user/issues";
        self.list_issues(url, options).await
    }

    /// List issues in an organization assigned to the authenticated user
    ///
    /// GitHub API docs: https://developer.github.com/v3/issues/#list-organization-issues-assigned-to-the-authenticated-user
    pub async fn list_for_org(
        &self,
        org: &str,
        options: Option<ListIssuesOptions>,
    ) -> Result<Response<Vec<Issue>>> {
        let url = format!("orgs/{}/issues", org);
        self.list_issues(&url, options).await
    }

    pub async fn list_for_repo(
        &self,
        owner: &str,
        repo: &str,
        options: Option<ListIssuesForRepoOptions>,
    ) -> Result<Response<Vec<Issue>>> {
        let url = format!("repos/{}/{}/issues", owner, repo);
        self.list_issues(&url, options).await
    }

    /// Get an Issue
    ///
    /// GitHub API docs: https://developer.github.com/v3/issues/#get-an-issue
    pub async fn get(&self, owner: &str, repo: &str, issue_number: u64) -> Result<Response<Issue>> {
        let url = format!("repos/{}/{}/issues/{}", owner, repo, issue_number);
        let response = self
            .inner
            .get(&url)
            // TODO: remove custom Accept headers when APIs fully launch.
            .header(reqwest::header::ACCEPT, MEDIA_TYPE_INTEGRATION_PREVIEW)
            // For the 'reactions' object in an Issue
            .header(reqwest::header::ACCEPT, MEDIA_TYPE_REACTIONS_PREVIEW)
            .send()
            .await?;

        self.inner.json(response).await
    }

    /// Create an Issue
    ///
    /// `IssueRequest` must have the field `title` set and `state` unset
    /// TODO maybe early return?
    ///
    /// GitHub API docs: https://developer.github.com/v3/issues/#create-an-issue
    pub async fn create(
        &self,
        owner: &str,
        repo: &str,
        issue: IssueRequest,
    ) -> Result<Response<Issue>> {
        let url = format!("repos/{}/{}/issues/", owner, repo);
        let response = self.inner.post(&url).json(&issue).send().await?;

        self.inner.json(response).await
    }

    /// Update an Issue
    ///
    /// GitHub API docs: https://developer.github.com/v3/issues/#update-an-issue
    pub async fn update(
        &self,
        owner: &str,
        repo: &str,
        issue_number: u64,
        issue: IssueRequest,
    ) -> Result<Response<Issue>> {
        let url = format!("repos/{}/{}/issues/{}", owner, repo, issue_number);
        let response = self.inner.patch(&url).json(&issue).send().await?;

        self.inner.json(response).await
    }

    /// Lock an Issue
    ///
    /// GitHub API docs: https://developer.github.com/v3/issues/#lock-an-issue
    pub async fn lock(
        &self,
        owner: &str,
        repo: &str,
        issue_number: u64,
        lock_reason: Option<LockReason>,
    ) -> Result<Response<()>> {
        #[derive(Debug, Serialize)]
        struct LockRequest {
            lock_reason: LockReason,
        }

        let url = format!("repos/{}/{}/issues/{}/lock", owner, repo, issue_number);
        let mut request_builder = self.inner.put(&url);

        if let Some(lock_reason) = lock_reason {
            request_builder = request_builder
                // For the supplying a lock reason
                .header(reqwest::header::ACCEPT, MEDIA_TYPE_LOCK_REASON_PREVIEW)
                .json(&LockRequest { lock_reason });
        }

        let response = request_builder.send().await?;

        self.inner.empty(response).await
    }

    /// Unlock an issue
    ///
    /// GitHub API docs: https://developer.github.com/v3/issues/#unlock-an-issue
    pub async fn unlock(&self, owner: &str, repo: &str, issue_number: u64) -> Result<Response<()>> {
        let url = format!("repos/{}/{}/issues/{}/lock", owner, repo, issue_number);
        let response = self.inner.delete(&url).send().await?;

        self.inner.empty(response).await
    }

    // Assignee Endpoint
    // https://developer.github.com/v3/issues/assignees/

    /// List all available assignees (owners and collaborators) to which issues may be assigned.
    ///
    /// GitHub API docs: https://developer.github.com/v3/issues/assignees/#list-assignees
    pub async fn list_assignees(
        &self,
        owner: &str,
        repo: &str,
        options: PaginationOptions,
    ) -> Result<Response<Vec<User>>> {
        let url = format!("repos/{}/{}/assignees", owner, repo);
        let response = self.inner.get(&url).query(&options).send().await?;

        self.inner.json(response).await
    }

    /// Checks if a user has permission to be assigned to an issue in this repository
    ///
    /// GitHub API docs: https://developer.github.com/v3/issues/assignees/#check-assignee
    pub async fn check_assignee(
        &self,
        owner: &str,
        repo: &str,
        assignee: &str,
    ) -> Result<Response<()>> {
        let url = format!("repos/{}/{}/assignees/{}", owner, repo, assignee);
        let response = self.inner.get(&url).send().await?;

        //TODO actually return a bool instead based off the the response code:
        // 204: the assignee can be assigned to the issue
        // 404: the assignee cannot be assigned to the issue
        self.inner.empty(response).await
    }

    /// Add up to 10 assignees to an issue. Users already assigned to an issue are not replaced.
    ///
    /// GitHub API docs: https://developer.github.com/v3/issues/assignees/#add-assignees-to-an-issue
    pub async fn add_assignees(
        &self,
        owner: &str,
        repo: &str,
        issue_number: u64,
        assignees: Vec<String>,
    ) -> Result<Response<Issue>> {
        #[derive(Debug, Serialize)]
        struct AddAssigneesRequest {
            assignees: Vec<String>,
        }

        let request = AddAssigneesRequest { assignees };
        let url = format!("repos/{}/{}/issues/{}/assignees", owner, repo, issue_number);
        let response = self.inner.post(&url).json(&request).send().await?;

        self.inner.json(response).await
    }

    /// Removes one or more assignees from an issue
    ///
    /// GitHub API docs: https://developer.github.com/v3/issues/assignees/#remove-assignees-from-an-issue
    pub async fn remove_assignees(
        &self,
        owner: &str,
        repo: &str,
        issue_number: u64,
        assignees: Vec<String>,
    ) -> Result<Response<Issue>> {
        #[derive(Debug, Serialize)]
        struct RemoveAssigneesRequest {
            assignees: Vec<String>,
        }

        let request = RemoveAssigneesRequest { assignees };
        let url = format!("repos/{}/{}/issues/{}/assignees", owner, repo, issue_number);
        let response = self.inner.delete(&url).json(&request).send().await?;

        self.inner.json(response).await
    }

    // Comments Endpoint
    // https://developer.github.com/v3/issues/comments/

    /// List comments on an Issue
    ///
    /// GitHub API docs: https://developer.github.com/v3/issues/comments/#list-comments-on-an-issue
    pub async fn list_comments(
        &self,
        owner: &str,
        repo: &str,
        issue_number: u64,
        options: Option<ListIssueCommentsOptions>,
    ) -> Result<Response<Vec<Comment>>> {
        let url = format!("repos/{}/{}/issues/{}/comments", owner, repo, issue_number);
        let response = self
            .inner
            .get(&url)
            // For the 'reactions' object in an Issue
            .header(reqwest::header::ACCEPT, MEDIA_TYPE_REACTIONS_PREVIEW)
            .query(&options)
            .send()
            .await?;

        self.inner.json(response).await
    }

    /// List comments in a Respsitory
    ///
    /// GitHub API docs: https://developer.github.com/v3/issues/comments/#list-comments-in-a-repository
    pub async fn list_comments_for_repo(
        &self,
        owner: &str,
        repo: &str,
        options: Option<ListIssueCommentsOptions>,
    ) -> Result<Response<Vec<Comment>>> {
        let url = format!("repos/{}/{}/issues/comments", owner, repo);
        let response = self
            .inner
            .get(&url)
            // For the 'reactions' object in an Issue
            .header(reqwest::header::ACCEPT, MEDIA_TYPE_REACTIONS_PREVIEW)
            .query(&options)
            .send()
            .await?;

        self.inner.json(response).await
    }

    /// Get a single Comment
    ///
    /// GitHub API docs: https://developer.github.com/v3/issues/comments/#get-a-single-comment
    pub async fn get_comment(
        &self,
        owner: &str,
        repo: &str,
        comment_id: u64,
    ) -> Result<Response<Comment>> {
        let url = format!("repos/{}/{}/issues/comments/{}", owner, repo, comment_id);
        let response = self
            .inner
            .get(&url)
            // For the 'performed_via_github_app' object
            .header(reqwest::header::ACCEPT, MEDIA_TYPE_INTEGRATION_PREVIEW)
            // For the 'reactions' object
            .header(reqwest::header::ACCEPT, MEDIA_TYPE_REACTIONS_PREVIEW)
            .send()
            .await?;

        self.inner.json(response).await
    }

    /// Create a Comment
    ///
    /// GitHub API docs: https://developer.github.com/v3/issues/comments/#create-a-comment
    pub async fn create_comment(
        &self,
        owner: &str,
        repo: &str,
        issue_number: u64,
        body: &str,
    ) -> Result<Response<Comment>> {
        #[derive(Debug, Serialize)]
        struct CreateCommentRequest<'a> {
            body: &'a str,
        }

        let request = CreateCommentRequest { body };
        let url = format!("repos/{}/{}/issues/{}/comments", owner, repo, issue_number);
        let response = self.inner.post(&url).json(&request).send().await?;

        self.inner.json(response).await
    }

    /// Update a Comment
    ///
    /// GitHub API docs: https://developer.github.com/v3/issues/comments/#edit-a-comment
    pub async fn update_comment(
        &self,
        owner: &str,
        repo: &str,
        comment_id: u64,
        body: &str,
    ) -> Result<Response<Comment>> {
        #[derive(Debug, Serialize)]
        struct UpdateCommentRequest<'a> {
            body: &'a str,
        }

        let request = UpdateCommentRequest { body };
        let url = format!("repos/{}/{}/issues/comments/{}", owner, repo, comment_id);
        let response = self.inner.patch(&url).json(&request).send().await?;

        self.inner.json(response).await
    }

    /// Delete a Comment
    ///
    /// GitHub API docs: https://developer.github.com/v3/issues/comments/#delete-a-comment
    pub async fn delete_comment(
        &self,
        owner: &str,
        repo: &str,
        comment_id: u64,
    ) -> Result<Response<()>> {
        let url = format!("repos/{}/{}/issues/comments/{}", owner, repo, comment_id);
        let response = self.inner.delete(&url).send().await?;

        self.inner.empty(response).await
    }

    // TODO
    // Events Endpoint
    // https://developer.github.com/v3/issues/events/

    // Labels Endpoint
    // https://developer.github.com/v3/issues/labels/

    /// List labels for this Repository
    ///
    /// Github API docs: https://developer.github.com/v3/issues/labels/#list-all-labels-for-this-repository
    pub async fn list_labels_for_repo(
        &self,
        owner: &str,
        repo: &str,
        options: Option<PaginationOptions>,
    ) -> Result<Response<Vec<Label>>> {
        let url = format!("repos/{}/{}/labels", owner, repo);
        let response = self.inner.get(&url).query(&options).send().await?;

        self.inner.json(response).await
    }

    /// Get a single label
    ///
    /// GitHub API docs: https://developer.github.com/v3/issues/labels/#get-a-single-label
    pub async fn get_label(&self, owner: &str, repo: &str, name: &str) -> Result<Response<Label>> {
        let url = format!("repos/{}/{}/labels/{}", owner, repo, name);
        let response = self.inner.get(&url).send().await?;

        self.inner.json(response).await
    }

    /// Create a label
    ///
    /// GitHub API docs: https://developer.github.com/v3/issues/labels/#create-a-label
    pub async fn create_label(
        &self,
        owner: &str,
        repo: &str,
        name: &str,
        color: &str,
        description: Option<&str>,
    ) -> Result<Response<Label>> {
        #[derive(Debug, Serialize)]
        struct CreateLabelRequest<'a> {
            name: &'a str,
            color: &'a str,
            description: Option<&'a str>,
        }

        let request = CreateLabelRequest {
            name,
            color,
            description,
        };
        let url = format!("repos/{}/{}/labels", owner, repo);
        let response = self.inner.post(&url).json(&request).send().await?;

        self.inner.json(response).await
    }

    /// Update a label
    ///
    /// GitHub API docs: https://developer.github.com/v3/issues/labels/#update-a-label
    pub async fn update_label(
        &self,
        owner: &str,
        repo: &str,
        name: &str,
        new_name: Option<&str>,
        color: Option<&str>,
        description: Option<&str>,
    ) -> Result<Response<Label>> {
        #[derive(Debug, Serialize)]
        struct UpdateLabelRequest<'a> {
            new_name: Option<&'a str>,
            color: Option<&'a str>,
            description: Option<&'a str>,
        }

        let request = UpdateLabelRequest {
            new_name,
            color,
            description,
        };
        let url = format!("repos/{}/{}/labels/{}", owner, repo, name);
        let response = self.inner.patch(&url).json(&request).send().await?;

        self.inner.json(response).await
    }

    /// Delete a label
    ///
    /// GitHub API docs: https://developer.github.com/v3/issues/labels/#delete-a-label
    pub async fn delete_label(&self, owner: &str, repo: &str, name: &str) -> Result<Response<()>> {
        let url = format!("repos/{}/{}/labels/{}", owner, repo, name);
        let response = self.inner.delete(&url).send().await?;

        self.inner.empty(response).await
    }

    /// List all labels on an issue
    ///
    /// GitHub API docs: https://developer.github.com/v3/issues/labels/#list-labels-on-an-issue
    pub async fn list_labels_on_issue(
        &self,
        owner: &str,
        repo: &str,
        issue_number: u64,
        options: Option<PaginationOptions>,
    ) -> Result<Response<Vec<Label>>> {
        let url = format!("repos/{}/{}/issues/{}/labels", owner, repo, issue_number);
        let response = self.inner.get(&url).query(&options).send().await?;

        self.inner.json(response).await
    }

    /// Add labels to an issue
    ///
    /// GitHub API docs: https://developer.github.com/v3/issues/labels/#add-labels-to-an-issue
    pub async fn add_lables(
        &self,
        owner: &str,
        repo: &str,
        issue_number: u64,
        labels: Vec<String>,
    ) -> Result<Response<Vec<Label>>> {
        #[derive(Debug, Serialize)]
        struct AddLabelRequest {
            labels: Vec<String>,
        }

        let request = AddLabelRequest { labels };
        let url = format!("repos/{}/{}/issues/{}/labels", owner, repo, issue_number);
        let response = self.inner.post(&url).json(&request).send().await?;

        self.inner.json(response).await
    }

    /// Remove a label from an issue
    ///
    /// GitHub API docs: https://developer.github.com/v3/issues/labels/#remove-a-label-from-an-issue
    pub async fn remove_label(
        &self,
        owner: &str,
        repo: &str,
        issue_number: u64,
        label: &str,
    ) -> Result<Response<Vec<Label>>> {
        let url = format!(
            "repos/{}/{}/issues/{}/labels/{}",
            owner, repo, issue_number, label
        );
        let response = self.inner.delete(&url).send().await?;

        self.inner.json(response).await
    }

    /// Replace all labels on an issue
    ///
    /// GitHub API docs: https://developer.github.com/v3/issues/labels/#replace-all-labels-for-an-issue
    pub async fn replace_all_labels(
        &self,
        owner: &str,
        repo: &str,
        issue_number: u64,
        labels: Vec<String>,
    ) -> Result<Response<Vec<Label>>> {
        #[derive(Debug, Serialize)]
        struct ReplaceLabelRequest {
            labels: Vec<String>,
        }

        let request = ReplaceLabelRequest { labels };
        let url = format!("repos/{}/{}/issues/{}/labels", owner, repo, issue_number);
        let response = self.inner.put(&url).json(&request).send().await?;

        self.inner.json(response).await
    }

    /// Removes all labels on an issue
    ///
    /// GitHub API docs: https://developer.github.com/v3/issues/labels/#remove-all-labels-from-an-issue
    pub async fn remove_all_labels(
        &self,
        owner: &str,
        repo: &str,
        issue_number: u64,
    ) -> Result<Response<()>> {
        let url = format!("repos/{}/{}/issues/{}/labels", owner, repo, issue_number);
        let response = self.inner.delete(&url).send().await?;

        self.inner.empty(response).await
    }

    /// List labels for every issue in a milestone
    ///
    /// GitHub API docs: https://developer.github.com/v3/issues/labels/#get-labels-for-every-issue-in-a-milestone
    pub async fn list_labels_for_milestone(
        &self,
        owner: &str,
        repo: &str,
        milestone_number: u64,
        options: Option<PaginationOptions>,
    ) -> Result<Response<Vec<Label>>> {
        let url = format!(
            "repos/{}/{}/milestones/{}/labels",
            owner, repo, milestone_number
        );
        let response = self.inner.get(&url).query(&options).send().await?;

        self.inner.json(response).await
    }

    // TODO
    // Milestone Endpoint
    // https://developer.github.com/v3/issues/milestones/

    // TODO
    // Timeline Endpoint
    // https://developer.github.com/v3/issues/timeline/
}
