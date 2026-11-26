//! Compare a fragment tree to a FigCraft document subtree.

use crate::widget::FragmentNode;
use figcraft_color::{Color, Paint};
use figcraft_doc::{
    CrossAlign, Document, LayoutMode, MainAlign, NodeId, NodeKind, Sizing, StrokeAlign, TextAlignH,
};
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct Diff {
    pub path: String,
    pub message: String,
}

pub fn compare_fragment_to_doc(
    doc: &Document,
    root_id: NodeId,
    fragment: &FragmentNode,
) -> Vec<Diff> {
    let mut diffs = Vec::new();
    compare_node(doc, root_id, fragment, "root", &mut diffs);
    diffs
}

fn compare_node(
    doc: &Document,
    id: NodeId,
    frag: &FragmentNode,
    path: &str,
    diffs: &mut Vec<Diff>,
) {
    let Some(node) = doc.node(id) else {
        diffs.push(Diff {
            path: path.into(),
            message: "missing node in document".into(),
        });
        return;
    };

    let expected_type = match frag.type_name.as_str() {
        "frame" => "FRAME",
        "rectangle" => "RECTANGLE",
        "ellipse" => "ELLIPSE",
        "text" => "TEXT",
        other => other,
    };
    if node.kind.type_name() != expected_type {
        diffs.push(Diff {
            path: path.into(),
            message: format!(
                "type: json `{}` → doc `{}`",
                frag.type_name,
                node.kind.type_name()
            ),
        });
    }

    if node.name != frag.name {
        diffs.push(Diff {
            path: path.into(),
            message: format!("name: json `{}` → doc `{}`", frag.name, node.name),
        });
    }

    check_f64(path, "width", frag.width.unwrap_or(0.0), node.width, diffs);
    check_f64(path, "height", frag.height.unwrap_or(0.0), node.height, diffs);
    // transform translation ≈ x/y
    let t = node.transform.translation();
    check_f64(path, "x", frag.x.unwrap_or(0.0), t.x, diffs);
    check_f64(path, "y", frag.y.unwrap_or(0.0), t.y, diffs);

    if frag.type_name == "text" {
        let expected = frag.text.clone().unwrap_or_default();
        let actual = node
            .kind
            .text()
            .map(|t| t.characters.clone())
            .unwrap_or_default();
        if expected != actual {
            diffs.push(Diff {
                path: path.into(),
                message: format!("text: json `{expected}` → doc `{actual}`"),
            });
        }
    }

    if let Some(props) = &frag.props {
        compare_props(node, props, path, diffs);
    }

    let frag_children = frag.children.as_deref().unwrap_or(&[]);
    if node.children.len() != frag_children.len() {
        diffs.push(Diff {
            path: path.into(),
            message: format!(
                "children len: json {} → doc {}",
                frag_children.len(),
                node.children.len()
            ),
        });
    }
    let n = frag_children.len().min(node.children.len());
    for i in 0..n {
        let child_path = format!("{path}/{}", frag_children[i].name);
        compare_node(doc, node.children[i], &frag_children[i], &child_path, diffs);
    }
}

fn compare_props(
    node: &figcraft_doc::Node,
    props: &serde_json::Map<String, Value>,
    path: &str,
    diffs: &mut Vec<Diff>,
) {
    for (key, value) in props {
        match key.as_str() {
            "fill" => {
                let Some(expected) = value.as_str() else { continue };
                let actual = first_solid_hex(&node.fills);
                if !color_eq(expected, &actual) {
                    diffs.push(Diff {
                        path: path.into(),
                        message: format!("fill: json `{expected}` → doc `{actual}`"),
                    });
                }
            }
            "stroke" => {
                let Some(expected) = value.as_str() else { continue };
                let actual = first_solid_hex(&node.strokes);
                if !color_eq(expected, &actual) {
                    diffs.push(Diff {
                        path: path.into(),
                        message: format!("stroke: json `{expected}` → doc `{actual}`"),
                    });
                }
            }
            "strokeWeight" => {
                if let Some(v) = value.as_f64() {
                    check_f64(path, "strokeWeight", v, node.stroke.weight, diffs);
                }
            }
            "strokeAlign" => {
                if let Some(s) = value.as_str() {
                    let actual = match node.stroke.align {
                        StrokeAlign::Inside => "inside",
                        StrokeAlign::Center => "center",
                        StrokeAlign::Outside => "outside",
                    };
                    if s != actual {
                        diffs.push(Diff {
                            path: path.into(),
                            message: format!("strokeAlign: json `{s}` → doc `{actual}`"),
                        });
                    }
                }
            }
            "cornerRadius" => {
                if let Some(v) = value.as_f64() {
                    let r = node.corners.radii[0];
                    check_f64(path, "cornerRadius", v, r, diffs);
                }
            }
            "layout" => {
                if let Some(s) = value.as_str() {
                    let actual = match node.layout.as_ref().map(|l| l.mode) {
                        Some(LayoutMode::Horizontal) => "horizontal",
                        Some(LayoutMode::Vertical) => "vertical",
                        Some(LayoutMode::Grid) => "grid",
                        None => "none",
                    };
                    if s != actual {
                        diffs.push(Diff {
                            path: path.into(),
                            message: format!("layout: json `{s}` → doc `{actual}`"),
                        });
                    }
                }
            }
            "gap" => {
                if let Some(v) = value.as_f64() {
                    let actual = node.layout.as_ref().map(|l| l.gap).unwrap_or(0.0);
                    check_f64(path, "gap", v, actual, diffs);
                }
            }
            "padding" => {
                let expected = match value {
                    Value::Number(n) => {
                        let v = n.as_f64().unwrap_or(0.0);
                        [v, v, v, v]
                    }
                    Value::Array(a) if a.len() == 4 => {
                        let mut out = [0.0; 4];
                        for (i, item) in a.iter().enumerate() {
                            out[i] = item.as_f64().unwrap_or(0.0);
                        }
                        out
                    }
                    _ => continue,
                };
                let actual = node.layout.as_ref().map(|l| l.padding).unwrap_or([0.0; 4]);
                for i in 0..4 {
                    check_f64(path, &format!("padding[{i}]"), expected[i], actual[i], diffs);
                }
            }
            "mainAlign" => {
                if let Some(s) = value.as_str() {
                    let actual = match node.layout.as_ref().map(|l| l.main_align) {
                        Some(MainAlign::Min) => "min",
                        Some(MainAlign::Center) => "center",
                        Some(MainAlign::Max) => "max",
                        Some(MainAlign::SpaceBetween) => "spaceBetween",
                        Some(MainAlign::SpaceAround) => "spaceAround",
                        Some(MainAlign::SpaceEvenly) => "spaceEvenly",
                        None => "none",
                    };
                    if s != actual {
                        diffs.push(Diff {
                            path: path.into(),
                            message: format!("mainAlign: json `{s}` → doc `{actual}`"),
                        });
                    }
                }
            }
            "crossAlign" => {
                if let Some(s) = value.as_str() {
                    let actual = match node.layout.as_ref().map(|l| l.cross_align) {
                        Some(CrossAlign::Min) => "min",
                        Some(CrossAlign::Center) => "center",
                        Some(CrossAlign::Max) => "max",
                        Some(CrossAlign::Baseline) => "baseline",
                        None => "none",
                    };
                    if s != actual {
                        diffs.push(Diff {
                            path: path.into(),
                            message: format!("crossAlign: json `{s}` → doc `{actual}`"),
                        });
                    }
                }
            }
            "wrap" => {
                if let Some(v) = value.as_bool() {
                    let actual = node.layout.as_ref().map(|l| l.wrap).unwrap_or(false);
                    if v != actual {
                        diffs.push(Diff {
                            path: path.into(),
                            message: format!("wrap: json `{v}` → doc `{actual}`"),
                        });
                    }
                }
            }
            "sizingH" => {
                if let Some(s) = value.as_str() {
                    let actual = sizing_str(node.child_layout.sizing_h);
                    if s != actual {
                        diffs.push(Diff {
                            path: path.into(),
                            message: format!("sizingH: json `{s}` → doc `{actual}`"),
                        });
                    }
                }
            }
            "sizingV" => {
                if let Some(s) = value.as_str() {
                    let actual = sizing_str(node.child_layout.sizing_v);
                    if s != actual {
                        diffs.push(Diff {
                            path: path.into(),
                            message: format!("sizingV: json `{s}` → doc `{actual}`"),
                        });
                    }
                }
            }
            "fontFamily" | "fontStyle" | "fontSize" | "textAlign" => {
                let Some(text) = node.kind.text() else {
                    diffs.push(Diff {
                        path: path.into(),
                        message: format!("`{key}` on non-text node"),
                    });
                    continue;
                };
                match key.as_str() {
                    "fontFamily" => {
                        if let Some(s) = value.as_str() {
                            if text.style.font_family != s {
                                diffs.push(Diff {
                                    path: path.into(),
                                    message: format!(
                                        "fontFamily: json `{s}` → doc `{}`",
                                        text.style.font_family
                                    ),
                                });
                            }
                        }
                    }
                    "fontStyle" => {
                        if let Some(s) = value.as_str() {
                            if text.style.font_style != s {
                                diffs.push(Diff {
                                    path: path.into(),
                                    message: format!(
                                        "fontStyle: json `{s}` → doc `{}`",
                                        text.style.font_style
                                    ),
                                });
                            }
                        }
                    }
                    "fontSize" => {
                        if let Some(v) = value.as_f64() {
                            check_f64(path, "fontSize", v, text.style.font_size, diffs);
                        }
                    }
                    "textAlign" => {
                        if let Some(s) = value.as_str() {
                            let actual = match text.align_h {
                                TextAlignH::Left => "left",
                                TextAlignH::Center => "center",
                                TextAlignH::Right => "right",
                                TextAlignH::Justified => "justified",
                            };
                            if s != actual {
                                diffs.push(Diff {
                                    path: path.into(),
                                    message: format!("textAlign: json `{s}` → doc `{actual}`"),
                                });
                            }
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
}

fn sizing_str(s: Sizing) -> &'static str {
    match s {
        Sizing::Fixed => "fixed",
        Sizing::Hug => "hug",
        Sizing::Fill => "fill",
    }
}

fn first_solid_hex(paints: &[Paint]) -> String {
    paints
        .first()
        .and_then(|p| p.solid_color())
        .map(|c| c.to_css_hex())
        .unwrap_or_else(|| "(none)".into())
}

fn color_eq(expected: &str, actual: &str) -> bool {
    let Some(e) = Color::from_hex(expected.trim()) else {
        return false;
    };
    let Some(a) = Color::from_hex(actual.trim()) else {
        return expected.trim().eq_ignore_ascii_case(actual.trim());
    };
    const EPS: f32 = 1.5 / 255.0;
    (e.r - a.r).abs() < EPS
        && (e.g - a.g).abs() < EPS
        && (e.b - a.b).abs() < EPS
        && (e.a - a.a).abs() < EPS
}

fn check_f64(path: &str, field: &str, expected: f64, actual: f64, diffs: &mut Vec<Diff>) {
    if (expected - actual).abs() > 0.01 {
        diffs.push(Diff {
            path: path.into(),
            message: format!("{field}: json {expected} → doc {actual}"),
        });
    }
}

/// Root child of the first page (the exported widget).
pub fn page_root_id(doc: &Document) -> Option<NodeId> {
    let page = doc.first_page()?;
    let page_node = doc.node(page)?;
    page_node.children.first().copied()
}

pub fn assert_kind_is_container(kind: &NodeKind) -> bool {
    kind.is_container()
}
