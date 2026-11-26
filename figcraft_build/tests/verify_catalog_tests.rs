use figcraft_build::catalog::load_catalog;
use figcraft_build::compare::{compare_fragment_to_doc, page_root_id};
use figcraft_build::export_widget::{build_widget_document, default_widget_out};
use figcraft_build::root::resolve_repo_root;
use figcraft_build::validate::{SchemaValidator, default_catalog_path, schema_path, validate_catalog};
use figcraft_build::widget::load_widget;

#[test]
fn every_catalog_widget_roundtrips_matching_fragment() {
    let root = resolve_repo_root(None);
    let catalog = load_catalog(&default_catalog_path(&root)).unwrap();
    let schema = SchemaValidator::from_schema_path(&schema_path(&root)).unwrap();
    validate_catalog(&root, &catalog, &schema).unwrap();

    for entry in &catalog.widgets {
        let doc = build_widget_document(&root, &catalog, &entry.class).unwrap();
        let out = default_widget_out(&root, &entry.class).unwrap();
        if let Some(parent) = out.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        figcraft_format::save_file(&doc, &out, None).unwrap();
        let bytes = std::fs::read(&out).unwrap();
        let (loaded, issues) = figcraft_format::load_bytes(&bytes).unwrap();
        assert!(
            loaded.validate().is_empty(),
            "{} validate: {:?}",
            entry.class,
            loaded.validate()
        );
        let _ = issues;

        let widget = load_widget(&catalog.resolve_path(&root, entry)).unwrap();
        let root_id = page_root_id(&loaded).expect("page root");
        let diffs = compare_fragment_to_doc(&loaded, root_id, &widget.figcraft.root);
        assert!(
            diffs.is_empty(),
            "{} fragment mismatch: {:?}",
            entry.class,
            diffs
                .iter()
                .map(|d| format!("{}: {}", d.path, d.message))
                .collect::<Vec<_>>()
        );
    }
}

#[test]
fn color_compare_handles_hash_and_alpha() {
    use figcraft_build::props::parse_color;
    use figcraft_color::Paint;
    use serde_json::json;

    let c = parse_color(&json!("#FFFFFFCC"), "t", "fill").unwrap();
    let paint = Paint::solid(c);
    let hex = paint.solid_color().unwrap().to_css_hex();
    assert!(
        hex.eq_ignore_ascii_case("#FFFFFFCC"),
        "got {hex}"
    );

    let t = parse_color(&json!("#00000000"), "t", "fill").unwrap();
    let paint = Paint::solid(t);
    let hex = paint.solid_color().unwrap().to_css_hex();
    assert!(
        hex.eq_ignore_ascii_case("#00000000"),
        "got {hex}"
    );
}
