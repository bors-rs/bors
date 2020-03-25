use super::NodeId;
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct License {
    pub key: String,
    pub name: String,
    pub node_id: NodeId,
    pub spdx_id: String,
    pub url: Option<String>,

    // Populated by `LicenseClient::get`
    pub html_url: Option<String>,
    pub description: Option<String>,
    pub implementation: Option<String>,
    pub permissions: Option<Vec<String>>,
    pub conditions: Option<Vec<String>>,
    pub limitation: Option<Vec<String>>,
    pub body: Option<String>,
    pub featured: Option<bool>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct RepositoryLicense {
    pub name: String,
    pub path: String,
    pub sha: String,
    pub size: usize,
    pub url: String,
    pub html_url: String,
    pub git_url: String,
    pub download_url: String,
    #[serde(rename = "type")]
    pub blob_type: String, // TODO check that this is actually a blob
    pub content: String,
    pub encoding: String,
    pub license: License,
}
