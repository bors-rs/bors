use crate::github::Webhook;
use std::fmt::Debug;

mod commands;

pub trait Handler: Send + Debug {
    fn route(&self, webhook: &Webhook) -> bool;
    fn handle(&self, webhook: &Webhook);
}

#[derive(Debug)]
pub struct Handlers {
    list: Vec<Box<dyn Handler>>,
}

impl Handlers {
    pub fn new() -> Self {
        let mut list: Vec<Box<dyn Handler>> = Vec::new();

        // add each handler here
        list.push(Box::new(commands::CommandHandler));
        
        Handlers {
            list
        }
    }

    pub fn handle(&self, webhook: Webhook) {
        for handler in &self.list {
            if handler.route(&webhook) {
                handler.handle(&webhook);
            }
        }
    }
}
