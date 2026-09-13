//! Opt-in project-owned native component adapters, discovered by the existing catalog.
use std::path::{Path, PathBuf};

const SOURCE: &str = include_str!("../templates/TimelineAdapters.hpp");

/// Publish once. Installed sources belong to the project; upgrades never replace edits.
pub fn install(root: &Path) -> Result<PathBuf, String> {
    let path = crate::assets::inside(root, "assets/scripts/TimelineAdapters.hpp")?;
    if path.exists() {
        if std::fs::read(&path).map_err(|e| e.to_string())? == SOURCE.as_bytes() {
            return Ok(path);
        }
        return Err(format!(
            "{} already exists. Its contents were preserved; merge adapter updates manually.",
            path.display()
        ));
    }
    std::fs::create_dir_all(path.parent().ok_or("Missing script directory")?)
        .map_err(|e| e.to_string())?;
    crate::assets::atomic_write(&path, SOURCE.as_bytes(), None)?;
    Ok(path)
}
