use std::collections::HashMap;

pub trait HackSettings {
    fn get_settings(&self) -> HashMap<String, serde_json::Value>;
    fn apply_settings(&mut self, settings: &HashMap<String, serde_json::Value>);
}
