use super::{DateTime, NodeId, Oid, User};
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct Repository {
    pub id: u64,
    pub node_id: NodeId,
    pub name: String,
    pub full_name: String,
    pub private: bool,
    pub owner: User,
    pub html_url: String,
    pub description: Option<String>,
    pub fork: bool,
    pub url: String,
    pub forks_url: String,
    pub keys_url: String,
    pub collaborators_url: String,
    pub teams_url: String,
    pub hooks_url: String,
    pub issue_events_url: String,
    pub events_url: String,
    pub assignees_url: String,
    pub branches_url: String,
    pub tags_url: String,
    pub blobs_url: String,
    pub git_tags_url: String,
    pub git_refs_url: String,
    pub trees_url: String,
    pub statuses_url: String,
    pub languages_url: String,
    pub stargazers_url: String,
    pub contributors_url: String,
    pub subscribers_url: String,
    pub subscription_url: String,
    pub commits_url: String,
    pub git_commits_url: String,
    pub comments_url: String,
    pub issue_comment_url: String,
    pub contents_url: String,
    pub compare_url: String,
    pub merges_url: String,
    pub archive_url: String,
    pub downloads_url: String,
    pub issues_url: String,
    pub pulls_url: String,
    pub milestones_url: String,
    pub notifications_url: String,
    pub labels_url: String,
    pub releases_url: String,
    pub deployments_url: String,
    pub created_at: DateTime,
    pub updated_at: DateTime,
    pub pushed_at: DateTime,
    pub git_url: String,
    pub ssh_url: String,
    pub clone_url: String,
    pub svn_url: String,
    pub homepage: Option<String>,
    pub size: u64,
    pub stargazers_count: u64,
    pub watchers_count: u64,
    pub language: Option<String>,
    pub has_issues: bool,
    pub has_projects: bool,
    pub has_downloads: bool,
    pub has_wiki: bool,
    pub has_pages: bool,
    pub forks_count: u64,
    pub mirror_url: Option<String>,
    pub archived: bool,
    pub disabled: bool,
    pub open_issues_count: u64,
    pub license: Option<String>,
    pub forks: u64,
    pub open_issues: u64,
    pub watchers: u64,
    pub default_branch: String,
    // parent: Option<Box<Repository>>,
    // source: Option<Box<Repository>>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Author {
    pub name: String,
    pub email: String,
    pub username: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Commit {
    pub id: Oid,
    pub tree_id: Oid,
    pub distinct: bool,
    pub message: String,
    pub timestamp: DateTime,
    pub url: String,
    pub author: Author,
    pub committer: Author,
    /// List of added files
    pub added: Vec<String>,
    /// List of removed files
    pub removed: Vec<String>,
    /// List of modified files
    pub modified: Vec<String>,
}

#[cfg(test)]
mod test {
    use super::Repository;

    #[test]
    fn repo() {
        const REPO_JSON: &str = include_str!("../test-input/repo.json");
        let _repo: Repository = serde_json::from_str(REPO_JSON).unwrap();
    }
}
