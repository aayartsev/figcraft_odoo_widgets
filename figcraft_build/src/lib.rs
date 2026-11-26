//! Validate Odoo widget JSON and build a FigCraft atlas document.

pub mod atlas;
pub mod catalog;
pub mod convert;
pub mod export_widget;
pub mod props;
pub mod root;
pub mod validate;
pub mod widget;

pub use atlas::{AtlasOptions, build_atlas};
pub use catalog::{Catalog, CatalogEntry, load_catalog};
pub use convert::fragment_to_document;
pub use export_widget::{build_widget_document, default_widget_out};
pub use root::resolve_repo_root;
pub use validate::{ValidationError, validate_catalog, validate_widget};
pub use widget::{FragmentNode, WidgetDoc, load_widget};
