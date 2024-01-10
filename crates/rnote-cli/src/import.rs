// Imports
use crate::{cli, validators};
use rnote_engine::engine::EngineSnapshot;
use rnote_engine::Engine;
use std::path::Path;

pub(crate) async fn run_import(
    rnote_file: &Path,
    input_file: &Path,
    xopp_dpi: f64,
) -> anyhow::Result<()> {
    validators::file_has_ext(rnote_file, "rnote")?;
    // Xopp files don't require file extensions
    validators::path_is_file(input_file)?;

    let mut engine = Engine::default();

    apply_import_prefs(&mut engine, xopp_dpi)?;

    let rnote_file_disp = rnote_file.display().to_string();
    let input_file_disp = input_file.display().to_string();
    let progressbar = cli::new_progressbar(format!(
        "Importing \"{input_file_disp}\" to: \"{rnote_file_disp}\""
    ));

    if let Err(e) = import_file(&mut engine, input_file, rnote_file).await {
        let abandon_msg =
            format!("Import \"{input_file_disp}\" to \"{rnote_file_disp}\" failed, Err: {e:?}");
        if progressbar.is_hidden() {
            println!("{abandon_msg}");
        }
        progressbar.abandon_with_message(abandon_msg);
        return Err(e);
    } else {
        let finish_msg = format!("Import \"{input_file_disp}\" to \"{rnote_file_disp}\" succeeded");
        if progressbar.is_hidden() {
            println!("{finish_msg}");
        }
        progressbar.finish_with_message(finish_msg);
    }

    Ok(())
}

pub(crate) fn apply_import_prefs(engine: &mut Engine, xopp_dpi: f64) -> anyhow::Result<()> {
    engine.import_prefs.xopp_import_prefs.dpi = xopp_dpi;
    Ok(())
}

pub(crate) async fn import_file(
    engine: &mut Engine,
    input_file: &Path,
    rnote_file: &Path,
) -> anyhow::Result<()> {
    let Some(rnote_file_name) = rnote_file
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
    else {
        return Err(anyhow::anyhow!("Failed to get filename from rnote_file"));
    };
    let input_bytes = cli::read_bytes_from_file(&input_file).await?;
    let snapshot =
        EngineSnapshot::load_from_xopp_bytes(input_bytes, engine.import_prefs.xopp_import_prefs)
            .await?;
    let _ = engine.load_snapshot(snapshot);
    let rnote_bytes = engine.save_as_rnote_bytes(rnote_file_name).await??;
    cli::create_overwrite_file_w_bytes(&rnote_file, &rnote_bytes).await?;

    Ok(())
}
