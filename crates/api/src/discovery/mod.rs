//! `api.json`: tells local scripts where the API is listening.

use std::io;
use std::path::Path;

use serde::Serialize;

#[derive(Serialize)]
struct Discovery<'a> {
    base_url: &'a str,
    pid: u32,
    version: &'a str,
}

/// Writes the file atomically (temp file, then rename).
pub fn write(path: &Path, base_url: &str) -> io::Result<()> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let body = serde_json::to_vec_pretty(&Discovery {
        base_url,
        pid: std::process::id(),
        version: env!("CARGO_PKG_VERSION"),
    })?;
    let temp = path.with_extension("json.tmp");
    std::fs::write(&temp, body)?;
    std::fs::rename(&temp, path)
}

pub fn remove(path: &Path) {
    if let Err(error) = std::fs::remove_file(path)
        && error.kind() != io::ErrorKind::NotFound
    {
        tracing::warn!(%error, path = %path.display(), "could not remove API discovery file");
    }
}
