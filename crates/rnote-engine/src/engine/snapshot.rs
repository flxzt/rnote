// Imports
use crate::document::background;
use crate::engine::import::XoppImportPrefs;
use crate::fileformats::{rnoteformat, xoppformat, FileFormatLoader};
use crate::store::{ChronoComponent, StrokeKey};
use crate::strokes::Stroke;
use crate::{Camera, Document, Engine};
use anyhow::Context;
use futures::channel::oneshot;
use serde::{Deserialize, Serialize};
use slotmap::{HopSlotMap, SecondaryMap};
use std::sync::Arc;
use tracing::error;

use super::save::SavePrefs;

// An engine snapshot, used when loading/saving the current document from/into a file.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename = "engine_snapshot")]
pub struct EngineSnapshot {
    #[serde(rename = "document")]
    pub document: Document,
    #[serde(rename = "camera")]
    pub camera: Camera,
    #[serde(rename = "stroke_components")]
    pub stroke_components: Arc<HopSlotMap<StrokeKey, Arc<Stroke>>>,
    #[serde(rename = "chrono_components")]
    pub chrono_components: Arc<SecondaryMap<StrokeKey, Arc<ChronoComponent>>>,
    #[serde(rename = "chrono_counter")]
    pub chrono_counter: u32,
    // save_prefs is skipped as it is extracted and incorporated into the header when saving
    #[serde(skip, default)]
    pub save_prefs: SavePrefs,
}

impl Default for EngineSnapshot {
    fn default() -> Self {
        Self {
            document: Document::default(),
            camera: Camera::default(),
            stroke_components: Arc::new(HopSlotMap::with_key()),
            chrono_components: Arc::new(SecondaryMap::new()),
            chrono_counter: 0,
            save_prefs: SavePrefs::default(),
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
                // support for legacy files
                // gzip magic number
                if bytes
                    .get(..2)
                    .ok_or_else(|| anyhow::anyhow!("Not an Rnote file"))?
                    == [0x1f, 0x8b]
                {
                    let legacy = rnoteformat::legacy::LegacyRnoteFile::load_from_bytes(&bytes)?;
                    return Ok(ijson::from_value(&legacy.engine_snapshot)?);
                }

                let rnote_file = rnoteformat::RnoteFile::load_from_bytes(&bytes)
                    .context("Loading RnoteFile from bytes failed.")?;
                Self::try_from(rnote_file)
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
                engine.document.format.set_dpi(xopp_import_prefs.dpi);

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
                    .format
                    .set_width(crate::utils::convert_value_dpi(
                        doc_width,
                        xoppformat::XoppFile::DPI,
                        xopp_import_prefs.dpi,
                    ));
                engine
                    .document
                    .format
                    .set_height(crate::utils::convert_value_dpi(
                        doc_height / (no_pages as f64),
                        xoppformat::XoppFile::DPI,
                        xopp_import_prefs.dpi,
                    ));

                if let Some(first_page) = xopp_file.xopp_root.pages.first() {
                    if let xoppformat::XoppBackgroundType::Solid {
                        color: _color,
                        style: _style,
                    } = &first_page.background.bg_type
                    {
                        // Xopp background styles are not compatible with Rnotes, so everything is plain for now
                        engine.document.background.pattern = background::PatternStyle::None;
                    }
                }

                // Offsetting as rnote has one global coordinate space
                let mut offset = na::Vector2::<f64>::zeros();

                for page in xopp_file.xopp_root.pages.into_iter() {
                    for layers in page.layers.into_iter() {
                        // import strokes
                        for new_xoppstroke in layers.strokes.into_iter() {
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
                                        "Creating Stroke from XoppStroke failed while loading Xopp bytess, Err: {e:?}",
                                    );
                                }
                            }
                        }

                        // import images
                        for new_xoppimage in layers.images.into_iter() {
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
                error!("Sending result to receiver while loading Xopp bytes failed. Receiver already dropped");
            }
        });

        snapshot_receiver.await?
    }
}
