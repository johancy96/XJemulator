use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub lang: crate::i18n::Lang,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            lang: crate::i18n::Lang::default(),
        }
    }
}

impl AppConfig {
    pub fn load() -> Self {
        if let Ok(content) = fs::read_to_string("config.toml") {
            if let Ok(config) = toml::from_str(&content) {
                return config;
            }
        }
        Self::default()
    }

    pub fn save(&self) {
        if let Ok(content) = toml::to_string(self) {
            let _ = fs::write("config.toml", content);
        }
    }
}
