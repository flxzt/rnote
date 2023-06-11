// Imports
use super::{EngineConfig, EngineSnapshot, RnoteEngine};
use crate::store::StrokeKey;
use crate::strokes::Stroke;
use crate::{render, DrawBehaviour};
use anyhow::Context;
use futures::channel::oneshot;
use p2d::bounding_volume::{Aabb, BoundingVolume};
use piet::RenderContext;
use rnote_compose::helpers::Vector2Helpers;
use rnote_compose::transform::TransformBehaviour;
use rnote_fileformats::rnoteformat::RnoteFile;
use rnote_fileformats::{xoppformat, FileFormatSaver};
use serde::{Deserialize, Serialize};

/// Document export format.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Serialize,
    Deserialize,
    num_derive::FromPrimitive,
    num_derive::ToPrimitive,
)]
#[serde(rename = "doc_export_format")]
pub enum DocExportFormat {
    #[serde(rename = "svg")]
    Svg,
    #[serde(rename = "pdf")]
    Pdf,
    #[serde(rename = "xopp")]
    Xopp,
}

impl Default for DocExportFormat {
    fn default() -> Self {
        Self::Pdf
    }
}

impl TryFrom<u32> for DocExportFormat {
    type Error = anyhow::Error;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        num_traits::FromPrimitive::from_u32(value).ok_or_else(|| {
            anyhow::anyhow!(
                "DocExportFormat try_from::<u32>() for value {} failed",
                value
            )
        })
    }
}

impl DocExportFormat {
    /// File extension for the format.
    pub fn file_ext(self) -> String {
        match self {
            DocExportFormat::Svg => String::from("svg"),
            DocExportFormat::Pdf => String::from("pdf"),
            DocExportFormat::Xopp => String::from("xopp"),
        }
    }
}

/// Document export preferences.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(default, rename = "doc_export_prefs")]
pub struct DocExportPrefs {
    /// Whether the background should be exported.
    #[serde(rename = "with_background")]
    pub with_background: bool,
    /// Whether the background pattern should be exported.
    #[serde(rename = "with_pattern")]
    pub with_pattern: bool,
    /// The export format.
    #[serde(rename = "export_format")]
    pub export_format: DocExportFormat,
}

impl Default for DocExportPrefs {
    fn default() -> Self {
        Self {
            with_background: true,
            with_pattern: true,
            export_format: DocExportFormat::default(),
        }
    }
}

/// Document pages export format.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Serialize,
    Deserialize,
    num_derive::FromPrimitive,
    num_derive::ToPrimitive,
)]
#[serde(rename = "doc_pages_export_format")]
pub enum DocPagesExportFormat {
    #[serde(rename = "svg")]
    Svg,
    #[serde(rename = "png")]
    Png,
    #[serde(rename = "jpeg")]
    Jpeg,
}

impl Default for DocPagesExportFormat {
    fn default() -> Self {
        Self::Svg
    }
}

/// Document pages export preferences.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(default, rename = "doc_pages_export_prefs")]
pub struct DocPagesExportPrefs {
    /// Whether the background should be exported.
    #[serde(rename = "with_background")]
    pub with_background: bool,
    /// Whether the background pattern should be exported.
    #[serde(rename = "with_pattern")]
    pub with_pattern: bool,
    /// Export format
    #[serde(rename = "export_format")]
    pub export_format: DocPagesExportFormat,
    /// The bitmap scale-factor in relation to the actual size.
    #[serde(rename = "bitmap_scalefactor")]
    pub bitmap_scalefactor: f64,
    /// Quality when exporting as Jpeg.
    #[serde(rename = "jpg_quality")]
    pub jpeg_quality: u8,
}

impl Default for DocPagesExportPrefs {
    fn default() -> Self {
        Self {
            with_background: true,
            with_pattern: true,
            export_format: DocPagesExportFormat::default(),
            bitmap_scalefactor: 1.8,
            jpeg_quality: 85,
        }
    }
}

impl TryFrom<u32> for DocPagesExportFormat {
    type Error = anyhow::Error;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        num_traits::FromPrimitive::from_u32(value).ok_or_else(|| {
            anyhow::anyhow!(
                "DocPagesExportFormat try_from::<u32>() for value {} failed",
                value
            )
        })
    }
}

impl DocPagesExportFormat {
    pub fn file_ext(self) -> String {
        match self {
            Self::Svg => String::from("svg"),
            Self::Png => String::from("png"),
            Self::Jpeg => String::from("jpg"),
        }
    }
}

/// Selection export format.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Serialize,
    Deserialize,
    num_derive::FromPrimitive,
    num_derive::ToPrimitive,
)]
#[serde(rename = "selection_export_format")]
pub enum SelectionExportFormat {
    #[serde(rename = "svg")]
    Svg,
    #[serde(rename = "png")]
    Png,
    #[serde(rename = "jpeg")]
    Jpeg,
}

impl Default for SelectionExportFormat {
    fn default() -> Self {
        Self::Svg
    }
}

impl SelectionExportFormat {
    pub fn file_ext(self) -> String {
        match self {
            SelectionExportFormat::Svg => String::from("svg"),
            SelectionExportFormat::Png => String::from("png"),
            SelectionExportFormat::Jpeg => String::from("jpg"),
        }
    }
}

impl TryFrom<u32> for SelectionExportFormat {
    type Error = anyhow::Error;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        num_traits::FromPrimitive::from_u32(value).ok_or_else(|| {
            anyhow::anyhow!(
                "SelectionExportFormat try_from::<u32>() for value {} failed",
                value
            )
        })
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(default, rename = "selection_export_prefs")]
pub struct SelectionExportPrefs {
    /// Whether the background should be exported.
    #[serde(rename = "with_background")]
    pub with_background: bool,
    /// Whether the background pattern should be exported.
    #[serde(rename = "with_pattern")]
    pub with_pattern: bool,
    /// Export format.
    #[serde(rename = "export_format")]
    pub export_format: SelectionExportFormat,
    /// The bitmap scale-factor in relation to the actual size.
    #[serde(rename = "bitmap_scalefactor")]
    pub bitmap_scalefactor: f64,
    /// Quality when exporting as Jpeg.
    #[serde(rename = "jpg_quality")]
    pub jpeg_quality: u8,
    /// The margins of the export extending the bounds of the selection.
    #[serde(rename = "margin")]
    pub margin: f64,
}

impl Default for SelectionExportPrefs {
    fn default() -> Self {
        Self {
            with_background: true,
            with_pattern: false,
            export_format: SelectionExportFormat::Svg,
            bitmap_scalefactor: 1.8,
            jpeg_quality: 85,
            margin: 12.0,
        }
    }
}

/// Export preferences.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[serde(default, rename = "export_prefs")]
pub struct ExportPrefs {
    /// Document export preferences.
    #[serde(rename = "doc_export_prefs")]
    pub doc_export_prefs: DocExportPrefs,
    //// Document pages export preferences.
    #[serde(rename = "doc_pages_export_prefs")]
    pub doc_pages_export_prefs: DocPagesExportPrefs,
    /// Selection export preferences.
    #[serde(rename = "selection_export_prefs")]
    pub selection_export_prefs: SelectionExportPrefs,
}

impl RnoteEngine {
    /// The used image scale-factor for any strokes that are converted to bitmap images on export.
    pub const STROKE_EXPORT_IMAGE_SCALE: f64 = 1.8;

    /// Save the current document as a .rnote file.
    pub fn save_as_rnote_bytes(
        &self,
        file_name: String,
    ) -> anyhow::Result<oneshot::Receiver<anyhow::Result<Vec<u8>>>> {
        let (oneshot_sender, oneshot_receiver) = oneshot::channel::<anyhow::Result<Vec<u8>>>();

        let engine_snapshot = self.take_snapshot();

        rayon::spawn(move || {
            let result = || -> anyhow::Result<Vec<u8>> {
                let rnote_file = RnoteFile {
                    engine_snapshot: serde_json::to_value(engine_snapshot)?,
                };

                rnote_file.save_as_bytes(&file_name)
            };

            if let Err(_data) = oneshot_sender.send(result()) {
                log::error!("sending result to receiver in save_as_rnote_bytes() failed. Receiver already dropped");
            }
        });

        Ok(oneshot_receiver)
    }

    /// Extract the current engine configuration.
    pub fn extract_engine_config(&self) -> EngineConfig {
        EngineConfig {
            document: self.document,
            pens_config: self.pens_config.clone(),
            penholder: self.penholder.clone_config(),
            import_prefs: self.import_prefs,
            export_prefs: self.export_prefs,
            pen_sounds: self.pen_sounds(),
        }
    }

    /// Export the current engine config as Json string.
    pub fn export_engine_config_as_json(&self) -> anyhow::Result<String> {
        Ok(serde_json::to_string(&self.extract_engine_config())?)
    }

    /// Export the entire engine state as Json string.
    ///
    /// Only intended to be used for debugging.
    pub fn export_state_as_json(&self) -> anyhow::Result<String> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    /// Export the document.
    pub fn export_doc(
        &self,
        title: String,
        doc_export_prefs_override: Option<DocExportPrefs>,
    ) -> oneshot::Receiver<Result<Vec<u8>, anyhow::Error>> {
        let doc_export_prefs =
            doc_export_prefs_override.unwrap_or(self.export_prefs.doc_export_prefs);

        match doc_export_prefs.export_format {
            DocExportFormat::Svg => self.export_doc_as_svg_bytes(doc_export_prefs_override),
            DocExportFormat::Pdf => self.export_doc_as_pdf_bytes(title, doc_export_prefs_override),
            DocExportFormat::Xopp => {
                self.export_doc_as_xopp_bytes(title, doc_export_prefs_override)
            }
        }
    }

    /// Export the doc with the strokes as Svg.
    fn export_doc_as_svg_bytes(
        &self,
        doc_export_prefs_override: Option<DocExportPrefs>,
    ) -> oneshot::Receiver<Result<Vec<u8>, anyhow::Error>> {
        let (oneshot_sender, oneshot_receiver) = oneshot::channel::<anyhow::Result<Vec<u8>>>();

        let doc_export_prefs =
            doc_export_prefs_override.unwrap_or(self.export_prefs.doc_export_prefs);
        let stroke_keys = self.store.stroke_keys_as_rendered();
        let snapshot = self.take_snapshot();
        let content_bounds = self
            .bounds_w_content_extended()
            .unwrap_or_else(|| snapshot.document.bounds());

        rayon::spawn(move || {
            let result = || -> anyhow::Result<Vec<u8>> {
                let doc_svg =
                    gen_doc_svg(content_bounds, stroke_keys, &snapshot, doc_export_prefs)?;
                Ok(rnote_compose::utils::add_xml_header(
                    rnote_compose::utils::wrap_svg_root(
                        doc_svg.svg_data.as_str(),
                        Some(doc_svg.bounds),
                        Some(doc_svg.bounds),
                        false,
                    )
                    .as_str(),
                )
                .into_bytes())
            };

            if let Err(_data) = oneshot_sender.send(result()) {
                log::error!("sending result to receiver in export_doc_as_svg_bytes() failed. Receiver already dropped");
            }
        });

        oneshot_receiver
    }

    /// Export the doc with the strokes as Pdf.
    fn export_doc_as_pdf_bytes(
        &self,
        title: String,
        doc_export_prefs_override: Option<DocExportPrefs>,
    ) -> oneshot::Receiver<anyhow::Result<Vec<u8>>> {
        let (oneshot_sender, oneshot_receiver) = oneshot::channel::<anyhow::Result<Vec<u8>>>();

        let doc_export_prefs =
            doc_export_prefs_override.unwrap_or(self.export_prefs.doc_export_prefs);
        let snapshot = self.take_snapshot();

        let pages_strokes = self
            .pages_bounds_w_content()
            .into_iter()
            .map(|page_bounds| {
                let strokes_in_viewport = self
                    .store
                    .stroke_keys_as_rendered_intersecting_bounds(page_bounds);

                (page_bounds, strokes_in_viewport)
            })
            .collect::<Vec<(Aabb, Vec<StrokeKey>)>>();

        rayon::spawn(move || {
            let result = || -> anyhow::Result<Vec<u8>> {
                let format_size = na::vector![
                    snapshot.document.format.width,
                    snapshot.document.format.height
                ];

                let background_svg = if doc_export_prefs.with_background {
                    Some(
                        snapshot
                            .document
                            .background
                            .gen_svg(snapshot.document.bounds(), doc_export_prefs.with_pattern)?,
                    )
                } else {
                    None
                };

                let surface =
                    cairo::PdfSurface::for_stream(format_size[0], format_size[1], Vec::<u8>::new())
                        .context("pdfsurface creation failed")?;

                surface
                    .set_metadata(cairo::PdfMetadata::Title, title.as_str())
                    .context("set pdf surface title metadata failed")?;
                surface
                    .set_metadata(
                        cairo::PdfMetadata::CreateDate,
                        crate::utils::now_formatted_string().as_str(),
                    )
                    .context("set pdf surface date metadata failed")?;

                // New scope to avoid errors when flushing
                {
                    let cairo_cx =
                        cairo::Context::new(&surface).context("cario cx new() failed")?;

                    for (i, (page_bounds, page_strokes)) in pages_strokes.into_iter().enumerate() {
                        // We can't render the background svg with piet, so we have to do it with cairo.
                        cairo_cx.save()?;

                        cairo_cx.translate(-page_bounds.mins[0], -page_bounds.mins[1]);

                        if let Some(background_svg) = background_svg.clone() {
                            render::Svg::draw_svgs_to_cairo_context(&[background_svg], &cairo_cx)?;
                        }
                        cairo_cx.restore()?;

                        // Draw the strokes with piet.
                        let mut piet_cx = piet_cairo::CairoRenderContext::new(&cairo_cx);
                        piet_cx.save().map_err(|e| anyhow::anyhow!("{e:?}"))?;

                        piet_cx.transform(kurbo::Affine::translate(
                            -page_bounds.mins.coords.to_kurbo_vec(),
                        ));

                        for stroke in page_strokes.into_iter() {
                            if let Some(stroke) = snapshot.stroke_components.get(stroke) {
                                stroke
                                    .draw(&mut piet_cx, RnoteEngine::STROKE_EXPORT_IMAGE_SCALE)?;
                            }
                        }

                        cairo_cx.show_page().map_err(|e| {
                            anyhow::anyhow!(
                                "show_page() failed when exporting page {i} as pdf, Err: {e:?}"
                            )
                        })?;

                        piet_cx.restore().map_err(|e| anyhow::anyhow!("{e:?}"))?;
                    }
                }
                let data = *surface
                    .finish_output_stream()
                    .map_err(|e| {
                        anyhow::anyhow!(
                            "finish_outputstream() failed in export_doc_as_pdf_bytes with Err: {e:?}"
                        )
                    })?
                    .downcast::<Vec<u8>>()
                    .map_err(|e| {
                        anyhow::anyhow!(
                            "downcast() finished output stream failed in export_doc_as_pdf_bytes with Err: {e:?}"
                        )
                    })?;

                Ok(data)
            };

            if let Err(_data) = oneshot_sender.send(result()) {
                log::error!("sending result to receiver in export_doc_as_pdf_bytes() failed. Receiver already dropped");
            }
        });

        oneshot_receiver
    }

    /// Export the document as a Xournal++ .xopp file.
    fn export_doc_as_xopp_bytes(
        &self,
        title: String,
        _doc_export_prefs_override: Option<DocExportPrefs>,
    ) -> oneshot::Receiver<Result<Vec<u8>, anyhow::Error>> {
        let (oneshot_sender, oneshot_receiver) = oneshot::channel::<anyhow::Result<Vec<u8>>>();

        let snapshot = self.take_snapshot();

        let pages_strokes: Vec<(Aabb, Vec<Stroke>)> = self
            .pages_bounds_w_content()
            .into_iter()
            .map(|page_bounds| {
                let page_keys = self
                    .store
                    .stroke_keys_as_rendered_intersecting_bounds(page_bounds);

                let strokes = self.store.clone_strokes(&page_keys);

                (page_bounds, strokes)
            })
            .collect();

        rayon::spawn(move || {
            let result = || -> anyhow::Result<Vec<u8>> {
                // Only one background for all pages
                let xopp_background = xoppformat::XoppBackground {
                    name: None,
                    bg_type: xoppformat::XoppBackgroundType::Solid {
                        color: crate::utils::xoppcolor_from_color(
                            snapshot.document.background.color,
                        ),
                        style: xoppformat::XoppBackgroundSolidStyle::Plain,
                    },
                };

                // xopp spec needs at least one page in vec,
                // but it is fine because pages_bounds_w_content() always produces at least one.
                let pages = pages_strokes
                    .into_iter()
                    .map(|(page_bounds, strokes)| {
                        // Translate strokes to to page mins and convert to XoppStrokStyle
                        let xopp_strokestyles = strokes
                            .into_iter()
                            .filter_map(|mut stroke| {
                                stroke.translate(-page_bounds.mins.coords);

                                stroke.into_xopp(snapshot.document.format.dpi)
                            })
                            .collect::<Vec<xoppformat::XoppStrokeType>>();

                        // Extract the strokes
                        let xopp_strokes = xopp_strokestyles
                            .iter()
                            .filter_map(|stroke| {
                                if let xoppformat::XoppStrokeType::XoppStroke(xoppstroke) = stroke {
                                    Some(xoppstroke.clone())
                                } else {
                                    None
                                }
                            })
                            .collect::<Vec<xoppformat::XoppStroke>>();

                        // Extract the texts
                        let xopp_texts = xopp_strokestyles
                            .iter()
                            .filter_map(|stroke| {
                                if let xoppformat::XoppStrokeType::XoppText(xopptext) = stroke {
                                    Some(xopptext.clone())
                                } else {
                                    None
                                }
                            })
                            .collect::<Vec<xoppformat::XoppText>>();

                        // Extract the images
                        let xopp_images = xopp_strokestyles
                            .iter()
                            .filter_map(|stroke| {
                                if let xoppformat::XoppStrokeType::XoppImage(xoppstroke) = stroke {
                                    Some(xoppstroke.clone())
                                } else {
                                    None
                                }
                            })
                            .collect::<Vec<xoppformat::XoppImage>>();

                        // In Rnote images are always rendered below strokes and text.
                        // To match this behaviour accurately, images are separated into another layer.
                        let image_layer = xoppformat::XoppLayer {
                            name: None,
                            strokes: vec![],
                            texts: vec![],
                            images: xopp_images,
                        };

                        let strokes_layer = xoppformat::XoppLayer {
                            name: None,
                            strokes: xopp_strokes,
                            texts: xopp_texts,
                            images: vec![],
                        };

                        let page_dimensions = crate::utils::convert_coord_dpi(
                            page_bounds.extents(),
                            snapshot.document.format.dpi,
                            xoppformat::XoppFile::DPI,
                        );

                        xoppformat::XoppPage {
                            width: page_dimensions[0],
                            height: page_dimensions[1],
                            background: xopp_background.clone(),
                            layers: vec![image_layer, strokes_layer],
                        }
                    })
                    .collect::<Vec<xoppformat::XoppPage>>();

                let xopp_title = String::from("Xournal++ document - see https://github.com/xournalpp/xournalpp (exported from Rnote - see https://github.com/flxzt/rnote)");

                let xopp_root = xoppformat::XoppRoot {
                    title: xopp_title,
                    fileversion: String::from("4"),
                    preview: String::from(""),
                    pages,
                };
                let xopp_file = xoppformat::XoppFile { xopp_root };

                xopp_file.save_as_bytes(&title)
            };

            if let Err(_data) = oneshot_sender.send(result()) {
                log::error!("sending result to receiver in export_doc_as_xopp_bytes() failed. Receiver already dropped");
            }
        });

        oneshot_receiver
    }

    /// Export the document pages.
    pub fn export_doc_pages(
        &self,
        doc_pages_export_prefs_override: Option<DocPagesExportPrefs>,
    ) -> oneshot::Receiver<Result<Vec<Vec<u8>>, anyhow::Error>> {
        let doc_pages_export_prefs =
            doc_pages_export_prefs_override.unwrap_or(self.export_prefs.doc_pages_export_prefs);

        match doc_pages_export_prefs.export_format {
            DocPagesExportFormat::Svg => {
                self.export_doc_pages_as_svgs_bytes(doc_pages_export_prefs_override)
            }
            DocPagesExportFormat::Png | DocPagesExportFormat::Jpeg => {
                self.export_doc_pages_as_bitmap_bytes(doc_pages_export_prefs_override)
            }
        }
    }

    /// Export the document as Svg.
    fn export_doc_pages_as_svgs_bytes(
        &self,
        doc_pages_export_prefs_override: Option<DocPagesExportPrefs>,
    ) -> oneshot::Receiver<Result<Vec<Vec<u8>>, anyhow::Error>> {
        let (oneshot_sender, oneshot_receiver) = oneshot::channel::<anyhow::Result<Vec<Vec<u8>>>>();

        let doc_pages_export_prefs =
            doc_pages_export_prefs_override.unwrap_or(self.export_prefs.doc_pages_export_prefs);

        let snapshot = self.take_snapshot();

        let pages_strokes: Vec<(Aabb, Vec<StrokeKey>)> = self
            .pages_bounds_w_content()
            .into_iter()
            .map(|page_bounds| {
                let page_strokes = self
                    .store
                    .stroke_keys_as_rendered_intersecting_bounds(page_bounds);

                (page_bounds, page_strokes)
            })
            .collect();

        rayon::spawn(move || {
            let result = || -> anyhow::Result<Vec<Vec<u8>>> {
                let doc_pages_svgs: Vec<Vec<u8>> =
                    gen_doc_pages_svgs(pages_strokes, &snapshot, doc_pages_export_prefs)?
                        .into_iter()
                        .map(|page_svg| {
                            rnote_compose::utils::add_xml_header(
                                rnote_compose::utils::wrap_svg_root(
                                    page_svg.svg_data.as_str(),
                                    Some(page_svg.bounds),
                                    Some(page_svg.bounds),
                                    false,
                                )
                                .as_str(),
                            )
                            .into_bytes()
                        })
                        .collect();

                Ok(doc_pages_svgs)
            };

            if let Err(_data) = oneshot_sender.send(result()) {
                log::error!("sending result to receiver in export_doc_pages_as_svgs_bytes() failed. Receiver already dropped");
            }
        });

        oneshot_receiver
    }

    /// Export the document pages as bitmap.
    ///
    /// Returns an error if the format pref is not set to a bitmap variant.
    fn export_doc_pages_as_bitmap_bytes(
        &self,
        doc_pages_export_prefs_override: Option<DocPagesExportPrefs>,
    ) -> oneshot::Receiver<Result<Vec<Vec<u8>>, anyhow::Error>> {
        let (oneshot_sender, oneshot_receiver) = oneshot::channel::<anyhow::Result<Vec<Vec<u8>>>>();

        let doc_pages_export_prefs =
            doc_pages_export_prefs_override.unwrap_or(self.export_prefs.doc_pages_export_prefs);

        let snapshot = self.take_snapshot();

        let pages_strokes: Vec<(Aabb, Vec<StrokeKey>)> = self
            .pages_bounds_w_content()
            .into_iter()
            .map(|page_bounds| {
                let page_strokes = self
                    .store
                    .stroke_keys_as_rendered_intersecting_bounds(page_bounds);

                (page_bounds, page_strokes)
            })
            .collect();

        rayon::spawn(move || {
            let result = || -> Result<Vec<Vec<u8>>, anyhow::Error> {
                let bitmapimage_format = match doc_pages_export_prefs.export_format {
                    DocPagesExportFormat::Svg => return Err(anyhow::anyhow!("extracting bitmap image format from doc pages export prefs failed, not set to a bitmap format")),
                    DocPagesExportFormat::Png => image::ImageOutputFormat::Png,
                    DocPagesExportFormat::Jpeg => {
                        image::ImageOutputFormat::Jpeg(doc_pages_export_prefs.jpeg_quality)
                    }
                };

                gen_doc_pages_svgs(pages_strokes, &snapshot, doc_pages_export_prefs)?
                    .into_iter()
                    .map(|page_svg| {
                        let page_svg_bounds = page_svg.bounds;

                        render::Image::gen_image_from_svg(
                            page_svg,
                            page_svg_bounds,
                            doc_pages_export_prefs.bitmap_scalefactor,
                        )?
                        .into_encoded_bytes(bitmapimage_format.clone())
                    })
                    .collect()
            };
            if let Err(_data) = oneshot_sender.send(result()) {
                log::error!("sending result to receiver in export_doc_pages_as_bitmap_bytes() failed. Receiver already dropped");
            }
        });

        oneshot_receiver
    }

    /// Exports the current selection.
    pub fn export_selection(
        &self,
        selection_export_prefs_override: Option<SelectionExportPrefs>,
    ) -> oneshot::Receiver<Result<Option<Vec<u8>>, anyhow::Error>> {
        let selection_export_prefs =
            selection_export_prefs_override.unwrap_or(self.export_prefs.selection_export_prefs);

        match selection_export_prefs.export_format {
            SelectionExportFormat::Svg => {
                self.export_selection_as_svg_bytes(selection_export_prefs_override)
            }
            SelectionExportFormat::Png | SelectionExportFormat::Jpeg => {
                self.export_selection_as_bitmap_bytes(selection_export_prefs_override)
            }
        }
    }

    /// Exports the selection as Svg.
    fn export_selection_as_svg_bytes(
        &self,
        selection_export_prefs_override: Option<SelectionExportPrefs>,
    ) -> oneshot::Receiver<Result<Option<Vec<u8>>, anyhow::Error>> {
        let (oneshot_sender, oneshot_receiver) =
            oneshot::channel::<anyhow::Result<Option<Vec<u8>>>>();

        let selection_export_prefs =
            selection_export_prefs_override.unwrap_or(self.export_prefs.selection_export_prefs);
        let snapshot = self.take_snapshot();
        let selection_keys = self.store.selection_keys_as_rendered();
        let selection_bounds = self
            .store
            .bounds_for_strokes(&selection_keys)
            .map(|b| b.loosened(selection_export_prefs.margin));

        rayon::spawn(move || {
            let result = || -> Result<Option<Vec<u8>>, anyhow::Error> {
                let Some(selection_bounds) = selection_bounds else {
                    return Ok(None);
                };

                let Some(selection_svg) = gen_selection_svg(selection_keys, selection_bounds, &snapshot, selection_export_prefs)? else {
                    return Ok(None);
                };

                Ok(Some(
                    rnote_compose::utils::add_xml_header(
                        rnote_compose::utils::wrap_svg_root(
                            selection_svg.svg_data.as_str(),
                            Some(selection_svg.bounds),
                            Some(selection_svg.bounds),
                            false,
                        )
                        .as_str(),
                    )
                    .into_bytes(),
                ))
            };
            if let Err(_data) = oneshot_sender.send(result()) {
                log::error!("sending result to receiver in export_selection_as_svgs_bytes() failed. Receiver already dropped");
            }
        });

        oneshot_receiver
    }

    /// Export the selection a bitmap bytes.
    ///
    /// Returns an error if the format pref is not set to a bitmap format
    fn export_selection_as_bitmap_bytes(
        &self,
        selection_export_prefs_override: Option<SelectionExportPrefs>,
    ) -> oneshot::Receiver<Result<Option<Vec<u8>>, anyhow::Error>> {
        let (oneshot_sender, oneshot_receiver) =
            oneshot::channel::<anyhow::Result<Option<Vec<u8>>>>();

        let selection_export_prefs =
            selection_export_prefs_override.unwrap_or(self.export_prefs.selection_export_prefs);
        let snapshot = self.take_snapshot();
        let selection_keys = self.store.selection_keys_as_rendered();
        let selection_bounds = self
            .store
            .bounds_for_strokes(&selection_keys)
            .map(|b| b.loosened(selection_export_prefs.margin));

        rayon::spawn(move || {
            let result = || -> Result<Option<Vec<u8>>, anyhow::Error> {
                let Some(selection_bounds) = selection_bounds else {
                    return Ok(None);
                };

                let Some(selection_svg) = gen_selection_svg(selection_keys, selection_bounds, &snapshot, selection_export_prefs)? else {
                return Ok(None);
            };

                let selection_svg_bounds = selection_svg.bounds;

                let bitmapimage_format = match selection_export_prefs.export_format {
                SelectionExportFormat::Svg => return Err(anyhow::anyhow!("export_selection_as_bitmap_bytes() failed, export preferences have Svg as export format")),
                SelectionExportFormat::Png => image::ImageOutputFormat::Png,
                SelectionExportFormat::Jpeg => {
                    image::ImageOutputFormat::Jpeg(selection_export_prefs.jpeg_quality)
                }
            };

                Ok(Some(
                    render::Image::gen_image_from_svg(
                        selection_svg,
                        selection_svg_bounds,
                        selection_export_prefs.bitmap_scalefactor,
                    )?
                    .into_encoded_bytes(bitmapimage_format)?,
                ))
            };
            if let Err(_data) = oneshot_sender.send(result()) {
                log::error!("sending result to receiver in export_selection_as_bitmap_bytes() failed. Receiver already dropped");
            }
        });

        oneshot_receiver
    }
}

/// Generates the doc Svg.
///
/// Without root or Xml header.
fn gen_doc_svg(
    doc_w_content_bounds: Aabb,
    stroke_keys: Vec<StrokeKey>,
    snapshot: &EngineSnapshot,
    doc_export_prefs: DocExportPrefs,
) -> Result<render::Svg, anyhow::Error> {
    let mut doc_svg = if doc_export_prefs.with_background {
        snapshot
            .document
            .background
            .gen_svg(doc_w_content_bounds, doc_export_prefs.with_pattern)?
    } else {
        render::Svg {
            svg_data: String::new(),
            bounds: doc_w_content_bounds,
        }
    };

    doc_svg.merge([render::Svg::gen_with_piet_cairo_backend(
        |piet_cx| {
            for key in stroke_keys {
                if let Some(stroke) = snapshot.stroke_components.get(key) {
                    stroke.draw(piet_cx, RnoteEngine::STROKE_EXPORT_IMAGE_SCALE)?;
                }
            }

            Ok(())
        },
        doc_w_content_bounds,
    )?]);

    // The simplification also moves the bounds to mins: [0.0, 0.0], maxs: extents
    if let Err(e) = doc_svg.simplify() {
        log::warn!("simplifying svg in gen_doc_svg() failed, Err: {e:?}");
    };

    Ok(doc_svg)
}

/// Generates the doc pages Svg's.
///
/// Without root or Xml header.
fn gen_doc_pages_svgs(
    pages_strokes: Vec<(Aabb, Vec<StrokeKey>)>,
    snapshot: &EngineSnapshot,
    doc_pages_export_prefs: DocPagesExportPrefs,
) -> Result<Vec<render::Svg>, anyhow::Error> {
    let mut pages_svgs = vec![];

    for (page_bounds, strokes) in pages_strokes {
        let mut page_svg = if doc_pages_export_prefs.with_background {
            snapshot
                .document
                .background
                .gen_svg(page_bounds, doc_pages_export_prefs.with_pattern)?
        } else {
            render::Svg {
                svg_data: String::new(),
                bounds: page_bounds,
            }
        };

        page_svg.merge([render::Svg::gen_with_piet_cairo_backend(
            |piet_cx| {
                for key in strokes {
                    if let Some(stroke) = snapshot.stroke_components.get(key) {
                        stroke.draw(piet_cx, RnoteEngine::STROKE_EXPORT_IMAGE_SCALE)?;
                    }
                }
                Ok(())
            },
            page_bounds,
        )?]);

        // The simplification also moves the bounds to mins: [0.0, 0.0], maxs: extents
        if let Err(e) = page_svg.simplify() {
            log::warn!("simplifying svg in gen_doc_pages_svgs() failed, Err: {e:?}");
        };

        pages_svgs.push(page_svg);
    }

    Ok(pages_svgs)
}

/// Generates the selection Svg.
///
/// Without root or Xml header.
fn gen_selection_svg(
    selection_keys: Vec<StrokeKey>,
    selection_bounds: Aabb,
    snapshot: &EngineSnapshot,
    selection_export_prefs: SelectionExportPrefs,
) -> Result<Option<render::Svg>, anyhow::Error> {
    if selection_keys.is_empty() {
        return Ok(None);
    }

    let mut selection_svg = if selection_export_prefs.with_background {
        snapshot
            .document
            .background
            .gen_svg(selection_bounds, selection_export_prefs.with_pattern)?
    } else {
        render::Svg {
            svg_data: String::new(),
            bounds: selection_bounds,
        }
    };

    selection_svg.merge([render::Svg::gen_with_piet_cairo_backend(
        |piet_cx| {
            for key in selection_keys {
                if let Some(stroke) = snapshot.stroke_components.get(key) {
                    stroke.draw(piet_cx, RnoteEngine::STROKE_EXPORT_IMAGE_SCALE)?;
                }
            }
            Ok(())
        },
        selection_bounds,
    )?]);

    // The simplification also moves the bounds to mins: [0.0, 0.0], maxs: extents
    if let Err(e) = selection_svg.simplify() {
        log::warn!("simplifying svg in gen_selection_svg() failed, Err: {e:?}");
    };

    Ok(Some(selection_svg))
}
