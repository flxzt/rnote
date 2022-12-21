use rnote_engine::engine::export::{DocExportFormat, DocExportPrefs};
use rnote_engine::engine::EngineSnapshot;
use smol::fs::File;
use smol::io::{AsyncReadExt, AsyncWriteExt};
use std::path::{Path, PathBuf};

use clap::{Parser, Subcommand};

use rnote_engine::RnoteEngine;

/// rnote-cli
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub(crate) command: Commands,
}

#[derive(Subcommand)]
pub(crate) enum Commands {
    /// Imports the specified input file and saves it as a rnote save file.
    /// Currently only `.xopp` files can be imported.
    Import {
        /// the rnote save file
        rnote_file: PathBuf,
        /// the import input file
        #[arg(short = 'i', long)]
        input_file: PathBuf,
        /// When importing a .xopp file, the import dpi can be specified.{n}
        /// Else the default (96) is used.
        #[arg(long)]
        xopp_dpi: Option<f64>,
    },
    /// Exports the Rnote file and saves it in the output file.{n}
    /// The export format is recognized from the file extension of the output file.{n}
    /// Currently `.svg`, `.xopp` and `.pdf` are supported.
    Export {
        /// the rnote save file
        rnote_file: PathBuf,
        /// the export output file
        #[arg(short = 'o', long)]
        output_file: PathBuf,
        /// export with background
        #[arg(short = 'b', long)]
        with_background: Option<bool>,
        /// export with background pattern
        #[arg(short = 'p', long)]
        with_pattern: Option<bool>,
    },
}

pub(crate) async fn run() -> anyhow::Result<()> {
    let mut engine = RnoteEngine::default();

    let cli = Cli::parse();

    match cli.command {
        Commands::Import {
            rnote_file,
            input_file,
            xopp_dpi,
        } => {
            println!("Importing..");

            // apply given arguments to import prefs
            if let Some(xopp_dpi) = xopp_dpi {
                engine.import_prefs.xopp_import_prefs.dpi = xopp_dpi;
            }

            import_file(&mut engine, input_file, rnote_file).await?;
        }
        Commands::Export {
            rnote_file,
            output_file,
            with_background,
            with_pattern,
        } => {
            println!("Exporting..");

            // apply given arguments to export prefs
            engine.export_prefs.doc_export_prefs =
                create_doc_export_prefs_from_args(&output_file, with_background, with_pattern)?;

            export_to_file(&mut engine, rnote_file, output_file).await?;
        }
    }
    println!("Finished!");

    Ok(())
}

pub(crate) async fn import_file(
    engine: &mut RnoteEngine,
    input_file: PathBuf,
    rnote_file: PathBuf,
) -> anyhow::Result<()> {
    let mut input_bytes = vec![];
    let Some(rnote_file_name) = rnote_file.file_name().map(|s| s.to_string_lossy().to_string()) else {
        return Err(anyhow::anyhow!("Failed to get filename from rnote_file."));
    };

    let mut ifh = File::open(input_file).await?;
    ifh.read_to_end(&mut input_bytes).await?;

    let snapshot =
        EngineSnapshot::load_from_xopp_bytes(input_bytes, engine.import_prefs.xopp_import_prefs)
            .await?;

    let _ = engine.load_snapshot(snapshot);

    let rnote_bytes = engine.save_as_rnote_bytes(rnote_file_name)?.await??;

    let mut ofh = File::create(rnote_file).await?;
    ofh.write_all(&rnote_bytes).await?;
    ofh.sync_all().await?;

    Ok(())
}

pub(crate) fn create_doc_export_prefs_from_args(
    output_file: impl AsRef<Path>,
    with_background: Option<bool>,
    with_pattern: Option<bool>,
) -> anyhow::Result<DocExportPrefs> {
    let format = match output_file
        .as_ref()
        .extension()
        .and_then(|ext| ext.to_str())
    {
        Some("svg") => DocExportFormat::Svg,
        Some("xopp") => DocExportFormat::Xopp,
        Some("pdf") => DocExportFormat::Pdf,
        Some(ext) => {
            return Err(anyhow::anyhow!(
                "could not create doc export prefs, unsupported export file extension `{ext}`"
            ))
        }
        None => {
            return Err(anyhow::anyhow!(
                "Output file needs to have an extension to determine the file type."
            ))
        }
    };

    let mut prefs = DocExportPrefs {
        export_format: format,
        ..Default::default()
    };

    if let Some(with_background) = with_background {
        prefs.with_background = with_background;
    }
    if let Some(with_pattern) = with_pattern {
        prefs.with_pattern = with_pattern;
    }

    Ok(prefs)
}

pub(crate) async fn export_to_file(
    engine: &mut RnoteEngine,
    rnote_file: PathBuf,
    output_file: PathBuf,
) -> anyhow::Result<()> {
    let Some(export_file_name) = output_file.file_name().map(|s| s.to_string_lossy().to_string()) else {
        return Err(anyhow::anyhow!("Failed to get filename from output_file."));
    };

    let mut rnote_bytes = vec![];
    File::open(rnote_file)
        .await?
        .read_to_end(&mut rnote_bytes)
        .await?;

    let engine_snapshot = EngineSnapshot::load_from_rnote_bytes(rnote_bytes).await?;
    let _ = engine.load_snapshot(engine_snapshot);

    // We applied the prefs previously to the engine
    let export_bytes = engine.export_doc(export_file_name, None).await??;

    let mut fh = File::create(output_file).await?;
    fh.write_all(&export_bytes).await?;
    fh.sync_all().await?;

    Ok(())
}
