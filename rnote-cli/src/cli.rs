use smol::fs::File;
use smol::io::{AsyncReadExt, AsyncWriteExt};
use std::path::PathBuf;

use clap::{Parser, Subcommand};

use rnote_engine::RnoteEngine;

/// Rnote Cli
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub(crate) command: Commands,
}

#[derive(Subcommand)]
pub(crate) enum Commands {
    /// Converts the input file (expecting a rnote save file) and saves it in the output file.
    /// The export format is recognized from the file extension of the output file.
    /// Currently `.svg`, `.xopp`, and `.pdf` are supported.
    Convert {
        /// The input file
        #[arg(short, long)]
        input_file: PathBuf,
        /// The output file
        #[arg(short, long)]
        output_file: PathBuf,
    },
}

pub(crate) async fn run() -> anyhow::Result<()> {
    let mut engine = RnoteEngine::new(None);

    let cli = Cli::parse();

    match cli.command {
        Commands::Convert {
            input_file,
            output_file,
        } => {
            println!("Converting..");

            convert_file(&mut engine, input_file, output_file).await?;
        }
    }

    println!("Finished!");

    Ok(())
}

pub(crate) async fn convert_file(
    engine: &mut RnoteEngine,
    input_file: PathBuf,
    output_file: PathBuf,
) -> anyhow::Result<()> {
    let mut input_bytes = vec![];

    File::open(input_file)
        .await?
        .read_to_end(&mut input_bytes)
        .await?;

    let store_snapshot = engine.open_from_rnote_bytes_p1(input_bytes)?.await??;

    engine.open_from_store_snapshot_p2(&store_snapshot)?;

    let export_title = output_file
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| String::from("output_file").into());

    let export_bytes = match output_file.extension().and_then(|ext| ext.to_str()) {
        Some("svg") => engine.export_doc_as_svg_bytes(None).await??,
        Some("xopp") => {
            engine
                .export_doc_as_xopp_bytes(export_title, None)
                .await??
        }
        Some("pdf") => engine.export_doc_as_pdf_bytes(export_title, None).await??,
        Some(ext) => {
            return Err(anyhow::anyhow!(
                "unsupported extension `{ext}` for output file"
            ))
        }
        None => {
            return Err(anyhow::anyhow!(
                "Output file needs to have an extension to determine the file type."
            ))
        }
    };

    File::create(output_file)
        .await?
        .write_all(&export_bytes)
        .await?;

    Ok(())
}
