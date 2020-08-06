use crate::{config::RepoConfig, graphql::GithubClient, project_board::ProjectBoard, Result};
use github::Oid;
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    str::FromStr,
};
use thiserror::Error;

#[derive(Clone, Debug)]
pub struct PullRequestState {
    pub number: u64,
    pub id: u64,
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
    pub approved: bool,
    pub maintainer_can_modify: bool, // Use to enable 'rebase' merging and having github know a PR has been merged
    pub mergeable: bool,
    pub labels: HashSet<String>,

    pub status: Status,
    pub project_card_id: Option<u64>,

    pub canary_requested: bool,
}

#[derive(Clone, Debug)]
pub struct TestResult {
    pub passed: bool,
    pub details_url: String,
}

#[derive(Clone, Copy, Debug, PartialOrd, PartialEq, Ord, Eq)]
pub enum StatusType {
    Testing,
    Canary,
    Queued,
    InReview,
}

#[derive(Clone, Debug)]
pub enum Status {
    InReview,
    Queued,
    Testing {
        merge_oid: Oid,
        tests_started_at: std::time::Instant,
        test_results: HashMap<String, TestResult>,
    },
    Canary {
        merge_oid: Oid,
        tests_started_at: std::time::Instant,
        test_results: HashMap<String, TestResult>,
    },
    // Failed {
    //     merge_oid: Oid,
    //     test_results: HashMap<String, TestResult>,
    // },
    // Success {
    //     merge_oid: Oid,
    //     test_results: HashMap<String, TestResult>,
    // },
}

impl Status {
    pub fn is_queued(&self) -> bool {
        matches!(self, Status::Queued)
    }

    pub fn is_testing(&self) -> bool {
        matches!(self, Status::Testing { .. })
    }

    pub fn is_canary(&self) -> bool {
        matches!(self, Status::Canary { .. })
    }

    pub fn testing(merge_oid: Oid) -> Status {
        Status::Testing {
            merge_oid,
            tests_started_at: std::time::Instant::now(),
            test_results: HashMap::new(),
        }
    }

    pub fn canary(merge_oid: Oid) -> Status {
        Status::Canary {
            merge_oid,
            tests_started_at: std::time::Instant::now(),
            test_results: HashMap::new(),
        }
    }

    pub fn status_type(&self) -> StatusType {
        match self {
            Status::InReview => StatusType::InReview,
            Status::Queued => StatusType::Queued,
            Status::Testing { .. } => StatusType::Testing,
            Status::Canary { .. } => StatusType::Canary,
        }
    }
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
            id: pull.id,
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
            approved: false,
            maintainer_can_modify: pull.maintainer_can_modify.unwrap_or(false),
            mergeable: pull.mergeable.unwrap_or(false),
            labels,
            status: Status::InReview,
            project_card_id: None,
            canary_requested: false,
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

    // Update the Head Oid of a PR and kick it out of the queue if the Oid doesn't match the
    // currently being tested 'merge_oid'
    pub async fn update_head(
        &mut self,
        oid: Oid,
        config: &RepoConfig,
        github: &GithubClient,
        project_board: Option<&ProjectBoard>,
    ) -> Result<()> {
        self.head_ref_oid = oid.clone();

        match &self.status {
            // If the oid we're being updated to is the same as the merge_oid then we don't need to
            // do anything
            Status::Testing { merge_oid, .. } | Status::Canary { merge_oid, .. }
                if merge_oid == &oid => {}
            Status::InReview => {}
            _ => {
                self.update_status(Status::InReview, config, github, project_board)
                    .await?;

                if let Status::Testing { .. } | Status::Queued = &self.status {
                    let msg = ":exclamation: Land has been canceled due to this PR being updated with new commits. \
                    Please issue another Land command if you want to requeue this PR.";

                    github
                        .issues()
                        .create_comment(
                            config.repo().owner(),
                            config.repo().name(),
                            self.number,
                            msg,
                        )
                        .await?;
                }
            }
        }

        Ok(())
    }

    pub async fn update_status(
        &mut self,
        status: Status,
        _config: &RepoConfig,
        github: &GithubClient,
        project_board: Option<&ProjectBoard>,
    ) -> Result<()> {
        self.status = status;

        if let Some(board) = project_board {
            board.move_pr_to_status_column(github, &self).await?;
        }

        Ok(())
    }

    pub async fn add_label(
        &mut self,
        config: &RepoConfig,
        github: &GithubClient,
        label: &str,
    ) -> Result<()> {
        github
            .issues()
            .add_lables(
                config.owner(),
                config.name(),
                self.number,
                vec![label.into()],
            )
            .await?;
        self.labels.insert(label.into());
        Ok(())
    }

    pub fn has_label(&self, label: &str) -> bool {
        self.labels.contains(label)
    }

    pub fn priority(&self, config: &RepoConfig) -> Priority {
        if self.has_label(config.labels().high_priority()) {
            Priority::High
        } else if self.has_label(config.labels().low_priority()) {
            Priority::Low
        } else {
            Priority::Normal
        }
    }

    pub async fn remove_label(
        &mut self,
        config: &RepoConfig,
        github: &GithubClient,
        label: &str,
    ) -> Result<()> {
        if self.labels.contains(label) {
            github
                .issues()
                .remove_label(config.owner(), config.name(), self.number, label)
                .await?;
            self.labels.remove(label);
        }

        Ok(())
    }

    pub fn add_build_result(
        &mut self,
        build_name: &str,
        details_url: &str,
        conclusion: github::Conclusion,
    ) {
        if let Status::Testing {
            ref mut test_results,
            ..
        }
        | Status::Canary {
            ref mut test_results,
            ..
        } = self.status
        {
            test_results.insert(
                build_name.to_owned(),
                TestResult {
                    details_url: details_url.to_owned(),
                    passed: matches!(conclusion, github::Conclusion::Success),
                },
            );
        }
    }
}

pub enum TestSuiteResult {
    Pending,
    TimedOut,
    Passed,
    Failed { name: String, result: TestResult },
}

impl TestSuiteResult {
    pub fn new(
        tests_started_at: std::time::Instant,
        test_results: &HashMap<String, TestResult>,
        config: &RepoConfig,
    ) -> Self {
        // Check if there were any test failures from configured checks
        if let Some((name, result)) = config
            .checks()
            .filter_map(|name| test_results.get(name).map(|result| (name, result)))
            .find(|(_name, result)| !result.passed)
        {
            TestSuiteResult::Failed {
                name: name.to_owned(),
                result: result.to_owned(),
            }
        // Check if all tests have completed and passed
        } else if config
            .checks()
            .map(|name| test_results.get(name))
            .all(|result| result.map(|r| r.passed).unwrap_or(false))
        {
            TestSuiteResult::Passed
        // Check if the test has timed-out
        } else if tests_started_at.elapsed() >= config.timeout() {
            TestSuiteResult::TimedOut
        } else {
            TestSuiteResult::Pending
        }
    }
}

#[derive(Clone, Copy, Debug, PartialOrd, PartialEq, Ord, Eq, Serialize)]
pub enum Priority {
    High,
    Normal,
    Low,
}

#[derive(Error, Debug)]
#[error("invalid priority")]
pub struct ParsePriorityError;

impl FromStr for Priority {
    type Err = ParsePriorityError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "high" => Ok(Priority::High),
            "normal" => Ok(Priority::Normal),
            "low" => Ok(Priority::Low),
            _ => Err(ParsePriorityError),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
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
