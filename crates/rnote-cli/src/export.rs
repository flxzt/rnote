use std::{
    fmt::Display,
    path::{Path, PathBuf},
    time::Duration,
};

use anyhow::Context;
use clap::{Args, Subcommand, ValueEnum};
use parry2d_f64::{bounding_volume::Aabb, na::Vector2};
use rnote_compose::SplitOrder;
use rnote_engine::{
    engine::{
        export::{
            DocExportFormat, DocExportPrefs, DocPagesExportFormat, DocPagesExportPrefs,
            SelectionExportFormat, SelectionExportPrefs,
        },
        EngineSnapshot,
    },
    RnoteEngine,
};
use smol::{
    fs::File,
    io::{AsyncReadExt, AsyncWriteExt},
};

use crate::cli::OnConflict;
use crate::validators;

#[derive(Subcommand, Debug)]
pub(crate) enum ExportCommands {
    /// Export the entire document.{n}
    /// When using --output-file, only one input file can be given.{n}
    /// The export format is recognized from the file extension of the output file.{n}
    /// When using --output-format, the file name and path of the rnote file is used with the extension changed.{n}
    /// --output-file and --output-format are mutually exclusive but one of them is required.{n}
    /// Currently `.svg`, `.xopp` and `.pdf` are supported.{n}
    Doc {
        #[command(flatten)]
        file: FileArgs<DocOutputFormat>,
        /// The page order when documents with layouts that expand in horizontal and vertical directions are cut into pages.
        #[arg(long, default_value_t = PageOrder::default())]
        page_order: PageOrder,
    },
    /// Export each page of the documents individually.{n}
    /// Both --output-dir and --output-format need to be set.{n}
    #[command(alias = "pages")]
    DocPages {
        /// the directory the pages get exported to
        #[arg(short = 'o', long)]
        output_dir: PathBuf,
        /// The directory the pages get exported to.{n}
        /// Uses file stem of input file if not set.{n}
        /// Cannot be used when exporting multiple files
        #[arg(short = 's', long)]
        output_file_stem: Option<String>,
        /// the export output format
        #[arg(short = 'f', long)]
        output_format: DocPagesOutputFormat,
        /// The page order when documents with layouts that expand in horizontal and vertical directions are cut into pages
        #[arg(long, default_value_t = PageOrder::default())]
        page_order: PageOrder,
        /// bitmap scale factor in relation to the actual size on the document
        #[arg(long, default_value_t = DocPagesExportPrefs::default().bitmap_scalefactor)]
        bitmap_scalefactor: f64,
        /// quality of the jpeg image
        #[arg(long, default_value_t = DocPagesExportPrefs::default().jpeg_quality)]
        jpeg_quality: u8,
    },
    /// Export a selection in a document.{n}
    /// When using --output-file, only one input file can be given.{n}
    /// The export format is recognized from the file extension of the output file.{n}
    /// When using --output-format, the same file name is used with the extension changed.{n}
    /// --output-file and --output-format are mutually exclusive but one of them is required.{n}
    /// Available selection arguments - one of:{n}
    /// --all: select all strokes{n}
    /// --rect X Y width height: Select all strokes in given area{n}
    /// When not using --all, use --bounds to switch between intersecting and inside bounds
    #[command(alias = "sel")]
    Selection {
        #[command(flatten)]
        file: FileArgs<SelectionOutputFormat>,
        #[command(subcommand)]
        selection: SelectionCommands,
        #[arg(short = 'i', long, default_value_t = Bounds::default(), global = true)]
        /// if the strokes inside or intersecting with the given bounds are exported. Ignored when using all
        bounds: Bounds,
        /// bitmap scale factor in relation to the actual size on the document
        #[arg(long, default_value_t = SelectionExportPrefs::default().bitmap_scalefactor, global = true)]
        bitmap_scalefactor: f64,
        /// quality of the jpeg image
        #[arg(long, default_value_t = SelectionExportPrefs::default().jpeg_quality, global = true)]
        jpeg_quality: u8,
        /// margin around the document
        #[arg(long, default_value_t = SelectionExportPrefs::default().margin, global = true)]
        margin: f64,
    },
}

#[derive(Subcommand, Debug)]
pub(crate) enum SelectionCommands {
    /// Export all strokes
    All,
    /// Export a rectangular area of the document{n}
    /// usage: X Y deltaX deltaY{n}
    /// Goes to given coordiates and selects all strokes in a given rectangle based of the given delta values
    Rect {
        x: f64,
        y: f64,
        width: f64,
        height: f64,
    },
}

#[derive(Args, Debug)]
#[group(required = true, multiple = false)]
pub(crate) struct FileArgs<T: ValueEnum + 'static + Send + Sync> {
    /// the export output file. Only allows for one input file. Exclusive with --output-format
    #[arg(short = 'o', long, global = true)]
    output_file: Option<PathBuf>,
    /// the export output format. Exclusive with --output-file
    #[arg(short = 'f', long, global = true)]
    output_format: Option<T>,
}

#[derive(ValueEnum, Clone, Debug, Default)]
pub(crate) enum Bounds {
    #[default]
    Contains,
    Intersects,
}

impl Display for Bounds {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Bounds::Contains => "contains",
                Self::Intersects => "intersects",
            }
        )
    }
}

#[derive(ValueEnum, Clone, Debug)]
pub(crate) enum DocOutputFormat {
    Pdf,
    Xopp,
    Svg,
}
impl From<&DocOutputFormat> for DocExportFormat {
    fn from(val: &DocOutputFormat) -> Self {
        match val {
            DocOutputFormat::Pdf => DocExportFormat::Pdf,
            DocOutputFormat::Xopp => DocExportFormat::Xopp,
            DocOutputFormat::Svg => DocExportFormat::Svg,
        }
    }
}

#[derive(ValueEnum, Clone, Debug)]
pub(crate) enum SelectionOutputFormat {
    Svg,
    Png,
    Jpeg,
}

impl From<&SelectionOutputFormat> for SelectionExportFormat {
    fn from(val: &SelectionOutputFormat) -> Self {
        match val {
            SelectionOutputFormat::Svg => SelectionExportFormat::Svg,
            SelectionOutputFormat::Png => SelectionExportFormat::Png,
            SelectionOutputFormat::Jpeg => SelectionExportFormat::Jpeg,
        }
    }
}

#[derive(ValueEnum, Clone, Debug)]
pub(crate) enum DocPagesOutputFormat {
    Svg,
    Png,
    Jpeg,
}

impl From<&DocPagesOutputFormat> for DocPagesExportFormat {
    fn from(val: &DocPagesOutputFormat) -> Self {
        match val {
            DocPagesOutputFormat::Svg => DocPagesExportFormat::Svg,
            DocPagesOutputFormat::Png => DocPagesExportFormat::Png,
            DocPagesOutputFormat::Jpeg => DocPagesExportFormat::Jpeg,
        }
    }
}

#[derive(ValueEnum, Clone, Debug, Default)]
pub(crate) enum PageOrder {
    #[default]
    Horizontal,
    Vertical,
}

impl Display for PageOrder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                PageOrder::Horizontal => "horizontal",
                PageOrder::Vertical => "vertical",
            }
        )
    }
}

impl From<&PageOrder> for SplitOrder {
    fn from(val: &PageOrder) -> Self {
        match val {
            PageOrder::Horizontal => SplitOrder::RowMajor,
            PageOrder::Vertical => SplitOrder::ColumnMajor,
        }
    }
}

pub(crate) async fn run_export(
    export_commands: ExportCommands,
    engine: &mut RnoteEngine,
    rnote_files: Vec<PathBuf>,
    background: bool,
    pattern: bool,
    on_conflict: OnConflict,
) -> anyhow::Result<()> {
    if rnote_files.is_empty() {
        return Err(anyhow::anyhow!("No rnote files to export!"));
    }
    let output_file: Option<PathBuf>;
    // apply given arguments to export prefs
    match &export_commands {
        ExportCommands::Doc { file, page_order } => {
            output_file = file.output_file.clone();
            engine.export_prefs.doc_export_prefs = create_doc_export_prefs_from_args(
                output_file.as_deref(),
                file.output_format.as_ref(),
                background,
                pattern,
                page_order,
            )?
        }
        ExportCommands::DocPages {
            output_format,
            page_order,
            bitmap_scalefactor,
            jpeg_quality,
            output_file_stem,
            ..
        } => {
            if rnote_files.len() > 1 && output_file_stem.is_some() {
                return Err(anyhow::anyhow!(
                    "You cannot use --file-stem when exporting multiple rnote files"
                ));
            }
            engine.export_prefs.doc_pages_export_prefs = DocPagesExportPrefs {
                export_format: output_format.into(),
                with_background: background,
                with_pattern: pattern,
                page_order: page_order.into(),
                bitmap_scalefactor: *bitmap_scalefactor,
                jpeg_quality: *jpeg_quality,
            };
            output_file = None
        }
        ExportCommands::Selection {
            file,
            bitmap_scalefactor,
            jpeg_quality,
            margin,
            ..
        } => {
            output_file = file.output_file.clone();
            engine.export_prefs.selection_export_prefs = create_selection_export_prefs_from_args(
                output_file.as_deref(),
                file.output_format.as_ref(),
                background,
                pattern,
                *bitmap_scalefactor,
                *jpeg_quality,
                *margin,
            )?
        }
    }
    match output_file {
        Some(ref output_file) => {
            match rnote_files.get(0) {
                Some(rnote_file) => {
                    validators::file_has_ext(rnote_file, "rnote")?;
                    let new_path = check_file_conflict(output_file, &on_conflict)?;
                    // Replace output file if suffix added
                    let output_file = new_path.as_ref().unwrap_or(output_file);

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
                    if let Err(e) = export_to_file(
                        engine,
                        rnote_file,
                        output_file,
                        &export_commands,
                        &on_conflict,
                    )
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
            }
        }

        None => {
            let doc_pages = matches!(export_commands, ExportCommands::DocPages { .. });
            let output_ext = match &export_commands {
                ExportCommands::Doc { .. } => engine
                    .export_prefs
                    .doc_export_prefs
                    .export_format
                    .file_ext(),
                ExportCommands::DocPages { .. } => engine
                    .export_prefs
                    .doc_pages_export_prefs
                    .export_format
                    .file_ext(),
                ExportCommands::Selection { .. } => engine
                    .export_prefs
                    .selection_export_prefs
                    .export_format
                    .file_ext(),
            };
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
                let new_path = if doc_pages {
                    // conflicts are allowed here, generated output path will be ignored
                    None
                } else {
                    match check_file_conflict(output_file, &on_conflict) {
                        Ok(r) => r,
                        Err(e) => {
                            println!("{e}");
                            continue;
                        }
                    }
                };
                let output_file = new_path.as_ref().unwrap_or(output_file);

                let rnote_file_disp = rnote_file.display().to_string();
                let output_file_disp = output_file.display().to_string();
                let pb = indicatif::ProgressBar::new_spinner();
                pb.set_draw_target(indicatif::ProgressDrawTarget::stdout());
                pb.enable_steady_tick(Duration::from_millis(8));
                pb.set_message(match doc_pages {
                    true => format!("Exporting \"{rnote_file_disp}\""),
                    false => format!("Exporting \"{rnote_file_disp}\" to: \"{output_file_disp}\""),
                });

                // export
                if let Err(e) = export_to_file(
                    engine,
                    &rnote_file,
                    output_file,
                    &export_commands,
                    &on_conflict,
                )
                .await
                {
                    let msg = format!(
                        "Export \"{rnote_file_disp}\" to: \"{output_file_disp}\" failed, Err {e:?}"
                    );
                    if pb.is_hidden() {
                        println!("{msg}")
                    }
                    pb.abandon_with_message(msg);
                    return Err(e);
                } else {
                    let msg = match doc_pages {
                        false => format!(
                            "Export \"{rnote_file_disp}\" to: \"{output_file_disp}\" succeeded"
                        ),
                        true => format!("Export \"{rnote_file_disp}\" succeeded"),
                    };
                    if pb.is_hidden() {
                        println!("{msg}")
                    }
                    pb.finish_with_message(msg);
                }
            }
        }
    }

    println!("Export Finished!");
    Ok(())
}

pub(crate) fn create_doc_export_prefs_from_args(
    output_file: Option<impl AsRef<Path>>,
    output_format: Option<&DocOutputFormat>,
    background: bool,
    pattern: bool,
    page_order: &PageOrder,
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
        (None, Some(out_format)) => Ok(out_format.into()),
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

    let prefs = DocExportPrefs {
        export_format: format,
        with_background: background,
        with_pattern: pattern,
        page_order: page_order.into(),
    };

    Ok(prefs)
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
pub(crate) fn create_selection_export_prefs_from_args(
    output_file: Option<impl AsRef<Path>>,
    output_format: Option<&SelectionOutputFormat>,
    background: bool,
    pattern: bool,
    bitmap_scalefactor: f64,
    jpeg_quality: u8,
    margin: f64,
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
        (None, Some(out_format)) => Ok(out_format.into()),
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

    let prefs = SelectionExportPrefs {
        export_format: format,
        with_background: background,
        with_pattern: pattern,
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
            "Could not create selection export prefs, unsupported export file extension `{ext}`"
        )),
    }
}

pub(crate) fn check_file_conflict(
    output_file: &Path,
    mut on_conflict: &OnConflict,
) -> anyhow::Result<Option<PathBuf>> {
    if !output_file.exists() {
        return Ok(None);
    }
    let options = &[
        OnConflict::Ask,
        OnConflict::Overwrite,
        OnConflict::Skip,
        OnConflict::Suffix,
    ];
    while matches!(on_conflict, OnConflict::Ask) {
        match dialoguer::Select::new()
            .with_prompt(format!("File {} already exits:", output_file.display()))
            .items(options)
            .interact()
        {
            Ok(0) => {
                if let Err(e) = open::that(output_file) {
                    println!(
                        "Failed to open {} with default program, {e}",
                        output_file.display()
                    );
                }
            }
            Ok(c) => on_conflict = &options[c],
            Err(e) => {
                return Err(anyhow::anyhow!(
                "Failed to show select promt, retry or select an behavior with --on-conflict, {e}"
            ))
            }
        };
    }
    match on_conflict {
        OnConflict::Ask => Err(anyhow::anyhow!("Failed to save user choice!")),
        OnConflict::Overwrite => Ok(None),
        OnConflict::Skip => Err(anyhow::anyhow!("Skipped {}", output_file.display())),
        OnConflict::Suffix => {
            let mut i = 0;
            let mut new_path = output_file.to_path_buf();
            let Some(file_stem) = new_path.file_stem().map(|s| s.to_string_lossy().to_string()) else {
                return Err(anyhow::anyhow!("Failed to get file stem"));
            };
            let ext = new_path
                .extension()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or(String::new());
            while new_path.exists() {
                i += 1;
                new_path.set_file_name(format!("{file_stem}_{i}.{ext}"))
            }
            Ok(Some(new_path))
        }
    }
}

pub(crate) async fn export_to_file(
    engine: &mut RnoteEngine,
    rnote_file: impl AsRef<Path>,
    output_file: impl AsRef<Path>,
    export_commands: &ExportCommands,
    on_conflict: &OnConflict,
) -> anyhow::Result<()> {
    let rnote_file = rnote_file.as_ref();
    let rnote_bytes = {
        let mut out = vec![];
        File::open(rnote_file).await?.read_to_end(&mut out).await?;
        out
    };
    let engine_snapshot = EngineSnapshot::load_from_rnote_bytes(rnote_bytes).await?;
    let _ = engine.load_snapshot(engine_snapshot);

    match export_commands {
        ExportCommands::Selection {
            selection, bounds, ..
        } => {
            let (strokes, err_msg) = match selection {
                SelectionCommands::Rect {
                    x,
                    y,
                    width,
                    height,
                } => {
                    let v1 = Vector2::new(*x, *y);
                    let v2 = v1 + Vector2::new(*width, *height);
                    let points = vec![v1.into(), v2.into()];
                    let aabb = Aabb::from_points(&points);
                    (
                        match bounds {
                            Bounds::Contains => {
                                engine.store.stroke_keys_as_rendered_in_bounds(aabb)
                            }
                            Bounds::Intersects => engine
                                .store
                                .stroke_keys_as_rendered_intersecting_bounds(aabb),
                        },
                        "No strokes in given rectangle",
                    )
                }
                SelectionCommands::All => {
                    (engine.store.stroke_keys_as_rendered(), "Document is empty")
                }
            };
            if strokes.is_empty() {
                return Err(anyhow::anyhow!("{err_msg}"));
            }
            engine.store.set_selected_keys(&strokes, true);
            let export_bytes = engine
                .export_selection(None)
                .await??
                .context("No strokes selected")?;
            store_export(output_file, &export_bytes).await?;
        }
        ExportCommands::Doc { .. } => {
            let Some(export_file_name) = output_file.as_ref().file_name().map(|s| s.to_string_lossy().to_string()) else {
                return Err(anyhow::anyhow!("Failed to get filename from output_file"));
            };
            let export_bytes = engine.export_doc(export_file_name, None).await??;
            store_export(output_file, &export_bytes).await?;
        }
        ExportCommands::DocPages {
            output_dir,
            output_file_stem,
            output_format,
            ..
        } => {
            validators::path_is_dir(output_dir)?;
            // The output file cannnot be set with this subcommand
            drop(output_file);

            let export_bytes = engine.export_doc_pages(None).await??;
            let out_ext = DocPagesExportFormat::from(output_format).file_ext();
            let output_file_stem = match output_file_stem {
                Some(o) => o.clone(),
                None => match rnote_file.file_stem() {
                    Some(stem) => stem.to_string_lossy().to_string(),
                    None => {
                        return Err(anyhow::anyhow!(
                            "Failed to generate output_file_stem from rnote_file"
                        ))
                    }
                },
            };
            for (i, bytes) in export_bytes.into_iter().enumerate() {
                store_export(
                    &doc_page_output_file(i, output_dir, &out_ext, &output_file_stem, on_conflict)?,
                    &bytes,
                )
                .await
                .context(format!(
                    "Failed to export page {i} of document {}",
                    rnote_file.display()
                ))?
            }
        }
    };
    Ok(())
}

fn doc_page_output_file(
    i: usize,
    output_dir: &Path,
    out_ext: &str,
    output_file_stem: &str,
    on_conflict: &OnConflict,
) -> anyhow::Result<PathBuf> {
    let mut out = output_dir.join(format!(
        "{output_file_stem} - page {}{}.{out_ext}",
        if i + 1 < 10 { "0" } else { "" },
        i + 1,
    ));
    if let Some(new_out) = check_file_conflict(out.as_ref(), on_conflict)? {
        out = new_out;
    }
    Ok(out)
}

async fn store_export(output_file: impl AsRef<Path>, bytes: &[u8]) -> anyhow::Result<()> {
    let mut fh = File::create(output_file).await?;
    fh.write_all(bytes).await?;
    fh.sync_all().await?;
    Ok(())
}
