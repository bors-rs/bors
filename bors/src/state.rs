use github::Oid;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};

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
    pub test_results: HashMap<String, TestResult>,
    pub status: Status,
}

#[derive(Debug)]
pub struct TestResult {
    passed: bool,
    details_url: String,
}

#[derive(Debug)]
pub enum Status {
    InReview,
    /// Queued for landing
    ReadyToLand,
    Testing,
    // TODO add a Success state
}

impl PullRequestState {
    pub fn from_pull_request(pull: &github::PullRequest) -> Self {
        let state = match pull.state {
            github::State::Open => github::PullRequestState::Open,
            github::State::Closed => github::PullRequestState::Closed,
        };

        let labels = pull.labels.iter().map(|l| l.name.clone()).collect();

        Self {
            number: pull.number,
            author: Some(pull.user.login.clone()),
            title: pull.title.clone(),
            body: pull.body.clone().unwrap_or_default(),
            head_ref_oid: pull.head.sha.clone(),
            head_ref_name: pull.head.git_ref.clone(),
            head_repo: pull.head.repo.as_ref().map(Repo::from_repository),
            base_ref_name: pull.base.git_ref.clone(),
            base_ref_oid: pull.base.sha.clone(),
            state,
            is_draft: pull.draft.unwrap_or(false),
            approved_by: HashSet::new(),
            maintainer_can_modify: pull.maintainer_can_modify.unwrap_or(false),
            mergeable: pull.mergeable.unwrap_or(false),
            labels,
            priority: 0,
            delegate: false,
            merge_oid: None,
            test_results: HashMap::new(),
            status: Status::InReview,
        }
    }

    /// Check if either the PR is marked as being draft or if the PR title seems to indicate that
    /// it is still "WIP"
    pub fn is_draft(&self) -> bool {
        self.is_draft
            || ["WIP", "TODO", "[WIP]", "[TODO]"]
                .iter()
                .any(|s| self.title.starts_with(s))
    }

    // XXX this should probably update the status of the PR as well, like if the PR is in the queue
    // to land it should be kicked out
    pub fn update_head(&mut self, oid: Oid) {
        self.head_ref_oid = oid;
    }

    pub fn add_build_result(
        &mut self,
        build_name: &str,
        details_url: &str,
        conclusion: github::Conclusion,
    ) {
        self.test_results.insert(
            build_name.to_owned(),
            TestResult {
                details_url: details_url.to_owned(),
                passed: matches!(conclusion, github::Conclusion::Success),
            },
        );
    }
}

#[derive(Clone, Debug, Deserialize)]
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

    pub fn from_repository(r: &github::Repository) -> Self {
        Self {
            owner: r.owner.login.clone(),
            name: r.name.clone(),
        }
    }

    pub fn owner(&self) -> &str {
        &self.owner
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn to_github_ssh_url(&self) -> String {
        format!("git@github.com:{}/{}.git", self.owner, self.name)
    }
}
