//! Convert a widget fragment tree into FigCraft document nodes.

use crate::props::{apply_props, text_data};
use crate::widget::FragmentNode;
use anyhow::{Result, bail};
use figcraft_doc::{Document, NodeId, NodeKind};

/// Build a standalone document whose page holds a single converted fragment root.
pub fn fragment_to_document(root: &FragmentNode, doc_name: &str) -> Result<Document> {
    let mut doc = Document::new(doc_name);
    let page = doc
        .first_page()
        .ok_or_else(|| anyhow::anyhow!("Document::new must create a page"))?;
    convert_node(&mut doc, page, root, "root")?;
    Ok(doc)
}

/// Convert `node` as a child of `parent`, return created id.
pub fn convert_node(
    doc: &mut Document,
    parent: NodeId,
    node: &FragmentNode,
    path: &str,
) -> Result<NodeId> {
    let kind = node_kind(node, path)?;
    let x = node.x.unwrap_or(0.0);
    let y = node.y.unwrap_or(0.0);
    let w = node.width.unwrap_or(0.0);
    let h = node.height.unwrap_or(0.0);

    let id = doc
        .create(parent, kind, &node.name, x, y, w, h)
        .map_err(|e| anyhow::anyhow!("create node at {path}: {e}"))?;

    if let Some(props) = &node.props {
        let n = doc
            .node_mut(id)
            .ok_or_else(|| anyhow::anyhow!("missing node after create at {path}"))?;
        apply_props(n, props, path)?;
    }

    if let Some(children) = &node.children {
        for (i, child) in children.iter().enumerate() {
            let child_path = format!("{path}/{}", child.name);
            convert_node(doc, id, child, &child_path)
                .map_err(|e| anyhow::anyhow!("child[{i}] {child_path}: {e}"))?;
        }
    }

    Ok(id)
}

fn node_kind(node: &FragmentNode, path: &str) -> Result<NodeKind> {
    match node.type_name.as_str() {
        "frame" => Ok(NodeKind::Frame),
        "rectangle" => Ok(NodeKind::Rectangle),
        "ellipse" => Ok(NodeKind::Ellipse {
            start: 0.0,
            sweep: std::f64::consts::TAU,
            inner: 0.0,
        }),
        "text" => {
            let characters = node.text.clone().unwrap_or_default();
            Ok(NodeKind::Text(text_data(characters)))
        }
        other => bail!("unsupported fragment type `{other}` at {path}"),
    }
}

/// Walk a document tree collecting (name, type_name, width, height) for determinism checks.
pub fn collect_tree_summary(doc: &Document, id: NodeId) -> Vec<(String, String, f64, f64)> {
    let mut out = Vec::new();
    walk_summary(doc, id, &mut out);
    out
}

fn walk_summary(doc: &Document, id: NodeId, out: &mut Vec<(String, String, f64, f64)>) {
    let Some(node) = doc.node(id) else {
        return;
    };
    out.push((
        node.name.clone(),
        node.kind.type_name().to_string(),
        node.width,
        node.height,
    ));
    for child in &node.children {
        walk_summary(doc, *child, out);
    }
}
