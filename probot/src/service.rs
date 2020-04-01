use github::Webhook;
use std::fmt::Debug;

//TODO maybe don't need the route function?
#[async_trait::async_trait]
pub trait Service: Send + Sync + Debug {
    fn route(&self, webhook: &Webhook) -> bool;
    async fn handle(&self, webhook: &Webhook);
}
