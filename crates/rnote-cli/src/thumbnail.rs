// Inspired by: https://github.com/ayykamp/rnote-thumbnailer/blob/main/src/main.rs
// Author: ayykamp <kamp@ayyy.dev>

use anyhow::Context;
use image::DynamicImage;
use image::imageops::FilterType;
use rnote_engine::Engine;
use rnote_engine::engine::EngineSnapshot;
use rnote_engine::engine::export::{SelectionExportFormat, SelectionExportPrefs};
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

pub(crate) async fn run_thumbnail(
    rnote_file: PathBuf,
    output_size: u32,
    output: std::ffi::OsString,
) -> anyhow::Result<()> {
    let mut engine = Engine::default();
    let mut rnote_file_bytes = vec![];

    let mut fh = File::open(rnote_file)?;
    fh.read_to_end(&mut rnote_file_bytes)?;
    let engine_snapshot = EngineSnapshot::load_from_rnote_bytes(rnote_file_bytes).await?;

    // We dont care about the return values of these functions
    let _ = engine.load_snapshot(engine_snapshot);
    let _ = engine.select_all_strokes();

    let prefs = SelectionExportPrefs {
        export_format: SelectionExportFormat::Png,
        ..Default::default()
    };
    let export_bytes = engine
        .export_selection(Some(prefs))
        .await??
        .context("Exporting selection failed, no strokes selected.")?;

    let mut image = image::load_from_memory(&export_bytes)?;
    let (width, height) = (image.width(), image.height());

    if std::cmp::max(width, height) > output_size {
        let ratio = if width >= height {
            // Landscape
            width as f64 / output_size as f64
        } else {
            // Portrait
            height as f64 / output_size as f64
        };
        let nwidth = width as f64 / ratio;
        let nheight = height as f64 / ratio;
        image = DynamicImage::from(image::imageops::resize(
            &image,
            nwidth as u32,
            nheight as u32,
            FilterType::Nearest,
        ));
    }

    image.save(output)?;
    Ok(())
}
