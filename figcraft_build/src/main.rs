use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use figcraft_build::atlas::{AtlasOptions, build_atlas};
use figcraft_build::catalog::load_catalog;
use figcraft_build::compare::{compare_fragment_to_doc, page_root_id};
use figcraft_build::export_widget::{build_widget_document, default_widget_out};
use figcraft_build::root::resolve_repo_root;
use figcraft_build::validate::{SchemaValidator, default_catalog_path, schema_path, validate_catalog};
use figcraft_build::widget::load_widget;
use std::fs;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "figcraft_build", about = "Validate widgets and build FigCraft atlas")]
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
    /// Validate catalog + widgets against schema and invariants
    Validate,
    /// Build dist/atlas_all_widgets.figcraft
    Atlas {
        /// Output .figcraft path (default: <root>/dist/atlas_all_widgets.figcraft)
        #[arg(long)]
        out: Option<PathBuf>,
    },
    /// Build one widget into dist/<snake_class>.figcraft
    Widget {
        /// Widget class from catalog (e.g. Navbar)
        class: String,
        /// Output path (default: dist/<snake_case(class)>.figcraft)
        #[arg(long)]
        out: Option<PathBuf>,
    },
    /// Build every catalog widget into dist/ and compare each to its JSON fragment
    VerifyAll {
        /// Stop after the first mismatch
        #[arg(long)]
        fail_fast: bool,
    },
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
        Commands::Atlas { out } => {
            validate_catalog(&root, &catalog, &schema)?;
            let doc = build_atlas(&root, &catalog, &AtlasOptions::default())?;
            let out_path = out.unwrap_or_else(|| root.join("dist/atlas_all_widgets.figcraft"));
            if let Some(parent) = out_path.parent() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("create {}", parent.display()))?;
            }
            figcraft_format::save_file(&doc, &out_path, None)
                .with_context(|| format!("write {}", out_path.display()))?;
            println!("Wrote {}", out_path.display());
        }
        Commands::Widget { class, out } => {
            validate_catalog(&root, &catalog, &schema)?;
            let doc = build_widget_document(&root, &catalog, &class)?;
            let out_path = match out {
                Some(p) => p,
                None => default_widget_out(&root, &class)?,
            };
            if let Some(parent) = out_path.parent() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("create {}", parent.display()))?;
            }
            figcraft_format::save_file(&doc, &out_path, None)
                .with_context(|| format!("write {}", out_path.display()))?;
            println!("Wrote {}", out_path.display());
        }
        Commands::VerifyAll { fail_fast } => {
            validate_catalog(&root, &catalog, &schema)?;
            let dist = root.join("dist");
            fs::create_dir_all(&dist)?;
            let mut failed = 0usize;
            for entry in &catalog.widgets {
                let doc = build_widget_document(&root, &catalog, &entry.class)?;
                let out_path = default_widget_out(&root, &entry.class)?;
                figcraft_format::save_file(&doc, &out_path, None)?;

                let bytes = fs::read(&out_path)?;
                let (loaded, _issues) = figcraft_format::load_bytes(&bytes)
                    .with_context(|| format!("reload {}", out_path.display()))?;
                let widget = load_widget(&catalog.resolve_path(&root, entry))?;
                let root_id = page_root_id(&loaded)
                    .ok_or_else(|| anyhow::anyhow!("{}: no page root", entry.class))?;
                let diffs = compare_fragment_to_doc(&loaded, root_id, &widget.figcraft.root);
                if diffs.is_empty() {
                    println!("OK  {} -> {}", entry.class, out_path.display());
                } else {
                    failed += 1;
                    println!("FAIL {} ({} diffs)", entry.class, diffs.len());
                    for d in diffs.iter().take(20) {
                        println!("  - {}: {}", d.path, d.message);
                    }
                    if fail_fast {
                        anyhow::bail!("verify-all stopped on {}", entry.class);
                    }
                }
            }
            if failed > 0 {
                anyhow::bail!("{failed} widget(s) failed fragment↔figcraft compare");
            }
            println!("All {} widgets OK", catalog.widgets.len());
        }
    }
    Ok(())
}
