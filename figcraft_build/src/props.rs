//! Map fragment `props` keys onto a FigCraft [`figcraft_doc::Node`].

use anyhow::{Result, bail};
use figcraft_color::{Color, Paint};
use figcraft_doc::{
    AutoLayout, ChildLayout, Corners, CrossAlign, LayoutMode, MainAlign, Node, Sizing, StrokeAlign,
    TextAlignH, TextData,
};
use serde_json::Value;
use std::collections::BTreeSet;

/// Props keys allowed in convert v1 (union of keys present in the catalog).
pub const ALLOWED_PROPS: &[&str] = &[
    "fill",
    "stroke",
    "strokeWeight",
    "strokeAlign",
    "cornerRadius",
    "layout",
    "gap",
    "padding",
    "mainAlign",
    "crossAlign",
    "wrap",
    "sizingH",
    "sizingV",
    "fontFamily",
    "fontStyle",
    "fontSize",
    "textAlign",
];

fn allowed_set() -> BTreeSet<&'static str> {
    ALLOWED_PROPS.iter().copied().collect()
}

pub fn reject_unknown_props(props: &serde_json::Map<String, Value>, path: &str) -> Result<()> {
    let allowed = allowed_set();
    for key in props.keys() {
        if !allowed.contains(key.as_str()) {
            bail!("unknown props key `{key}` at {path}");
        }
    }
    Ok(())
}

pub fn apply_props(node: &mut Node, props: &serde_json::Map<String, Value>, path: &str) -> Result<()> {
    reject_unknown_props(props, path)?;

    // Layout-related keys need an AutoLayout; use zero gaps/padding until set explicitly.
    let needs_layout = props.keys().any(|k| {
        matches!(
            k.as_str(),
            "layout" | "gap" | "padding" | "mainAlign" | "crossAlign" | "wrap"
        )
    });
    if needs_layout && node.layout.is_none() {
        node.layout = Some(blank_auto_layout());
    }

    for (key, value) in props {
        match key.as_str() {
            "fill" => {
                let color = parse_color(value, path, "fill")?;
                node.fills = vec![Paint::solid(color)];
            }
            "stroke" => {
                let color = parse_color(value, path, "stroke")?;
                node.strokes = vec![Paint::solid(color)];
            }
            "strokeWeight" => {
                node.stroke.weight = as_f64(value, path, "strokeWeight")?;
            }
            "strokeAlign" => {
                node.stroke.align = parse_stroke_align(value, path)?;
            }
            "cornerRadius" => {
                let r = as_f64(value, path, "cornerRadius")?;
                node.corners = Corners {
                    radii: [r, r, r, r],
                    smoothing: 0.0,
                };
            }
            "layout" => {
                let mode = parse_layout_mode(value, path)?;
                ensure_layout(node).mode = mode;
            }
            "gap" => {
                ensure_layout(node).gap = as_f64(value, path, "gap")?;
            }
            "padding" => {
                ensure_layout(node).padding = parse_padding(value, path)?;
            }
            "mainAlign" => {
                ensure_layout(node).main_align = parse_main_align(value, path)?;
            }
            "crossAlign" => {
                ensure_layout(node).cross_align = parse_cross_align(value, path)?;
            }
            "wrap" => {
                ensure_layout(node).wrap = as_bool(value, path, "wrap")?;
            }
            "sizingH" => {
                ensure_child_layout(node).sizing_h = parse_sizing(value, path, "sizingH")?;
            }
            "sizingV" => {
                ensure_child_layout(node).sizing_v = parse_sizing(value, path, "sizingV")?;
            }
            "fontFamily" | "fontStyle" | "fontSize" | "textAlign" => {
                apply_text_prop(node, key, value, path)?;
            }
            other => bail!("unknown props key `{other}` at {path}"),
        }
    }
    Ok(())
}

fn blank_auto_layout() -> AutoLayout {
    AutoLayout {
        mode: LayoutMode::Horizontal,
        wrap: false,
        gap: 0.0,
        cross_gap: 0.0,
        padding: [0.0; 4],
        main_align: MainAlign::Min,
        cross_align: CrossAlign::Min,
        content_align: Default::default(),
        grid_columns: 2,
        grid_rows: 2,
        column_tracks: Vec::new(),
        row_tracks: Vec::new(),
        strokes_included: true,
        reverse_z: false,
    }
}

fn ensure_layout(node: &mut Node) -> &mut AutoLayout {
    if node.layout.is_none() {
        node.layout = Some(blank_auto_layout());
    }
    node.layout.as_mut().expect("layout just set")
}

fn ensure_child_layout(node: &mut Node) -> &mut ChildLayout {
    &mut node.child_layout
}

fn apply_text_prop(node: &mut Node, key: &str, value: &Value, path: &str) -> Result<()> {
    let Some(text) = node.kind.text_mut() else {
        bail!("props `{key}` requires a text node at {path}");
    };
    match key {
        "fontFamily" => {
            text.style.font_family = as_str(value, path, "fontFamily")?.to_string();
        }
        "fontStyle" => {
            text.style.font_style = as_str(value, path, "fontStyle")?.to_string();
        }
        "fontSize" => {
            text.style.font_size = as_f64(value, path, "fontSize")?;
        }
        "textAlign" => {
            text.align_h = parse_text_align(value, path)?;
        }
        _ => unreachable!(),
    }
    Ok(())
}

pub fn parse_color(value: &Value, path: &str, key: &str) -> Result<Color> {
    let s = as_str(value, path, key)?;
    Color::from_hex(s).ok_or_else(|| anyhow::anyhow!("invalid color `{s}` for `{key}` at {path}"))
}

fn parse_padding(value: &Value, path: &str) -> Result<[f64; 4]> {
    match value {
        Value::Number(n) => {
            let v = n
                .as_f64()
                .ok_or_else(|| anyhow::anyhow!("invalid padding number at {path}"))?;
            Ok([v, v, v, v])
        }
        Value::Array(arr) if arr.len() == 4 => {
            let mut out = [0.0; 4];
            for (i, item) in arr.iter().enumerate() {
                out[i] = as_f64(item, path, "padding")?;
            }
            Ok(out)
        }
        _ => bail!("padding must be a number or [t,r,b,l] at {path}"),
    }
}

fn parse_layout_mode(value: &Value, path: &str) -> Result<LayoutMode> {
    match as_str(value, path, "layout")? {
        "horizontal" => Ok(LayoutMode::Horizontal),
        "vertical" => Ok(LayoutMode::Vertical),
        "grid" => Ok(LayoutMode::Grid),
        other => bail!("unsupported layout `{other}` at {path}"),
    }
}

fn parse_main_align(value: &Value, path: &str) -> Result<MainAlign> {
    match as_str(value, path, "mainAlign")? {
        "min" => Ok(MainAlign::Min),
        "center" => Ok(MainAlign::Center),
        "max" => Ok(MainAlign::Max),
        "spaceBetween" => Ok(MainAlign::SpaceBetween),
        "spaceAround" => Ok(MainAlign::SpaceAround),
        "spaceEvenly" => Ok(MainAlign::SpaceEvenly),
        other => bail!("unsupported mainAlign `{other}` at {path}"),
    }
}

fn parse_cross_align(value: &Value, path: &str) -> Result<CrossAlign> {
    match as_str(value, path, "crossAlign")? {
        "min" => Ok(CrossAlign::Min),
        "center" => Ok(CrossAlign::Center),
        "max" => Ok(CrossAlign::Max),
        "baseline" => Ok(CrossAlign::Baseline),
        other => bail!("unsupported crossAlign `{other}` at {path}"),
    }
}

fn parse_sizing(value: &Value, path: &str, key: &str) -> Result<Sizing> {
    match as_str(value, path, key)? {
        "fixed" => Ok(Sizing::Fixed),
        "hug" => Ok(Sizing::Hug),
        "fill" => Ok(Sizing::Fill),
        other => bail!("unsupported {key} `{other}` at {path}"),
    }
}

fn parse_stroke_align(value: &Value, path: &str) -> Result<StrokeAlign> {
    match as_str(value, path, "strokeAlign")? {
        "inside" => Ok(StrokeAlign::Inside),
        "center" => Ok(StrokeAlign::Center),
        "outside" => Ok(StrokeAlign::Outside),
        other => bail!("unsupported strokeAlign `{other}` at {path}"),
    }
}

fn parse_text_align(value: &Value, path: &str) -> Result<TextAlignH> {
    match as_str(value, path, "textAlign")? {
        "left" => Ok(TextAlignH::Left),
        "center" => Ok(TextAlignH::Center),
        "right" => Ok(TextAlignH::Right),
        "justified" => Ok(TextAlignH::Justified),
        other => bail!("unsupported textAlign `{other}` at {path}"),
    }
}

fn as_str<'a>(value: &'a Value, path: &str, key: &str) -> Result<&'a str> {
    value
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("`{key}` must be a string at {path}"))
}

fn as_f64(value: &Value, path: &str, key: &str) -> Result<f64> {
    value
        .as_f64()
        .ok_or_else(|| anyhow::anyhow!("`{key}` must be a number at {path}"))
}

fn as_bool(value: &Value, path: &str, key: &str) -> Result<bool> {
    value
        .as_bool()
        .ok_or_else(|| anyhow::anyhow!("`{key}` must be a boolean at {path}"))
}

/// Helper for tests / convert: create TextData with characters.
pub fn text_data(characters: impl Into<String>) -> TextData {
    TextData::new(characters)
}

#[cfg(test)]
mod tests {
    use super::*;
    use figcraft_doc::{NodeId, NodeKind};
    use serde_json::json;

    fn frame_node() -> Node {
        Node::new(NodeId(1), NodeKind::Frame, "F", 100.0, 40.0)
    }

    #[test]
    fn parse_hex_and_alpha() {
        let c = parse_color(&json!("#714B67"), "t", "fill").unwrap();
        assert!((c.a - 1.0).abs() < f32::EPSILON);
        let a = parse_color(&json!("#FFFFFFCC"), "t", "fill").unwrap();
        assert!((a.a - (0xCCu8 as f32 / 255.0)).abs() < 0.02);
        let paint = Paint::solid(c);
        assert!(paint.solid_color().is_some());
    }

    #[test]
    fn padding_number_and_array() {
        assert_eq!(parse_padding(&json!(8), "t").unwrap(), [8.0; 4]);
        assert_eq!(
            parse_padding(&json!([0, 12, 0, 12]), "t").unwrap(),
            [0.0, 12.0, 0.0, 12.0]
        );
    }

    #[test]
    fn unknown_prop_errors() {
        let mut n = frame_node();
        let props = json!({"nope": 1}).as_object().unwrap().clone();
        assert!(apply_props(&mut n, &props, "w").is_err());
    }

    #[test]
    fn sizing_ok() {
        let mut n = frame_node();
        let props = json!({"sizingH": "fill", "sizingV": "hug"})
            .as_object()
            .unwrap()
            .clone();
        apply_props(&mut n, &props, "w").unwrap();
        assert_eq!(n.child_layout.sizing_h, Sizing::Fill);
        assert_eq!(n.child_layout.sizing_v, Sizing::Hug);
    }
}
