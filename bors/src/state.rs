use github::Oid;
use serde::Deserialize;
use std::collections::HashSet;

#[derive(Debug)]
pub struct PullRequestState {
    pub number: u64,
    pub author: Option<String>,
    pub title: String,
    pub body: String,

    pub head_ref_oid: Oid,
    pub head_ref_name: String,
    /// (owner, name)
    pub head_repo: Option<Repo>,

    //pub base_repo: Option<Repo>,
    pub base_ref_name: String,
    pub base_ref_oid: Oid,

    pub state: github::PullRequestState,
    pub is_draft: bool,
    pub approved_by: HashSet<String>,
    pub maintainer_can_modify: bool, // Use to enable 'rebase' merging and having github know a PR has been merged
    pub mergeable: bool,
    pub labels: Vec<String>,

    //tests_started_at: Instant,
    pub priority: u32,
    pub delegate: bool,
    pub merge_oid: Option<Oid>,
}

#[derive(Debug, Deserialize)]
pub struct Repo {
    owner: String,
    name: String,
}

impl Repo {
    pub fn new<O: Into<String>, N: Into<String>>(owner: O, name: N) -> Self {
        Self {
            owner: owner.into(),
            name: name.into(),
        }
    }

    pub fn owner(&self) -> &str {
        &self.owner
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}
