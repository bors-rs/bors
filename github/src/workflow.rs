use crate::{CheckStatus, Conclusion, DateTime, EventType, NodeId, Oid};
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct Workflow {
    pub id: u64,
    pub node_id: NodeId,
    pub name: String,
    pub path: String,
    pub state: String,
    pub created_at: DateTime,
    pub updated_at: DateTime,
    pub url: String,
    pub html_url: String,
    pub badge_url: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct WorkflowRun {
    pub id: u64,
    pub name: String,
    pub node_id: NodeId,
    pub head_branch: String,
    pub head_sha: Oid,
    pub run_number: u64,
    pub event: EventType,
    pub status: CheckStatus,
    pub conclusion: Option<Conclusion>,
    pub workflow_id: u64,
    pub check_suite_id: u64,
    pub check_suite_node_id: NodeId,
    pub url: String,
    pub html_url: String,
    // pull_requests: Vec
    pub created_at: DateTime,
    pub updated_at: DateTime,
    pub jobs_url: String,
    pub logs_url: String,
    pub check_suite_url: String,
    pub artifacts_url: String,
    pub cancel_url: String,
    pub rerun_url: String,
    pub workflow_url: String,
    // head_commit
    // repository
    // head_repository
}
