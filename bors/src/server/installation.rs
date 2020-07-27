use crate::{config::RepoConfig, event_processor::EventProcessorSender};
use github::Event;

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

    pub async fn state(&self) -> String {
        let (queue, pulls) = self.event_processor.get_state().await.unwrap();

        format!("Queue: {:#?}\n\nPulls: {:#?}", queue, pulls)
    }
}
