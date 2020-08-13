use crate::{
    config::RepoConfig,
    event_processor::EventProcessorSender,
    state::{Priority, PullRequestState},
};
use github::Event;
use serde::Serialize;

#[derive(Debug)]
pub struct Installation {
    config: RepoConfig,
    event_processor: EventProcessorSender,
}

impl Installation {
    pub fn new(config: RepoConfig, event_processor: EventProcessorSender) -> Self {
        Self {
            config,
            event_processor,
        }
    }

    pub fn config(&self) -> &RepoConfig {
        &self.config
    }

    pub fn owner(&self) -> &str {
        self.config.owner()
    }

    pub fn name(&self) -> &str {
        self.config.name()
    }

    pub fn secret(&self) -> Option<&str> {
        self.config.secret()
    }

    #[allow(dead_code)]
    pub fn event_processor(&self) -> &EventProcessorSender {
        &self.event_processor
    }

    pub async fn handle_webhook(&self, event: &Event, delivery_id: &str) {
        self.event_processor
            .webhook(event.clone(), delivery_id.to_owned())
            .await
            .unwrap();
    }

    pub async fn state(&self) -> Vec<PullRequestState> {
        let (_queue, pulls) = self.event_processor.get_state().await.unwrap();

        let mut pulls = pulls.into_iter().map(|(_, v)| v).collect::<Vec<_>>();
        pulls.sort_unstable_by_key(|p| p.to_queue_entry(self.config()));
        pulls
    }

    pub async fn sync(&self) {
        self.event_processor.sync().await.unwrap();
    }

    pub async fn repo_liquid_object(&self) -> liquid::Object {
        let pull_requests = self.state().await;
        let pull_requests = pull_requests
            .into_iter()
            .map(|p| LiquidPullRequest::from_pull_request_state(p, self.config()))
            .collect::<Vec<_>>();

        let object = liquid::object!({
            "repo": self.config().repo(),
            "total": pull_requests.len(),
            "pull_requests": pull_requests,
        });

        object
    }
}

// Type used for Liquid templating
#[derive(Debug, Serialize)]
struct LiquidPullRequest {
    number: u64,
    title: String,
    status: &'static str,
    mergeable: &'static str,
    head_ref: String,
    approved: &'static str,
    maintainer_can_modify: &'static str,
    priority: Priority,
}

impl LiquidPullRequest {
    fn from_pull_request_state(pr: PullRequestState, config: &RepoConfig) -> Self {
        let priority = pr.priority(config);

        use crate::state::Status;
        let status = match pr.status {
            Status::InReview => "",
            Status::Queued(_) => "queued",
            Status::Testing { .. } => "testing",
            Status::Canary { .. } => "canary",
        };

        let mergeable = if pr.mergeable { "yes" } else { "no" };
        let approved = if pr.approved { "yes" } else { "no" };
        let maintainer_can_modify = if pr.maintainer_can_modify {
            "yes"
        } else {
            "no"
        };

        let head_ref = if let Some(author) = pr.author {
            format!("{}:{}", author, pr.head_ref_name)
        } else {
            pr.head_ref_name
        };

        Self {
            number: pr.number,
            title: pr.title,
            status,
            mergeable,
            approved,
            maintainer_can_modify,
            head_ref,
            priority,
        }
    }
}
