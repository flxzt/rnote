// Imports
use crate::cli::{self, OnConflict};
use crate::validators;
use anyhow::Context;
use p2d::bounding_volume::Aabb;
use rnote_compose::SplitOrder;
use rnote_engine::engine::export::{
    DocExportFormat, DocExportPrefs, DocPagesExportFormat, DocPagesExportPrefs,
    SelectionExportFormat, SelectionExportPrefs,
};
use rnote_engine::engine::EngineSnapshot;
use rnote_engine::{Engine, SelectionCollision};
use std::path::{Path, PathBuf};

#[allow(clippy::too_many_arguments)]
pub(crate) async fn run_export(
    rnote_files: Vec<PathBuf>,
    no_background: bool,
    no_pattern: bool,
    optimize_printing: bool,
    on_conflict: OnConflict,
    open: bool,
    export_command: cli::ExportCommand,
) -> anyhow::Result<()> {
    if rnote_files.is_empty() {
        return Err(anyhow::anyhow!(
            "There must be at least one rnote file specified for exporting."
        ));
    }

    let mut engine = Engine::default();
    let mut on_conflict_overwrite = None;
    let output_file = match &export_command {
        cli::ExportCommand::Doc { file_args, .. } => file_args.output_file.as_ref(),
        cli::ExportCommand::Selection { file_args, .. } => file_args.output_file.as_ref(),
        cli::ExportCommand::DocPages {
            output_file_stem, ..
        } => {
            if rnote_files.len() > 1 && output_file_stem.is_some() {
                return Err(anyhow::anyhow!(
                    "The option \"--file-stem\" cannot be used when exporting multiple rnote files."
                ));
            }
            None
        }
    };

    apply_export_prefs(
        &mut engine,
        &export_command,
        output_file,
        no_background,
        no_pattern,
        optimize_printing,
    )?;

    match output_file {
        Some(output_file) => {
            let Some(rnote_file) = rnote_files.first() else {
                return Err(anyhow::anyhow!(
                    "There must be at least one rnote file specified for exporting."
                ));
            };

            validators::file_has_ext(rnote_file, "rnote")?;
            let output_file = get_output_file_path(
                output_file,
                on_conflict,
                &mut on_conflict_overwrite,
                &export_command,
            )?;
            if rnote_files.len() > 1 {
                return Err(anyhow::anyhow!("Expected only a single rnote file. The option \"--output-format\" must be used when exporting multiple files."));
            }

            let rnote_file_disp = rnote_file.display().to_string();
            let output_file_disp = output_file.display().to_string();
            let progressbar = cli::new_progressbar(format!(
                "Exporting \"{rnote_file_disp}\" to: \"{output_file_disp}\"."
            ));

            if let Err(e) = export_to_file(
                &mut engine,
                rnote_file,
                output_file,
                &export_command,
                on_conflict,
                &mut on_conflict_overwrite,
                open,
            )
            .await
            {
                let abandon_msg = format!(
                    "Export \"{rnote_file_disp}\" to: \"{output_file_disp}\" failed, Err {e:?}"
                );
                if progressbar.is_hidden() {
                    println!("{abandon_msg}")
                }
                progressbar.abandon_with_message(abandon_msg);
                return Err(e);
            } else {
                let finish_msg =
                    format!("Export \"{rnote_file_disp}\" to: \"{output_file_disp}\" succeeded.");
                if progressbar.is_hidden() {
                    println!("{finish_msg}")
                }
                progressbar.finish_with_message(finish_msg);
            }
        }
        None => {
            let exporting_doc_pages = matches!(export_command, cli::ExportCommand::DocPages { .. });
            let output_ext = file_ext_from_export_command(&mut engine, &export_command);
            let output_files = rnote_files
                .iter()
                .map(|file| {
                    let mut output = file.clone();
                    output.set_extension(&output_ext);
                    output
                })
                .collect::<Vec<PathBuf>>();

            for (rnote_file, output_file) in rnote_files.iter().zip(output_files.iter()) {
                validators::file_has_ext(rnote_file, "rnote")?;
                let output_file = match get_output_file_path(
                    output_file,
                    on_conflict,
                    &mut on_conflict_overwrite,
                    &export_command,
                ) {
                    Ok(file) => file,
                    Err(e) => {
                        println!("Failed to generate output file path, Err: {e:?}");
                        continue;
                    }
                };
                let rnote_file_disp = rnote_file.display().to_string();
                let output_file_disp = output_file.display().to_string();
                let progressbar_msg = match exporting_doc_pages {
                    true => format!("Exporting \"{rnote_file_disp}\"."),
                    false => format!("Exporting \"{rnote_file_disp}\" to: \"{output_file_disp}\"."),
                };
                let progressbar = cli::new_progressbar(progressbar_msg);

                if let Err(e) = export_to_file(
                    &mut engine,
                    &rnote_file,
                    output_file,
                    &export_command,
                    on_conflict,
                    &mut on_conflict_overwrite,
                    open,
                )
                .await
                {
                    let abandon_msg = match exporting_doc_pages {
                        true => format!("Export \"{rnote_file_disp}\" failed, Err {e:?}"),
                        false => format!(
                        "Export \"{rnote_file_disp}\" to: \"{output_file_disp}\" failed, Err {e:?}"
                    ),
                    };
                    if progressbar.is_hidden() {
                        println!("{abandon_msg}")
                    }
                    progressbar.abandon_with_message(abandon_msg);
                    return Err(e);
                } else {
                    let finish_msg = match exporting_doc_pages {
                        false => format!(
                            "Export \"{rnote_file_disp}\" to: \"{output_file_disp}\" succeeded."
                        ),
                        true => format!("Export \"{rnote_file_disp}\" succeeded."),
                    };
                    if progressbar.is_hidden() {
                        println!("{finish_msg}")
                    }
                    progressbar.finish_with_message(finish_msg);
                }
            }
        }
    }

    Ok(())
}

fn apply_export_prefs(
    engine: &mut Engine,
    export_command: &cli::ExportCommand,
    output_file: Option<&PathBuf>,
    no_background: bool,
    no_pattern: bool,
    optimize_printing: bool,
) -> anyhow::Result<()> {
    match &export_command {
        cli::ExportCommand::Doc {
            file_args,
            page_order,
        } => {
            engine.export_prefs.doc_export_prefs = create_doc_export_prefs_from_args(
                output_file,
                file_args.output_format,
                no_background,
                no_pattern,
                optimize_printing,
                *page_order,
            )?;
        }
        cli::ExportCommand::DocPages {
            export_format: output_format,
            page_order,
            bitmap_scalefactor,
            jpeg_quality,
            ..
        } => {
            engine.export_prefs.doc_pages_export_prefs = create_doc_pages_export_prefs_from_args(
                *output_format,
                no_background,
                no_pattern,
                optimize_printing,
                *page_order,
                *bitmap_scalefactor,
                *jpeg_quality,
            )?;
        }
        cli::ExportCommand::Selection {
            file_args,
            bitmap_scalefactor,
            jpeg_quality,
            margin,
            ..
        } => {
            engine.export_prefs.selection_export_prefs = create_selection_export_prefs_from_args(
                output_file,
                file_args.output_format,
                no_background,
                no_pattern,
                optimize_printing,
                *bitmap_scalefactor,
                *jpeg_quality,
                *margin,
            )?;
        }
    }
    Ok(())
}

fn file_ext_from_export_command(
    engine: &mut Engine,
    export_command: &cli::ExportCommand,
) -> String {
    match export_command {
        cli::ExportCommand::Doc { .. } => engine
            .export_prefs
            .doc_export_prefs
            .export_format
            .file_ext(),
        cli::ExportCommand::DocPages { .. } => engine
            .export_prefs
            .doc_pages_export_prefs
            .export_format
            .file_ext(),
        cli::ExportCommand::Selection { .. } => engine
            .export_prefs
            .selection_export_prefs
            .export_format
            .file_ext(),
    }
}

pub(crate) fn create_doc_export_prefs_from_args(
    output_file: Option<impl AsRef<Path>>,
    output_format: Option<DocExportFormat>,
    no_background: bool,
    no_pattern: bool,
    optimize_printing: bool,
    page_order: SplitOrder,
) -> anyhow::Result<DocExportPrefs> {
    let format = match (output_file, output_format) {
        (Some(file), None) => match file.as_ref().extension().and_then(|ext| ext.to_str()) {
            Some(extension) => doc_export_format_from_ext_str(extension)?,
            None => return Err(anyhow::anyhow!(
                "The output file \"{}\" needs to have a supported extension to determine its file type.",
                file.as_ref().display()
            )),
        },
        (None, Some(out_format)) => out_format,
        // should be unreachable because the arguments are exclusive (clap conflicts_with)
        (Some(_), Some(_)) => {
            return Err(anyhow::anyhow!(
                "\"--output-file\" and \"--output-format\" are mutually exclusive."
            ))
        }
        // should be unreachable because either --output-file or --output-format is required
        (None, None) => {
            return Err(anyhow::anyhow!(
                "Either \"--output-file\" or \"--output-format\" is required."
            ))
        }
    };

    let prefs = DocExportPrefs {
        export_format: format,
        with_background: !no_background,
        with_pattern: !no_pattern,
        optimize_printing,
        page_order,
    };

    Ok(prefs)
}

fn doc_export_format_from_ext_str(format: &str) -> anyhow::Result<DocExportFormat> {
    match format {
        "svg" => Ok(DocExportFormat::Svg),
        "xopp" => Ok(DocExportFormat::Xopp),
        "pdf" => Ok(DocExportFormat::Pdf),
        ext => Err(anyhow::anyhow!(
            "Exporting document to format with extension \"{ext}\" is not supported."
        )),
    }
}

pub(crate) fn create_doc_pages_export_prefs_from_args(
    export_format: DocPagesExportFormat,
    no_background: bool,
    no_pattern: bool,
    optimize_printing: bool,
    page_order: SplitOrder,
    bitmap_scalefactor: f64,
    jpeg_quality: u8,
) -> anyhow::Result<DocPagesExportPrefs> {
    Ok(DocPagesExportPrefs {
        export_format,
        with_background: !no_background,
        with_pattern: !no_pattern,
        optimize_printing,
        page_order,
        bitmap_scalefactor,
        jpeg_quality,
    })
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn create_selection_export_prefs_from_args(
    output_file: Option<impl AsRef<Path>>,
    output_format: Option<SelectionExportFormat>,
    no_background: bool,
    no_pattern: bool,
    optimize_printing: bool,
    bitmap_scalefactor: f64,
    jpeg_quality: u8,
    margin: f64,
) -> anyhow::Result<SelectionExportPrefs> {
    let format = match (output_file, output_format) {
        (Some(file), None) => match file.as_ref().extension().and_then(|ext| ext.to_str()) {
            Some(extension) => get_selection_export_format(extension)?,
            None => {
                return Err(anyhow::anyhow!(
                    "The output file \"{}\" needs to have a supported extension to determine its file type.", file.as_ref().display()
                ))
            }
        },
        (None, Some(out_format)) => out_format,
        // should be unreachable because the arguments are exclusive (clap conflicts_with)
        (Some(_), Some(_)) => {
            return Err(anyhow::anyhow!(
                "\"--output-file\" and \"--output-format\" are mutually exclusive."
            ))
        }
        // should be unreachable because either --output-file or --output-format is required
        (None, None) => {
            return Err(anyhow::anyhow!(
                "Either \"--output-file\" or \"--output-format\" is required."
            ))
        }
    };

    let prefs = SelectionExportPrefs {
        export_format: format,
        with_background: !no_background,
        with_pattern: !no_pattern,
        optimize_printing,
        bitmap_scalefactor,
        jpeg_quality,
        margin,
    };

    Ok(prefs)
}

fn get_selection_export_format(format: &str) -> anyhow::Result<SelectionExportFormat> {
    match format {
        "svg" => Ok(SelectionExportFormat::Svg),
        "png" => Ok(SelectionExportFormat::Png),
        "jpg" | "jpeg" => Ok(SelectionExportFormat::Jpeg),
        ext => Err(anyhow::anyhow!(
            "Exporting selection to format with extension \"{ext}\" is not supported."
        )),
    }
}

pub(crate) fn get_output_file_path(
    initial_output_file: &Path,
    on_conflict: OnConflict,
    on_conflict_overwrite: &mut Option<OnConflict>,
    export_command: &cli::ExportCommand,
) -> anyhow::Result<PathBuf> {
    match export_command {
        // output file will be ignored when parsing output file
        cli::ExportCommand::DocPages { .. } => Ok(initial_output_file.to_path_buf()),
        _ => Ok(file_conflict_prompt_action(
            initial_output_file,
            on_conflict,
            on_conflict_overwrite,
        )?
        .unwrap_or(initial_output_file.to_path_buf())),
    }
}

/// Opens a dialog/prompt when a file conflict (file already exists) is detected.
///
/// Returns a new path for the output file optionally.
pub(crate) fn file_conflict_prompt_action(
    output_file: &Path,
    mut on_conflict: OnConflict,
    on_conflict_overwrite: &mut Option<OnConflict>,
) -> anyhow::Result<Option<PathBuf>> {
    if !output_file.exists() {
        return Ok(None);
    }
    if atty::isnt(atty::Stream::Stdout) {
        return Err(anyhow::anyhow!(
            "File conflict for file \"{}\" detected and terminal is not interactive. Option \"--on-conflict\" needs to be supplied.", output_file.display()
        ));
    }
    match on_conflict_overwrite {
        Some(o) => on_conflict = *o,
        None => {
            let options = &[
                OnConflict::Ask,
                OnConflict::Overwrite,
                OnConflict::AlwaysOverwrite,
                OnConflict::Skip,
                OnConflict::AlwaysSkip,
                OnConflict::Suffix,
                OnConflict::AlwaysSuffix,
            ];
            while matches!(on_conflict, OnConflict::Ask) {
                match dialoguer::Select::new()
                    .with_prompt(format!("File \"{}\" already exists:", output_file.display()))
                    .items(options)
                    .default(1)
                    .interact()
                {
                    Ok(0) => {
                            cli::open_file_default_app(output_file)?
                    }
                    Ok(c) => on_conflict = options[c],
                    Err(e) => {
                        return Err(anyhow::anyhow!(
                            "Failed to show select prompt, retry or select the behavior with\"--on-conflict\", Err {e:?}"
                        ))
                    }
                };
            }
        }
    };
    match on_conflict {
        OnConflict::Ask => {
            return Err(anyhow::anyhow!(
                "on-conflict behaviour is still Ask after prompting the user."
            ))
        }
        OnConflict::AlwaysOverwrite => {
            on_conflict = OnConflict::Overwrite;
            *on_conflict_overwrite = Some(on_conflict);
        }
        OnConflict::AlwaysSkip => {
            on_conflict = OnConflict::Skip;
            *on_conflict_overwrite = Some(on_conflict);
        }
        OnConflict::AlwaysSuffix => {
            on_conflict = OnConflict::Suffix;
            *on_conflict_overwrite = Some(on_conflict);
        }
        OnConflict::Overwrite | OnConflict::Skip | OnConflict::Suffix => (),
    }
    match on_conflict {
        OnConflict::Ask => Err(anyhow::anyhow!(
            "on-conflict behaviour is still Ask after prompting the user."
        )),
        OnConflict::Overwrite => Ok(None),
        OnConflict::Skip => Err(anyhow::anyhow!("Skipped {}", output_file.display())),
        OnConflict::Suffix => {
            let mut i = 0;
            let mut new_path = output_file.to_path_buf();
            let Some(file_stem) = new_path
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
            else {
                return Err(anyhow::anyhow!("Failed to get file stem"));
            };
            let ext = new_path
                .extension()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            while new_path.exists() {
                i += 1;
                new_path.set_file_name(format!("{file_stem}_{i}.{ext}"))
            }
            Ok(Some(new_path))
        }
        OnConflict::AlwaysOverwrite | OnConflict::AlwaysSkip | OnConflict::AlwaysSuffix => {
            Err(anyhow::anyhow!(
                "on-conflict behaviour is still {on_conflict} after applying overwrite."
            ))
        }
    }
}

pub(crate) async fn export_to_file(
    engine: &mut Engine,
    rnote_file: impl AsRef<Path>,
    output_file: impl AsRef<Path>,
    export_command: &cli::ExportCommand,
    on_conflict: OnConflict,
    on_conflict_overwrite: &mut Option<OnConflict>,
    open: bool,
) -> anyhow::Result<()> {
    let rnote_bytes = cli::read_bytes_from_file(&rnote_file).await?;
    let engine_snapshot = EngineSnapshot::load_from_rnote_bytes(rnote_bytes).await?;
    let _ = engine.load_snapshot(engine_snapshot);

    match export_command {
        cli::ExportCommand::Selection {
            selection,
            selection_collision,
            ..
        } => {
            select_strokes_for_selection_args(engine, selection, *selection_collision);
            let export_bytes = engine
                .export_selection(None)
                .await??
                .context("Exporting selection failed, no strokes selected.")?;
            cli::create_overwrite_file_w_bytes(&output_file, &export_bytes).await?;
            if open {
                cli::open_file_default_app(output_file)?;
            }
        }
        cli::ExportCommand::Doc { .. } => {
            let Some(export_file_name) = output_file
                .as_ref()
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
            else {
                return Err(anyhow::anyhow!(
                    "Failed to get file name from output-file \"{}\".",
                    output_file.as_ref().display()
                ));
            };
            let export_bytes = engine.export_doc(export_file_name, None).await??;
            cli::create_overwrite_file_w_bytes(&output_file, &export_bytes).await?;
            if open {
                cli::open_file_default_app(output_file)?;
            }
        }
        cli::ExportCommand::DocPages {
            output_dir,
            output_file_stem,
            export_format: output_format,
            ..
        } => {
            validators::path_is_dir(output_dir)?;
            // The output file cannot be set with this subcommand
            drop(output_file);

            let pages_export_bytes = engine.export_doc_pages(None).await??;
            let out_ext = output_format.file_ext();
            let output_file_stem = match output_file_stem {
                Some(o) => o.clone(),
                None => match rnote_file.as_ref().file_stem() {
                    Some(stem) => stem.to_string_lossy().to_string(),
                    None => {
                        return Err(anyhow::anyhow!(
                            "Failed to get file stem from rnote file \"{}\"",
                            rnote_file.as_ref().display()
                        ))
                    }
                },
            };
            let pages_amount = pages_export_bytes.len();
            for (page_i, bytes) in pages_export_bytes.into_iter().enumerate() {
                let output_file = doc_page_determine_output_file(
                    page_i,
                    pages_amount,
                    output_dir,
                    &out_ext,
                    &output_file_stem,
                    on_conflict,
                    on_conflict_overwrite,
                )?;
                cli::create_overwrite_file_w_bytes(&output_file, &bytes)
                    .await
                    .context(format!(
                        "Failed to export page {page_i} of document \"{}\".",
                        rnote_file.as_ref().display()
                    ))?
            }
            if open {
                cli::open_file_default_app(output_dir)?;
            }
        }
    };
    Ok(())
}

fn select_strokes_for_selection_args(
    engine: &mut Engine,
    selection: &cli::SelectionCommand,
    selection_collision: SelectionCollision,
) {
    match selection {
        cli::SelectionCommand::Rect {
            x,
            y,
            width,
            height,
        } => {
            let mins = na::vector![*x, *y];
            let maxs = mins + na::vector![*width, *height];
            let bounds = Aabb::new(mins.into(), maxs.into());
            let _ = engine.select_with_bounds(bounds, selection_collision);
        }
        cli::SelectionCommand::All => {
            let _ = engine.select_all_strokes();
        }
    };
}

fn doc_page_determine_output_file(
    mut page_i: usize,
    pages_amount: usize,
    output_dir: &Path,
    out_ext: &str,
    output_file_stem: &str,
    on_conflict: OnConflict,
    on_conflict_overwrite: &mut Option<OnConflict>,
) -> anyhow::Result<PathBuf> {
    // user facing number is one-indexed
    page_i += 1;
    let leading_zeros = pages_amount.to_string().len();
    let number = format!("{page_i:0fill$}", fill = leading_zeros);
    let mut out = output_dir.join(format!("{output_file_stem} - page {number}.{out_ext}"));
    if let Some(new_out) =
        file_conflict_prompt_action(out.as_ref(), on_conflict, on_conflict_overwrite)?
    {
        out = new_out;
    }
    Ok(out)
}
