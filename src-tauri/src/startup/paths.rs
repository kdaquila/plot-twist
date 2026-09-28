//! Per-user folders: `%APPDATA%\plot-twist` (settings) and `%LOCALAPPDATA%\plot-twist`
//! (logs, API discovery file).

use std::path::PathBuf;

const APP_DIR: &str = "plot-twist";

fn env_dir(var: &str) -> PathBuf {
    std::env::var_os(var)
        .map_or_else(std::env::temp_dir, PathBuf::from)
        .join(APP_DIR)
}

pub fn settings_file() -> PathBuf {
    env_dir("APPDATA").join("settings.json")
}

pub fn local_dir() -> PathBuf {
    env_dir("LOCALAPPDATA")
}

pub fn logs_dir() -> PathBuf {
    local_dir().join("logs")
}

pub fn api_discovery_file() -> PathBuf {
    local_dir().join("api.json")
}
