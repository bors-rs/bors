use github::{Event, EventType};
use std::fmt::Debug;

#[async_trait::async_trait]
pub trait Service: Send + Sync + Debug {
    /// Returns the name of the service.
    fn name(&self) -> &'static str;

    /// Event Handling
    fn route(&self, event_type: EventType) -> bool;
    async fn handle(&self, event: &Event, delivery_id: &str);

    // TODO: HTTP routing
}
