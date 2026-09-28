//! User preferences persisted across restarts: theme, API port, recent files.

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::session::lock;

pub const DEFAULT_API_PORT: u16 = 47811;
const MAX_RECENT_FILES: usize = 10;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export)]
pub enum Theme {
    #[default]
    System,
    Light,
    Dark,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(default)]
#[ts(export)]
pub struct Settings {
    pub theme: Theme,
    pub api_port: u16,
    /// Most recent first, at most 10.
    pub recent_files: Vec<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            theme: Theme::System,
            api_port: DEFAULT_API_PORT,
            recent_files: Vec::new(),
        }
    }
}

impl Settings {
    /// Reads settings; a missing or corrupt file yields defaults (logged, never an error).
    pub fn load(path: &Path) -> Self {
        let text = match std::fs::read_to_string(path) {
            Ok(text) => text,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Self::default(),
            Err(e) => {
                tracing::warn!(path = %path.display(), error = %e, "could not read settings");
                return Self::default();
            }
        };
        serde_json::from_str(&text).unwrap_or_else(|e| {
            tracing::warn!(path = %path.display(), error = %e, "corrupt settings; using defaults");
            Self::default()
        })
    }

    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let text = serde_json::to_string_pretty(self).map_err(std::io::Error::other)?;
        let temp = path.with_extension("json.tmp");
        std::fs::write(&temp, text)?;
        std::fs::rename(temp, path)
    }

    /// Moves `path` to the front of the recent list.
    pub fn record_recent(&mut self, path: &str) {
        self.remove_recent(path);
        self.recent_files.insert(0, path.to_owned());
        self.recent_files.truncate(MAX_RECENT_FILES);
    }

    pub fn remove_recent(&mut self, path: &str) {
        // Windows paths are case-insensitive.
        self.recent_files.retain(|p| !p.eq_ignore_ascii_case(path));
    }
}

/// Settings plus the file they persist to (none in tests).
#[derive(Debug)]
pub struct SettingsStore {
    path: Option<PathBuf>,
    settings: Mutex<Settings>,
}

impl SettingsStore {
    pub fn open(path: PathBuf) -> Self {
        let settings = Settings::load(&path);
        Self {
            path: Some(path),
            settings: Mutex::new(settings),
        }
    }

    pub fn in_memory() -> Self {
        Self {
            path: None,
            settings: Mutex::new(Settings::default()),
        }
    }

    pub fn get(&self) -> Settings {
        lock(&self.settings).clone()
    }

    /// Applies `change`, saves, and returns the new settings. Save failures are logged.
    pub fn update(&self, change: impl FnOnce(&mut Settings)) -> Settings {
        let mut settings = lock(&self.settings);
        change(&mut settings);
        if let Some(path) = &self.path
            && let Err(e) = settings.save(path)
        {
            tracing::warn!(path = %path.display(), error = %e, "could not save settings");
        }
        settings.clone()
    }
}
