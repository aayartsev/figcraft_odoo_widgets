use crate::catalog::Catalog;
use crate::native_doc::{
    class_to_snake, collect_slot_names, ensure_frame_root, load_native_document, page_root_id,
};
use anyhow::{Context, Result};
use jsonschema::Validator;
use serde_json::Value;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("{0}")]
    Message(String),
}

pub struct SchemaValidator {
    validator: Validator,
}

impl SchemaValidator {
    pub fn from_schema_path(path: &Path) -> Result<Self> {
        let text = fs::read_to_string(path)
            .with_context(|| format!("read schema {}", path.display()))?;
        let schema: Value = serde_json::from_str(&text)
            .with_context(|| format!("parse schema {}", path.display()))?;
        let validator = Validator::new(&schema)
            .map_err(|e| anyhow::anyhow!("compile JSON Schema: {e}"))?;
        Ok(Self { validator })
    }

    pub fn validate_value(&self, value: &Value, path: &Path) -> Result<()> {
        let errors: Vec<String> = self
            .validator
            .iter_errors(value)
            .map(|e| e.to_string())
            .collect();
        if !errors.is_empty() {
            anyhow::bail!(
                "schema validation failed for {}: {}",
                path.display(),
                errors.join("; ")
            );
        }
        Ok(())
    }
}

pub fn validate_catalog(root: &Path, catalog: &Catalog, catalog_schema: &SchemaValidator) -> Result<()> {
    let catalog_path = default_catalog_path(root);
    let catalog_raw = fs::read_to_string(&catalog_path)
        .with_context(|| format!("read {}", catalog_path.display()))?;
    let catalog_value: Value = serde_json::from_str(&catalog_raw)
        .with_context(|| format!("parse {}", catalog_path.display()))?;
    catalog_schema.validate_value(&catalog_value, &catalog_path)?;

    let mut seen_paths = BTreeSet::new();
    for entry in &catalog.widgets {
        let widget_path = root.join(&entry.path);
        if !widget_path.exists() {
            anyhow::bail!(
                "catalog path missing for class {}: {}",
                entry.class,
                entry.path
            );
        }
        let file_name = widget_path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("");
        if file_name == "_template.json" {
            anyhow::bail!(
                "_template.json must not appear in catalog (found {})",
                entry.path
            );
        }
        if !seen_paths.insert(entry.path.clone()) {
            anyhow::bail!("duplicate catalog path {}", entry.path);
        }

        let expected_stem = class_to_snake(&entry.class);
        let stem = widget_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("");
        if stem != expected_stem {
            anyhow::bail!(
                "file name `{}` must be snake_case of class `{}` (expected `{expected_stem}.json`)",
                file_name,
                entry.class
            );
        }

        let raw = fs::read_to_string(&widget_path)
            .with_context(|| format!("read {}", widget_path.display()))?;
        let doc = load_native_document(&raw)
            .with_context(|| format!("native document {}", widget_path.display()))?;
        let problems = doc.validate();
        if !problems.is_empty() {
            anyhow::bail!(
                "Document::validate failed for {}: {:?}",
                widget_path.display(),
                problems
            );
        }

        let root_id = page_root_id(&doc).ok_or_else(|| {
            anyhow::anyhow!("{}: no widget root on first page", widget_path.display())
        })?;
        ensure_frame_root(&doc, root_id, &entry.class)
            .with_context(|| format!("{}", widget_path.display()))?;

        let root_node = doc.node(root_id).unwrap();
        let eps = 0.51_f64;
        if (root_node.width - entry.size.width).abs() > eps
            || (root_node.height - entry.size.height).abs() > eps
        {
            anyhow::bail!(
                "{}: catalog size {}×{} != root {}×{}",
                widget_path.display(),
                entry.size.width,
                entry.size.height,
                root_node.width,
                root_node.height
            );
        }

        let declared: BTreeSet<String> = entry.slots.iter().map(|s| s.name.clone()).collect();
        let mut found = BTreeSet::new();
        collect_slot_names(&doc, root_id, &mut found);
        if declared != found {
            anyhow::bail!(
                "slots mismatch in {}: catalog {:?}, found slot:* {:?}",
                widget_path.display(),
                declared.iter().cloned().collect::<Vec<_>>(),
                found.iter().cloned().collect::<Vec<_>>()
            );
        }
    }
    Ok(())
}

pub fn schema_path(root: &Path) -> PathBuf {
    root.join("schemas/catalog.schema.json")
}

pub fn default_catalog_path(root: &Path) -> PathBuf {
    root.join("catalog.json")
}
