// Imports
use crate::cli;
use rnote_engine::Engine;
use std::path::Path;

pub(crate) async fn run_create(rnote_file: &Path) -> anyhow::Result<()> {
    let engine = Engine::default();
    let Some(rnote_file_name) = rnote_file
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
    else {
        return Err(anyhow::anyhow!(
            "Failed to get filename from the supplied file '{}'",
            rnote_file.display()
        ));
    };
    let rnote_bytes = engine.save_as_rnote_bytes(rnote_file_name).await??;
    cli::create_overwrite_file_w_bytes(rnote_file, &rnote_bytes).await?;
    Ok(())
}
