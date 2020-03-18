use crate::github::{Comment, Event, EventType, IssueCommentEvent, PullRequestReviewEvent, Review, Webhook};
use crate::handlers::Handler;
use log::info;

#[derive(Debug)]
pub struct CommandHandler;

impl Handler for CommandHandler {
    fn route(&self, webhook: &Webhook) -> bool {
        matches!(
            webhook.event_type,
            EventType::IssueComment | EventType::PullRequestReview
        )
    }

    fn handle(&self, webhook: &Webhook) {
        let body = match &webhook.event {
            Event::IssueComment(IssueCommentEvent { comment: Comment { ref body, .. }, .. }) => body,
            Event::PullRequestReview(PullRequestReviewEvent { review: Review { ref body, .. }, .. }) => body,
            _ => &None,
        };

        if let Some(ref body) = body {
            info!("comment body = \n{}", body);
        }
    }
}
