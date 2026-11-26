use anyhow::{Context, Result};
use serde::Deserialize;
use serde_json::Value;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WidgetDoc {
    pub schema_version: u32,
    pub class: String,
    pub odoo_version: String,
    pub category: String,
    pub description: String,
    pub size: Size,
    pub slots: Vec<Slot>,
    #[serde(default)]
    pub slot_anchors: Option<Value>,
    pub variants: Vec<String>,
    #[serde(default)]
    pub tags: Option<Vec<String>>,
    pub figcraft: FigcraftMeta,
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

#[derive(Debug, Clone, Deserialize)]
pub struct FigcraftMeta {
    pub kind: String,
    pub root: FragmentNode,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FragmentNode {
    #[serde(rename = "type")]
    pub type_name: String,
    pub name: String,
    #[serde(default)]
    pub x: Option<f64>,
    #[serde(default)]
    pub y: Option<f64>,
    #[serde(default)]
    pub width: Option<f64>,
    #[serde(default)]
    pub height: Option<f64>,
    #[serde(default)]
    pub text: Option<String>,
    #[serde(default)]
    pub props: Option<serde_json::Map<String, Value>>,
    #[serde(default)]
    pub children: Option<Vec<FragmentNode>>,
}

pub fn load_widget(path: &Path) -> Result<WidgetDoc> {
    let text =
        fs::read_to_string(path).with_context(|| format!("read widget {}", path.display()))?;
    let doc: WidgetDoc = serde_json::from_str(&text)
        .with_context(|| format!("parse widget {}", path.display()))?;
    Ok(doc)
}

/// PascalCase class → snake_case file stem (`WebClientShell` → `web_client_shell`).
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

#[cfg(test)]
mod tests {
    use super::class_to_snake;

    #[test]
    fn snake_case_examples() {
        assert_eq!(class_to_snake("Navbar"), "navbar");
        assert_eq!(class_to_snake("WebClientShell"), "web_client_shell");
        assert_eq!(class_to_snake("NavbarSystrayItem"), "navbar_systray_item");
    }
}
