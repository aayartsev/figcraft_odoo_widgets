use figcraft_build::convert::{collect_tree_summary, fragment_to_document};
use figcraft_build::widget::FragmentNode;
use serde_json::json;

fn node_from(value: serde_json::Value) -> FragmentNode {
    serde_json::from_value(value).expect("fragment node")
}

#[test]
fn convert_frame_text_rectangle() {
    let root = node_from(json!({
        "type": "frame",
        "name": "Box",
        "x": 0,
        "y": 0,
        "width": 120,
        "height": 40,
        "props": { "fill": "#FFFFFF", "layout": "horizontal", "gap": 8 },
        "children": [
            {
                "type": "text",
                "name": "Label",
                "width": 40,
                "height": 16,
                "text": "Hello",
                "props": { "fontSize": 14, "fill": "#212529" }
            },
            {
                "type": "rectangle",
                "name": "Dot",
                "width": 8,
                "height": 8,
                "props": { "fill": "#714B67", "cornerRadius": 2 }
            }
        ]
    }));

    let doc = fragment_to_document(&root, "test").unwrap();
    let page = doc.first_page().unwrap();
    let summary = collect_tree_summary(&doc, page);
    let names: Vec<_> = summary.iter().map(|(n, ..)| n.as_str()).collect();
    assert!(names.contains(&"Box"));
    assert!(names.contains(&"Label"));
    assert!(names.contains(&"Dot"));

    let label = summary.iter().find(|(n, ..)| n == "Label").unwrap();
    assert_eq!(label.1, "TEXT");

    // characters preserved
    let page_node = doc.node(page).unwrap();
    let box_id = page_node.children[0];
    let box_node = doc.node(box_id).unwrap();
    let text_id = box_node.children[0];
    let text_node = doc.node(text_id).unwrap();
    let text = text_node.kind.text().unwrap();
    assert_eq!(text.characters, "Hello");
}

#[test]
fn unsupported_type_errors() {
    let root = node_from(json!({
        "type": "vector",
        "name": "V",
        "width": 10,
        "height": 10
    }));
    assert!(fragment_to_document(&root, "x").is_err());
}
