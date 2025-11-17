// Imports
use super::*;

impl RnoteFileInterfaceV1 {
    // The generic argument `ES` dictates to what type the gutted serialized `EngineSnapshot` is deserialized into.
    //   → `ES` = `EngineSnapshot` when going straight from bytes to `EngineSnapshot`
    //   → `ES` = `ijson::IValue` when going from bytes to `CompatBridgeV1`
    // The generic argument `SC` dictates to what type the stroke-chrono pair chunks are deserialized into.
    //   → `SC` = `Vec<(Stroke, ChronoComponent)>` when going straight from bytes to `EngineSnapshot`
    //   → `SC` = `ijson::IValue` when going from bytes to `CompatBridgeV1`
    fn bytes_to_base<ES, SC>(
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
        let stroke_chrono_pair_chunk_vec = header
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

        Ok((engine_snapshot, stroke_chrono_pair_chunk_vec))
    }

    fn components_to_engine_snapshot(
        mut engine_snapshot: EngineSnapshot,
        stroke_chrono_pair_chunk_vec: Vec<Vec<(Stroke, ChronoComponent)>>,
    ) -> EngineSnapshot {
        let capacity = stroke_chrono_pair_chunk_vec
            .iter()
            .map(Vec::len)
            .sum::<usize>();

        let mut stroke_components: HopSlotMap<StrokeKey, Arc<Stroke>> =
            HopSlotMap::with_capacity_and_key(capacity);
        let mut chrono_components: SecondaryMap<StrokeKey, Arc<ChronoComponent>> =
            SecondaryMap::with_capacity(capacity);

        stroke_chrono_pair_chunk_vec
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

    pub fn bytes_to_engine_snapshot(
        cursor: BCursor,
        header_size: usize,
    ) -> anyhow::Result<EngineSnapshot> {
        let (engine_snapshot, stroke_chrono_pair_chunk_vec) = Self::bytes_to_base::<
            EngineSnapshot,
            Vec<(Stroke, ChronoComponent)>,
        >(cursor, header_size)?;

        Ok(Self::components_to_engine_snapshot(
            engine_snapshot,
            stroke_chrono_pair_chunk_vec,
        ))
    }

    pub fn bytes_to_sc_bridge(
        cursor: BCursor,
        header_size: usize,
    ) -> anyhow::Result<CompatBridgeV1> {
        Self::bytes_to_base::<ijson::IValue, ijson::IValue>(cursor, header_size).map(
            |(engine_snapshot_gutted, stroke_chrono_pair_chunks)| CompatBridgeV1 {
                engine_snapshot_gutted,
                stroke_chrono_pair_chunks,
            },
        )
    }

    pub fn bridge_to_engine_snapshot(bridge: CompatBridgeV1) -> anyhow::Result<EngineSnapshot> {
        let engine_snapshot: EngineSnapshot = ijson::from_value(&bridge.engine_snapshot_gutted)?;

        let stroke_chrono_pair_chunk_vec = bridge
            .stroke_chrono_pair_chunks
            .par_iter()
            .map(ijson::from_value::<Vec<(Stroke, ChronoComponent)>>)
            .collect::<Result<Vec<Vec<(Stroke, ChronoComponent)>>, serde_json::Error>>()?;

        Ok(Self::components_to_engine_snapshot(
            engine_snapshot,
            stroke_chrono_pair_chunk_vec,
        ))
    }
}

type EngineStrokes = Arc<HopSlotMap<StrokeKey, Arc<Stroke>>>;
type EngineChronos = Arc<SecondaryMap<StrokeKey, Arc<ChronoComponent>>>;

impl TryFrom<LegacyRnoteFile> for CompatBridgeV1 {
    type Error = anyhow::Error;

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
            ijson::to_value::<EngineStrokes>(Default::default()).unwrap(),
        );

        let mut raw_chronos: ijson::IArray = engine_snapshot
            .remove("chrono_components")
            .ok_or_else(|| anyhow::anyhow!("`engine_snapshot` has no value `chrono_components`"))?
            .into_array()
            .map_err(|_| anyhow::anyhow!("`chrono_components` is not a JSON array"))?;

        engine_snapshot.insert(
            "chrono_components",
            ijson::to_value::<EngineChronos>(Default::default()).unwrap(),
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
            return Ok(CompatBridgeV1 {
                engine_snapshot_gutted: value.engine_snapshot,
                stroke_chrono_pair_chunks: vec![],
            });
        }

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

        Ok(CompatBridgeV1 {
            engine_snapshot_gutted: value.engine_snapshot,
            stroke_chrono_pair_chunks,
        })
    }
}
