// Imports
use crate::{cli, validators};
use rnote_engine::engine::EngineSnapshot;
use std::path::{Path, PathBuf};

pub(crate) async fn run_test(rnote_files: &[PathBuf]) -> anyhow::Result<()> {
    for rnote_file in rnote_files.iter() {
        validators::file_has_ext(rnote_file, "rnote")?;
        let file_disp = rnote_file.display().to_string();
        let progressbar = cli::new_progressbar(format!("Testing file \"{file_disp}\""));

        if let Err(e) = test_file(rnote_file).await {
            let abandon_msg = format!("Test failed, Err: {e:?}");
            if progressbar.is_hidden() {
                println!("{abandon_msg}");
            }
            progressbar.abandon_with_message(abandon_msg);
            return Err(e);
        } else {
            let finish_msg = format!("Test succeeded for file \"{file_disp}\"");
            if progressbar.is_hidden() {
                println!("{finish_msg}");
            }
            progressbar.finish_with_message(finish_msg);
        }
    }

    Ok(())
}

pub(crate) async fn test_file(rnote_file: impl AsRef<Path>) -> anyhow::Result<()> {
    let rnote_bytes = cli::read_bytes_from_file(&rnote_file).await?;
    let _ = EngineSnapshot::load_from_rnote_bytes(rnote_bytes).await?;
    // Loading a valid snapshot into the engine can't fail, so we skip it.
    Ok(())
}
