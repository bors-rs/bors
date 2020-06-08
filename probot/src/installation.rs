use crate::service::Service;

#[derive(Debug)]
pub struct Installation {
    owner: String,
    name: String,
    secret: Option<String>,
    services: Vec<Box<dyn Service>>,
}

impl Installation {
    pub fn new<O: Into<String>, N: Into<String>>(owner: O, name: N) -> Self {
        Self {
            owner: owner.into(),
            name: name.into(),
            secret: None,
            services: Vec::new(),
        }
    }

    pub fn with_secret<T: Into<String>>(&mut self, secret: T) -> &mut Self {
        self.secret = Some(secret.into());
        self
    }

    pub fn with_service(&mut self, service: Box<dyn Service>) -> &mut Self {
        self.services.push(service);
        self
    }

    pub fn owner(&self) -> &str {
        &self.owner
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn secret(&self) -> Option<&str> {
        self.secret.as_deref()
    }

    pub fn services(&self) -> &[Box<dyn Service>] {
        &self.services
    }
}
