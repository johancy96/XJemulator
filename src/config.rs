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
        let path = crate::paths::AppPaths::config_file();
        if let Ok(content) = fs::read_to_string(path) {
            if let Ok(config) = toml::from_str(&content) {
                return config;
            }
        }
        Self::default()
    }
}
