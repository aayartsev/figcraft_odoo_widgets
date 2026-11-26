//! Validate Odoo widget catalog against native FigCraft JSON files.

pub mod catalog;
pub mod native_doc;
pub mod root;
pub mod validate;

pub use catalog::{Catalog, CatalogEntry, load_catalog};
pub use native_doc::{class_to_snake, load_native_document, page_root_id};
pub use root::resolve_repo_root;
pub use validate::{ValidationError, default_catalog_path, schema_path, validate_catalog};
