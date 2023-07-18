use anyhow::Context;
use clap::{ArgAction, Parser, Subcommand};
use rnote_engine::engine::export::{
    DocExportFormat, DocExportPrefs, SelectionExportFormat, SelectionExportPrefs,
};
use rnote_engine::engine::EngineSnapshot;
use rnote_engine::RnoteEngine;
use smol::fs::File;
use smol::io::{AsyncReadExt, AsyncWriteExt};
use std::path::{Path, PathBuf};
use std::time::Duration;

///    rnote_cli  Copyright (C) 2023  The Rnote Authors{n}{n}
///    This program is free software; you can redistribute it and/or modify it under the terms of the GPL v3 or (at your option) any later version.
#[derive(Parser)]
#[command(author, version, about, long_about = None, arg_required_else_help = true)]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub(crate) command: Commands,
}

#[derive(Subcommand)]
pub(crate) enum Commands {
    /// Tests if the specified files can be opened and are valid rnote files.
    Test {
        /// the rnote files
        rnote_files: Vec<PathBuf>,
    },
    /// Imports the specified input file and saves it as a rnote save file.{n}
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
    /// Exports the Rnote file(s) and saves it in the desired format.{n}
    /// When using --output-file, only one input file can be given.{n}
    /// The export format is recognized from the file extension of the output file.{n}
    /// When using --output-format, the same file name is used with the extension changed.{n}
    /// --output-file and --output-format are mutually exclusive but one of them is required.{n}
    /// Currently `.svg`, `.xopp` and `.pdf` are supported.{n}
    /// Usages: {n}
    /// rnote-cli export --output-file [filename.(svg|xopp|pdf)] [1 file]{n}
    /// rnote-cli export --output-format [svg|xopp|pdf] [list of files]
    Export {
        /// the rnote save file
        rnote_files: Vec<PathBuf>,
        /// the export output file. Only allows for one input file. Exclusive with output-format.
        #[arg(short = 'o', long, conflicts_with("output_format"), required(true))]
        output_file: Option<PathBuf>,
        /// the export output format. Exclusive with output-file.
        #[arg(short = 'f', long, conflicts_with("output_file"), required(true))]
        output_format: Option<String>,
        /// export without background
        #[arg(short = 'b', long, action = ArgAction::SetTrue)]
        without_background: bool,
        /// export without background pattern
        #[arg(short = 'p', long, action = ArgAction::SetTrue)]
        without_pattern: bool,
        /// crop document to fit all strokes
        #[arg(short = 'c', long, action = ArgAction::SetFalse)]
        crop_to_content: bool,
    },
}

pub(crate) async fn run() -> anyhow::Result<()> {
    let mut engine = RnoteEngine::default();

    let cli = Cli::parse();

    match cli.command {
        Commands::Test { rnote_files } => {
            println!("Testing..");

            for rnote_file in rnote_files.into_iter() {
                let file_disp = rnote_file.display().to_string();
                let pb = indicatif::ProgressBar::new_spinner();
                pb.set_draw_target(indicatif::ProgressDrawTarget::stdout());
                pb.set_message(format!("Testing file \"{file_disp}\""));
                pb.enable_steady_tick(Duration::from_millis(8));

                // test
                if let Err(e) = test_file(&mut engine, rnote_file).await {
                    let msg = format!("Test failed, Err: {e:?}");
                    if pb.is_hidden() {
                        println!("{msg}");
                    }
                    pb.abandon_with_message(msg);
                    return Err(e);
                } else {
                    let msg = format!("Test succeeded for file \"{file_disp}\"");
                    if pb.is_hidden() {
                        println!("{msg}");
                    }
                    pb.finish_with_message(msg);
                }
            }

            println!("Tests finished successfully!");
        }
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

            let rnote_file_disp = rnote_file.display().to_string();
            let input_file_disp = input_file.display().to_string();
            let pb = indicatif::ProgressBar::new_spinner().with_message(format!(
                "Importing \"{input_file_disp}\" to: \"{rnote_file_disp}\""
            ));
            pb.set_draw_target(indicatif::ProgressDrawTarget::stdout());
            pb.enable_steady_tick(Duration::from_millis(8));

            // import
            if let Err(e) = import_file(&mut engine, input_file, rnote_file).await {
                let msg = format!(
                    "Import \"{input_file_disp}\" to \"{rnote_file_disp}\" failed, Err: {e:?}"
                );
                if pb.is_hidden() {
                    println!("{msg}");
                }
                pb.abandon_with_message(msg);
                return Err(e);
            } else {
                let msg =
                    format!("Import \"{input_file_disp}\" to \"{rnote_file_disp}\" succeeded");
                if pb.is_hidden() {
                    println!("{msg}");
                }
                pb.finish_with_message(msg);
            }

            println!("Import finished!");
        }
        Commands::Export {
            rnote_files,
            output_file,
            output_format,
            without_background,
            without_pattern,
            crop_to_content,
        } => {
            println!("Exporting..");

            // apply given arguments to export prefs
            match crop_to_content {
                true => {
                    engine.export_prefs.selection_export_prefs =
                        create_selection_export_prefs_from_args(
                            output_file.as_deref(),
                            output_format.as_deref(),
                            without_background,
                            without_pattern,
                        )?;
                }
                false => {
                    engine.export_prefs.doc_export_prefs = create_doc_export_prefs_from_args(
                        output_file.as_deref(),
                        output_format.as_deref(),
                        without_background,
                        without_pattern,
                    )?
                }
            }
            match output_file {
                Some(ref output_file) => match rnote_files.get(0) {
                    Some(rnote_file) => {
                        if rnote_files.len() > 1 {
                            return Err(anyhow::anyhow!("Was expecting only 1 file. Use --output-format when exporting multiple files."));
                        }

                        let rnote_file_disp = rnote_file.display().to_string();
                        let output_file_disp = output_file.display().to_string();
                        let pb = indicatif::ProgressBar::new_spinner().with_message(format!(
                            "Exporting \"{rnote_file_disp}\" to: \"{output_file_disp}\""
                        ));
                        pb.set_draw_target(indicatif::ProgressDrawTarget::stdout());
                        pb.enable_steady_tick(Duration::from_millis(8));

                        // export
                        if let Err(e) =
                            export_to_file(&mut engine, rnote_file, output_file, crop_to_content)
                                .await
                        {
                            let msg = format!("Export \"{rnote_file_disp}\" to: \"{output_file_disp}\" failed, Err {e:?}");
                            if pb.is_hidden() {
                                println!("{msg}")
                            }
                            pb.abandon_with_message(msg);
                            return Err(e);
                        } else {
                            let msg = format!(
                                "Export \"{rnote_file_disp}\" to: \"{output_file_disp}\" succeeded"
                            );
                            if pb.is_hidden() {
                                println!("{msg}")
                            }
                            pb.finish_with_message(msg);
                        }
                    }
                    None => return Err(anyhow::anyhow!("Failed to get filename from rnote_files")),
                },
                None => {
                    let output_files = rnote_files
                        .iter()
                        .map(|file| {
                            let mut output = file.clone();
                            output.set_extension(
                                engine
                                    .export_prefs
                                    .doc_export_prefs
                                    .export_format
                                    .file_ext(),
                            );
                            output
                        })
                        .collect::<Vec<PathBuf>>();

                    for (rnote_file, output_file) in rnote_files.iter().zip(output_files.iter()) {
                        let rnote_file_disp = rnote_file.display().to_string();
                        let output_file_disp = output_file.display().to_string();
                        let pb = indicatif::ProgressBar::new_spinner();
                        pb.set_draw_target(indicatif::ProgressDrawTarget::stdout());
                        pb.set_message(format!(
                            "Exporting \"{rnote_file_disp}\" to: \"{output_file_disp}\""
                        ));
                        pb.enable_steady_tick(Duration::from_millis(8));

                        // export
                        if let Err(e) =
                            export_to_file(&mut engine, &rnote_file, &output_file, crop_to_content)
                                .await
                        {
                            let msg = format!("Export \"{rnote_file_disp}\" to: \"{output_file_disp}\" failed, Err {e:?}");
                            if pb.is_hidden() {
                                println!("{msg}")
                            }
                            pb.abandon_with_message(msg);
                            return Err(e);
                        } else {
                            let msg = format!(
                                "Export \"{rnote_file_disp}\" to: \"{output_file_disp}\" succeeded"
                            );
                            if pb.is_hidden() {
                                println!("{msg}")
                            }
                            pb.finish_with_message(msg);
                        }
                    }
                }
            }

            println!("Export Finished!");
        }
    }

    Ok(())
}

pub(crate) async fn test_file(
    _engine: &mut RnoteEngine,
    rnote_file: PathBuf,
) -> anyhow::Result<()> {
    let mut rnote_bytes = vec![];
    File::open(rnote_file)
        .await?
        .read_to_end(&mut rnote_bytes)
        .await?;

    let _ = EngineSnapshot::load_from_rnote_bytes(rnote_bytes).await?;
    // Loading a valid engine snapshot can't fail, so we skip it
    Ok(())
}

pub(crate) async fn import_file(
    engine: &mut RnoteEngine,
    input_file: PathBuf,
    rnote_file: PathBuf,
) -> anyhow::Result<()> {
    let mut input_bytes = vec![];
    let Some(rnote_file_name) = rnote_file.file_name().map(|s| s.to_string_lossy().to_string()) else {
        return Err(anyhow::anyhow!("Failed to get filename from rnote_file"));
    };

    let mut ifh = File::open(input_file).await?;
    ifh.read_to_end(&mut input_bytes).await?;

    let snapshot =
        EngineSnapshot::load_from_xopp_bytes(input_bytes, engine.import_prefs.xopp_import_prefs)
            .await?;

    let _ = engine.load_snapshot(snapshot);

    let rnote_bytes = engine.save_as_rnote_bytes(rnote_file_name).await??;

    let mut ofh = File::create(rnote_file).await?;
    ofh.write_all(&rnote_bytes).await?;
    ofh.sync_all().await?;

    Ok(())
}

fn get_doc_export_format(format: &str) -> anyhow::Result<DocExportFormat> {
    match format {
        "svg" => Ok(DocExportFormat::Svg),
        "xopp" => Ok(DocExportFormat::Xopp),
        "pdf" => Ok(DocExportFormat::Pdf),
        ext => Err(anyhow::anyhow!(
            "Could not create doc export prefs, unsupported export file extension `{ext}`"
        )),
    }
}

pub(crate) fn create_doc_export_prefs_from_args(
    output_file: Option<impl AsRef<Path>>,
    output_format: Option<&str>,
    without_background: bool,
    without_pattern: bool,
) -> anyhow::Result<DocExportPrefs> {
    let format = match (output_file, output_format) {
        (Some(file), None) => match file.as_ref().extension().and_then(|ext| ext.to_str()) {
            Some(extension) => get_doc_export_format(extension),
            None => {
                return Err(anyhow::anyhow!(
                    "Output file needs to have an extension to determine the file type"
                ))
            }
        },
        (None, Some(out_format)) => get_doc_export_format(out_format),
        // unreachable because they are exclusive (conflicts_with)
        (Some(_), Some(_)) => {
            return Err(anyhow::anyhow!(
                "--output-file and --output-format are mutually exclusive."
            ))
        }
        // unreachable because they are required
        (None, None) => {
            return Err(anyhow::anyhow!(
                "--output-file or --output-format is required."
            ))
        }
    }?;

    let mut prefs = DocExportPrefs {
        export_format: format,
        ..Default::default()
    };

    prefs.with_background = !without_background;
    prefs.with_pattern = !without_pattern;
    Ok(prefs)
}

pub(crate) fn create_selection_export_prefs_from_args(
    output_file: Option<impl AsRef<Path>>,
    output_format: Option<&str>,
    without_background: bool,
    without_pattern: bool,
) -> anyhow::Result<SelectionExportPrefs> {
    let format = match (output_file, output_format) {
        (Some(file), None) => match file.as_ref().extension().and_then(|ext| ext.to_str()) {
            Some(extension) => get_selection_export_format(extension),
            None => {
                return Err(anyhow::anyhow!(
                    "Output file needs to have an extension to determine the file type"
                ))
            }
        },
        (None, Some(out_format)) => get_selection_export_format(out_format),
        // unreachable because they are exclusive (conflicts_with)
        (Some(_), Some(_)) => {
            return Err(anyhow::anyhow!(
                "--output-file and --output-format are mutually exclusive."
            ))
        }
        // unreachable because they are required
        (None, None) => {
            return Err(anyhow::anyhow!(
                "--output-file or --output-format is required."
            ))
        }
    }?;

    let mut prefs = SelectionExportPrefs {
        export_format: format,
        ..Default::default()
    };

    prefs.with_background = !without_background;
    prefs.with_pattern = !without_pattern;

    Ok(prefs)
}

fn get_selection_export_format(format: &str) -> anyhow::Result<SelectionExportFormat> {
    match format {
        "svg" => Ok(SelectionExportFormat::Svg),
        "png" => Ok(SelectionExportFormat::Png),
        "jpg" | "jpeg" => Ok(SelectionExportFormat::Jpeg),
        ext => Err(anyhow::anyhow!(
            "Could not create selection export prefs, unsupported export file extension `{ext}`"
        )),
    }
}

pub(crate) async fn export_to_file(
    engine: &mut RnoteEngine,
    rnote_file: impl AsRef<Path>,
    output_file: impl AsRef<Path>,
    crop_to_content: bool,
) -> anyhow::Result<()> {
    let Some(export_file_name) = output_file.as_ref().file_name().map(|s| s.to_string_lossy().to_string()) else {
        return Err(anyhow::anyhow!("Failed to get filename from output_file"));
    };

    let mut rnote_bytes = vec![];
    File::open(rnote_file)
        .await?
        .read_to_end(&mut rnote_bytes)
        .await?;

    let engine_snapshot = EngineSnapshot::load_from_rnote_bytes(rnote_bytes).await?;
    let _ = engine.load_snapshot(engine_snapshot);
    if crop_to_content {
        let all_strokes = engine.store.stroke_keys_unordered();
        if all_strokes.is_empty() {
            return Err(anyhow::anyhow!(
                "Cannot export empty document with --crop-to-content enabled"
            ));
        }
        engine.store.set_selected_keys(&all_strokes, true);
    }

    // We applied the prefs previously to the engine
    let export_bytes = match crop_to_content {
        true => engine
            .export_selection(None)
            .await??
            .context("Failed to export selection")?,
        false => engine.export_doc(export_file_name, None).await??,
    };
    let mut fh = File::create(output_file).await?;
    fh.write_all(&export_bytes).await?;
    fh.sync_all().await?;

    Ok(())
}
