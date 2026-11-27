use anyhow::{Context, Result, bail};
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

/// Stable category order for docs / future compose.
pub const CATEGORY_ORDER: &[&str] = &[
    "shell",
    "navbar_widgets",
    "menu",
    "views",
    "fields",
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
#[serde(rename_all = "camelCase")]
pub struct CatalogEntry {
    pub class: String,
    pub category: String,
    pub path: String,
    pub description: String,
    pub size: Size,
    pub slots: Vec<Slot>,
    pub variants: Vec<String>,
    #[serde(default)]
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Size {
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Slot {
    pub name: String,
    pub accepts: Vec<String>,
}

pub fn load_catalog(path: &Path) -> Result<Catalog> {
    let text = fs::read_to_string(path)
        .with_context(|| format!("read catalog {}", path.display()))?;
    let catalog: Catalog = serde_json::from_str(&text)
        .with_context(|| format!("parse catalog {}", path.display()))?;
    if catalog.schema_version != 2 {
        bail!(
            "unsupported catalog schemaVersion {} (expected 2)",
            catalog.schema_version
        );
    }
    Ok(catalog)
}

impl Catalog {
    pub fn resolve_path(&self, root: &Path, entry: &CatalogEntry) -> PathBuf {
        root.join(&entry.path)
    }
}
