use figcraft_build::convert::fragment_to_document;
use figcraft_build::widget::FragmentNode;
use serde_json::json;

#[test]
fn save_load_roundtrip_validates() {
    let root: FragmentNode = serde_json::from_value(json!({
        "type": "frame",
        "name": "R",
        "width": 80,
        "height": 40,
        "props": { "fill": "#714B67", "layout": "vertical", "gap": 4, "padding": 8 },
        "children": [{
            "type": "text",
            "name": "T",
            "width": 40,
            "height": 16,
            "text": "Hi",
            "props": { "fontSize": 12, "fill": "#FFFFFF" }
        }]
    }))
    .unwrap();

    let doc = fragment_to_document(&root, "roundtrip").unwrap();
    let bytes = figcraft_format::save_bytes(&doc, None).unwrap();
    let (back, issues) = figcraft_format::load_bytes(&bytes).unwrap();
    assert!(
        back.validate().is_empty(),
        "validate issues: {:?}",
        back.validate()
    );
    // repairs/issues from load are acceptable if non-critical
    let _ = issues;
}
