use super::{DateTime, NodeId, ReactionSummary, Repository, User};
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
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
    pub milestone: Option<Milestone>,
    pub reactions: Option<ReactionSummary>,
    pub locked: bool,
    pub active_lock_reason: Option<String>,
    pub comments: u64,
    pub pull_request: Option<PullRequestRef>,
    pub closed_at: Option<DateTime>,
    pub created_at: DateTime,
    pub updated_at: DateTime,
    pub repository: Option<Repository>,
}

impl Issue {
    pub fn is_pull_request(&self) -> bool {
        self.pull_request.is_some()
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct PullRequestRef {
    pub url: String,
    pub html_url: String,
    pub diff_url: String,
    pub patch_url: String,
}

#[derive(Clone, Debug, Deserialize)]
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
    pub created_at: DateTime,
    pub updated_at: DateTime,
    pub closed_at: Option<DateTime>,
    pub due_on: DateTime,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Comment {
    pub id: u64,
    pub node_id: NodeId,
    pub url: String,
    pub html_url: String,
    pub body: Option<String>,
    pub reactions: Option<ReactionSummary>,
    pub user: User,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

impl Comment {
    pub fn body(&self) -> Option<&str> {
        self.body.as_ref().map(AsRef::as_ref)
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct Label {
    pub id: u64,
    pub node_id: NodeId,
    pub url: String,
    pub name: String,
    pub description: Option<String>,
    pub color: String,
    pub default: bool,
}
