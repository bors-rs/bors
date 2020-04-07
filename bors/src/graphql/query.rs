use graphql_client::GraphQLQuery;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/github-schema.graphql",
    query_path = "src/graphql/add_reaction.graphql",
    response_derives = "Debug"
)]
pub struct AddReaction;

impl From<github::ReactionType> for add_reaction::ReactionContent {
    fn from(reaction: github::ReactionType) -> Self {
        use add_reaction::ReactionContent;
        use github::ReactionType;

        match reaction {
            ReactionType::ThumbsUp => ReactionContent::THUMBS_UP,
            ReactionType::ThumbsDown => ReactionContent::THUMBS_DOWN,
            ReactionType::Laugh => ReactionContent::LAUGH,
            ReactionType::Confused => ReactionContent::CONFUSED,
            ReactionType::Heart => ReactionContent::HEART,
            ReactionType::Hooray => ReactionContent::HOORAY,
            ReactionType::Rocket => ReactionContent::ROCKET,
            ReactionType::Eyes => ReactionContent::EYES,
        }
    }
}

type GitObjectID = github::Oid;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/github-schema.graphql",
    query_path = "src/graphql/list_pulls.graphql",
    response_derives = "Debug"
)]
pub struct ListPulls;

impl From<list_pulls::PullRequestState> for github::PullRequestState {
    fn from(state: list_pulls::PullRequestState) -> Self {
        match state {
            list_pulls::PullRequestState::OPEN => github::PullRequestState::Open,
            list_pulls::PullRequestState::CLOSED => github::PullRequestState::Closed,
            list_pulls::PullRequestState::MERGED => github::PullRequestState::Merged,

            // Treat Other states as being closed
            list_pulls::PullRequestState::Other(_) => github::PullRequestState::Closed,
        }
    }
}

impl From<list_pulls::ListPullsRepositoryPullRequestsNodes> for crate::state::PullRequestState {
    fn from(pull: list_pulls::ListPullsRepositoryPullRequestsNodes) -> Self {
        let list_pulls::ListPullsRepositoryPullRequestsNodes {
            number,
            author,
            is_draft,
            maintainer_can_modify,
            mergeable,
            labels,
            head_ref_name,
            head_ref_oid,
            body,
            base_ref_name,
            base_ref_oid,
            title,
            state,
            head_repository,
        } = pull;

        let labels = labels
            .into_iter()
            .flat_map(|nodes| {
                nodes
                    .nodes
                    .into_iter()
                    .flat_map(|labels| labels.into_iter().flat_map(|label| label.map(|l| l.name)))
            })
            .collect();

        let head_repo = if let Some(repo) = head_repository {
            let mut iter = repo.name_with_owner.split('/');
            match (iter.next(), iter.next()) {
                (Some(owner), Some(name)) => Some(crate::state::Repo::new(owner, name)),
                _ => None,
            }
        } else {
            None
        };

        Self {
            number: number as u64,
            author: author.map(|a| a.login),
            title,
            body,

            head_ref_name,
            head_ref_oid,
            head_repo,

            base_ref_name,
            base_ref_oid,

            is_draft,
            maintainer_can_modify,
            mergeable: matches!(mergeable, list_pulls::MergeableState::MERGEABLE),
            labels,
            state: state.into(),

            approved_by: Vec::new(),
            priority: 0,
            delegate: false,
            merge_oid: None,
        }
    }
}
