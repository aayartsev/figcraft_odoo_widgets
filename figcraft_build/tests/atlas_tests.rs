use figcraft_build::atlas::{
    ROW_WRAP_WIDTH, WIDGET_GAP, atlas_section_names, build_atlas_from_fragments,
};
use figcraft_build::atlas::AtlasOptions;
use figcraft_build::widget::FragmentNode;
use serde_json::json;

fn frag(name: &str, w: f64) -> FragmentNode {
    serde_json::from_value(json!({
        "type": "frame",
        "name": name,
        "width": w,
        "height": 20,
        "props": { "fill": "#FFFFFF" }
    }))
    .unwrap()
}

#[test]
fn section_order_follows_spec_not_input_order() {
    let items = vec![
        ("fields".into(), "FieldA".into(), frag("FieldA", 50.0)),
        ("shell".into(), "ShellA".into(), frag("ShellA", 50.0)),
        ("views".into(), "ViewA".into(), frag("ViewA", 50.0)),
    ];
    let doc = build_atlas_from_fragments(&items, &AtlasOptions::default()).unwrap();
    let sections = atlas_section_names(&doc).unwrap();
    assert_eq!(
        sections,
        vec![
            "category:shell".to_string(),
            "category:views".to_string(),
            "category:fields".to_string(),
        ]
    );
}

#[test]
fn empty_category_skipped() {
    let items = vec![("menu".into(), "M".into(), frag("M", 40.0))];
    let doc = build_atlas_from_fragments(&items, &AtlasOptions::default()).unwrap();
    let sections = atlas_section_names(&doc).unwrap();
    assert_eq!(sections, vec!["category:menu".to_string()]);
}

#[test]
fn row_wraps_when_width_exceeds_limit() {
    // Two widgets that individually fit but together with gap exceed wrap width.
    let w = (ROW_WRAP_WIDTH / 2.0) + 10.0;
    assert!(w + WIDGET_GAP + w > ROW_WRAP_WIDTH);

    let items = vec![
        ("shell".into(), "A".into(), frag("A", w)),
        ("shell".into(), "B".into(), frag("B", w)),
    ];
    let doc = build_atlas_from_fragments(&items, &AtlasOptions::default()).unwrap();
    let page = doc.first_page().unwrap();
    let atlas = doc.node(page).unwrap().children[0];
    let section = doc.node(atlas).unwrap().children[0];
    let section_node = doc.node(section).unwrap();
    // children: title text + row:0 + row:1
    let row_names: Vec<_> = section_node
        .children
        .iter()
        .filter_map(|id| doc.node(*id).map(|n| n.name.clone()))
        .filter(|n| n.starts_with("row:"))
        .collect();
    assert_eq!(row_names, vec!["row:0".to_string(), "row:1".to_string()]);
}
