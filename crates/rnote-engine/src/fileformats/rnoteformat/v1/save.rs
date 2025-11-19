use anyhow::Context;

// Imports
use super::*;

impl RnoteFileInterfaceV1 {
    /// Attempts to convert an `EngineSnapshot` to bytes.
    pub fn engine_snapshot_to_bytes(
        mut engine_snapshot: EngineSnapshot,
        compression_method: CompressionMethod,
    ) -> anyhow::Result<Vec<u8>> {
        let engine_strokes = std::mem::take(&mut engine_snapshot.stroke_components);
        let engine_chronos = std::mem::take(&mut engine_snapshot.chrono_components);

        let mut core_info = ChunkInfo {
            c_size: 0,
            uc_size: 0,
        };
        let core_bytes = compression_method
            .compress(
                serde_json::to_vec(&engine_snapshot)
                    .inspect(|encoded| core_info.uc_size = encoded.len())?,
            )
            .inspect(|compressed| core_info.c_size = compressed.len())?;

        // Not the nicest-looking approach, but avoids the drawbacks of other approaches I've tried.
        // Namely, this should play somewhat nicely with Rayon's load-balancing and leaves us with
        // `nb_threads` chunks of JSON-serialized data.
        let local_buffer: ThreadLocal<RefCell<Vec<u8>>> = ThreadLocal::new();
        engine_strokes
            .iter()
            .map(|(key, stroke)| engine_chronos.get(key).map(|chrono| (stroke, chrono)))
            .collect::<Option<Vec<(_, _)>>>()
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "At least one `Stroke` without an associated `ChronoComponent` was encountered."
                )
            })?
            .into_par_iter()
            .for_each_init(
                || {
                    local_buffer.get_or(|| {
                        std::cell::RefCell::new({
                            let mut vec = Vec::with_capacity(4096);
                            vec.push(b'[');
                            vec
                        })
                    })
                },
                |&mut cell, stroke_chrono_pair| {
                    let mut buf = cell.borrow_mut();
                    if buf.len() > 1 {
                        buf.push(b',');
                    }
                    serde_json::to_writer(&mut *buf, &stroke_chrono_pair).unwrap() // Fine to unwrap here
                },
            );

        let mut chunk_vec = local_buffer
            .into_iter()
            .map(RefCell::into_inner)
            .collect_vec();

        // Compress the chunks and gather info on their size pre- and post-compression at the same time.
        let chunk_info_vec = chunk_vec
            .par_iter_mut()
            .map(|chunk| {
                chunk.push(b']');
                let uc_size = chunk.len();
                compression_method
                    .compress_mut(chunk)
                    .with_context(|| "Zstd compression failed.")?;
                let c_size = chunk.len();
                Ok(ChunkInfo { c_size, uc_size })
            })
            .collect::<anyhow::Result<Vec<ChunkInfo>>>()?;

        let header = RnoteFileHeaderV1 {
            compression_method,
            core_info,
            chunk_info_vec,
        };
        let header_bytes = serde_json::to_vec(&header)?;

        let prelude_bytes = Prelude::new(
            RnoteFileInterfaceV1::FILE_VERSION,
            semver::Version::parse(crate::utils::crate_version())?,
            header_bytes.len(),
        )
        .try_to_bytes()?;

        Ok([prelude_bytes, header_bytes, core_bytes, chunk_vec.concat()].concat())
    }
}
