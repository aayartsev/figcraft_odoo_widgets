use anyhow::{Context, Result, bail};
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

/// Stable category order for the atlas (format spec table).
pub const CATEGORY_ORDER: &[&str] = &[
    "shell",
    "navbar_widgets",
    "menu",
    "views",
    "fields",
    "custom_views",
    "discussion",
    "custom",
];

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Catalog {
    pub schema_version: u32,
    pub odoo_version: String,
    pub widgets: Vec<CatalogEntry>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CatalogEntry {
    pub class: String,
    pub category: String,
    pub path: String,
}

pub fn load_catalog(path: &Path) -> Result<Catalog> {
    let text = fs::read_to_string(path)
        .with_context(|| format!("read catalog {}", path.display()))?;
    let catalog: Catalog = serde_json::from_str(&text)
        .with_context(|| format!("parse catalog {}", path.display()))?;
    if catalog.schema_version != 1 {
        bail!(
            "unsupported catalog schemaVersion {} (expected 1)",
            catalog.schema_version
        );
    }
    Ok(catalog)
}

impl Catalog {
    pub fn entries_by_category_order(&self) -> Vec<(String, Vec<&CatalogEntry>)> {
        let mut out = Vec::new();
        for &cat in CATEGORY_ORDER {
            let entries: Vec<&CatalogEntry> = self
                .widgets
                .iter()
                .filter(|w| w.category == cat)
                .collect();
            if !entries.is_empty() {
                out.push((cat.to_string(), entries));
            }
        }
        out
    }

    pub fn resolve_path(&self, root: &Path, entry: &CatalogEntry) -> PathBuf {
        root.join(&entry.path)
    }
}
