use anyhow::Result;
use clap::{Parser, Subcommand};
use figcraft_build::catalog::load_catalog;
use figcraft_build::root::resolve_repo_root;
use figcraft_build::validate::{SchemaValidator, default_catalog_path, schema_path, validate_catalog};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "figcraft_build",
    about = "Validate Odoo widget catalog (native FigCraft JSON)"
)]
struct Cli {
    /// Repository root containing catalog.json and widgets/
    #[arg(long, global = true)]
    root: Option<PathBuf>,

    /// Path to catalog.json (relative to root or absolute)
    #[arg(long, global = true)]
    catalog: Option<PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Validate catalog + native widget documents
    Validate,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let root = resolve_repo_root(cli.root.as_deref());
    let catalog_path = cli
        .catalog
        .clone()
        .unwrap_or_else(|| default_catalog_path(&root));
    let catalog_path = if catalog_path.is_absolute() {
        catalog_path
    } else {
        root.join(catalog_path)
    };

    let schema = SchemaValidator::from_schema_path(&schema_path(&root))?;
    let catalog = load_catalog(&catalog_path)?;

    match cli.command {
        Commands::Validate => {
            validate_catalog(&root, &catalog, &schema)?;
            println!(
                "OK: {} widgets in {}",
                catalog.widgets.len(),
                catalog_path.display()
            );
        }
    }
    Ok(())
}
