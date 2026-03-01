// Inspired by: https://github.com/ayykamp/rnote-thumbnailer/blob/main/src/main.rs
// Author: ayykamp <kamp@ayyy.dev>

use anyhow::{Context, anyhow};
use async_fs::File;
use core::time::Duration;
use futures::{AsyncReadExt, AsyncWriteExt, FutureExt, select};
use rnote_engine::Engine;
use rnote_engine::engine::EngineSnapshot;
use rnote_engine::engine::export::SelectionExportFormat;
use smol::Timer;
use std::path::PathBuf;

pub(crate) async fn run_thumbnail(
    rnote_file: PathBuf,
    size: u32,
    output: PathBuf,
    timeout: Option<Duration>,
) -> anyhow::Result<()> {
    let mut engine = Engine::default();
    let mut rnote_file_bytes = vec![];

    let mut fh = File::open(rnote_file).await?;
    fh.read_to_end(&mut rnote_file_bytes).await?;
    let engine_snapshot = EngineSnapshot::load_from_rnote_bytes(rnote_file_bytes).await?;

    // We dont care about the return values of these functions
    let _ = engine.load_snapshot(engine_snapshot);
    let mut timeout = if let Some(timeout) = timeout {
        Timer::after(timeout).fuse()
    } else {
        Timer::never().fuse()
    };
    let mut export_op = engine
        .generate_thumbnail(size, SelectionExportFormat::Png)
        .fuse();
    let export_bytes = select! {
        res = export_op => res??.context("Generating thumbnail failed, empty document.")?,
        _ = timeout => return Err(anyhow!("Timeout reached"))
    };
    let mut fh = File::create(output).await?;
    fh.write_all(&export_bytes).await?;
    fh.sync_all().await?;

    Ok(())
}
