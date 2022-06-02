use std::ops::Range;

use futures::channel::oneshot;
use rnote_fileformats::{rnoteformat, xoppformat, FileFormatLoader};
use serde::{Deserialize, Serialize};

use crate::document::{background, Background, Format};
use crate::pens::penholder::PenStyle;
use crate::store::{StoreSnapshot, StrokeKey};
use crate::strokes::{BitmapImage, Stroke, VectorImage};
use crate::{Document, RnoteEngine, StrokeStore, WidgetFlags};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename = "pdf_import_pages_type")]
pub enum PdfImportPagesType {
    #[serde(rename = "bitmap")]
    Bitmap,
    #[serde(rename = "vector")]
    Vector,
}

impl Default for PdfImportPagesType {
    fn default() -> Self {
        Self::Vector
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
}

impl Default for PdfImportPrefs {
    fn default() -> Self {
        Self {
            pages_type: PdfImportPagesType::default(),
            page_width_perc: 50.0,
        }
    }
}

impl RnoteEngine {
    /// opens a .rnote file. We need to split this into two methods,
    /// because we can't have it as a async function and await when the engine is wrapped in a refcell without causing panics :/
    pub fn open_from_rnote_bytes_p1(
        &mut self,
        bytes: Vec<u8>,
    ) -> anyhow::Result<oneshot::Receiver<anyhow::Result<StoreSnapshot>>> {
        let rnote_file = rnoteformat::RnotefileMaj0Min5::load_from_bytes(&bytes)?;

        self.document = serde_json::from_value(rnote_file.document)?;

        let (store_snapshot_sender, store_snapshot_receiver) =
            oneshot::channel::<anyhow::Result<StoreSnapshot>>();

        rayon::spawn(move || {
            let result = || -> anyhow::Result<StoreSnapshot> {
                Ok(serde_json::from_value(rnote_file.store_snapshot)?)
            };

            if let Err(_data) = store_snapshot_sender.send(result()) {
                log::error!("sending result to receiver in open_from_rnote_bytes() failed. Receiver already dropped.");
            }
        });

        Ok(store_snapshot_receiver)
    }

    // Part two for opening a file. imports the store snapshot.
    pub fn open_from_store_snapshot_p2(
        &mut self,
        store_snapshot: &StoreSnapshot,
    ) -> anyhow::Result<()> {
        self.store.import_snapshot(store_snapshot);

        self.update_pens_states();

        Ok(())
    }

    /// Opens a  Xournal++ .xopp file, and replaces the current state with it.
    pub fn open_from_xopp_bytes(&mut self, bytes: Vec<u8>) -> anyhow::Result<()> {
        let xopp_file = xoppformat::XoppFile::load_from_bytes(&bytes)?;

        // Extract the largest width of all pages, add together all heights
        let (doc_width, doc_height) = xopp_file
            .xopp_root
            .pages
            .iter()
            .map(|page| (page.width, page.height))
            .fold((0_f64, 0_f64), |prev, next| {
                // Max of width, sum heights
                (prev.0.max(next.0), prev.1 + next.1)
            });
        let no_pages = xopp_file.xopp_root.pages.len() as u32;

        let mut doc = Document::default();
        let mut format = Format::default();
        let mut background = Background::default();
        let mut store = StrokeStore::default();
        // We set the doc dpi to the hardcoded xournal++ dpi, so no need to convert values or coordinates anywhere
        doc.format.dpi = xoppformat::XoppFile::DPI;

        doc.x = 0.0;
        doc.y = 0.0;
        doc.width = doc_width;
        doc.height = doc_height;

        format.width = doc_width;
        format.height = doc_height / f64::from(no_pages);

        if let Some(first_page) = xopp_file.xopp_root.pages.get(0) {
            if let xoppformat::XoppBackgroundType::Solid {
                color: _color,
                style: _style,
            } = &first_page.background.bg_type
            {
                // Background styles would not align with Rnotes background patterns, so everything is plain
                background.pattern = background::PatternStyle::None;
            }
        }

        // Offsetting as rnote has one global coordinate space
        let mut offset = na::Vector2::<f64>::zeros();

        for (_page_i, page) in xopp_file.xopp_root.pages.into_iter().enumerate() {
            for layers in page.layers.into_iter() {
                // import strokes
                for new_xoppstroke in layers.strokes.into_iter() {
                    match Stroke::from_xoppstroke(new_xoppstroke, offset) {
                        Ok(new_stroke) => {
                            store.insert_stroke(new_stroke);
                        }
                        Err(e) => {
                            log::error!(
                                "from_xoppstroke() failed in open_from_xopp_bytes() with Err {}",
                                e
                            );
                        }
                    }
                }

                // import images
                for new_xoppimage in layers.images.into_iter() {
                    match Stroke::from_xoppimage(new_xoppimage, offset) {
                        Ok(new_image) => {
                            store.insert_stroke(new_image);
                        }
                        Err(e) => {
                            log::error!(
                                "from_xoppimage() failed in open_from_xopp_bytes() with Err {}",
                                e
                            );
                        }
                    }
                }
            }

            // Only add to y offset, results in vertical pages
            offset[1] += page.height;
        }

        doc.background = background;
        doc.format = format;

        // Import into engine
        self.document = doc;
        self.store.import_snapshot(&*store.take_store_snapshot());

        self.update_pens_states();

        Ok(())
    }

    //// generates a vectorimage for the bytes ( from a SVG file )
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

    //// generates a bitmapimage for the bytes ( from a bitmap image file (PNG, JPG) )
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

    //// generates strokes for each page for the bytes ( from a PDF file )
    pub fn generate_strokes_from_pdf_bytes(
        &self,
        bytes: Vec<u8>,
        pos: na::Vector2<f64>,
        page_range: Option<Range<u32>>,
    ) -> oneshot::Receiver<anyhow::Result<Vec<Stroke>>> {
        let (oneshot_sender, oneshot_receiver) = oneshot::channel::<anyhow::Result<Vec<Stroke>>>();
        let pdf_import_prefs = self.pdf_import_prefs;

        let page_width = (self.document.format.width * (pdf_import_prefs.page_width_perc / 100.0))
            .round() as i32;

        rayon::spawn(move || {
            let result = || -> anyhow::Result<Vec<Stroke>> {
                match pdf_import_prefs.pages_type {
                    PdfImportPagesType::Bitmap => {
                        let bitmapimages = BitmapImage::import_from_pdf_bytes(
                            &bytes,
                            pos,
                            Some(page_width),
                            page_range,
                        )?
                        .into_iter()
                        .map(Stroke::BitmapImage)
                        .collect::<Vec<Stroke>>();
                        Ok(bitmapimages)
                    }
                    PdfImportPagesType::Vector => {
                        let vectorimages = VectorImage::import_from_pdf_bytes(
                            &bytes,
                            pos,
                            Some(page_width),
                            page_range,
                        )?
                        .into_iter()
                        .map(Stroke::VectorImage)
                        .collect::<Vec<Stroke>>();
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
    pub fn import_generated_strokes(&mut self, strokes: Vec<Stroke>) -> WidgetFlags {
        let mut widget_flags = self.store.record();

        let all_strokes = self.store.keys_unordered();
        self.store.set_selected_keys(&all_strokes, false);

        widget_flags.merge_with_other(self.change_pen_style(PenStyle::Selector));

        let inserted = strokes
            .into_iter()
            .map(|stroke| self.store.insert_stroke(stroke))
            .collect::<Vec<StrokeKey>>();

        // after inserting the strokes, but before set the inserted strokes selected
        self.resize_to_fit_strokes();

        inserted.into_iter().for_each(|key| {
            self.store.set_selected(key, true);
        });

        self.update_pens_states();
        self.update_rendering_current_viewport();

        widget_flags.redraw = true;
        widget_flags.resize = true;
        widget_flags.indicate_changed_store = true;
        widget_flags.refresh_ui = true;

        widget_flags
    }
}
