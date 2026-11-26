use figcraft_build::catalog::load_catalog;
use figcraft_build::native_doc::class_to_snake;
use figcraft_build::root::resolve_repo_root;
use figcraft_build::validate::{SchemaValidator, default_catalog_path, schema_path, validate_catalog};

#[test]
fn snake_case_helper() {
    assert_eq!(class_to_snake("Navbar"), "navbar");
    assert_eq!(class_to_snake("WebClientShell"), "web_client_shell");
    assert_eq!(class_to_snake("DateTimeField"), "date_time_field");
}

#[test]
fn real_catalog_validate() {
    let root = resolve_repo_root(None);
    let catalog = load_catalog(&default_catalog_path(&root)).unwrap();
    let schema = SchemaValidator::from_schema_path(&schema_path(&root)).unwrap();
    validate_catalog(&root, &catalog, &schema).unwrap();
}
