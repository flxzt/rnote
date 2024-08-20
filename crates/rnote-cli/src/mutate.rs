use rnote_engine::engine::EngineSnapshot;
use rnote_engine::fileformats::rnoteformat::RnoteHeader;
use rnote_engine::fileformats::FileFormatSaver;
use rnote_engine::fileformats::{rnoteformat::RnoteFile, FileFormatLoader};
use smol::io::AsyncWriteExt;
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
    for (idx, mut filepath) in rnote_files.into_iter().enumerate() {
        println!("Working on file {} out of {}", idx + 1, total_len);
        let mut bytes: Vec<u8> = Vec::new();
        OpenOptions::new()
            .read(true)
            .open(&filepath)
            .await?
            .read_to_end(&mut bytes)
            .await?;

        let rnote_file = RnoteFile::load_from_bytes(&bytes)?;

        let serialization = if let Some(ref str) = serialization_method {
            rnote_engine::fileformats::rnoteformat::SerM::from_str(str).unwrap()
        } else {
            rnote_file.head.serialization
        };

        let mut compression = if let Some(ref str) = compression_method {
            rnote_engine::fileformats::rnoteformat::CompM::from_str(str).unwrap()
        } else {
            rnote_file.head.compression
        };

        if let Some(lvl) = compression_level {
            compression.update_compression_level(lvl)?;
        }

        let method_lock = (rnote_file.head.method_lock | lock) && !unlock;
        let uc_data = serialization.serialize(&EngineSnapshot::try_from(rnote_file)?)?;
        let uc_size = uc_data.len() as u64;

        let rnote_file = RnoteFile {
            head: RnoteHeader {
                serialization,
                compression,
                uc_size,
                method_lock,
            },
            body: uc_data,
        };

        if not_in_place {
            let file_stem = filepath
                .file_stem()
                .ok_or(anyhow::anyhow!("File does not contain a valid file stem"))?
                .to_str()
                .ok_or(anyhow::anyhow!("File does not contain a valid file stem"))?;
            filepath.set_file_name(format!("{}_mutated.rnote", file_stem));
        }

        let data = rnote_file.save_as_bytes("")?;
        println!(
            "attempting to write {:.3} [MB] of data",
            data.len() as f64 / 1e6
        );

        let mut file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(&filepath)
            .await?;

        file.write_all(&data).await?;
        file.sync_all().await?;

        // TODO - use a two-step file saving process (once and if the file format is approved)
    }
    Ok(())
}
