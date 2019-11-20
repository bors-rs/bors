use super::{DateTime, Label, Milestone, NodeId, Oid, Repository, Team, User};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct CommitRef {
    pub label: String,
    #[serde(rename = "ref")]
    pub git_ref: String,
    pub sha: Oid,
    pub user: User,
    pub repo: Repository,
}

#[derive(Debug, Deserialize)]
pub struct PullRequest {
    pub url: String,
    pub id: u64,
    pub node_id: NodeId,
    pub html_url: String,
    pub diff_url: String,
    pub patch_url: String,
    pub issue_url: String,
    pub number: u64,
    pub state: String,
    pub locked: bool,
    pub title: String,
    pub user: User,
    pub body: Option<String>,
    pub created_at: DateTime,
    pub updated_at: DateTime,
    pub closed_at: Option<DateTime>,
    pub merged_at: Option<DateTime>,
    pub merge_commit_sha: Option<Oid>,
    pub assignee: Option<User>,
    pub assignees: Vec<User>,
    pub requested_reviewers: Vec<User>,
    pub requested_teams: Vec<Team>,
    pub labels: Vec<Label>,
    pub milestone: Option<Milestone>,
    pub commits_url: String,
    pub review_comments_url: String,
    pub review_comment_url: String,
    pub comments_url: String,
    pub statuses_url: String,
    pub head: CommitRef,
    pub base: CommitRef,
    // pub _links: ???
    pub author_association: String,
    pub draft: bool,
    pub merged: bool,
    pub mergeable: Option<bool>,
    pub rebaseable: Option<bool>,
    pub mergeable_state: String,
    pub merged_by: Option<User>,
    pub comments: u64,
    pub review_comments: u64,
    pub maintainer_can_modify: bool,
    pub commits: u64,
    pub additions: u64,
    pub deletions: u64,
    pub changed_files: u64,
}

#[derive(Debug, Deserialize)]
pub struct Review {
    pub id: u64,
    pub node_id: NodeId,
    pub user: User,
    pub body: Option<String>,
    pub commit_id: Oid,
    pub submitted_at: DateTime,
    pub state: String, // Maybe make a type for this
    pub html_url: String,
    pub pull_request_url: String,
    pub author_association: String,
    // pub _links
}

#[derive(Debug, Deserialize)]
pub struct ReviewComment {
    pub url: String,
    pub id: u64,
    pub node_id: NodeId,
    pub pull_request_review_id: u64,
    pub diff_hunk: String,
    pub path: String,
    pub position: u64,
    pub original_position: u64,
    pub commit_id: Oid,
    pub original_commit_id: Oid,
    pub in_reply_to_id: u64,
    pub user: User,
    pub body: Option<String>,
    pub created_at: DateTime,
    pub updated_at: DateTime,
    pub html_url: String,
    pub pull_request_url: String,
    pub author_association: String,
    // pub _links
    pub start_line: u64,
    pub original_start_line: u64,
    pub start_side: String,
    pub line: u64,
    pub original_line: u64,
    pub side: String,
}

#[cfg(test)]
mod test {
    use super::PullRequest;

    #[test]
    fn pull_request() {
        const PR_JSON: &str = include_str!("../test-input/pr.json");
        let _pr: PullRequest = serde_json::from_str(PR_JSON).unwrap();
    }
}
