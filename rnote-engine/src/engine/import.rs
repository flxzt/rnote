use std::ops::Range;
use std::path::PathBuf;
use std::time::Instant;

use futures::channel::oneshot;
use serde::{Deserialize, Serialize};

use crate::pens::Pen;
use crate::pens::PenStyle;
use crate::store::chrono_comp::StrokeLayer;
use crate::store::StrokeKey;
use crate::strokes::{BitmapImage, Stroke, VectorImage};
use crate::{RnoteEngine, WidgetFlags};

use super::{EngineConfig, EngineViewMut};

#[derive(
    Debug, Clone, Copy, Serialize, Deserialize, num_derive::FromPrimitive, num_derive::ToPrimitive,
)]
#[serde(rename = "pdf_import_pages_type")]
pub enum PdfImportPagesType {
    #[serde(rename = "bitmap")]
    Bitmap = 0,
    #[serde(rename = "vector")]
    Vector,
}

impl Default for PdfImportPagesType {
    fn default() -> Self {
        Self::Vector
    }
}

impl TryFrom<u32> for PdfImportPagesType {
    type Error = anyhow::Error;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        num_traits::FromPrimitive::from_u32(value).ok_or_else(|| {
            anyhow::anyhow!(
                "PdfImportPagesType try_from::<u32>() for value {} failed",
                value
            )
        })
    }
}

#[derive(
    Debug, Clone, Copy, Serialize, Deserialize, num_derive::FromPrimitive, num_derive::ToPrimitive,
)]
#[serde(rename = "pdf_import_page_spacing")]
pub enum PdfImportPageSpacing {
    #[serde(rename = "continuous")]
    Continuous = 0,
    #[serde(rename = "one_per_document_page")]
    OnePerDocumentPage,
}

impl Default for PdfImportPageSpacing {
    fn default() -> Self {
        Self::Continuous
    }
}

impl TryFrom<u32> for PdfImportPageSpacing {
    type Error = anyhow::Error;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        num_traits::FromPrimitive::from_u32(value).ok_or_else(|| {
            anyhow::anyhow!(
                "PdfImportPageSpacing try_from::<u32>() for value {} failed",
                value
            )
        })
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename = "pdf_import_prefs")]
pub struct PdfImportPrefs {
    /// The pdf pages type
    #[serde(rename = "pages_type")]
    pub pages_type: PdfImportPagesType,
    /// The pdf page width in percentage to the format width
    #[serde(rename = "page_width_perc")]
    pub page_width_perc: f64,
    /// The pdf page spacing
    #[serde(rename = "page_spacing")]
    pub page_spacing: PdfImportPageSpacing,
}

impl Default for PdfImportPrefs {
    fn default() -> Self {
        Self {
            pages_type: PdfImportPagesType::default(),
            page_width_perc: 50.0,
            page_spacing: PdfImportPageSpacing::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename = "xopp_import_prefs")]
pub struct XoppImportPrefs {
    /// The import DPI
    #[serde(rename = "pages_type")]
    pub dpi: f64,
}

impl Default for XoppImportPrefs {
    fn default() -> Self {
        Self { dpi: 96.0 }
    }
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[serde(default, rename = "import_prefs")]
pub struct ImportPrefs {
    #[serde(rename = "pdf_import_prefs")]
    pub pdf_import_prefs: PdfImportPrefs,
    #[serde(rename = "xopp_import_prefs")]
    pub xopp_import_prefs: XoppImportPrefs,
}

impl RnoteEngine {
    /// Imports and replace the engine config. If pen sounds should be enabled the rnote data dir must be provided
    /// NOT for opening files
    pub fn load_engine_config(
        &mut self,
        serialized_config: &str,
        data_dir: Option<PathBuf>,
    ) -> anyhow::Result<WidgetFlags> {
        let mut widget_flags = WidgetFlags::default();
        let engine_config = serde_json::from_str::<EngineConfig>(serialized_config)?;

        self.document = serde_json::from_value(engine_config.document)?;
        self.pens_config = serde_json::from_value(engine_config.pens_config)?;
        self.penholder = serde_json::from_value(engine_config.penholder)?;
        self.import_prefs = serde_json::from_value(engine_config.import_prefs)?;
        self.export_prefs = serde_json::from_value(engine_config.export_prefs)?;
        self.pen_sounds = serde_json::from_value(engine_config.pen_sounds)?;

        // Set the pen sounds to update the audioplayer
        self.set_pen_sounds(self.pen_sounds, data_dir);

        // Reinstall the pen
        widget_flags.merge(
            self.penholder
                .reinstall_pen_current_style(&mut EngineViewMut {
                    tasks_tx: self.tasks_tx.clone(),
                    pens_config: &mut self.pens_config,
                    doc: &mut self.document,
                    store: &mut self.store,
                    camera: &mut self.camera,
                    audioplayer: &mut self.audioplayer,
                }),
        );

        widget_flags.redraw = true;
        widget_flags.refresh_ui = true;

        Ok(widget_flags)
    }

    /// generates a vectorimage for the bytes ( from a SVG file )
    pub fn generate_vectorimage_from_bytes(
        &self,
        pos: na::Vector2<f64>,
        bytes: Vec<u8>,
    ) -> oneshot::Receiver<anyhow::Result<VectorImage>> {
        let (oneshot_sender, oneshot_receiver) = oneshot::channel::<anyhow::Result<VectorImage>>();

        rayon::spawn(move || {
            let result = || -> anyhow::Result<VectorImage> {
                let svg_str = String::from_utf8(bytes)?;

                VectorImage::import_from_svg_data(&svg_str, pos, None)
            };

            if let Err(_data) = oneshot_sender.send(result()) {
                log::error!("sending result to receiver in generate_vectorimage_from_bytes() failed. Receiver already dropped.");
            }
        });

        oneshot_receiver
    }

    /// generates a bitmapimage for the bytes ( from a bitmap image file (PNG, JPG) )
    pub fn generate_bitmapimage_from_bytes(
        &self,
        pos: na::Vector2<f64>,
        bytes: Vec<u8>,
    ) -> oneshot::Receiver<anyhow::Result<BitmapImage>> {
        let (oneshot_sender, oneshot_receiver) = oneshot::channel::<anyhow::Result<BitmapImage>>();

        rayon::spawn(move || {
            let result = || -> anyhow::Result<BitmapImage> {
                BitmapImage::import_from_image_bytes(&bytes, pos)
            };

            if let Err(_data) = oneshot_sender.send(result()) {
                log::error!("sending result to receiver in generate_bitmapimage_from_bytes() failed. Receiver already dropped.");
            }
        });

        oneshot_receiver
    }

    /// generates image strokes for each page for the bytes ( from a PDF file )
    #[allow(clippy::type_complexity)]
    pub fn generate_pdf_pages_from_bytes(
        &self,
        bytes: Vec<u8>,
        insert_pos: na::Vector2<f64>,
        page_range: Option<Range<u32>>,
    ) -> oneshot::Receiver<anyhow::Result<Vec<(Stroke, Option<StrokeLayer>)>>> {
        let (oneshot_sender, oneshot_receiver) =
            oneshot::channel::<anyhow::Result<Vec<(Stroke, Option<StrokeLayer>)>>>();
        let pdf_import_prefs = self.import_prefs.pdf_import_prefs;

        let format = self.document.format.clone();

        rayon::spawn(move || {
            let result = || -> anyhow::Result<Vec<(Stroke, Option<StrokeLayer>)>> {
                match pdf_import_prefs.pages_type {
                    PdfImportPagesType::Bitmap => {
                        let bitmapimages = BitmapImage::import_from_pdf_bytes(
                            &bytes,
                            pdf_import_prefs,
                            insert_pos,
                            page_range,
                            &format,
                        )?
                        .into_iter()
                        .map(|s| (Stroke::BitmapImage(s), Some(StrokeLayer::Document)))
                        .collect::<Vec<(Stroke, Option<StrokeLayer>)>>();
                        Ok(bitmapimages)
                    }
                    PdfImportPagesType::Vector => {
                        let vectorimages = VectorImage::import_from_pdf_bytes(
                            &bytes,
                            pdf_import_prefs,
                            insert_pos,
                            page_range,
                            &format,
                        )?
                        .into_iter()
                        .map(|s| (Stroke::VectorImage(s), Some(StrokeLayer::Document)))
                        .collect::<Vec<(Stroke, Option<StrokeLayer>)>>();
                        Ok(vectorimages)
                    }
                }
            };

            if let Err(_data) = oneshot_sender.send(result()) {
                log::error!("sending result to receiver in import_pdf_bytes() failed. Receiver already dropped.");
            }
        });

        oneshot_receiver
    }

    /// Imports the generated strokes into the store
    pub fn import_generated_strokes(
        &mut self,
        strokes: Vec<(Stroke, Option<StrokeLayer>)>,
    ) -> WidgetFlags {
        let mut widget_flags = self.store.record(Instant::now());

        // we need to always deselect all strokes, even tough changing the pen style deselects too, however only when the pen is actually changed.
        let all_strokes = self.store.stroke_keys_as_rendered();
        self.store.set_selected_keys(&all_strokes, false);

        widget_flags.merge(self.change_pen_style(PenStyle::Selector));

        let inserted = strokes
            .into_iter()
            .map(|(stroke, layer)| self.store.insert_stroke(stroke, layer))
            .collect::<Vec<StrokeKey>>();

        // after inserting the strokes, but before set the inserted strokes selected
        self.resize_to_fit_strokes();

        self.store.set_selected_keys(&inserted, true);

        if let Err(e) = self.update_rendering_current_viewport() {
            log::error!("failed to update rendering for current viewport while importing generated strokes, Err: {e:?}");
        }

        widget_flags.redraw = true;
        widget_flags.resize = true;
        widget_flags.indicate_changed_store = true;
        widget_flags.refresh_ui = true;

        widget_flags
    }

    /// inserts text
    pub fn insert_text(
        &mut self,
        text: String,
        pos: na::Vector2<f64>,
    ) -> anyhow::Result<WidgetFlags> {
        let mut widget_flags = self.store.record(Instant::now());

        // we need to always deselect all strokes, even tough changing the pen style deselects too, however only when the pen is actually changed.
        let all_strokes = self.store.stroke_keys_as_rendered();
        self.store.set_selected_keys(&all_strokes, false);

        widget_flags.merge(self.change_pen_style(PenStyle::Typewriter));

        if let Pen::Typewriter(typewriter) = self.penholder.current_pen_mut() {
            widget_flags.merge(typewriter.insert_text(
                text,
                Some(pos),
                &mut EngineViewMut {
                    tasks_tx: self.tasks_tx.clone(),
                    pens_config: &mut self.pens_config,
                    doc: &mut self.document,
                    store: &mut self.store,
                    camera: &mut self.camera,
                    audioplayer: &mut self.audioplayer,
                },
            ));
        }

        Ok(widget_flags)
    }
}
