// Imports
use super::*;

impl RnoteFileInterfaceV1 {
    /// Handles the decompression and deserialization step. Created in order to avoiding code duplication.
    /// The generic argument `ES` dictates to what type the gutted serialized `EngineSnapshot` is deserialized into.
    ///   → `ES` = `EngineSnapshot` when going straight from bytes to `EngineSnapshot`
    ///   → `ES` = `ijson::IValue` when going from bytes to `CompatV1`
    /// The generic argument `SC` dictates to what type the stroke-chrono pair chunks are deserialized into.
    ///   → `SC` = `Vec<(Stroke, ChronoComponent)>` when going straight from bytes to `EngineSnapshot`
    ///   → `SC` = `ijson::IValue` when going from bytes to `CompatV1`
    fn bytes_to_deserialized<ES, SC>(
        mut cursor: BCursor,
        header_size: usize,
    ) -> anyhow::Result<(ES, Vec<SC>)>
    where
        ES: Send + DeserializeOwned,
        SC: Send + DeserializeOwned,
    {
        let header: RnoteFileHeaderV1 = serde_json::from_slice(cursor.try_capture(header_size)?)?;

        let engine_snapshot: ES = cursor
            .try_capture(header.core_info.c_size)
            .and_then(|c_data| {
                header
                    .compression_method
                    .decompress(c_data, header.core_info.uc_size)
            })
            .and_then(|uc_data| {
                serde_json::from_slice::<ES>(&uc_data).map_err(anyhow::Error::from)
            })?;

        // Not the most readable block of code but quite functional, we first gather
        // every serialized and compressed chunk with their respective uncompressed size,
        // to then decompress and deserialize them in parallel afterward.
        let stroke_chrono_pair_chunks = header
            .chunk_info_vec
            .into_iter()
            .map(|info| {
                cursor
                    .try_capture(info.c_size)
                    .map(|slice| (slice, info.uc_size))
            })
            .collect::<Result<Vec<(&[u8], usize)>, anyhow::Error>>()?
            .into_par_iter()
            .map(|(data, uc_size)| {
                header
                    .compression_method
                    .decompress(data, uc_size)
                    .and_then(|uc_data| {
                        serde_json::from_slice::<SC>(&uc_data).map_err(anyhow::Error::from)
                    })
            })
            .collect::<Result<Vec<SC>, anyhow::Error>>()?;

        Ok((engine_snapshot, stroke_chrono_pair_chunks))
    }

    /// Handles the recreation of the full `EngineSnapshot` from its gutted form and the
    /// chunks containing stroke-chrono pairs. Created in order to avoiding code duplication.
    fn components_to_engine_snapshot(
        mut engine_snapshot: EngineSnapshot,
        stroke_chrono_pair_chunks: Vec<Vec<(Stroke, ChronoComponent)>>,
    ) -> EngineSnapshot {
        let mut capacity = stroke_chrono_pair_chunks
            .iter()
            .map(Vec::len)
            .sum::<usize>();

        // We want some extra breathing room, here we are not trying to predict the number of
        // additional strokes a user is going to create after opening the file, instead we want
        // to minimize re-allocations, especially for very large slotmaps.
        capacity += capacity.div_ceil(6).clamp(400, 2000);

        let mut stroke_components: HopSlotMap<StrokeKey, Arc<Stroke>> =
            HopSlotMap::with_capacity_and_key(capacity);
        let mut chrono_components: SecondaryMap<StrokeKey, Arc<ChronoComponent>> =
            SecondaryMap::with_capacity(capacity);

        stroke_chrono_pair_chunks
            .into_iter()
            .flatten()
            .for_each(|(stroke, chrono)| {
                let key = stroke_components.insert(Arc::new(stroke));
                chrono_components.insert(key, Arc::new(chrono));
            });

        engine_snapshot.stroke_components = Arc::new(stroke_components);
        engine_snapshot.chrono_components = Arc::new(chrono_components);

        engine_snapshot
    }

    /// This function attempts to directly load the `EngineSnapshot`, skipping the compatibility
    /// representation for efficiency, should be used when the Rnote version specified by the file
    /// matches the current version of the application or is still directly compatible.
    pub fn bytes_to_engine_snapshot(
        cursor: BCursor,
        header_size: usize,
    ) -> anyhow::Result<EngineSnapshot> {
        let (engine_snapshot, stroke_chrono_pair_chunks) = Self::bytes_to_deserialized::<
            EngineSnapshot,
            Vec<(Stroke, ChronoComponent)>,
        >(cursor, header_size)?;

        Ok(Self::components_to_engine_snapshot(
            engine_snapshot,
            stroke_chrono_pair_chunks,
        ))
    }

    /// This function attempts to load an intermediate representation of the `EngineSnapshot`,
    /// for compatibility across differing versions of Rnote. Currently unused, but will be
    /// needed when an incompatible change to [`EngineSnapshot`] is made, refer to the docs
    /// in `rnoteformat/v1/version.rs` and `rnoteformat/mod.rs` for more information.
    #[allow(unused)]
    pub fn bytes_to_compat(cursor: BCursor, header_size: usize) -> anyhow::Result<CompatV1> {
        Self::bytes_to_deserialized::<ijson::IValue, ijson::IValue>(cursor, header_size).map(
            |(engine_snapshot_gutted, stroke_chrono_pair_chunks)| CompatV1 {
                engine_snapshot_gutted,
                stroke_chrono_pair_chunks,
            },
        )
    }

    /// This function attempts to convert a [CompatV1] or [CompatV1For] struct into an `EngineSnapshot`.
    /// Note that any upgrades must be performed before this function is called.
    pub fn compat_to_engine_snapshot<C: Into<CompatV1>>(
        compat: C,
    ) -> anyhow::Result<EngineSnapshot> {
        let compat: CompatV1 = compat.into();
        let engine_snapshot: EngineSnapshot = ijson::from_value(&compat.engine_snapshot_gutted)?;

        let stroke_chrono_pair_chunks = compat
            .stroke_chrono_pair_chunks
            .par_iter()
            .map(ijson::from_value::<Vec<(Stroke, ChronoComponent)>>)
            .collect::<Result<Vec<Vec<(Stroke, ChronoComponent)>>, serde_json::Error>>()?;

        Ok(Self::components_to_engine_snapshot(
            engine_snapshot,
            stroke_chrono_pair_chunks,
        ))
    }
}

impl TryFrom<LegacyRnoteFile> for CompatV1 {
    type Error = anyhow::Error;

    /// Attempts to convert a [LegacyRnoteFile] into a [CompatV1] struct, which entails a somewhat lengthy
    /// process as we need to free the `stroke_components` and `chrono_components` of their `slotmap` cage.
    fn try_from(mut value: LegacyRnoteFile) -> Result<Self, Self::Error> {
        let engine_snapshot = value
            .engine_snapshot
            .as_object_mut()
            .ok_or_else(|| anyhow::anyhow!("`engine_snapshot` is not a JSON object"))?;

        let mut raw_strokes: ijson::IArray = engine_snapshot
            .remove("stroke_components")
            .ok_or_else(|| anyhow::anyhow!("`engine_snapshot` has no value `stroke_components`"))?
            .into_array()
            .map_err(|_| anyhow::anyhow!("`stroke_components` is not a JSON array"))?;

        engine_snapshot.insert(
            "stroke_components",
            // Type definition of `stroke_components` in [`EngineSnapshot`]
            ijson::to_value::<Arc<HopSlotMap<StrokeKey, Arc<Stroke>>>>(Default::default()).unwrap(),
        );

        let mut raw_chronos: ijson::IArray = engine_snapshot
            .remove("chrono_components")
            .ok_or_else(|| anyhow::anyhow!("`engine_snapshot` has no value `chrono_components`"))?
            .into_array()
            .map_err(|_| anyhow::anyhow!("`chrono_components` is not a JSON array"))?;

        engine_snapshot.insert(
            "chrono_components",
            // Type definition of `chrono_components` in [`EngineSnapshot`]
            ijson::to_value::<Arc<SecondaryMap<StrokeKey, Arc<ChronoComponent>>>>(
                Default::default(),
            )
            .unwrap(),
        );

        let mut stroke_chrono_pair_vec: Vec<ijson::IValue> = raw_strokes
            .par_iter_mut()
            .zip_eq(raw_chronos.par_iter_mut())
            .filter_map(|(raw_stroke, raw_chrono)| {
                let stroke = raw_stroke
                    .remove("value")
                    .and_then(|value| (!value.is_null()).then_some(value))?;
                let chrono = raw_chrono
                    .remove("value")
                    .and_then(|value| (!value.is_null()).then_some(value))?;
                Some(ijson::IValue::from(ijson::IArray::from(vec![
                    stroke, chrono,
                ])))
            })
            .collect();

        if stroke_chrono_pair_vec.is_empty() {
            return Ok(CompatV1 {
                engine_snapshot_gutted: value.engine_snapshot,
                stroke_chrono_pair_chunks: vec![],
            });
        }

        // We could stuff everything into a single chunk but that would make any future required upgrades quite painful.
        let stroke_chrono_pair_chunks = {
            // The number of chunks to split `stroke_chrono_vec` into
            let nb_chunks = rayon::current_num_threads().max(1);

            let mut out: Vec<ijson::IValue> = Vec::with_capacity(nb_chunks);

            let nb_pairs = stroke_chrono_pair_vec.len();
            let base = nb_pairs / nb_chunks;

            for mult in (1..nb_chunks).rev() {
                out.push(stroke_chrono_pair_vec.split_off(mult * base).into());
            }
            out.push(stroke_chrono_pair_vec.into());

            out
        };

        Ok(CompatV1 {
            engine_snapshot_gutted: value.engine_snapshot,
            stroke_chrono_pair_chunks,
        })
    }
}
