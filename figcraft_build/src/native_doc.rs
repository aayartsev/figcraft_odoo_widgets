//! Load native FigCraft document JSON and find the widget root / slot frames.

use anyhow::{Result, bail};
use figcraft_doc::{Document, NodeId, NodeKind};
use std::collections::BTreeSet;

pub fn load_native_document(raw: &str) -> Result<Document> {
    let (doc, _issues) = figcraft_format::from_json(raw)
        .map_err(|e| anyhow::anyhow!("FigCraft JSON: {e}"))?;
    Ok(doc)
}

/// First non-page child of the first page (widget root).
pub fn page_root_id(doc: &Document) -> Option<NodeId> {
    let page = doc.first_page()?;
    let node = doc.node(page)?;
    node.children.first().copied()
}

pub fn collect_slot_names(doc: &Document, id: NodeId, out: &mut BTreeSet<String>) {
    let Some(node) = doc.node(id) else {
        return;
    };
    if let Some(name) = node.name.strip_prefix("slot:") {
        out.insert(name.to_string());
    }
    for child in &node.children {
        collect_slot_names(doc, *child, out);
    }
}

pub fn class_to_snake(class: &str) -> String {
    let mut out = String::new();
    for (i, ch) in class.chars().enumerate() {
        if ch.is_uppercase() {
            if i > 0 {
                out.push('_');
            }
            for c in ch.to_lowercase() {
                out.push(c);
            }
        } else {
            out.push(ch);
        }
    }
    out
}

pub fn ensure_frame_root(doc: &Document, root_id: NodeId, class: &str) -> Result<()> {
    let node = doc
        .node(root_id)
        .ok_or_else(|| anyhow::anyhow!("missing root node"))?;
    if node.name != class {
        bail!(
            "root layer name `{}` != catalog class `{}`",
            node.name,
            class
        );
    }
    if !matches!(node.kind, NodeKind::Frame) {
        bail!("widget root must be a frame, got {:?}", node.kind);
    }
    Ok(())
}
