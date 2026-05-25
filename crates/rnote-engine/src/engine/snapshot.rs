// Imports
use crate::document::background;
use crate::engine::import::XoppImportPrefs;
use crate::fileformats::{FileFormatLoader, rnoteformat, xoppformat};
use crate::store::{ChronoComponent, DocumentLayer, DocumentLayerId, LayerComponent, StrokeKey};
use crate::strokes::Stroke;
use crate::{Camera, Document, Engine};
use anyhow::Context;
use futures::channel::oneshot;
use serde::{Deserialize, Serialize};
use slotmap::{SecondaryMap, SlotMap};
use std::sync::Arc;
use tracing::error;

/// Trait for types which hold configuration needed for engine snapshots
pub trait Snapshotable {
    fn extract_snapshot_data(&self) -> Self;
}

// An engine snapshot, used when loading/saving the current document from/into a file.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename = "engine_snapshot")]
pub struct EngineSnapshot {
    #[serde(rename = "document")]
    pub document: Document,
    #[serde(rename = "camera")]
    pub camera: Camera,
    #[serde(rename = "stroke_components")]
    pub stroke_components: Arc<SlotMap<StrokeKey, Arc<Stroke>>>,
    #[serde(rename = "chrono_components")]
    pub chrono_components: Arc<SecondaryMap<StrokeKey, Arc<ChronoComponent>>>,
    #[serde(rename = "layer_components")]
    pub layer_components: Arc<SecondaryMap<StrokeKey, Arc<LayerComponent>>>,
    #[serde(rename = "layers")]
    pub layers: Arc<Vec<DocumentLayer>>,
    #[serde(rename = "active_layer_id")]
    pub active_layer_id: DocumentLayerId,
    #[serde(rename = "layer_counter")]
    pub layer_counter: u32,
    #[serde(rename = "chrono_counter")]
    pub chrono_counter: u32,
}

impl Default for EngineSnapshot {
    fn default() -> Self {
        Self {
            document: Document::default(),
            camera: Camera::default(),
            stroke_components: Arc::new(SlotMap::with_key()),
            chrono_components: Arc::new(SecondaryMap::new()),
            layer_components: Arc::new(SecondaryMap::new()),
            layers: Arc::new(vec![DocumentLayer::default()]),
            active_layer_id: DocumentLayer::default().id,
            layer_counter: DocumentLayer::default().id,
            chrono_counter: 0,
        }
    }
}

impl EngineSnapshot {
    /// Loads a snapshot from the bytes of a .rnote file.
    ///
    /// To import this snapshot into the current engine, use [`Engine::load_snapshot()`].
    pub async fn load_from_rnote_bytes(bytes: Vec<u8>) -> anyhow::Result<Self> {
        let (snapshot_sender, snapshot_receiver) = oneshot::channel::<anyhow::Result<Self>>();

        rayon::spawn(move || {
            let result = || -> anyhow::Result<Self> {
                let rnote_file = rnoteformat::RnoteFile::load_from_bytes(&bytes)
                    .context("loading RnoteFile from bytes failed.")?;
                Ok(ijson::from_value(&rnote_file.engine_snapshot)?)
            };

            if let Err(_data) = snapshot_sender.send(result()) {
                error!(
                    "Sending bytes result to receiver failed while loading rnote bytes in. Receiver already dropped."
                );
            }
        });

        snapshot_receiver.await?
    }
    /// Loads from the bytes of a Xournal++ .xopp file.
    ///
    /// To import this snapshot into the current engine, use [`Engine::load_snapshot()`].
    pub async fn load_from_xopp_bytes(
        bytes: Vec<u8>,
        xopp_import_prefs: XoppImportPrefs,
    ) -> anyhow::Result<Self> {
        let (snapshot_sender, snapshot_receiver) = oneshot::channel::<anyhow::Result<Self>>();

        rayon::spawn(move || {
            let result = || -> anyhow::Result<Self> {
                let xopp_file = xoppformat::XoppFile::load_from_bytes(&bytes)?;

                // Extract the largest width of all pages, add together all heights
                let (doc_width, doc_height) = xopp_file
                    .xopp_root
                    .pages
                    .iter()
                    .map(|page| (page.width, page.height))
                    .fold(
                        (0_f64, 0_f64),
                        |(prev_width, prev_height), (next_width, next_height)| {
                            (prev_width.max(next_width), prev_height + next_height)
                        },
                    );
                let no_pages = xopp_file.xopp_root.pages.len() as u32;

                let mut engine = Engine::default();

                // We convert all values from the hardcoded 72 DPI of Xopp files to the preferred dpi
                engine.document.config.format.set_dpi(xopp_import_prefs.dpi);

                engine.document.x = 0.0;
                engine.document.y = 0.0;
                engine.document.width = crate::utils::convert_value_dpi(
                    doc_width,
                    xoppformat::XoppFile::DPI,
                    xopp_import_prefs.dpi,
                );
                engine.document.height = crate::utils::convert_value_dpi(
                    doc_height,
                    xoppformat::XoppFile::DPI,
                    xopp_import_prefs.dpi,
                );

                engine
                    .document
                    .config
                    .format
                    .set_width(crate::utils::convert_value_dpi(
                        doc_width,
                        xoppformat::XoppFile::DPI,
                        xopp_import_prefs.dpi,
                    ));
                engine
                    .document
                    .config
                    .format
                    .set_height(crate::utils::convert_value_dpi(
                        doc_height / (no_pages as f64),
                        xoppformat::XoppFile::DPI,
                        xopp_import_prefs.dpi,
                    ));

                if let Some(first_page) = xopp_file.xopp_root.pages.first()
                    && let xoppformat::XoppBackgroundType::Solid {
                        color: _color,
                        style: _style,
                    } = &first_page.background.bg_type
                {
                    // Xopp background styles are not compatible with Rnotes, so everything is plain for now
                    engine.document.config.background.pattern = background::PatternStyle::None;
                }

                // Offsetting as rnote has one global coordinate space
                let mut offset = na::Vector2::<f64>::zeros();

                // Map from layer key to rnote DocumentLayerId for merging layers across pages.
                // Named layers use their name as key; unnamed layers use page-level index.
                let mut layer_id_map: std::collections::HashMap<String, DocumentLayerId> =
                    std::collections::HashMap::new();
                let mut is_first_layer = true;

                for page in xopp_file.xopp_root.pages.into_iter() {
                    let mut page_unnamed_index: usize = 0;

                    for xopp_layer in page.layers.into_iter() {
                        let layer_key = xopp_layer
                            .name
                            .as_ref()
                            .filter(|n| !n.is_empty())
                            .cloned()
                            .unwrap_or_else(|| {
                                let index = page_unnamed_index;
                                page_unnamed_index += 1;
                                format!("__unnamed_{}", index)
                            });

                        let layer_id = if is_first_layer {
                            is_first_layer = false;
                            if let Some(ref name) = xopp_layer.name {
                                if !name.is_empty() {
                                    engine.store.rename_layer(0, name.clone());
                                }
                            }
                            layer_id_map.insert(layer_key, 0);
                            0
                        } else if let Some(&existing_id) = layer_id_map.get(&layer_key) {
                            existing_id
                        } else {
                            let display_name = if layer_key.starts_with("__unnamed_") {
                                None
                            } else {
                                Some(layer_key.clone())
                            };
                            let new_id = engine.store.add_layer(display_name);
                            layer_id_map.insert(layer_key, new_id);
                            new_id
                        };

                        if !engine.store.set_active_layer(layer_id) {
                            error!(
                                "failed to set active layer to {} while importing xopp, skipping layer content",
                                layer_id
                            );
                            continue;
                        }

                        for new_xoppstroke in xopp_layer.strokes.into_iter() {
                            match Stroke::from_xoppstroke(
                                new_xoppstroke,
                                offset,
                                xopp_import_prefs.dpi,
                            ) {
                                Ok((new_stroke, layer)) => {
                                    engine.store.insert_stroke(new_stroke, Some(layer));
                                }
                                Err(e) => {
                                    error!(
                                        "Creating Stroke from XoppStroke failed while loading Xopp bytes, Err: {e:?}",
                                    );
                                }
                            }
                        }

                        for new_xoppimage in xopp_layer.images.into_iter() {
                            match Stroke::from_xoppimage(
                                new_xoppimage,
                                offset,
                                xopp_import_prefs.dpi,
                            ) {
                                Ok(new_image) => {
                                    engine.store.insert_stroke(new_image, None);
                                }
                                Err(e) => {
                                    error!(
                                        "Creating Stroke from XoppImage failed while loading Xopp bytes, Err: {e:?}",
                                    );
                                }
                            }
                        }

                        for new_xopptext in xopp_layer.texts.into_iter() {
                            match Stroke::from_xopptext(
                                new_xopptext,
                                offset,
                                xopp_import_prefs.dpi,
                            ) {
                                Ok(new_text) => {
                                    engine.store.insert_stroke(new_text, None);
                                }
                                Err(e) => {
                                    error!(
                                        "Creating Stroke from XoppText failed while loading Xopp bytes, Err: {e:?}",
                                    );
                                }
                            }
                        }
                    }

                    // Only add to y offset, results in vertical pages
                    offset[1] += crate::utils::convert_value_dpi(
                        page.height,
                        xoppformat::XoppFile::DPI,
                        xopp_import_prefs.dpi,
                    );
                }

                Ok(engine.take_snapshot())
            };

            if snapshot_sender.send(result()).is_err() {
                error!(
                    "Sending result to receiver while loading Xopp bytes failed. Receiver already dropped"
                );
            }
        });

        snapshot_receiver.await?
    }
}
