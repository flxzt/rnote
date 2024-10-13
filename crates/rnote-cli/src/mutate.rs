use rnote_engine::engine::EngineSnapshot;
use rnote_engine::fileformats::rnoteformat::RnoteHeader;
use rnote_engine::fileformats::FileFormatSaver;
use rnote_engine::fileformats::{rnoteformat::RnoteFile, FileFormatLoader};
use smol::{fs::OpenOptions, io::AsyncReadExt};
use std::path::PathBuf;
use std::str::FromStr;

pub(crate) async fn run_mutate(
    rnote_files: Vec<PathBuf>,
    not_in_place: bool,
    lock: bool,
    unlock: bool,
    serialization_method: Option<String>,
    compression_method: Option<String>,
    compression_level: Option<u8>,
) -> anyhow::Result<()> {
    let total_len = rnote_files.len();
    let mut total_delta: f64 = 0.0;
    for (idx, mut filepath) in rnote_files.into_iter().enumerate() {
        println!("Working on file {} out of {}", idx + 1, total_len);

        let file_read_operation = async {
            let mut read_file = OpenOptions::new().read(true).open(&filepath).await?;

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
        let bytes = file_read_operation.await?;
        let old_size_mb = bytes.len() as f64 / 1e6;
        let rnote_file = RnoteFile::load_from_bytes(&bytes)?;

        let serialization = if let Some(ref str) = serialization_method {
            rnote_engine::fileformats::rnoteformat::SerializationMethod::from_str(str).unwrap()
        } else {
            rnote_file.header.serialization
        };

        let mut compression = if let Some(ref str) = compression_method {
            rnote_engine::fileformats::rnoteformat::CompressionMethod::from_str(str).unwrap()
        } else {
            rnote_file.header.compression
        };

        if let Some(lvl) = compression_level {
            compression.update_compression_level(lvl)?;
        }

        let method_lock = (rnote_file.header.method_lock | lock) && !unlock;
        let uc_data = serialization.serialize(&EngineSnapshot::try_from(rnote_file)?)?;
        let uc_size = uc_data.len() as u64;
        let data = compression.compress(uc_data)?;

        let rnote_file = RnoteFile {
            header: RnoteHeader {
                serialization,
                compression,
                uc_size,
                method_lock,
            },
            body: data,
        };

        if not_in_place {
            let file_stem = filepath
                .file_stem()
                .ok_or_else(|| anyhow::anyhow!("File does not contain a valid file stem"))?
                .to_str()
                .ok_or_else(|| anyhow::anyhow!("File does not contain a valid file stem"))?;
            filepath.set_file_name(format!("{}_mut.rnote", file_stem));
        }

        let data = rnote_file.save_as_bytes("")?;
        let new_size_mb = data.len() as f64 / 1e6;
        rnote_engine::utils::atomic_save_to_file(&filepath, &data).await?;

        println!("{:.2} MB → {:.2} MB", old_size_mb, new_size_mb,);
        total_delta += new_size_mb - old_size_mb;
    }
    println!("\n⇒ ∆ = {:.2} MB", total_delta);
    Ok(())
}
