//! Export one catalog widget as a standalone `.figcraft` document.

use crate::catalog::Catalog;
use crate::convert::convert_node;
use crate::validate::validate_widget;
use crate::widget::{class_to_snake, load_widget};
use anyhow::{Context, Result, bail};
use figcraft_doc::Document;
use std::path::Path;

/// Build a document that mirrors the fragment JSON (text without size gets an estimated box).
pub fn build_widget_document(root: &Path, catalog: &Catalog, class: &str) -> Result<Document> {
    let entry = catalog
        .widgets
        .iter()
        .find(|w| w.class == class)
        .ok_or_else(|| anyhow::anyhow!("class `{class}` not found in catalog"))?;

    let path = catalog.resolve_path(root, entry);
    let widget = load_widget(&path).with_context(|| format!("load {}", path.display()))?;
    validate_widget(&widget, &path, Some(entry))?;

    let mut doc = Document::new(&widget.class);
    let page = doc
        .first_page()
        .ok_or_else(|| anyhow::anyhow!("Document::new must create a page"))?;

    // Keep the fragment root name from JSON so export matches the source tree.
    convert_node(&mut doc, page, &widget.figcraft.root, &widget.class)?;
    Ok(doc)
}

pub fn default_widget_out(root: &Path, class: &str) -> Result<std::path::PathBuf> {
    if class.is_empty() {
        bail!("empty class");
    }
    Ok(root.join("dist").join(format!("{}.figcraft", class_to_snake(class))))
}
