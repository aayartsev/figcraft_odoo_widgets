use std::path::{Path, PathBuf};

/// Resolve the repository root that contains `catalog.json` and `widgets/`.
///
/// Default: parent of the `figcraft_build` crate (`CARGO_MANIFEST_DIR/..`),
/// overridden by `--root` when provided.
pub fn resolve_repo_root(explicit: Option<&Path>) -> PathBuf {
    if let Some(p) = explicit {
        return p.to_path_buf();
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."))
}
