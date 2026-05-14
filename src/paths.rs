use directories::ProjectDirs;
use std::path::PathBuf;
use std::fs;

pub struct AppPaths;

impl AppPaths {
    fn get_project_dirs() -> ProjectDirs {
        ProjectDirs::from("com", "xjemulator", "xjemulator")
            .expect("No se pudo determinar el directorio de inicio del usuario")
    }

    pub fn config_dir() -> PathBuf {
        let dir = Self::get_project_dirs().config_dir().to_path_buf();
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

    pub fn profile_path(name: &str) -> PathBuf {
        let mut path = Self::profiles_dir().join(name);
        if path.extension().is_none() {
            path.set_extension("toml");
        }
        path
    }
}
