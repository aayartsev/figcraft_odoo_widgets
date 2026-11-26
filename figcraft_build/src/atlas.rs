//! Build the all-widgets atlas document.

use crate::catalog::Catalog;
use crate::convert::convert_node;
use crate::props::{apply_props, text_data};
use crate::widget::{FragmentNode, load_widget};
use anyhow::{Context, Result};
use figcraft_color::{Color, Paint};
use figcraft_doc::{
    ChildLayout, Document, LayoutMode, NodeId, NodeKind, Sizing, TextAlignH,
};
use figcraft_doc::geom::Affine;
use serde_json::{Map, Value, json};
use std::path::Path;

pub const ROW_WRAP_WIDTH: f64 = 2400.0;
pub const WIDGET_GAP: f64 = 40.0;

#[derive(Debug, Clone)]
pub struct AtlasOptions {
    pub doc_name: String,
}

impl Default for AtlasOptions {
    fn default() -> Self {
        Self {
            doc_name: "Odoo widgets atlas".into(),
        }
    }
}

pub fn build_atlas(root: &Path, catalog: &Catalog, opts: &AtlasOptions) -> Result<Document> {
    let mut doc = Document::new(&opts.doc_name);
    let page = doc
        .first_page()
        .ok_or_else(|| anyhow::anyhow!("Document::new must create a page"))?;

    let atlas_id = create_frame(
        &mut doc,
        page,
        "Atlas",
        0.0,
        0.0,
        100.0,
        100.0,
        atlas_root_props(),
    )?;

    for (category, entries) in catalog.entries_by_category_order() {
        let section_id = create_frame(
            &mut doc,
            atlas_id,
            &format!("category:{category}"),
            0.0,
            0.0,
            100.0,
            100.0,
            section_props(),
        )?;

        create_title_text(&mut doc, section_id, &category)?;

        let mut row_index: usize = 0;
        let mut row_id = create_row(&mut doc, section_id, row_index)?;
        let mut row_width: f64 = 0.0;

        for entry in entries {
            let path = catalog.resolve_path(root, entry);
            let widget = load_widget(&path)
                .with_context(|| format!("load widget for atlas: {}", path.display()))?;

            // Ensure root name is the class for atlas identification.
            let mut root_frag = widget.figcraft.root.clone();
            root_frag.name = widget.class.clone();

            let w = root_frag.width.unwrap_or(widget.size.width);
            if row_width > 0.0 && row_width + WIDGET_GAP + w > ROW_WRAP_WIDTH {
                row_index += 1;
                row_id = create_row(&mut doc, section_id, row_index)?;
                row_width = 0.0;
            }

            convert_node(
                &mut doc,
                row_id,
                &root_frag,
                &format!("atlas/{}/{}", category, widget.class),
            )?;

            if row_width > 0.0 {
                row_width += WIDGET_GAP;
            }
            row_width += w;
        }
    }

    // Without figcraft-engine::normalize, hug frames keep create() sizes and children
    // stay at (0,0). Resolve sizes and positions for atlas containers only.
    resolve_hug_subtree(&mut doc, atlas_id);
    Ok(doc)
}

fn atlas_root_props() -> Map<String, Value> {
    json!({
        "fill": "#F8F9FA",
        "layout": "vertical",
        "gap": 80,
        "padding": 40,
        "sizingH": "hug",
        "sizingV": "hug"
    })
    .as_object()
    .cloned()
    .expect("object")
}

fn section_props() -> Map<String, Value> {
    json!({
        "layout": "vertical",
        "gap": 24,
        "sizingH": "hug",
        "sizingV": "hug"
    })
    .as_object()
    .cloned()
    .expect("object")
}

fn row_props() -> Map<String, Value> {
    json!({
        "layout": "horizontal",
        "gap": 40,
        "sizingH": "hug",
        "sizingV": "hug"
    })
    .as_object()
    .cloned()
    .expect("object")
}

fn create_frame(
    doc: &mut Document,
    parent: NodeId,
    name: &str,
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    props: Map<String, Value>,
) -> Result<NodeId> {
    let id = doc
        .create(parent, NodeKind::Frame, name, x, y, w, h)
        .map_err(|e| anyhow::anyhow!("create frame `{name}`: {e}"))?;
    let node = doc
        .node_mut(id)
        .ok_or_else(|| anyhow::anyhow!("missing frame `{name}`"))?;
    apply_props(node, &props, name)?;
    node.clip = false;
    Ok(id)
}

fn create_row(doc: &mut Document, section: NodeId, index: usize) -> Result<NodeId> {
    create_frame(
        doc,
        section,
        &format!("row:{index}"),
        0.0,
        0.0,
        100.0,
        100.0,
        row_props(),
    )
}

fn create_title_text(doc: &mut Document, section: NodeId, category: &str) -> Result<NodeId> {
    let mut text = text_data(category);
    text.style.font_family = "Inter".into();
    text.style.font_style = "Semi Bold".into();
    text.style.font_size = 20.0;
    text.align_h = TextAlignH::Left;

    let id = doc
        .create(section, NodeKind::Text(text), category, 0.0, 0.0, 200.0, 28.0)
        .map_err(|e| anyhow::anyhow!("create title `{category}`: {e}"))?;

    if let Some(node) = doc.node_mut(id) {
        let color = Color::from_hex("#212529")
            .ok_or_else(|| anyhow::anyhow!("bad title color"))?;
        node.fills = vec![Paint::solid(color)];
        node.child_layout = ChildLayout {
            sizing_h: Sizing::Hug,
            sizing_v: Sizing::Hug,
            ..Default::default()
        };
    }
    Ok(id)
}

/// Build atlas from an in-memory list of (category, class, fragment) — for unit tests.
pub fn build_atlas_from_fragments(
    items: &[(String, String, FragmentNode)],
    opts: &AtlasOptions,
) -> Result<Document> {
    // Synthesize a minimal catalog-like grouping using CATEGORY_ORDER.
    use crate::catalog::CATEGORY_ORDER;
    use std::collections::BTreeMap;

    let mut by_cat: BTreeMap<String, Vec<&(String, String, FragmentNode)>> = BTreeMap::new();
    for item in items {
        by_cat.entry(item.0.clone()).or_default().push(item);
    }

    let mut doc = Document::new(&opts.doc_name);
    let page = doc.first_page().unwrap();
    let atlas_id = create_frame(
        &mut doc,
        page,
        "Atlas",
        0.0,
        0.0,
        100.0,
        100.0,
        atlas_root_props(),
    )?;

    for &cat in CATEGORY_ORDER {
        let Some(entries) = by_cat.get(cat) else {
            continue;
        };
        let section_id = create_frame(
            &mut doc,
            atlas_id,
            &format!("category:{cat}"),
            0.0,
            0.0,
            100.0,
            100.0,
            section_props(),
        )?;
        create_title_text(&mut doc, section_id, cat)?;

        let mut row_index = 0usize;
        let mut row_id = create_row(&mut doc, section_id, row_index)?;
        let mut row_width = 0.0;

        for (_c, class, frag) in entries {
            let mut root_frag = frag.clone();
            root_frag.name = class.clone();
            let w = root_frag.width.unwrap_or(0.0);
            if row_width > 0.0 && row_width + WIDGET_GAP + w > ROW_WRAP_WIDTH {
                row_index += 1;
                row_id = create_row(&mut doc, section_id, row_index)?;
                row_width = 0.0;
            }
            convert_node(&mut doc, row_id, &root_frag, class)?;
            if row_width > 0.0 {
                row_width += WIDGET_GAP;
            }
            row_width += w;
        }
    }

    resolve_hug_subtree(&mut doc, atlas_id);
    Ok(doc)
}

/// Bottom-up hug + place children for auto-layout frames (stand-in for normalize).
pub fn resolve_hug_subtree(doc: &mut Document, id: NodeId) {
    let children: Vec<NodeId> = doc
        .node(id)
        .map(|n| n.children.clone())
        .unwrap_or_default();
    for child in &children {
        resolve_hug_subtree(doc, *child);
    }

    let Some(node) = doc.node(id) else {
        return;
    };
    // Lightweight stand-in for normalize: any auto-layout frame gets sized/placed
    // from child box sizes already written into the document.
    let Some(layout) = node.layout.clone() else {
        return;
    };

    let child_ids = node.children.clone();
    let mut child_sizes: Vec<(f64, f64)> = Vec::new();
    for cid in &child_ids {
        if let Some(c) = doc.node(*cid) {
            child_sizes.push((c.width, c.height));
        }
    }
    if child_sizes.is_empty() {
        return;
    }

    let pad = layout.padding;
    let gap = layout.gap;
    let (content_w, content_h) = match layout.mode {
        LayoutMode::Horizontal => {
            let w = child_sizes.iter().map(|s| s.0).sum::<f64>()
                + gap * (child_sizes.len().saturating_sub(1) as f64);
            let h = child_sizes.iter().map(|s| s.1).fold(0.0_f64, f64::max);
            (w, h)
        }
        LayoutMode::Vertical => {
            let w = child_sizes.iter().map(|s| s.0).fold(0.0_f64, f64::max);
            let h = child_sizes.iter().map(|s| s.1).sum::<f64>()
                + gap * (child_sizes.len().saturating_sub(1) as f64);
            (w, h)
        }
        LayoutMode::Grid => {
            let w = child_sizes.iter().map(|s| s.0).fold(0.0_f64, f64::max);
            let h = child_sizes.iter().map(|s| s.1).sum::<f64>();
            (w, h)
        }
    };

    // Place children (padding top/left, then along main axis).
    let mut cursor = 0.0_f64;
    for (i, cid) in child_ids.iter().enumerate() {
        let (cw, _ch) = child_sizes[i];
        let (x, y) = match layout.mode {
            LayoutMode::Horizontal => (pad[3] + cursor, pad[0]),
            LayoutMode::Vertical | LayoutMode::Grid => (pad[3], pad[0] + cursor),
        };
        if let Some(child) = doc.node_mut(*cid) {
            child.transform = Affine::translate((x, y));
        }
        let step = match layout.mode {
            LayoutMode::Horizontal => cw,
            LayoutMode::Vertical | LayoutMode::Grid => child_sizes[i].1,
        };
        cursor += step + gap;
    }

    if let Some(n) = doc.node_mut(id) {
        // Keep JSON/create sizes when the axis is Fixed; only hug axes expand.
        if n.child_layout.sizing_h == Sizing::Hug {
            n.width = content_w + pad[1] + pad[3];
        }
        if n.child_layout.sizing_v == Sizing::Hug {
            n.height = content_h + pad[0] + pad[2];
        }
        n.clip = false;
    }
}

/// Collect top-level section names under Atlas (category:*).
pub fn atlas_section_names(doc: &Document) -> Result<Vec<String>> {
    let page = doc
        .first_page()
        .ok_or_else(|| anyhow::anyhow!("no page"))?;
    let page_node = doc
        .node(page)
        .ok_or_else(|| anyhow::anyhow!("missing page"))?;
    let atlas = page_node
        .children
        .iter()
        .find_map(|id| {
            doc.node(*id)
                .filter(|n| n.name == "Atlas")
                .map(|_| *id)
        })
        .ok_or_else(|| anyhow::anyhow!("Atlas frame not found"))?;
    let atlas_node = doc.node(atlas).unwrap();
    Ok(atlas_node
        .children
        .iter()
        .filter_map(|id| doc.node(*id).map(|n| n.name.clone()))
        .filter(|n| n.starts_with("category:"))
        .collect())
}
