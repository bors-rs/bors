use super::{Date, NodeId, Oid, User};
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
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

#[derive(Debug, Deserialize)]
pub struct Image {
    pub alt: String,
    pub image_url: String,
    pub caption: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CheckOutput {
    pub title: String,
    pub summary: String,
    pub text: String,
    pub annotations_count: Option<u64>,
    pub annotations_url: Option<String>,
    pub annotations: Option<Vec<Annotation>>,
    pub images: Option<Vec<Image>>,
}

pub struct CheckRun {
    pub id: u64,
    pub head_sha: Oid,
    pub node_id: NodeId,
    pub external_id: String,
    pub url: String,
    pub html_url: String,
    pub details_url: String,
    pub status: String,
    pub conclusion: Option<String>,
    pub started_at: Date,
    pub completed_at: Date,
    pub output: CheckOutput,
    pub name: String,
    pub check_suite: CheckSuite,
    pub app: App,
    // pull_requests: Vec<PullRequestRef>,
}

pub struct App {
    pub id: u64,
    pub slug: String,
    pub node_id: NodeId,
    pub owner: User,
    pub name: String,
    pub description: Option<String>,
    pub external_url: String,
    pub html_url: String,
    pub created_at: Date,
    pub updated_at: Date,
    pub permissions: HashMap<String, String>,
    pub events: Vec<String>,
}

pub struct CheckSuite {
    pub id: u64,
    pub node_id: NodeId,
    pub head_branch: Option<String>,
    pub head_sha: Oid,
    pub status: String,
    pub conclusion: Option<String>,
    pub url: String,
    pub before: Option<String>,
    pub after: Option<String>,
    // pull_requests: Vec<PullRequestRef>,
    pub app: App,
    pub created_at: Date,
    pub updated_at: Date,
}
