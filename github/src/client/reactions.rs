use crate::{
    client::{Client, PaginationOptions, Response, Result, MEDIA_TYPE_REACTIONS_PREVIEW},
    Reaction, ReactionType,
};
use serde::Serialize;

#[derive(Debug, Serialize)]
struct ReactionsRequest {
    content: ReactionType,
}

#[derive(Debug, Default, Serialize)]
pub struct ListReactionsOptions {
    pub content: Option<ReactionType>,

    #[serde(flatten)]
    pub pagination_options: PaginationOptions,
}

/// `ReactionsClient` handles communication with the reactions related methods of the GitHub API.
///
/// GitHub API docs: https://developer.github.com/v3/reactions/
pub struct ReactionsClient<'a> {
    inner: &'a Client,
}

impl<'a> ReactionsClient<'a> {
    pub(super) fn new(client: &'a Client) -> Self {
        Self { inner: client }
    }

    async fn list_reactions(
        &self,
        url: &str,
        options: Option<ListReactionsOptions>,
    ) -> Result<Response<Vec<Reaction>>> {
        let response = self
            .inner
            .get(url)
            // TODO: remove custom Accept headers when APIs fully launch.
            .header(reqwest::header::ACCEPT, MEDIA_TYPE_REACTIONS_PREVIEW)
            .query(&options)
            .send()
            .await?;

        self.inner.json(response).await
    }

    async fn create_reaction(
        &self,
        url: &str,
        reaction: ReactionType,
    ) -> Result<Response<Reaction>> {
        let request = ReactionsRequest { content: reaction };

        let response = self
            .inner
            .post(url)
            // TODO: remove custom Accept headers when APIs fully launch.
            .header(reqwest::header::ACCEPT, MEDIA_TYPE_REACTIONS_PREVIEW)
            .json(&request)
            .send()
            .await?;

        self.inner.json(response).await
    }

    async fn delete_reaction(&self, url: &str) -> Result<Response<()>> {
        let response = self
            .inner
            .delete(url)
            // TODO: remove custom Accept headers when APIs fully launch.
            .header(reqwest::header::ACCEPT, MEDIA_TYPE_REACTIONS_PREVIEW)
            .send()
            .await?;

        self.inner.empty(response)
    }

    /// List the reactions for a commit comment
    ///
    /// GitHub API docs: https://developer.github.com/v3/reactions/#list-reactions-for-a-commit-comment
    pub async fn list_for_commit_comment(
        &self,
        owner: &str,
        repo: &str,
        comment_id: u64,
        options: Option<ListReactionsOptions>,
    ) -> Result<Response<Vec<Reaction>>> {
        let url = format!("repos/{}/{}/comments/{}/reactions", owner, repo, comment_id);
        self.list_reactions(&url, options).await
    }

    /// Create a reaction for a commit comment
    ///
    /// Note that if a reaction of the provided type already exists,
    /// the existing reaction will be returned.
    ///
    /// GitHub API docs: https://developer.github.com/v3/reactions/#create-reaction-for-a-commit-comment
    pub async fn create_for_commit_comment(
        &self,
        owner: &str,
        repo: &str,
        comment_id: u64,
        reaction: ReactionType,
    ) -> Result<Response<Reaction>> {
        let url = format!("repos/{}/{}/comments/{}/reactions", owner, repo, comment_id);
        self.create_reaction(&url, reaction).await
    }

    /// Delete a reaction for a commit comment
    ///
    /// GitHub API docs: https://developer.github.com/v3/reactions/#delete-a-commit-comment-reaction
    pub async fn delete_for_commit_comment(
        &self,
        owner: &str,
        repo: &str,
        comment_id: u64,
        reaction_id: u64,
    ) -> Result<Response<()>> {
        let url = format!(
            "repos/{}/{}/comments/{}/reactions/{}",
            owner, repo, comment_id, reaction_id
        );
        self.delete_reaction(&url).await
    }

    /// Delete a reaction for a commit comment with a Repository Id
    ///
    /// GitHub API docs: https://developer.github.com/v3/reactions/#delete-a-commit-comment-reaction
    pub async fn delete_for_commit_comment_with_repository_id(
        &self,
        repository_id: u64,
        comment_id: u64,
        reaction_id: u64,
    ) -> Result<Response<()>> {
        let url = format!(
            "repositories/{}/comments/{}/reactions/{}",
            repository_id, comment_id, reaction_id
        );
        self.delete_reaction(&url).await
    }

    /// List the reactions for an issue
    ///
    /// GitHub API docs: https://developer.github.com/v3/reactions/#list-reactions-for-an-issue
    pub async fn list_for_issue(
        &self,
        owner: &str,
        repo: &str,
        issue_number: usize,
        options: Option<ListReactionsOptions>,
    ) -> Result<Response<Vec<Reaction>>> {
        let url = format!("repos/{}/{}/issues/{}/reactions", owner, repo, issue_number);
        self.list_reactions(&url, options).await
    }

    /// Create a reaction for an issue
    ///
    /// Note that if a reaction of the provided type already exists,
    /// the existing reaction will be returned.
    ///
    /// GitHub API docs: https://developer.github.com/v3/reactions/#create-reaction-for-an-issue
    pub async fn create_for_issue(
        &self,
        owner: &str,
        repo: &str,
        issue_number: u64,
        reaction: ReactionType,
    ) -> Result<Response<Reaction>> {
        let url = format!("repos/{}/{}/issues/{}/reactions", owner, repo, issue_number);
        self.create_reaction(&url, reaction).await
    }

    /// Delete a reaction for an issue
    ///
    /// GitHub API docs: https://developer.github.com/v3/reactions/#delete-an-issue-reaction
    pub async fn delete_for_issue(
        &self,
        owner: &str,
        repo: &str,
        issue_number: u64,
        reaction_id: u64,
    ) -> Result<Response<()>> {
        let url = format!(
            "repos/{}/{}/issues/{}/reactions/{}",
            owner, repo, issue_number, reaction_id
        );
        self.delete_reaction(&url).await
    }

    /// Delete a reaction for an issue with a Repository Id
    ///
    /// GitHub API docs: https://developer.github.com/v3/reactions/#delete-an-issue-reaction
    pub async fn delete_for_issue_with_repository_id(
        &self,
        repository_id: u64,
        issue_number: u64,
        reaction_id: u64,
    ) -> Result<Response<()>> {
        let url = format!(
            "repositories/{}/issues/{}/reactions/{}",
            repository_id, issue_number, reaction_id
        );
        self.delete_reaction(&url).await
    }

    /// List the reactions for an issue comment
    ///
    /// GitHub API docs: https://developer.github.com/v3/reactions/#list-reactions-for-an-issue-comment
    pub async fn list_for_issue_comment(
        &self,
        owner: &str,
        repo: &str,
        comment_id: usize,
        options: Option<ListReactionsOptions>,
    ) -> Result<Response<Vec<Reaction>>> {
        let url = format!(
            "repos/{}/{}/issues/comments/{}/reactions",
            owner, repo, comment_id
        );
        self.list_reactions(&url, options).await
    }

    /// Create a reaction for an issue comment
    ///
    /// Note that if a reaction of the provided type already exists,
    /// the existing reaction will be returned.
    ///
    /// GitHub API docs: https://developer.github.com/v3/reactions/#create-reaction-for-an-issue-comment
    pub async fn create_for_issue_comment(
        &self,
        owner: &str,
        repo: &str,
        comment_id: u64,
        reaction: ReactionType,
    ) -> Result<Response<Reaction>> {
        let url = format!(
            "repos/{}/{}/issues/comments/{}/reactions",
            owner, repo, comment_id
        );
        self.create_reaction(&url, reaction).await
    }

    /// Delete a reaction for an issue comment
    ///
    /// GitHub API docs: https://developer.github.com/v3/reactions/#delete-an-issue-comment-reaction
    pub async fn delete_for_issue_comment(
        &self,
        owner: &str,
        repo: &str,
        comment_id: u64,
        reaction_id: u64,
    ) -> Result<Response<()>> {
        let url = format!(
            "repos/{}/{}/issues/comments/{}/reactions/{}",
            owner, repo, comment_id, reaction_id
        );
        self.delete_reaction(&url).await
    }

    /// Delete a reaction for an issue comment with a Repository Id
    ///
    /// GitHub API docs: https://developer.github.com/v3/reactions/#delete-an-issue-comment-reaction
    pub async fn delete_for_issue_comment_with_repository_id(
        &self,
        repository_id: u64,
        comment_id: u64,
        reaction_id: u64,
    ) -> Result<Response<()>> {
        let url = format!(
            "repositories/{}/issues/comments/{}/reactions/{}",
            repository_id, comment_id, reaction_id
        );
        self.delete_reaction(&url).await
    }

    /// List the reactions for a pull request review comment
    ///
    /// GitHub API docs: https://developer.github.com/v3/reactions/#list-reactions-for-a-pull-request-review-comment
    pub async fn list_for_pull_request_review_comment(
        &self,
        owner: &str,
        repo: &str,
        comment_id: usize,
        options: Option<ListReactionsOptions>,
    ) -> Result<Response<Vec<Reaction>>> {
        let url = format!(
            "repos/{}/{}/pulls/comments/{}/reactions",
            owner, repo, comment_id
        );
        self.list_reactions(&url, options).await
    }

    /// Create a reaction for a pull request review comment
    ///
    /// Note that if a reaction of the provided type already exists,
    /// the existing reaction will be returned.
    ///
    /// GitHub API docs: https://developer.github.com/v3/reactions/#create-reaction-for-a-pull-request-review-comment
    pub async fn create_for_pull_request_review_comment(
        &self,
        owner: &str,
        repo: &str,
        comment_id: u64,
        reaction: ReactionType,
    ) -> Result<Response<Reaction>> {
        let url = format!(
            "repos/{}/{}/pulls/comments/{}/reactions",
            owner, repo, comment_id
        );
        self.create_reaction(&url, reaction).await
    }

    /// Delete a reaction for a pull request review comment
    ///
    /// GitHub API docs: https://developer.github.com/v3/reactions/#delete-a-pull-request-comment-reaction
    pub async fn delete_for_pull_request_review_comment(
        &self,
        owner: &str,
        repo: &str,
        comment_id: u64,
        reaction_id: u64,
    ) -> Result<Response<()>> {
        let url = format!(
            "repos/{}/{}/pulls/comments/{}/reactions/{}",
            owner, repo, comment_id, reaction_id
        );
        self.delete_reaction(&url).await
    }

    /// Delete a reaction for a pull request review comment with a Repository Id
    ///
    /// GitHub API docs: https://developer.github.com/v3/reactions/#delete-a-pull-request-comment-reaction
    pub async fn delete_for_pull_request_review_comment_with_repository_id(
        &self,
        repository_id: u64,
        comment_id: u64,
        reaction_id: u64,
    ) -> Result<Response<()>> {
        let url = format!(
            "repositories/{}/pulls/comments/{}/reactions/{}",
            repository_id, comment_id, reaction_id
        );
        self.delete_reaction(&url).await
    }

    //TODO look into adding methods for interacting with Team endpoints with Org and Team Ids

    /// List the reactions for a team discussion
    ///
    /// GitHub API docs: https://developer.github.com/v3/reactions/#list-reactions-for-a-team-discussion
    pub async fn list_for_team_discussion(
        &self,
        org: &str,
        team_slug: &str,
        discussion_number: usize,
        options: Option<ListReactionsOptions>,
    ) -> Result<Response<Vec<Reaction>>> {
        let url = format!(
            "orgs/{}/teams/{}/discussions/{}/reactions",
            org, team_slug, discussion_number
        );
        self.list_reactions(&url, options).await
    }

    /// Create a reaction for a team discussion
    ///
    /// Note that if a reaction of the provided type already exists,
    /// the existing reaction will be returned.
    ///
    /// GitHub API docs: https://developer.github.com/v3/reactions/#create-reaction-for-a-team-discussion
    pub async fn create_for_team_discussion(
        &self,
        org: &str,
        team_slug: &str,
        discussion_number: u64,
        reaction: ReactionType,
    ) -> Result<Response<Reaction>> {
        let url = format!(
            "orgs/{}/teams/{}/discussions/{}/reactions",
            org, team_slug, discussion_number
        );
        self.create_reaction(&url, reaction).await
    }

    /// Delete a reaction for a team discussion
    ///
    /// GitHub API docs: https://developer.github.com/v3/reactions/#delete-team-discussion-reaction
    pub async fn delete_for_team_discussion(
        &self,
        org: &str,
        team_slug: &str,
        discussion_number: usize,
        reaction_id: u64,
    ) -> Result<Response<()>> {
        let url = format!(
            "orgs/{}/teams/{}/discussions/{}/reactions/{}",
            org, team_slug, discussion_number, reaction_id
        );
        self.delete_reaction(&url).await
    }

    /// List the reactions for a team discussion comment
    ///
    /// GitHub API docs: https://developer.github.com/v3/reactions/#list-reactions-for-a-team-discussion-comment
    pub async fn list_for_team_discussion_comment(
        &self,
        org: &str,
        team_slug: &str,
        discussion_number: usize,
        comment_number: u64,
        options: Option<ListReactionsOptions>,
    ) -> Result<Response<Vec<Reaction>>> {
        let url = format!(
            "orgs/{}/teams/{}/discussions/{}/comments/{}/reactions",
            org, team_slug, discussion_number, comment_number
        );
        self.list_reactions(&url, options).await
    }

    /// Create a reaction for a team discussion comment
    ///
    /// Note that if a reaction of the provided type already exists,
    /// the existing reaction will be returned.
    ///
    /// GitHub API docs: https://developer.github.com/v3/reactions/#create-reaction-for-a-team-discussion-comment
    pub async fn create_for_team_discussion_comment(
        &self,
        org: &str,
        team_slug: &str,
        discussion_number: u64,
        comment_number: u64,
        reaction: ReactionType,
    ) -> Result<Response<Reaction>> {
        let url = format!(
            "orgs/{}/teams/{}/discussions/{}/comments/{}/reactions",
            org, team_slug, discussion_number, comment_number
        );
        self.create_reaction(&url, reaction).await
    }

    /// Delete a reaction for a team discussion comment
    ///
    /// GitHub API docs: https://developer.github.com/v3/reactions/#delete-team-discussion-comment-reaction
    pub async fn delete_for_team_discussion_comment(
        &self,
        org: &str,
        team_slug: &str,
        discussion_number: usize,
        comment_number: u64,
        reaction_id: u64,
    ) -> Result<Response<()>> {
        let url = format!(
            "orgs/{}/teams/{}/discussions/{}/comments/{}/reactions/{}",
            org, team_slug, discussion_number, comment_number, reaction_id
        );
        self.delete_reaction(&url).await
    }
}
