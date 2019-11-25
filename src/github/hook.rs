use super::{DateTime, EventType};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Hook {
    #[serde(rename = "type")]
    hook_type: String,
    id: u64,
    name: String,
    active: bool,
    events: Vec<EventType>,
    config: HookConfig,
    updated_at: DateTime,
    created_at: DateTime,
    url: String,
    test_url: String,
    ping_url: String,
    last_response: HookResponse,
}

#[derive(Debug, Deserialize)]
pub struct HookConfig {
    content_type: String,
    insecure_ssl: String,
    url: String,
}

#[derive(Debug, Deserialize)]
pub struct HookResponse {
    code: Option<String>,
    status: String,
    message: Option<String>,
}

#[cfg(test)]
mod test {
    use super::Hook;

    #[test]
    fn hook() {
        const JSON: &str = include_str!("../test-input/hook.json");
        let _: Hook = serde_json::from_str(JSON).unwrap();
    }
}
