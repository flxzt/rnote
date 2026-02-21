use indicatif::ProgressIterator;
use rnote_engine::fileformats::rnoteformat::{
    self, CompressionMethod, DEFAULT_ZSTD_COMPRESSION_INTEGER,
};
use smol::{fs::OpenOptions, io::AsyncReadExt};
use std::path::PathBuf;

pub(crate) async fn run_set_compression(
    rnote_files: Vec<PathBuf>,
    compression_method: String,
    compression_level: Option<i32>,
) -> anyhow::Result<()> {
    let mut compression_method = match compression_method.as_str() {
        "zstd" | "Zstd" => CompressionMethod::Zstd(DEFAULT_ZSTD_COMPRESSION_INTEGER),
        "none" | "None" => CompressionMethod::None,
        _ => unreachable!(),
    };

    if let Some(compression_level) = compression_level
        && !matches!(compression_method, CompressionMethod::None)
    {
        compression_method.update_compression_integer(compression_level)?;
    }

    let spinner = indicatif::ProgressBar::new_spinner().with_style(
        indicatif::ProgressStyle::default_spinner()
            .tick_chars("⊶⊷✔")
            .template("{spinner:.green} [{elapsed_precise}] ({pos}/{len}) Mutating '{msg}'")
            .unwrap(),
    );
    spinner.set_length(rnote_files.len() as u64);
    spinner.enable_steady_tick(std::time::Duration::from_millis(250));

    for filepath in rnote_files.iter().progress_with(spinner.clone()) {
        spinner.set_message(format!("{}", filepath.display()));
        let file_read_operation = async {
            let mut read_file = OpenOptions::new().read(true).open(filepath).await?;
            let mut bytes: Vec<u8> = {
                match read_file.metadata().await {
                    Ok(metadata) => {
                        Vec::with_capacity(usize::try_from(metadata.len()).unwrap_or(usize::MAX))
                    }
                    Err(err) => {
                        eprintln!("Failed to read file metadata, '{err}'");
                        Vec::new()
                    }
                }
            };
            read_file.read_to_end(&mut bytes).await?;
            Ok::<Vec<u8>, anyhow::Error>(bytes)
        };

        let mut bytes = file_read_operation.await?;
        let engine_snapshot = rnoteformat::load_engine_snapshot_from_bytes(&bytes)?;
        bytes = rnoteformat::save_engine_snapshot_to_bytes(engine_snapshot, compression_method)?;
        rnote_engine::utils::atomic_save_to_file(filepath, &bytes).await?
    }

    Ok(())
}
