use super::{DateTime, NodeId, User};
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct Project {
    pub id: u64,
    pub node_id: NodeId,
    pub url: String,
    pub html_url: String,
    pub columns_url: String,
    pub owner_url: String,
    pub name: String,
    pub body: Option<String>,
    pub number: u64,
    pub state: String,
    pub created_at: DateTime,
    pub updated_at: Option<DateTime>,
    pub creator: User,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ProjectCard {
    pub id: u64,
    pub node_id: NodeId,
    pub url: String,
    pub column_url: String,
    pub content_url: String,
    pub note: Option<String>,
    pub creator: User,
    pub created_at: DateTime,
    pub updated_at: Option<DateTime>,
    pub archived: bool,

    // populated by Webhook events
    pub column_id: Option<u64>,

    // Populated by Events API
    pub project_id: Option<u64>,
    pub project_url: Option<String>,
    pub column_name: Option<String>,
    pub previous_column_name: Option<String>,
}

impl ProjectCard {
    pub fn column_id(&self) -> Option<u64> {
        self.column_url.split('/').last()?.parse().ok()
    }

    pub fn issue_number(&self) -> Option<u64> {
        if self.note.is_some() {
            return None;
        }

        self.content_url.split('/').last()?.parse().ok()
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct ProjectColumn {
    pub id: u64,
    pub node_id: NodeId,
    pub url: String,
    pub name: String,
    pub project_url: String,
    pub cards_url: String,
    pub created_at: DateTime,
    pub updated_at: Option<DateTime>,
}
