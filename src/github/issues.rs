use super::{Date, NodeId, Repository, User};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Issue {
    pub id: u64,
    pub node_id: NodeId,
    pub url: String,
    pub repository_url: String,
    pub labels_url: String,
    pub comments_url: String,
    pub events_url: String,
    pub html_url: String,
    pub number: u64,
    pub state: String,
    pub title: String,
    pub body: Option<String>,
    pub user: User,
    pub labels: Vec<Label>,
    pub assignee: Option<User>,
    pub assignees: Vec<User>,
    pub milestone: Milestone,
    pub locked: bool,
    pub active_lock_reason: Option<String>,
    pub comments: u64,
    pub pull_request: Option<PullRequestRef>,
    pub closed_at: Option<Date>,
    pub created_at: Date,
    pub updated_at: Date,
    pub repository: Repository,
}

#[derive(Debug, Deserialize)]
pub struct PullRequestRef {
    pub url: String,
    pub html_url: String,
    pub diff_url: String,
    pub patch_url: String,
}

#[derive(Debug, Deserialize)]
pub struct Milestone {
    pub url: String,
    pub html_url: String,
    pub labels_url: String,
    pub id: u64,
    pub node_id: NodeId,
    pub number: u64,
    pub state: String,
    pub title: String,
    pub description: Option<String>,
    pub creator: User,
    pub open_issues: u64,
    pub closed_issues: u64,
    pub created_at: Date,
    pub updated_at: Date,
    pub closed_at: Option<Date>,
    pub due_on: Date,
}

#[derive(Debug, Deserialize)]
pub struct Comment {
    pub id: u64,
    pub node_id: NodeId,
    pub url: String,
    pub html_url: String,
    pub body: Option<String>,
    pub user: User,
    pub created_at: Date,
    pub updated_at: Date,
}

#[derive(Debug, Deserialize)]
pub struct Label {
    pub id: u64,
    pub node_id: NodeId,
    pub url: String,
    pub name: String,
    pub description: String,
    pub color: String,
    pub default: bool,
}
