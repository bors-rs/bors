use super::{NodeId, User};
use serde::{Deserialize, Serialize};

// GitHub API docs: https://developer.github.com/v3/reactions/

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ReactionType {
    #[serde(rename = "+1")]
    PlusOne,
    #[serde(rename = "-1")]
    MinusOne,
    Laugh,
    Confused,
    Heart,
    Hooray,
    Rocket,
    Eyes,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Reaction {
    pub id: u64,
    pub user: User,
    pub node_id: NodeId,
    pub content: ReactionType,
}
