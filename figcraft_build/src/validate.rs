use crate::catalog::Catalog;
use crate::widget::{FragmentNode, WidgetDoc, class_to_snake, load_widget};
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

pub fn validate_catalog(root: &Path, catalog: &Catalog, schema: &SchemaValidator) -> Result<()> {
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

        let raw = fs::read_to_string(&widget_path)
            .with_context(|| format!("read {}", widget_path.display()))?;
        let value: Value = serde_json::from_str(&raw)
            .with_context(|| format!("parse JSON {}", widget_path.display()))?;
        schema.validate_value(&value, &widget_path)?;

        let widget = load_widget(&widget_path)?;
        validate_widget(&widget, &widget_path, Some(entry))?;
    }
    Ok(())
}

pub fn validate_widget(
    widget: &WidgetDoc,
    path: &Path,
    catalog_entry: Option<&crate::catalog::CatalogEntry>,
) -> Result<()> {
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("");
    let expected = class_to_snake(&widget.class);
    if stem != expected {
        anyhow::bail!(
            "file name `{}` must be snake_case of class `{}` (expected `{expected}.json`)",
            path.file_name().and_then(|s| s.to_str()).unwrap_or("?"),
            widget.class
        );
    }

    if let Some(entry) = catalog_entry {
        if entry.class != widget.class {
            anyhow::bail!(
                "catalog class `{}` != widget class `{}` in {}",
                entry.class,
                widget.class,
                path.display()
            );
        }
        if entry.category != widget.category {
            anyhow::bail!(
                "catalog category `{}` != widget category `{}` in {}",
                entry.category,
                widget.category,
                path.display()
            );
        }
    }

    if widget.figcraft.kind != "fragment" {
        anyhow::bail!(
            "figcraft.kind must be `fragment` in {}",
            path.display()
        );
    }

    let slot_names: BTreeSet<String> = widget.slots.iter().map(|s| s.name.clone()).collect();
    let mut found_slots = BTreeSet::new();
    collect_slot_frames(&widget.figcraft.root, &mut found_slots);

    if slot_names != found_slots {
        anyhow::bail!(
            "slots mismatch in {}: declared {:?}, found slot:* {:?}",
            path.display(),
            slot_names.iter().cloned().collect::<Vec<_>>(),
            found_slots.iter().cloned().collect::<Vec<_>>()
        );
    }

    Ok(())
}

fn collect_slot_frames(node: &FragmentNode, out: &mut BTreeSet<String>) {
    if let Some(name) = node.name.strip_prefix("slot:") {
        out.insert(name.to_string());
    }
    if let Some(children) = &node.children {
        for child in children {
            collect_slot_frames(child, out);
        }
    }
}

pub fn schema_path(root: &Path) -> PathBuf {
    root.join("schemas/widget.schema.json")
}

pub fn default_catalog_path(root: &Path) -> PathBuf {
    root.join("catalog.json")
}
