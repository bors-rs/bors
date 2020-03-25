use super::{DateTime, EventType, NodeId, Oid, User};
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Clone, Debug, Deserialize)]
pub struct Annotation {
    pub path: String,
    pub start_line: u64,
    pub end_line: u64,
    pub start_column: Option<u64>,
    pub end_colum: Option<u64>,
    pub annotation_level: Option<String>,
    pub message: Option<String>,
    pub title: Option<String>,
    pub raw_details: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Image {
    pub alt: String,
    pub image_url: String,
    pub caption: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Conclusion {
    Success,
    Failure,
    Neutral,
    Cancelled,
    TimedOut,
    ActionRequired,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckStatus {
    Queued,
    InProgress,
    Completed,
}

#[derive(Clone, Debug, Deserialize)]
pub struct CheckOutput {
    pub title: Option<String>,
    pub summary: Option<String>,
    pub text: Option<String>,
    pub annotations_count: Option<u64>,
    pub annotations_url: Option<String>,
    pub annotations: Option<Vec<Annotation>>,
    pub images: Option<Vec<Image>>,
}

// Maybe rename these?
#[derive(Clone, Debug, Deserialize)]
pub struct CheckPullRequest {
    url: String,
    id: u64,
    number: u64,
    head: CheckBranch,
    base: CheckBranch,
}

#[derive(Clone, Debug, Deserialize)]
pub struct CheckBranch {
    #[serde(rename = "ref")]
    git_ref: String,
    sha: Oid,
    repo: CheckRepo,
}

#[derive(Clone, Debug, Deserialize)]
pub struct CheckRepo {
    id: u64,
    url: String,
    name: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct CheckRun {
    pub id: u64,
    pub head_sha: Oid,
    pub node_id: NodeId,
    pub external_id: String,
    pub url: String,
    pub html_url: String,
    pub details_url: String,
    pub status: CheckStatus,
    pub conclusion: Option<Conclusion>,
    pub started_at: DateTime,
    pub completed_at: Option<DateTime>,
    pub output: CheckOutput,
    pub name: String,
    pub check_suite: CheckSuite,
    pub app: App,
    pub pull_requests: Vec<CheckPullRequest>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct App {
    pub id: u64,
    pub slug: String,
    pub node_id: NodeId,
    pub owner: User,
    pub name: String,
    pub description: Option<String>,
    pub external_url: String,
    pub html_url: String,
    pub created_at: DateTime,
    pub updated_at: DateTime,
    pub permissions: HashMap<String, String>,
    pub events: Vec<EventType>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct CheckSuite {
    pub id: u64,
    pub node_id: NodeId,
    pub head_branch: Option<String>,
    pub head_sha: Oid,
    pub status: CheckStatus,
    pub conclusion: Option<Conclusion>,
    pub url: String,
    pub before: Option<String>,
    pub after: Option<String>,
    pub pull_requests: Vec<CheckPullRequest>,
    pub app: App,
    pub created_at: DateTime,
    pub updated_at: DateTime,
    pub latest_check_runs_count: Option<u64>,
    pub check_runs_url: Option<String>,
}

#[cfg(test)]
mod test {
    use super::{CheckRun, CheckSuite};

    #[test]
    fn check_run() {
        const CHECK_RUN_JSON: &str = include_str!("../test-input/check-run.json");
        let _: CheckRun = serde_json::from_str(CHECK_RUN_JSON).unwrap();
    }

    #[test]
    fn check_suite() {
        const CHECK_SUITE_JSON: &str = include_str!("../test-input/check-suite.json");
        let _: CheckSuite = serde_json::from_str(CHECK_SUITE_JSON).unwrap();
    }
}
