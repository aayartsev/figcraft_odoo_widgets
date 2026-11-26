use figcraft_build::validate::{SchemaValidator, validate_widget};
use figcraft_build::widget::{WidgetDoc, class_to_snake};
use serde_json::json;
use std::path::PathBuf;

fn minimal_widget_json() -> serde_json::Value {
    json!({
        "schemaVersion": 1,
        "class": "DemoWidget",
        "odooVersion": "20",
        "category": "custom",
        "description": "тест",
        "size": { "width": 100, "height": 40 },
        "slots": [{ "name": "body", "accepts": [] }],
        "variants": ["default"],
        "figcraft": {
            "kind": "fragment",
            "root": {
                "type": "frame",
                "name": "DemoWidget",
                "width": 100,
                "height": 40,
                "children": [{
                    "type": "frame",
                    "name": "slot:body",
                    "width": 80,
                    "height": 20
                }]
            }
        }
    })
}

#[test]
fn schema_accepts_valid_widget() {
    let root = figcraft_build::resolve_repo_root(None);
    let schema = SchemaValidator::from_schema_path(&root.join("schemas/widget.schema.json")).unwrap();
    schema
        .validate_value(&minimal_widget_json(), &PathBuf::from("demo_widget.json"))
        .unwrap();
}

#[test]
fn schema_rejects_broken() {
    let root = figcraft_build::resolve_repo_root(None);
    let schema = SchemaValidator::from_schema_path(&root.join("schemas/widget.schema.json")).unwrap();
    let mut bad = minimal_widget_json();
    bad.as_object_mut().unwrap().remove("class");
    assert!(
        schema
            .validate_value(&bad, &PathBuf::from("x.json"))
            .is_err()
    );
}

#[test]
fn slot_without_frame_errors() {
    let mut value = minimal_widget_json();
    value["figcraft"]["root"]["children"] = json!([]);
    let widget: WidgetDoc = serde_json::from_value(value).unwrap();
    let path = PathBuf::from("demo_widget.json");
    assert!(validate_widget(&widget, &path, None).is_err());
}

#[test]
fn snake_case_helper() {
    assert_eq!(class_to_snake("ListView"), "list_view");
}
