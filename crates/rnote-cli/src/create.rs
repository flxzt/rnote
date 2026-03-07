// Imports
use crate::cli;
use rnote_engine::Engine;
use std::path::Path;

pub(crate) async fn create_new_file(engine: &mut Engine, rnote_file: &Path) -> anyhow::Result<()> {
    let Some(rnote_file_name) = rnote_file
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
    else {
        return Err(anyhow::anyhow!("Failed to get filename from rnote_file"));
    };

    // export the default/empty engine state.
    let rnote_bytes = engine.save_as_rnote_bytes(rnote_file_name).await??;
    cli::create_overwrite_file_w_bytes(rnote_file, &rnote_bytes).await?;

    Ok(())
}

pub(crate) async fn run_create(input_file: &Path) -> anyhow::Result<()> {
    let mut engine = Engine::default();

    // Pass the Option through to the inner function
    if let Err(e) = create_new_file(&mut engine, input_file).await {
        return Err(e);
    }

    Ok(())
}
