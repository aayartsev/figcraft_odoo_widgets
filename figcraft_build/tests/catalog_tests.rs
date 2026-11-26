use figcraft_build::atlas::{AtlasOptions, build_atlas};
use figcraft_build::catalog::load_catalog;
use figcraft_build::root::resolve_repo_root;
use figcraft_build::validate::{SchemaValidator, default_catalog_path, schema_path, validate_catalog};
use std::collections::BTreeSet;

#[test]
fn real_catalog_validate_and_atlas() {
    let root = resolve_repo_root(None);
    let catalog = load_catalog(&default_catalog_path(&root)).unwrap();
    let schema = SchemaValidator::from_schema_path(&schema_path(&root)).unwrap();
    validate_catalog(&root, &catalog, &schema).unwrap();

    let doc = build_atlas(&root, &catalog, &AtlasOptions::default()).unwrap();
    let bytes = figcraft_format::save_bytes(&doc, None).unwrap();
    let (back, _issues) = figcraft_format::load_bytes(&bytes).unwrap();
    assert!(back.validate().is_empty());

    // Collect widget class names that appear as direct children of row:* frames.
    let page = back.first_page().unwrap();
    let mut classes = BTreeSet::new();
    collect_widget_classes(&back, page, &mut classes);

    let expected: BTreeSet<_> = catalog.widgets.iter().map(|w| w.class.clone()).collect();
    assert_eq!(
        classes.len(),
        expected.len(),
        "widget count mismatch: got {classes:?}"
    );
    assert_eq!(classes, expected);
}

fn collect_widget_classes(
    doc: &figcraft_doc::Document,
    id: figcraft_doc::NodeId,
    out: &mut BTreeSet<String>,
) {
    let Some(node) = doc.node(id) else {
        return;
    };
    if node.name.starts_with("row:") {
        for child in &node.children {
            if let Some(c) = doc.node(*child) {
                out.insert(c.name.clone());
            }
        }
    }
    for child in &node.children {
        collect_widget_classes(doc, *child, out);
    }
}
