use github::ReactionType;
use graphql_client::GraphQLQuery;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/github-schema.graphql",
    query_path = "src/graphql/add_reaction.graphql",
    response_derives = "Debug"
)]
pub struct AddReaction;

impl From<ReactionType> for add_reaction::ReactionContent {
    fn from(reaction: ReactionType) -> Self {
        use add_reaction::ReactionContent;

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
