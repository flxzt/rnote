// Imports
use crate::{export, import, mutate, test, thumbnail};

use anyhow::Context;
use clap::Parser;
use clap::builder::PossibleValuesParser;
use rnote_compose::SplitOrder;
use rnote_engine::SelectionCollision;
use rnote_engine::engine::export::{
    DocExportFormat, DocPagesExportFormat, DocPagesExportPrefs, SelectionExportFormat,
    SelectionExportPrefs,
};
use rnote_engine::engine::import::XoppImportPrefs;
use rnote_engine::fileformats::rnoteformat;
use smol::fs::File;
use smol::io::{AsyncReadExt, AsyncWriteExt};
use std::path::{Path, PathBuf};
use std::time::Duration;

///    rnote-cli{n}{n}
///    This program is free software; you can redistribute it{n}
///    and/or modify it under the terms of the GPL v3 or (at your option){n}
///    any later version.
#[derive(clap::Parser, Debug, Clone)]
#[command(author, version, about, long_about = None, arg_required_else_help = true)]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub(crate) command: Command,
}

#[derive(clap::Subcommand, Debug, Clone)]
pub(crate) enum Command {
    /// Tests if the specified files can be opened and are valid rnote files.
    Test {
        /// The rnote files.
        rnote_files: Vec<PathBuf>,
    },
    /// Imports the specified input file and saves it as a rnote save file.{n}
    /// Currently only `.xopp` files can be imported.
    Import {
        /// The rnote save file.
        rnote_file: PathBuf,
        /// The import input file.
        #[arg(short = 'i', long)]
        input_file: PathBuf,
        /// When importing a .xopp file, the import dpi can be specified.
        #[arg(long, default_value_t = XoppImportPrefs::default().dpi)]
        xopp_dpi: f64,
    },
    /// Exports the Rnote file(s) and saves it/them in the desired format.{n}
    /// See sub-commands for usage.
    Export {
        #[command(subcommand)]
        export_command: ExportCommand,
        /// The rnote save file.
        #[arg(global = true)]
        rnote_files: Vec<PathBuf>,
        /// The action that will be performed if the to be exported file(s) already exist(s).
        #[arg(long, default_value = "ask", global = true)]
        on_conflict: OnConflict,
        /// Export without background.
        #[arg(short = 'b', long, action = clap::ArgAction::SetTrue, global = true)]
        no_background: bool,
        /// Export without background pattern.
        #[arg(short = 'p', long, action = clap::ArgAction::SetTrue, global = true)]
        no_pattern: bool,
        /// Optimize the background and stroke colors for printing.
        #[arg(long, action = clap::ArgAction::SetTrue, global = true)]
        optimize_printing: bool,
        /// Inspect the result after the export is finished.{n}
        /// Opens output folder when using "doc-pages" sub-command.
        #[arg(long, action = clap::ArgAction::SetTrue, global = true)]
        open: bool,
    },
    /// Mutates one or more of the following for the specified Rnote file(s):{n}
    /// compression method, compression level, serialization method, method lock
    Mutate {
        /// The rnote save file(s) to mutate
        rnote_files: Vec<PathBuf>,
        /// Keep the original rnote save file(s)
        #[arg(long = "not-in-place", alias = "nip", action = clap::ArgAction::SetTrue)]
        not_in_place: bool,
        /// Sets method_lock to true, allowing a rnote save file to keep using non-default methods to serialize and compress itself
        #[arg(short = 'l', long, action = clap::ArgAction::SetTrue, conflicts_with = "unlock")]
        lock: bool,
        /// Sets method_lock to false, coercing the file to use default methods on the next save
        #[arg(short = 'u', long, action = clap::ArgAction::SetTrue, conflicts_with = "lock")]
        unlock: bool,
        #[arg(short = 's', long, action = clap::ArgAction::Set, value_parser = PossibleValuesParser::new(rnoteformat::SerializationMethod::VALID_STR_ARRAY))]
        serialization_method: Option<String>,
        #[arg(short = 'c', long, action = clap::ArgAction::Set, value_parser = PossibleValuesParser::new(rnoteformat::CompressionMethod::VALID_STR_ARRAY))]
        compression_method: Option<String>,
        #[arg(short = 'v', long, action = clap::ArgAction::Set)]
        compression_level: Option<u8>,
    },
    /// Generate rnote thumbail from a given file
    Thumbnail {
        /// Input rnote file
        rnote_file: PathBuf,
        /// Size of the thumbnail in bits
        #[arg(short, long, default_value_t = 256)]
        size: u32,
        /// Output path of the thumbnail
        output: PathBuf,
    },
}

#[derive(clap::ValueEnum, Debug, Clone, Copy, Default)]
pub(crate) enum OnConflict {
    #[default]
    /// Ask before overwriting.
    Ask,
    /// Overwrite existing files.
    Overwrite,
    #[value(skip)]
    AlwaysOverwrite,
    /// Skip the export.
    Skip,
    #[value(skip)]
    AlwaysSkip,
    /// Append a number as a suffix to the file name.
    Suffix,
    #[value(skip)]
    AlwaysSuffix,
}

#[derive(clap::Subcommand, Debug, Clone)]
pub(crate) enum ExportCommand {
    /// Export the entire document.{n}
    /// When using "--output-file", only a single input file can be specified.{n}
    /// The export format will be recognized from the file extension of the output file.{n}
    /// When using "--output-format", the file name and path of the rnote file is used with the extension changed.{n}
    /// "--output-file and "--output-format" are mutually exclusive and specifying one of them is required.
    Doc {
        #[command(flatten)]
        file_args: FileArgs<DocExportFormat>,
        /// The page order when documents with layouts that expand in horizontal and vertical directions are cut into
        /// pages.
        #[arg(long, default_value_t = Default::default())]
        page_order: SplitOrder,
    },
    /// Export each page of the document(s) individually.{n}
    /// Both "--output-dir" and "--output-format" need to be set.
    DocPages {
        /// The directory the pages get exported to.
        #[arg(short = 'o', long)]
        output_dir: PathBuf,
        /// The file name stem when naming the to be exported pages files.
        #[arg(short = 's', long)]
        output_file_stem: Option<String>,
        /// The export output format.
        #[arg(short = 'f', long)]
        export_format: DocPagesExportFormat,
        /// The page order when documents with layouts that expand in horizontal and vertical directions are cut into
        /// pages.
        #[arg(long, default_value_t = Default::default())]
        page_order: SplitOrder,
        /// The bitmap scale-factor in relation to the actual size on the document.
        #[arg(long, default_value_t = DocPagesExportPrefs::default().bitmap_scalefactor)]
        bitmap_scalefactor: f64,
        /// The quality of the generated image(s) when Jpeg is used as export format.
        #[arg(long, default_value_t = DocPagesExportPrefs::default().jpeg_quality)]
        jpeg_quality: u8,
    },
    /// Export a selection in a document.{n}
    /// When using "--output-file", only a single input file can be specified.{n}
    /// The export format is then recognized from the file extension of the output file.{n}
    /// When using "--output-format", the file name and path of the rnote file is used with the extension changed.{n}
    /// "--output-file and "--output-format" are mutually exclusive and specifying one of them is required.
    Selection {
        #[command(flatten)]
        file_args: FileArgs<SelectionExportFormat>,
        #[command(subcommand)]
        selection: SelectionCommand,
        #[arg(short = 'c', long, default_value_t = Default::default(), global = true)]
        /// If strokes that are contained or intersect with the given bounds are selected.{n}
        /// Ignored when using option "all".
        selection_collision: SelectionCollision,
        /// The bitmap scale-factor in relation to the actual size on the document.
        #[arg(long, default_value_t = SelectionExportPrefs::default().bitmap_scalefactor, global = true)]
        bitmap_scalefactor: f64,
        /// The quality of the generated image(s) when Jpeg is used as export format.
        #[arg(long, default_value_t = SelectionExportPrefs::default().jpeg_quality, global = true)]
        jpeg_quality: u8,
        /// The margin around the to be exported content.
        #[arg(long, default_value_t = SelectionExportPrefs::default().margin, global = true)]
        margin: f64,
    },
}

#[derive(clap::Subcommand, Debug, Clone, Copy)]
pub(crate) enum SelectionCommand {
    /// Export all strokes.
    #[command(alias = "a")]
    All,
    /// Export a rectangular area of the document.
    #[command(alias = "r")]
    Rect {
        /// X-position of the upper-left point.
        x: f64,
        /// Y-position of the upper-left point.
        y: f64,
        /// Width of the rectangle.
        width: f64,
        /// Weight of the rectangle.
        height: f64,
    },
}

#[derive(clap::Args, Debug, Clone)]
#[group(required = true, multiple = false)]
pub(crate) struct FileArgs<T: clap::ValueEnum + 'static + Send + Sync> {
    /// The export output file. Exclusive with "--output-format".
    #[arg(short = 'o', long, global = true)]
    pub(crate) output_file: Option<PathBuf>,
    /// The export output format. Exclusive with "--output-file".
    #[arg(short = 'f', long, global = true)]
    pub(crate) output_format: Option<T>,
}

impl std::fmt::Display for OnConflict {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Ask => "Open existing file for inspection and ask again",
                Self::Overwrite => "Overwrite existing file",
                Self::AlwaysOverwrite => "Always overwrite existing files",
                Self::Skip => "Skip file",
                Self::AlwaysSkip => "Always skip file",
                Self::Suffix => "Append a number as a suffix to the file name",
                Self::AlwaysSuffix => "Always append a number as a suffix to the file name",
            }
        )
    }
}

pub(crate) async fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Test { rnote_files } => {
            println!("Testing..");
            test::run_test(&rnote_files).await?;
            println!("Tests finished successfully!");
        }
        Command::Import {
            rnote_file,
            input_file,
            xopp_dpi,
        } => {
            println!("Importing..");
            import::run_import(&rnote_file, &input_file, xopp_dpi).await?;
            println!("Import finished!");
        }
        Command::Export {
            rnote_files,
            no_background,
            no_pattern,
            optimize_printing,
            on_conflict,
            open,
            export_command,
        } => {
            println!("Exporting..");
            export::run_export(
                rnote_files,
                no_background,
                no_pattern,
                optimize_printing,
                on_conflict,
                open,
                export_command,
            )
            .await?;
            println!("Export finished!");
        }
        Command::Mutate {
            rnote_files,
            not_in_place,
            lock,
            unlock,
            serialization_method,
            compression_method,
            compression_level,
        } => {
            println!("Mutating..\n");
            mutate::run_mutate(
                rnote_files,
                not_in_place,
                lock,
                unlock,
                serialization_method,
                compression_method,
                compression_level,
            )
            .await?;
            println!("Mutate finished!");
        }
        Command::Thumbnail {
            rnote_file,
            size,
            output,
        } => {
            println!("Thumbnail...");
            thumbnail::run_thumbnail(rnote_file, size, output).await?;
        }
    }

    Ok(())
}

pub(crate) fn new_progressbar(message: String) -> indicatif::ProgressBar {
    let pb = indicatif::ProgressBar::new_spinner().with_message(message);
    pb.set_draw_target(indicatif::ProgressDrawTarget::stdout());
    pb.enable_steady_tick(Duration::from_millis(8));
    pb
}

pub(crate) async fn read_bytes_from_file(file_path: impl AsRef<Path>) -> anyhow::Result<Vec<u8>> {
    let mut bytes = vec![];
    let mut fh = File::open(file_path).await?;
    fh.read_to_end(&mut bytes).await?;
    Ok(bytes)
}

pub(crate) async fn create_overwrite_file_w_bytes(
    output_file: impl AsRef<Path>,
    bytes: &[u8],
) -> anyhow::Result<()> {
    let mut fh = File::create(output_file).await?;
    fh.write_all(bytes).await?;
    fh.sync_all().await?;
    Ok(())
}

pub(crate) fn open_file_default_app(file_path: impl AsRef<Path>) -> anyhow::Result<()> {
    open::that_detached(file_path.as_ref()).with_context(|| {
        format!(
            "Failed to open output file/folder \"{}\".",
            file_path.as_ref().display()
        )
    })?;
    Ok(())
}
