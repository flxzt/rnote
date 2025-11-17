//! rnote-cli
//!
//! The cli interface is not (yet) stable and could change at any time.

use std::{
    io::{Read, Write},
    ops::Div,
    path::PathBuf,
    time::{Duration, Instant},
};

use itertools::Itertools;
use rnote_engine::{
    engine::EngineSnapshot,
    fileformats::{FileFormatLoader, rnoteformat},
};

// Modules
pub(crate) mod cli;
pub(crate) mod export;
pub(crate) mod import;
pub(crate) mod set_compression;
pub(crate) mod test;
pub(crate) mod thumbnail;
pub(crate) mod validators;

// Renames
extern crate nalgebra as na;
extern crate parry2d_f64 as p2d;

/*
fn main() -> anyhow::Result<()> {
    smol::block_on(async { cli::run().await })
}
*/

// temporary for testing purposes
// --- ✀ ---

fn warmup(iterations: u64, mem_size: usize) {
    let mut vec = vec![0u8; mem_size];
    let mut x = 0u64;
    for i in 0..iterations {
        x = x.wrapping_add(i);
        vec[i as usize % mem_size] ^= 0xAA;
    }
    std::hint::black_box(x);
    std::hint::black_box(vec);
}

fn main() -> anyhow::Result<()> {
    let benchmark_path = std::env::home_dir().unwrap().join("Downloads/benchmark/");

    let old_filepaths: Vec<PathBuf> = (2..=9)
        .map(|i| benchmark_path.join(format!("old/{i}.rnote")))
        .collect();
    let new_filepaths: Vec<PathBuf> = (2..=9)
        .map(|i| benchmark_path.join(format!("new/{i}.rnote")))
        .collect();

    let mut bytes: Vec<u8> = Vec::new();

    if std::env::var("CNF").is_ok() {
        println!("creating new files");
        for (old_fp, new_fp) in old_filepaths.iter().zip_eq(new_filepaths.iter()) {
            std::fs::OpenOptions::new()
                .read(true)
                .open(old_fp)?
                .read_to_end(&mut bytes)?;

            let es = rnoteformat::load_engine_snapshot_from_bytes(&bytes)?;
            let new_bytes = rnoteformat::save_engine_snapshot_to_bytes(
                es,
                rnoteformat::CompressionMethod::default(),
            )?;

            std::fs::OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(new_fp)?
                .write_all(&new_bytes)?;

            bytes.clear();
        }
    }

    warmup(100000, 500000);

    let mut files_orig_size: Vec<f64> = Vec::new(); // in megabytes
    let mut old_load_times: Vec<f64> = Vec::new();
    let mut old_save_times: Vec<f64> = Vec::new();

    for (idx, fp) in old_filepaths.iter().enumerate() {
        println!("For file n°{}", idx + 2);
        std::fs::OpenOptions::new()
            .read(true)
            .open(fp)?
            .read_to_end(&mut bytes)?;

        files_orig_size.push(bytes.len() as f64 / 1E6);

        let engine_snapshot: EngineSnapshot = ijson::from_value(
            &rnoteformat::LegacyRnoteFile::load_from_bytes(&bytes)
                .unwrap()
                .engine_snapshot,
        )?;
        let time = (0..3)
            .map(|_| {
                let start = Instant::now();
                ijson::from_value::<EngineSnapshot>(
                    &rnoteformat::LegacyRnoteFile::load_from_bytes(&bytes)
                        .unwrap()
                        .engine_snapshot,
                )
                .unwrap();
                Instant::now().duration_since(start)
            })
            .sum::<Duration>()
            .as_secs_f64()
            .div(3.0);
        println!("→ load avg: {:.2} [ms]", time * 1000.0);
        old_load_times.push(time);

        bytes.clear();

        let time = (0..3)
            .map(|_| {
                let start = Instant::now();
                compress_to_gzip(
                    &serde_json::to_vec(&rnoteformat::LegacyRnoteFile {
                        engine_snapshot: ijson::to_value(&engine_snapshot).unwrap(),
                    })
                    .unwrap(),
                )
                .unwrap();
                Instant::now().duration_since(start)
            })
            .sum::<Duration>()
            .as_secs_f64()
            .div(3.0);
        println!("→ save avg: {:.2} [ms]\n", time * 1000.0);
        old_save_times.push(time);
    }

    std::thread::sleep(Duration::from_secs(3));

    warmup(100000, 500000);

    let mut new_load_times: Vec<f64> = Vec::new();
    let mut new_save_times: Vec<f64> = Vec::new();

    for (idx, fp) in new_filepaths.iter().enumerate() {
        println!("For file n°{}", idx + 2);
        std::fs::OpenOptions::new()
            .read(true)
            .open(fp)?
            .read_to_end(&mut bytes)?;

        let engine_snapshot = rnoteformat::load_engine_snapshot_from_bytes(&bytes)?;

        let time = (0..3)
            .map(|_| {
                let start = Instant::now();
                rnoteformat::load_engine_snapshot_from_bytes(&bytes).unwrap();
                Instant::now().duration_since(start)
            })
            .sum::<Duration>()
            .as_secs_f64()
            .div(3.0);
        println!("→ load avg: {:.2} [ms]", time * 1000.0);
        new_load_times.push(time);

        bytes.clear();

        let time = (0..3)
            .map(|_| {
                let es = engine_snapshot.clone();
                let start = Instant::now();
                rnoteformat::save_engine_snapshot_to_bytes(
                    es,
                    rnoteformat::CompressionMethod::default(),
                )
                .unwrap();
                Instant::now().duration_since(start)
            })
            .sum::<Duration>()
            .as_secs_f64()
            .div(3.0);
        println!("→ save avg: {:.2} [ms]\n", time * 1000.0);
        new_save_times.push(time);
    }

    println!(
        "original_file_sizes = [{}]",
        files_orig_size
            .into_iter()
            .map(|x| format!("{x:.2}"))
            .join(", ")
    );

    println!(
        "old_load_times = [{}]",
        old_load_times
            .into_iter()
            .map(|x| format!("{:.2}", x * 1000.0))
            .join(", ")
    );

    println!(
        "old_save_times = [{}]",
        old_save_times
            .into_iter()
            .map(|x| format!("{:.2}", x * 1000.0))
            .join(", ")
    );

    println!(
        "new_load_times = [{}]",
        new_load_times
            .into_iter()
            .map(|x| format!("{:.2}", x * 1000.0))
            .join(", ")
    );

    println!(
        "new_save_times = [{}]",
        new_save_times
            .into_iter()
            .map(|x| format!("{:.2}", x * 1000.0))
            .join(", ")
    );

    Ok(())
}

fn compress_to_gzip(to_compress: &[u8]) -> Result<Vec<u8>, anyhow::Error> {
    let mut encoder = flate2::write::GzEncoder::new(Vec::<u8>::new(), flate2::Compression::new(5));
    encoder.write_all(to_compress)?;
    Ok(encoder.finish()?)
}
