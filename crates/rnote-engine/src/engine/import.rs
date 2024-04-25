// Imports
use super::{EngineConfig, EngineViewMut, StrokeContent};
use crate::document::Layout;
use crate::pens::Pen;
use crate::pens::PenStyle;
use crate::store::chrono_comp::StrokeLayer;
use crate::store::StrokeKey;
use crate::strokes::{resize::calculate_resize_ratio, resize::ImageSizeOption, Resize};
use crate::strokes::{BitmapImage, Stroke, VectorImage};
use crate::{CloneConfig, Engine, WidgetFlags};
use futures::channel::oneshot;
use rnote_compose::ext::Vector2Ext;
use rnote_compose::shapes::Shapeable;
use serde::{Deserialize, Serialize};
use std::ops::Range;
use std::path::PathBuf;
use std::time::Instant;

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

/// Pdf import preferences.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(default, rename = "pdf_import_prefs")]
pub struct PdfImportPrefs {
    /// Pdf page width in percentage to the format width.
    #[serde(rename = "page_width_perc")]
    pub page_width_perc: f64,
    /// Pdf page spacing.
    #[serde(rename = "page_spacing")]
    pub page_spacing: PdfImportPageSpacing,
    /// Pdf pages import type.
    #[serde(rename = "pages_type")]
    pub pages_type: PdfImportPagesType,
    /// The scalefactor when importing as bitmap image
    #[serde(rename = "bitmap_scalefactor")]
    pub bitmap_scalefactor: f64,
    /// Whether the imported Pdf pages have drawn borders
    #[serde(rename = "page_borders")]
    pub page_borders: bool,
    /// Whether the document layout should be adjusted to the Pdf
    #[serde(rename = "adjust_document")]
    pub adjust_document: bool,
}

impl Default for PdfImportPrefs {
    fn default() -> Self {
        Self {
            pages_type: PdfImportPagesType::default(),
            page_width_perc: 50.0,
            page_spacing: PdfImportPageSpacing::default(),
            bitmap_scalefactor: 1.8,
            page_borders: true,
            adjust_document: false,
        }
    }
}

/// Xournal++ `.xopp` file import preferences.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename = "xopp_import_prefs")]
pub struct XoppImportPrefs {
    /// Import DPI.
    #[serde(rename = "pages_type")]
    pub dpi: f64,
}

impl Default for XoppImportPrefs {
    fn default() -> Self {
        Self { dpi: 96.0 }
    }
}

/// Import preferences.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[serde(default, rename = "import_prefs")]
pub struct ImportPrefs {
    /// Pdf import preferences
    #[serde(rename = "pdf_import_prefs")]
    pub pdf_import_prefs: PdfImportPrefs,
    /// Xournal++ `.xopp` file import preferences
    #[serde(rename = "xopp_import_prefs")]
    pub xopp_import_prefs: XoppImportPrefs,
}

impl CloneConfig for ImportPrefs {
    fn clone_config(&self) -> Self {
        *self
    }
}

impl Engine {
    /// Loads the engine config
    pub fn load_engine_config(
        &mut self,
        engine_config: EngineConfig,
        data_dir: Option<PathBuf>,
    ) -> WidgetFlags {
        let mut widget_flags = WidgetFlags::default();

        self.document = engine_config.document;
        self.pens_config = engine_config.pens_config;
        self.penholder = engine_config.penholder;
        self.import_prefs = engine_config.import_prefs;
        self.export_prefs = engine_config.export_prefs;

        // Set the pen sounds to update the audioplayer
        self.set_pen_sounds(engine_config.pen_sounds, data_dir);

        self.set_optimize_epd(engine_config.optimize_epd);

        widget_flags |= self
            .penholder
            .reinstall_pen_current_style(&mut EngineViewMut {
                tasks_tx: self.tasks_tx.clone(),
                pens_config: &mut self.pens_config,
                document: &mut self.document,
                store: &mut self.store,
                camera: &mut self.camera,
                audioplayer: &mut self.audioplayer,
            });
        widget_flags |= self.doc_resize_to_fit_content();
        widget_flags.redraw = true;
        widget_flags.refresh_ui = true;
        widget_flags
    }

    /// Loads the config when syncing engine state between tabs.
    pub fn load_engine_config_sync_tab(
        &mut self,
        engine_config: EngineConfig,
        data_dir: Option<PathBuf>,
    ) -> WidgetFlags {
        let mut widget_flags = WidgetFlags::default();

        self.pens_config = engine_config.pens_config;
        self.penholder = engine_config.penholder;
        self.import_prefs = engine_config.import_prefs;
        self.export_prefs = engine_config.export_prefs;

        // Set the pen sounds to update the audioplayer
        self.set_pen_sounds(engine_config.pen_sounds, data_dir);

        self.set_optimize_epd(engine_config.optimize_epd);

        widget_flags |= self
            .penholder
            .reinstall_pen_current_style(&mut EngineViewMut {
                tasks_tx: self.tasks_tx.clone(),
                pens_config: &mut self.pens_config,
                document: &mut self.document,
                store: &mut self.store,
                camera: &mut self.camera,
                audioplayer: &mut self.audioplayer,
            });
        widget_flags |= self.doc_resize_to_fit_content();
        widget_flags.redraw = true;
        widget_flags.refresh_ui = true;
        widget_flags
    }

    /// Import and replaces the engine config.
    ///
    /// If pen sounds should be enabled the rnote data-dir must be provided.
    pub fn import_engine_config_from_json(
        &mut self,
        serialized_config: &str,
        data_dir: Option<PathBuf>,
    ) -> anyhow::Result<WidgetFlags> {
        let engine_config = serde_json::from_str::<EngineConfig>(serialized_config)?;
        Ok(self.load_engine_config(engine_config, data_dir))
    }

    /// Generate a vectorimage from the bytes.
    ///
    /// The bytes are expected to be from a valid UTF-8 encoded Svg string.
    pub fn generate_vectorimage_from_bytes(
        &self,
        pos: na::Vector2<f64>,
        bytes: Vec<u8>,
        respect_borders: bool,
    ) -> oneshot::Receiver<anyhow::Result<VectorImage>> {
        let (oneshot_sender, oneshot_receiver) = oneshot::channel::<anyhow::Result<VectorImage>>();

        let resize_struct = Resize {
            width: self.document.format.width(),
            height: self.document.format.height(),
            layout_fixed_width: self.document.layout.is_fixed_width(),
            max_viewpoint: Some(self.camera.viewport().maxs),
            restrain_to_viewport: true,
            respect_borders,
        };
        rayon::spawn(move || {
            let result = || -> anyhow::Result<VectorImage> {
                let svg_str = String::from_utf8(bytes)?;

                VectorImage::from_svg_str(
                    &svg_str,
                    pos,
                    ImageSizeOption::ResizeImage(resize_struct),
                )
            };

            if oneshot_sender.send(result()).is_err() {
                tracing::error!(
                    "Sending result to receiver while generating VectorImage from bytes failed. Receiver already dropped."
                );
            }
        });

        oneshot_receiver
    }

    /// Generate a bitmapimage for the bytes.
    ///
    /// The bytes are expected to be from a valid bitmap image (Png/Jpeg).
    pub fn generate_bitmapimage_from_bytes(
        &self,
        pos: na::Vector2<f64>,
        bytes: Vec<u8>,
        respect_borders: bool,
    ) -> oneshot::Receiver<anyhow::Result<BitmapImage>> {
        let (oneshot_sender, oneshot_receiver) = oneshot::channel::<anyhow::Result<BitmapImage>>();

        let resize_struct = Resize {
            width: self.document.format.width(),
            height: self.document.format.height(),
            layout_fixed_width: self.document.layout.is_fixed_width(),
            max_viewpoint: Some(self.camera.viewport().maxs),
            restrain_to_viewport: true,
            respect_borders,
        };
        rayon::spawn(move || {
            let result = || -> anyhow::Result<BitmapImage> {
                BitmapImage::from_image_bytes(
                    &bytes,
                    pos,
                    ImageSizeOption::ResizeImage(resize_struct),
                )
            };

            if oneshot_sender.send(result()).is_err() {
                tracing::error!(
                    "Sending result to receiver while generating BitmapImage from bytes failed. Receiver already dropped."
                );
            }
        });

        oneshot_receiver
    }

    /// Generate image strokes for each page for the bytes.
    ///
    /// The bytes are expected to be from a valid Pdf.
    ///
    /// Note: `insert_pos` does not have an effect when the `adjust_document` import pref is set true.
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
        let format = self.document.format;
        let insert_pos = if self.import_prefs.pdf_import_prefs.adjust_document {
            na::Vector2::<f64>::zeros()
        } else {
            insert_pos
        };

        rayon::spawn(move || {
            let result = || -> anyhow::Result<Vec<(Stroke, Option<StrokeLayer>)>> {
                match pdf_import_prefs.pages_type {
                    PdfImportPagesType::Bitmap => {
                        let bitmapimages = BitmapImage::from_pdf_bytes(
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
                        let vectorimages = VectorImage::from_pdf_bytes(
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

            if oneshot_sender.send(result()).is_err() {
                tracing::error!("Sending result to receiver while importing Pdf bytes failed. Receiver already dropped");
            }
        });

        oneshot_receiver
    }

    /// Import the generated strokes into the store.
    pub fn import_generated_content(
        &mut self,
        strokes: Vec<(Stroke, Option<StrokeLayer>)>,
        adjust_document: bool,
    ) -> WidgetFlags {
        let mut widget_flags = WidgetFlags::default();
        if strokes.is_empty() {
            return widget_flags;
        }
        let select = !adjust_document;

        // we need to always deselect all strokes. Even tough changing the pen style deselects too, it does only when
        // the pen is actually different.
        let all_strokes = self.store.stroke_keys_as_rendered();
        self.store.set_selected_keys(&all_strokes, false);

        if select {
            widget_flags |= self.change_pen_style(PenStyle::Selector);
        }

        if adjust_document {
            let max_size = strokes
                .iter()
                .map(|(stroke, _)| stroke.bounds().extents())
                .fold(na::Vector2::<f64>::zeros(), |acc, x| acc.maxs(&x));
            self.document.format.set_width(max_size[0]);
            self.document.format.set_height(max_size[1]);
            widget_flags |= self.set_doc_layout(Layout::FixedSize) | self.doc_resize_autoexpand()
        }

        let inserted = strokes
            .into_iter()
            .map(|(stroke, layer)| self.store.insert_stroke(stroke, layer))
            .collect::<Vec<StrokeKey>>();

        // resize after the strokes are inserted, but before they are set selected
        widget_flags |= self.doc_resize_to_fit_content();
        if select {
            self.store.set_selected_keys(&inserted, true);
        }
        widget_flags |= self.current_pen_update_state();
        widget_flags |= self.store.record(Instant::now());
        widget_flags.resize = true;
        widget_flags.store_modified = true;
        widget_flags.refresh_ui = true;

        widget_flags
    }

    /// Insert text.
    pub fn insert_text(&mut self, text: String, pos: Option<na::Vector2<f64>>) -> WidgetFlags {
        let mut widget_flags = WidgetFlags::default();

        // we need to always deselect all strokes. Even tough changing the pen style deselects too, but only when the pen is actually changed.
        let all_strokes = self.store.stroke_keys_as_rendered();
        self.store.set_selected_keys(&all_strokes, false);

        widget_flags |= self.change_pen_style(PenStyle::Typewriter);

        if let Pen::Typewriter(typewriter) = self.penholder.current_pen_mut() {
            widget_flags |= typewriter.insert_text(
                text,
                pos,
                &mut EngineViewMut {
                    tasks_tx: self.tasks_tx.clone(),
                    pens_config: &mut self.pens_config,
                    document: &mut self.document,
                    store: &mut self.store,
                    camera: &mut self.camera,
                    audioplayer: &mut self.audioplayer,
                },
            );
        }

        widget_flags |= self.store.record(Instant::now());
        widget_flags.redraw = true;
        widget_flags
    }

    /// Insert the stroke content.
    ///
    /// The data usually comes from the clipboard, drag-and-drop, ..
    pub fn insert_stroke_content(
        &mut self,
        content: StrokeContent,
        pos: na::Vector2<f64>,
        resize: ImageSizeOption,
    ) -> WidgetFlags {
        let mut widget_flags = WidgetFlags::default();

        // we need to always deselect all strokes
        // even though changing the pen style deselects too, but only when the pen is actually different.
        let all_strokes = self.store.stroke_keys_as_rendered();
        self.store.set_selected_keys(&all_strokes, false);
        widget_flags |= self.change_pen_style(PenStyle::Selector);

        // calculate ratio
        let ratio = match resize {
            ImageSizeOption::ResizeImage(resize) => {
                calculate_resize_ratio(resize, content.size().unwrap(), pos)
            }
            _ => 1.0f64,
        };
        let inserted_keys = self.store.insert_stroke_content(content, ratio, pos);

        // re generate view
        self.store.update_geometry_for_strokes(&inserted_keys);
        self.store.regenerate_rendering_in_viewport_threaded(
            self.tasks_tx.clone(),
            false,
            self.camera.viewport(),
            self.camera.image_scale(),
        );

        widget_flags |= self.penholder.current_pen_update_state(&mut EngineViewMut {
            tasks_tx: self.tasks_tx.clone(),
            pens_config: &mut self.pens_config,
            document: &mut self.document,
            store: &mut self.store,
            camera: &mut self.camera,
            audioplayer: &mut self.audioplayer,
        });

        widget_flags |= self.store.record(Instant::now());
        widget_flags.redraw = true;

        widget_flags
    }
}
