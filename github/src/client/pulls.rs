use crate::{
    client::{
        Client, PaginationOptions, Response, Result, SortDirection, SortPages, StateFilter,
        MEDIA_TYPE_DRAFT_PREVIEW, MEDIA_TYPE_LOCK_REASON_PREVIEW,
        MEDIA_TYPE_MULTI_LINE_COMMENTS_PREVIEW, MEDIA_TYPE_REACTIONS_PREVIEW,
        MEDIA_TYPE_UPDATE_PULL_REQUEST_BRANCH_PREVIEW,
    },
    DateTime, PullRequest, Review, ReviewComment, Team, User,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize)]
pub struct ListPullsOptions {
    /// Indicates the state of the pull requests to return. Default: open
    pub state: Option<StateFilter>,

    /// Filter pulls by head user or head organization and branch name in the format of
    /// user:ref-name or organization:ref-name. For example: github:new-script-format or
    /// octocat:test-branch.
    pub head: Option<String>,

    /// Filter pulls by base branch name. Example: gh-pages.
    pub base: Option<String>,

    /// What to sort results by. Default: created
    pub sort: Option<SortPages>,

    /// The direction of the sort. Default: desc
    pub direction: Option<SortDirection>,

    #[serde(flatten)]
    pub pagination_options: PaginationOptions,
}

#[derive(Debug, Default, Serialize)]
pub struct NewPullRequest {
    /// The title of the Pull Request
    pub title: String,

    /// The contents of the pull request cover letter
    pub body: Option<String>,

    /// The name of the branch where your changes are implemented. For cross-repository pull
    /// requests in the same network, namespace head with a user like this: username:branch.
    pub head: String,

    /// The name of the branch you want the changes pulled into. This should be an existing branch
    /// on the current repository. You cannot submit a pull request to one repository that requests
    /// a merge to a base of another repository.
    pub base: String,

    /// Indicates whether maintainers can modify the pull request
    pub maintainer_can_modify: Option<bool>,

    /// Indicates whether the pull request is a draft
    pub draft: Option<bool>,
}

#[derive(Debug, Default, Serialize)]
pub struct UpdatePullRequest {
    /// The title of the Pull Request
    pub title: Option<String>,

    /// The contents of the pull request cover letter
    pub body: Option<String>,

    /// The name of the branch where your changes are implemented. For cross-repository pull
    /// requests in the same network, namespace head with a user like this: username:branch.
    pub head: Option<String>,

    /// The name of the branch you want the changes pulled into. This should be an existing branch
    /// on the current repository. You cannot submit a pull request to one repository that requests
    /// a merge to a base of another repository.
    pub base: Option<String>,

    /// Indicates whether maintainers can modify the pull request
    pub maintainer_can_modify: Option<bool>,

    /// Indicates whether the pull request is a draft
    pub draft: Option<bool>,
}

#[derive(Debug, Default, Deserialize)]
pub struct Reviewers {
    users: Vec<User>,
    teams: Vec<Team>,
}

// The response from the update branch endpoint
#[derive(Debug, Default, Deserialize)]
pub struct PullRequestBranchUpdateResponse {
    message: String,
    url: String,
}

#[derive(Debug, Default, Serialize)]
pub struct MergePullRequest {
    /// Title for the automatic commit message
    commit_title: String,
    /// Extra detail to append to automatic commit message
    commit_message: Option<String>,
    /// Merge method to use. Possible values are merge, squash or rebase
    merge_method: MergeMethod,
    /// SHA that pull request head must match to allow merge
    sha: String,
}

#[derive(Debug, Serialize)]
pub enum MergeMethod {
    Merge,
    Squash,
    Rebase,
}

impl Default for MergeMethod {
    fn default() -> Self {
        MergeMethod::Merge
    }
}

#[derive(Debug, Default, Deserialize)]
pub struct MergePullRequestResponse {
    sha: String,
    merged: bool,
    message: String,
}

// A comment on part of a PullRequest review
#[derive(Debug, Default, Serialize)]
pub struct DraftReviewComment {
    /// The relative path to the file that necessitates a review comment.
    path: String,

    /// The position in the diff where you want to add a review comment. Note this value is not the
    /// same as the line number in the file.
    position: usize,

    /// Text of the review comment
    body: String,
}

#[derive(Debug, Default, Serialize)]
pub struct PullRequestReviewRequest {
    /// The SHA of the commit that needs a review. Not using the latest commit SHA may render your
    /// review comment outdated if a subsequent commit modifies the line you specify as the
    /// position. Defaults to the most recent commit in the pull request when you do not specify a
    /// value.
    commit_id: String,

    /// Required when using REQUEST_CHANGES or COMMENT for the event parameter. The body text of
    /// the pull request review.
    body: String,

    /// The review action you want to perform. The review actions include: APPROVE,
    /// REQUEST_CHANGES, or COMMENT. By leaving this blank, you set the review action state to
    /// PENDING, which means you will need to submit the pull request review when you are ready.
    event: String,

    comments: Option<Vec<DraftReviewComment>>,
}

#[derive(Debug, Default, Serialize)]
pub struct ListReviewCommentsOptions {
    /// What to sort results by. Default: created
    pub sort: SortPages,

    /// The direction of the sort. Default: desc
    pub direction: SortDirection,

    /// Only issues updated at or after this time are returned. This is a timestamp in ISO 8601 format: YYYY-MM-DDTHH:MM:SSZ.
    pub since: Option<DateTime>,

    #[serde(flatten)]
    pub pagination_options: PaginationOptions,
}

//TODO maybe use an enum for the different types, multi-line, single line
#[derive(Debug, Default, Serialize)]
pub struct CreateReviewCommentRequest {
    /// Required. The text of the review comment.
    body: String,

    /// Required. The SHA of the commit needing a comment. Not using the latest commit SHA may
    /// render your comment outdated if a subsequent commit modifies the line you specify as the
    /// position.
    commit_id: String,

    /// Required. The relative path to the file that necessitates a comment.
    path: String,

    /// Required without comfort-fade preview. The position in the diff where you want to add a
    /// review comment. Note this value is not the same as the line number in the file. For help
    /// finding the position value, read the note above.
    position: Option<usize>,

    /// Required with comfort-fade preview. In a split diff view, the side of the diff that the
    /// pull request's changes appear on. Can be LEFT or RIGHT. Use LEFT for deletions that appear
    /// in red. Use RIGHT for additions that appear in green or unchanged lines that appear in
    /// white and are shown for context. For a multi-line comment, side represents whether the last
    /// line of the comment range is a deletion or addition. For more information, see "Diff view
    /// options" in the GitHub Help documentation.
    side: Option<String>,
    /// Required with comfort-fade preview. The line of the blob in the pull request diff that the
    /// comment applies to. For a multi-line comment, the last line of the range that your comment
    /// applies to.
    line: Option<usize>,
    /// Required when using multi-line comments. To create multi-line comments, you must use the
    /// comfort-fade preview header. The start_line is the first line in the pull request diff that
    /// your multi-line comment applies to. To learn more about multi-line comments, see
    /// "Commenting on a pull request" in the GitHub Help documentation.
    start_line: Option<usize>,
    /// Required when using multi-line comments. To create multi-line comments, you must use the
    /// comfort-fade preview header. The start_side is the starting side of the diff that the
    /// comment applies to. Can be LEFT or RIGHT. To learn more about multi-line comments, see
    /// "Commenting on a pull request" in the GitHub Help documentation. See side in this table for
    /// additional context.
    start_side: Option<String>,
}

/// `PullsClient` handles communication with the pull request related methods of the GitHub API.
///
/// GitHub API docs: https://developer.github.com/v3/pulls/
pub struct PullsClient<'a> {
    inner: &'a Client,
}

impl<'a> PullsClient<'a> {
    pub(super) fn new(client: &'a Client) -> Self {
        Self { inner: client }
    }

    /// List pull requests
    ///
    /// GitHub API docs: https://developer.github.com/v3/pulls/#list-pull-requests
    pub async fn list(
        &self,
        owner: &str,
        repo: &str,
        options: Option<ListPullsOptions>,
    ) -> Result<Response<Vec<PullRequest>>> {
        let url = format!("repos/{}/{}/pulls", owner, repo);
        let response = self
            .inner
            .get(&url)
            // For the 'lock_reason' object
            .header(reqwest::header::ACCEPT, MEDIA_TYPE_LOCK_REASON_PREVIEW)
            // For the 'draft' parameter
            .header(reqwest::header::ACCEPT, MEDIA_TYPE_DRAFT_PREVIEW)
            .query(&options)
            .send()
            .await?;

        self.inner.json(response).await
    }

    /// Get a pull request
    ///
    /// GitHub API docs: https://developer.github.com/v3/pulls/#get-a-single-pull-request
    pub async fn get(
        &self,
        owner: &str,
        repo: &str,
        pull_number: u64,
    ) -> Result<Response<PullRequest>> {
        let url = format!("repos/{}/{}/pulls/{}", owner, repo, pull_number);
        let response = self
            .inner
            .get(&url)
            // For the 'lock_reason' object
            .header(reqwest::header::ACCEPT, MEDIA_TYPE_LOCK_REASON_PREVIEW)
            // For the 'draft' parameter
            .header(reqwest::header::ACCEPT, MEDIA_TYPE_DRAFT_PREVIEW)
            .send()
            .await?;

        self.inner.json(response).await
    }

    /// Create a pull request
    ///
    /// GitHub API docs: https://developer.github.com/v3/pulls/#create-a-pull-request
    pub async fn create(
        &self,
        owner: &str,
        repo: &str,
        pull_request: NewPullRequest,
    ) -> Result<Response<PullRequest>> {
        let url = format!("repos/{}/{}/pulls", owner, repo);
        let response = self
            .inner
            .post(&url)
            // For the 'draft' parameter
            .header(reqwest::header::ACCEPT, MEDIA_TYPE_DRAFT_PREVIEW)
            .json(&pull_request)
            .send()
            .await?;

        self.inner.json(response).await
    }

    /// Update a pull request
    ///
    /// GitHub API docs: https://developer.github.com/v3/pulls/#update-a-pull-request
    pub async fn update(
        &self,
        owner: &str,
        repo: &str,
        pull_number: u64,
        pull_request: UpdatePullRequest,
    ) -> Result<Response<PullRequest>> {
        let url = format!("repos/{}/{}/pulls/{}", owner, repo, pull_number);
        let response = self
            .inner
            .post(&url)
            // For the 'lock_reason' object
            .header(reqwest::header::ACCEPT, MEDIA_TYPE_LOCK_REASON_PREVIEW)
            // For the 'draft' parameter
            .header(reqwest::header::ACCEPT, MEDIA_TYPE_DRAFT_PREVIEW)
            .json(&pull_request)
            .send()
            .await?;

        self.inner.json(response).await
    }

    /// Updates the pull request branch with the latest upstream changes by merging HEAD from the
    /// base branch into the pull request branch.
    ///
    /// GitHub API docs: https://developer.github.com/v3/pulls/#update-a-pull-request-branch
    pub async fn update_branch(
        &self,
        owner: &str,
        repo: &str,
        pull_number: u64,
        expected_head_sha: Option<String>,
    ) -> Result<Response<PullRequestBranchUpdateResponse>> {
        #[derive(Debug, Serialize)]
        pub struct UpdateBranchRequest {
            expected_head_sha: Option<String>,
        }

        let request = UpdateBranchRequest { expected_head_sha };
        let url = format!("repos/{}/{}/pulls/{}", owner, repo, pull_number);
        let response = self
            .inner
            .post(&url)
            // Enable this preview endpoint
            .header(
                reqwest::header::ACCEPT,
                MEDIA_TYPE_UPDATE_PULL_REQUEST_BRANCH_PREVIEW,
            )
            .json(&request)
            .send()
            .await?;

        self.inner.json(response).await
    }

    //TODO add a RepositoryCommit type
    /// List commits on a pull request
    ///
    /// GitHub API docs: https://developer.github.com/v3/pulls/#list-commits-on-a-pull-request
    pub async fn list_commits(
        &self,
        owner: &str,
        repo: &str,
        pull_number: u64,
        options: Option<PaginationOptions>,
        //) -> Result<Response<Vec<RepositoryCommit>>> {
    ) -> Result<Response<()>> {
        let url = format!("repos/{}/{}/pulls/{}/commits", owner, repo, pull_number);
        let response = self.inner.get(&url).query(&options).send().await?;

        //self.inner.json(response).await
        self.inner.empty(response).await
    }

    //TODO add CommitFile type
    /// List files on a pull request
    ///
    /// GitHub API docs: https://developer.github.com/v3/pulls/#list-pull-requests-files
    pub async fn list_files(
        &self,
        owner: &str,
        repo: &str,
        pull_number: u64,
        options: Option<PaginationOptions>,
        //) -> Result<Response<Vec<CommitFile>>> {
    ) -> Result<Response<()>> {
        let url = format!("repos/{}/{}/pulls/{}/files", owner, repo, pull_number);
        let response = self.inner.get(&url).query(&options).send().await?;

        //self.inner.json(response).await
        self.inner.empty(response).await
    }

    /// Check if a pull request has been merged
    ///
    /// GitHub API docs: https://developer.github.com/v3/pulls/#get-if-a-pull-request-has-been-merged
    pub async fn is_merged(
        &self,
        owner: &str,
        repo: &str,
        pull_number: u64,
    ) -> Result<Response<bool>> {
        let url = format!("repos/{}/{}/pulls/{}/merge", owner, repo, pull_number);
        let response = self.inner.get(&url).send().await?;

        self.inner.boolean(response).await
    }

    /// Merge a pull request (hit the Merge Button)
    ///
    /// GitHub API docs: https://developer.github.com/v3/pulls/#merge-a-pull-request-merge-button
    pub async fn merge(
        &self,
        owner: &str,
        repo: &str,
        pull_number: u64,
        request: MergePullRequest,
    ) -> Result<Response<MergePullRequestResponse>> {
        let url = format!("repos/{}/{}/pulls/{}/merge", owner, repo, pull_number);
        let response = self.inner.put(&url).json(&request).send().await?;

        self.inner.json(response).await
    }

    // Review Endpoint
    // https://developer.github.com/v3/pulls/reviews/

    /// List reviews on a pull request
    ///
    /// GitHub API docs: https://developer.github.com/v3/pulls/reviews/#list-reviews-on-a-pull-request
    pub async fn list_reviews(
        &self,
        owner: &str,
        repo: &str,
        pull_number: u64,
        options: Option<PaginationOptions>,
    ) -> Result<Response<Vec<Review>>> {
        let url = format!("repos/{}/{}/pulls/{}/reviews", owner, repo, pull_number);
        let response = self.inner.get(&url).query(&options).send().await?;

        self.inner.json(response).await
    }

    /// Get a single review on a pull request
    ///
    /// GitHub API docs: https://developer.github.com/v3/pulls/reviews/#get-a-single-review
    pub async fn get_review(
        &self,
        owner: &str,
        repo: &str,
        pull_number: u64,
        review_id: u64,
    ) -> Result<Response<Review>> {
        let url = format!(
            "repos/{}/{}/pulls/{}/reviews/{}",
            owner, repo, pull_number, review_id
        );
        let response = self.inner.get(&url).send().await?;

        self.inner.json(response).await
    }

    /// Delete a pending review
    ///
    /// GitHub API docs: https://developer.github.com/v3/pulls/reviews/#delete-a-pending-review
    pub async fn delete_pending_review(
        &self,
        owner: &str,
        repo: &str,
        pull_number: u64,
        review_id: u64,
    ) -> Result<Response<Review>> {
        let url = format!(
            "repos/{}/{}/pulls/{}/reviews/{}",
            owner, repo, pull_number, review_id
        );
        let response = self.inner.delete(&url).send().await?;

        self.inner.json(response).await
    }

    /// List review comments for a particular review on a pull request
    ///
    /// GitHub API docs: https://developer.github.com/v3/pulls/reviews/#get-comments-for-a-single-review
    pub async fn get_comments_for_review(
        &self,
        owner: &str,
        repo: &str,
        pull_number: u64,
        review_id: u64,
        options: Option<PaginationOptions>,
    ) -> Result<Response<Vec<ReviewComment>>> {
        let url = format!(
            "repos/{}/{}/pulls/{}/reviews/{}/comments",
            owner, repo, pull_number, review_id
        );
        let response = self.inner.get(&url).query(&options).send().await?;

        self.inner.json(response).await
    }

    /// Create a Review
    ///
    /// GitHub API docs: https://developer.github.com/v3/pulls/reviews/#create-a-pull-request-review
    pub async fn create_review(
        &self,
        owner: &str,
        repo: &str,
        pull_number: u64,
        pull_request_review: PullRequestReviewRequest,
    ) -> Result<Response<Review>> {
        let url = format!("repos/{}/{}/pulls/{}/reviews", owner, repo, pull_number);
        let response = self
            .inner
            .post(&url)
            .json(&pull_request_review)
            .send()
            .await?;

        self.inner.json(response).await
    }

    /// Update a review summary
    ///
    /// GitHub API docs: https://developer.github.com/v3/pulls/reviews/#update-a-pull-request-review
    pub async fn update_review(
        &self,
        owner: &str,
        repo: &str,
        pull_number: u64,
        review_id: u64,
        body: String,
    ) -> Result<Response<Review>> {
        #[derive(Debug, Serialize)]
        struct UpdateReviewRequest {
            body: String,
        }

        let request = UpdateReviewRequest { body };
        let url = format!(
            "repos/{}/{}/pulls/{}/reviews/{}",
            owner, repo, pull_number, review_id
        );
        let response = self.inner.put(&url).json(&request).send().await?;

        self.inner.json(response).await
    }

    /// Submit a Review
    ///
    /// GitHub API docs: https://developer.github.com/v3/pulls/reviews/#submit-a-pull-request-review
    pub async fn submit_review(
        &self,
        owner: &str,
        repo: &str,
        pull_number: u64,
        review_id: u64,
        body: String,
        event: String,
    ) -> Result<Response<Review>> {
        #[derive(Debug, Serialize)]
        struct SubmitReviewRequest {
            body: String,
            event: String,
        }

        let request = SubmitReviewRequest { body, event };
        let url = format!(
            "repos/{}/{}/pulls/{}/reviews/{}/events",
            owner, repo, pull_number, review_id
        );
        let response = self.inner.post(&url).json(&request).send().await?;

        self.inner.json(response).await
    }

    /// Dismiss a Review
    ///
    /// GitHub API docs: https://developer.github.com/v3/pulls/reviews/#dismiss-a-pull-request-review
    pub async fn dismiss_review(
        &self,
        owner: &str,
        repo: &str,
        pull_number: u64,
        review_id: u64,
    ) -> Result<Response<Review>> {
        let url = format!(
            "repos/{}/{}/pulls/{}/reviews/{}/dismissals",
            owner, repo, pull_number, review_id
        );
        let response = self.inner.put(&url).send().await?;

        self.inner.json(response).await
    }

    // Review Comments Endpoint
    // https://developer.github.com/v3/pulls/comments/

    /// List review comments for a Pull Request
    ///
    /// GitHub API docs: https://developer.github.com/v3/pulls/comments/#list-comments-on-a-pull-request
    pub async fn list_review_comments(
        &self,
        owner: &str,
        repo: &str,
        pull_number: u64,
        options: Option<ListReviewCommentsOptions>,
    ) -> Result<Response<Vec<ReviewComment>>> {
        let url = format!("repos/{}/{}/pulls/{}/comments", owner, repo, pull_number);
        let response = self
            .inner
            .get(&url)
            // For the multi line comments
            .header(
                reqwest::header::ACCEPT,
                MEDIA_TYPE_MULTI_LINE_COMMENTS_PREVIEW,
            )
            // For the 'reactions' reaction summary object
            .header(reqwest::header::ACCEPT, MEDIA_TYPE_REACTIONS_PREVIEW)
            .query(&options)
            .send()
            .await?;

        self.inner.json(response).await
    }

    /// List review comments for a repository
    ///
    /// GitHub API docs: https://developer.github.com/v3/pulls/comments/#list-comments-in-a-repository
    pub async fn list_review_comments_for_repo(
        &self,
        owner: &str,
        repo: &str,
        options: Option<ListReviewCommentsOptions>,
    ) -> Result<Response<Vec<ReviewComment>>> {
        let url = format!("repos/{}/{}/pulls/comments", owner, repo);
        let response = self
            .inner
            .get(&url)
            // For the multi line comments
            .header(
                reqwest::header::ACCEPT,
                MEDIA_TYPE_MULTI_LINE_COMMENTS_PREVIEW,
            )
            // For the 'reactions' reaction summary object
            .header(reqwest::header::ACCEPT, MEDIA_TYPE_REACTIONS_PREVIEW)
            .query(&options)
            .send()
            .await?;

        self.inner.json(response).await
    }

    /// Get a single review commnet
    ///
    /// GitHub API docs: https://developer.github.com/v3/pulls/comments/#get-a-single-comment
    pub async fn get_review_comment(
        &self,
        owner: &str,
        repo: &str,
        comment_id: u64,
    ) -> Result<Response<ReviewComment>> {
        let url = format!("repos/{}/{}/pulls/comments/{}", owner, repo, comment_id);
        let response = self
            .inner
            .get(&url)
            // For the multi line comments
            .header(
                reqwest::header::ACCEPT,
                MEDIA_TYPE_MULTI_LINE_COMMENTS_PREVIEW,
            )
            // For the 'reactions' reaction summary object
            .header(reqwest::header::ACCEPT, MEDIA_TYPE_REACTIONS_PREVIEW)
            .send()
            .await?;

        self.inner.json(response).await
    }

    /// Create a review comment
    ///
    /// GitHub API docs: https://developer.github.com/v3/pulls/comments/#create-a-comment
    pub async fn create_review_comment(
        &self,
        owner: &str,
        repo: &str,
        pull_number: u64,
        review_request: CreateReviewCommentRequest,
    ) -> Result<Response<ReviewComment>> {
        let url = format!("repos/{}/{}/pulls/{}/comments", owner, repo, pull_number);
        let response = self
            .inner
            .post(&url)
            // For the multi line comments
            .header(
                reqwest::header::ACCEPT,
                MEDIA_TYPE_MULTI_LINE_COMMENTS_PREVIEW,
            )
            .json(&review_request)
            .send()
            .await?;

        self.inner.json(response).await
    }

    /// Create a review comment reply
    ///
    /// GitHub API docs: https://developer.github.com/v3/pulls/comments/#create-a-review-comment-reply
    pub async fn create_review_comment_reply(
        &self,
        owner: &str,
        repo: &str,
        pull_number: u64,
        comment_id: u64,
        body: &str,
    ) -> Result<Response<ReviewComment>> {
        #[derive(Debug, Serialize)]
        struct CreateReviewCommentReplyRequest<'a> {
            body: &'a str,
        }

        let request = CreateReviewCommentReplyRequest { body };
        let url = format!(
            "repos/{}/{}/pulls/{}/comments/{}/replies",
            owner, repo, pull_number, comment_id
        );
        let response = self
            .inner
            .post(&url)
            // For the multi line comments
            .header(
                reqwest::header::ACCEPT,
                MEDIA_TYPE_MULTI_LINE_COMMENTS_PREVIEW,
            )
            .json(&request)
            .send()
            .await?;

        self.inner.json(response).await
    }

    /// Edit a review comment
    ///
    /// GitHub API docs: https://developer.github.com/v3/pulls/comments/#edit-a-comment
    pub async fn edit_review_comment(
        &self,
        owner: &str,
        repo: &str,
        comment_id: u64,
        body: &str,
    ) -> Result<Response<ReviewComment>> {
        #[derive(Debug, Serialize)]
        struct EditReviewCommentRequest<'a> {
            body: &'a str,
        }

        let request = EditReviewCommentRequest { body };
        let url = format!("repos/{}/{}/pulls/comments/{}", owner, repo, comment_id);
        let response = self
            .inner
            .patch(&url)
            // For the multi line comments
            .header(
                reqwest::header::ACCEPT,
                MEDIA_TYPE_MULTI_LINE_COMMENTS_PREVIEW,
            )
            .json(&request)
            .send()
            .await?;

        self.inner.json(response).await
    }

    /// Delete a review comment
    ///
    /// GitHub API docs: https://developer.github.com/v3/pulls/comments/#delete-a-comment
    pub async fn delete_review_comment(
        &self,
        owner: &str,
        repo: &str,
        comment_id: u64,
    ) -> Result<Response<()>> {
        let url = format!("repos/{}/{}/pulls/comments/{}", owner, repo, comment_id);
        let response = self.inner.delete(&url).send().await?;

        self.inner.empty(response).await
    }

    // PullRequest Review Request endpoint
    // https://developer.github.com/v3/pulls/review_requests/

    /// List pull requests
    ///
    /// GitHub API docs: https://developer.github.com/v3/pulls/review_requests/#list-review-requests
    pub async fn list_reviewers(
        &self,
        owner: &str,
        repo: &str,
        pull_number: u64,
        options: Option<PaginationOptions>,
    ) -> Result<Response<Reviewers>> {
        let url = format!("repos/{}/{}/pulls/{}", owner, repo, pull_number);
        let response = self.inner.get(&url).query(&options).send().await?;

        self.inner.json(response).await
    }

    /// Request a Review Request
    ///
    /// GitHub API docs: https://developer.github.com/v3/pulls/review_requests/#create-a-review-request
    pub async fn create_review_request(
        &self,
        owner: &str,
        repo: &str,
        pull_number: u64,
        reviewers: Vec<String>,
        team_reviwers: Vec<String>,
    ) -> Result<Response<PullRequest>> {
        #[derive(Debug, Serialize)]
        struct CreateReviewRequest {
            reviewers: Vec<String>,
            team_reviwers: Vec<String>,
        }

        let request = CreateReviewRequest {
            reviewers,
            team_reviwers,
        };

        let url = format!(
            "repos/{}/{}/pulls/{}/requested_reviewers",
            owner, repo, pull_number
        );
        let response = self.inner.post(&url).json(&request).send().await?;

        self.inner.json(response).await
    }

    /// Remove a Review Request
    ///
    /// GitHub API docs: https://developer.github.com/v3/pulls/review_requests/#delete-a-review-request
    pub async fn remove_review_request(
        &self,
        owner: &str,
        repo: &str,
        pull_number: u64,
        reviewers: Vec<String>,
        team_reviwers: Vec<String>,
    ) -> Result<Response<PullRequest>> {
        #[derive(Debug, Serialize)]
        struct RemoveReviewRequest {
            reviewers: Vec<String>,
            team_reviwers: Vec<String>,
        }

        let request = RemoveReviewRequest {
            reviewers,
            team_reviwers,
        };

        let url = format!(
            "repos/{}/{}/pulls/{}/requested_reviewers",
            owner, repo, pull_number
        );
        let response = self.inner.delete(&url).json(&request).send().await?;

        self.inner.json(response).await
    }
}
