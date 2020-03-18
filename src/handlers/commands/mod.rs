use crate::github::Webhook;
use crate::handlers::Handler;

#[derive(Debug)]
pub struct CommandHandler;

impl Handler for CommandHandler {
    fn route(&self, _webhook: &Webhook) -> bool {
        false
    }

    fn handle(&self, _webhook: &Webhook) {
    }
}
