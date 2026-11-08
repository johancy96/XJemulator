use directories::ProjectDirs;
use std::fs;
use std::path::PathBuf;

pub struct AppPaths;

impl AppPaths {
    pub fn config_dir() -> PathBuf {
        let dir = if let Some(proj) = ProjectDirs::from("com", "xjemulator", "xjemulator") {
            proj.config_dir().to_path_buf()
        } else if let Ok(home) = std::env::var("HOME") {
            PathBuf::from(home).join(".config").join("xjemulator")
        } else {
            PathBuf::from("/tmp/xjemulator")
        };

        if !dir.exists() {
            let _ = fs::create_dir_all(&dir);
        }
        dir
    }

    pub fn profiles_dir() -> PathBuf {
        let dir = Self::config_dir().join("profiles");
        if !dir.exists() {
            let _ = fs::create_dir_all(&dir);
        }
        dir
    }

    pub fn config_file() -> PathBuf {
        Self::config_dir().join("config.toml")
    }

    pub fn telemetry_dir() -> PathBuf {
        let dir = Self::config_dir().join("telemetry");
        if !dir.exists() {
            let _ = fs::create_dir_all(&dir);
        }
        dir
    }

    /// Sanitiza un nombre de perfil para evitar Path Traversal y caracteres inválidos.
    pub fn sanitize_profile_name(name: &str) -> String {
        let base = name.trim().strip_suffix(".toml").unwrap_or(name.trim());
        let sanitized: String = base
            .chars()
            .map(|c| {
                if c.is_alphanumeric() || c == '_' || c == '-' {
                    c
                } else if c.is_whitespace() {
                    '_'
                } else {
                    '_'
                }
            })
            .collect();

        let clean = sanitized.trim_matches('_').to_string();
        if clean.is_empty() {
            "nuevo_perfil".to_string()
        } else {
            clean
        }
    }

    pub fn profile_path(name: &str) -> PathBuf {
        let safe_name = Self::sanitize_profile_name(name);
        let mut path = Self::profiles_dir().join(safe_name);
        path.set_extension("toml");
        path
    }
}
