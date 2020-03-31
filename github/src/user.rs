use super::{DateTime, NodeId};
use serde::Deserialize;

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub enum UserType {
    Bot,
    Organization,
    User,
}

#[derive(Clone, Debug, Deserialize)]
pub struct User {
    pub login: String,
    pub id: u64,
    pub node_id: NodeId,
    pub avatar_url: String,
    pub gravatar_id: String,
    pub url: String,
    pub html_url: String,
    pub followers_url: String,
    pub following_url: String,
    pub gists_url: String,
    pub starred_url: String,
    pub subscriptions_url: String,
    pub organizations_url: String,
    pub repos_url: String,
    pub events_url: String,
    pub received_events_url: String,
    #[serde(rename = "type")]
    pub user_type: UserType,
    pub site_admin: bool,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Team {
    id: u64,
    node_id: NodeId,
    url: String,
    html_url: String,
    name: String,
    slug: String,
    description: Option<String>,
    privacy: String,
    permission: String,
    members_url: String,
    repositories_url: String,
    parent: Option<Box<Team>>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Pusher {
    name: String,
    email: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Key {
    id: u64,
    key: String,
    url: String,
    title: String,
    read_only: bool,
    created_at: DateTime,
    verified: bool,
}

#[cfg(test)]
mod test {
    use super::{User, UserType};

    #[test]
    fn user() {
        const USER_JSON: &str = r#"
            {
                "login": "Codertocat",
                "id": 21031067,
                "node_id": "MDQ6VXNlcjIxMDMxMDY3",
                "avatar_url": "https://avatars1.githubusercontent.com/u/21031067?v=4",
                "gravatar_id": "",
                "url": "https://api.github.com/users/Codertocat",
                "html_url": "https://github.com/Codertocat",
                "followers_url": "https://api.github.com/users/Codertocat/followers",
                "following_url": "https://api.github.com/users/Codertocat/following{/other_user}",
                "gists_url": "https://api.github.com/users/Codertocat/gists{/gist_id}",
                "starred_url": "https://api.github.com/users/Codertocat/starred{/owner}{/repo}",
                "subscriptions_url": "https://api.github.com/users/Codertocat/subscriptions",
                "organizations_url": "https://api.github.com/users/Codertocat/orgs",
                "repos_url": "https://api.github.com/users/Codertocat/repos",
                "events_url": "https://api.github.com/users/Codertocat/events{/privacy}",
                "received_events_url": "https://api.github.com/users/Codertocat/received_events",
                "type": "User",
                "site_admin": false
            }
        "#;

        let user: User = serde_json::from_str(USER_JSON).unwrap();
        assert_eq!(user.user_type, UserType::User);
    }

    #[test]
    fn org() {
        const ORGANIZATION_JSON: &str = r#"
            {
                "login": "Octocoders",
                "id": 38302899,
                "node_id": "MDEyOk9yZ2FuaXphdGlvbjM4MzAyODk5",
                "avatar_url": "https://avatars1.githubusercontent.com/u/38302899?v=4",
                "gravatar_id": "",
                "url": "https://api.github.com/users/Octocoders",
                "html_url": "https://github.com/Octocoders",
                "followers_url": "https://api.github.com/users/Octocoders/followers",
                "following_url": "https://api.github.com/users/Octocoders/following{/other_user}",
                "gists_url": "https://api.github.com/users/Octocoders/gists{/gist_id}",
                "starred_url": "https://api.github.com/users/Octocoders/starred{/owner}{/repo}",
                "subscriptions_url": "https://api.github.com/users/Octocoders/subscriptions",
                "organizations_url": "https://api.github.com/users/Octocoders/orgs",
                "repos_url": "https://api.github.com/users/Octocoders/repos",
                "events_url": "https://api.github.com/users/Octocoders/events{/privacy}",
                "received_events_url": "https://api.github.com/users/Octocoders/received_events",
                "type": "Organization",
                "site_admin": false
            }
        "#;

        let user: User = serde_json::from_str(ORGANIZATION_JSON).unwrap();
        assert_eq!(user.user_type, UserType::Organization);
    }
}
