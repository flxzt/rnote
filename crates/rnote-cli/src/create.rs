// Creates
use crate::{cli, validators};
use rnote_engine::Engine;
use rnote_engine::engine::{EngineConfigShared, EngineSnapshot, snapshot};
use std::path::Path;

pub(crate) async fn create_new_file(
    engine: &mut Engine,
    rnote_file: &Path,
    template_file: Option<&Path>, // Changed to Option
) -> anyhow::Result<()> {
    let Some(rnote_file_name) = rnote_file
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
    else {
        return Err(anyhow::anyhow!("Failed to get filename from rnote_file"));
    };

    // Only load a snapshot if a template path was actually provided
    if let Some(path) = template_file {
        let input_bytes = cli::read_bytes_from_file(path).await?;
        let snapshot = EngineSnapshot::load_from_rnote_bytes(input_bytes).await?;
        let _ = engine.load_snapshot(snapshot);
    }

    // If no template was loaded, engine.save_as_rnote_bytes will
    // export the default/empty engine state.
    let rnote_bytes = engine.save_as_rnote_bytes(rnote_file_name).await??;
    cli::create_overwrite_file_w_bytes(rnote_file, &rnote_bytes).await?;

    Ok(())
}

pub(crate) async fn run_create(
    input_file: &Path,
    template_file: Option<&Path>, // Changed to Option
) -> anyhow::Result<()> {
    // Only validate the extension if the user actually provided a template
    if let Some(path) = template_file {
        validators::file_has_ext(path, "rnote")?;
    }

    let config = EngineConfigShared::default();
    let mut engine = Engine::default();
    let _ = engine.install_config(&config, None);

    // Pass the Option through to the inner function
    if let Err(e) = create_new_file(&mut engine, input_file, template_file).await {
        println!("Cannot Create a file: {:?}", e);
        return Err(e);
    } else {
        println!("File Created Successfully");
    }

    Ok(())
}
