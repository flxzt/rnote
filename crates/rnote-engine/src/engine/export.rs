// Imports
use super::{Engine, EngineConfig, StrokeContent};
use crate::fileformats::rnoteformat::RnoteFile;
use crate::fileformats::{xoppformat, FileFormatSaver};
use crate::CloneConfig;
use anyhow::Context;
use futures::channel::oneshot;
use rayon::prelude::*;
use rnote_compose::transform::Transformable;
use rnote_compose::SplitOrder;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

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
#[cfg_attr(feature = "cli", derive(clap::ValueEnum))]
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
    /// Whether the background and stroke colors should be optimized for printing.
    #[serde(rename = "optimize_printing")]
    pub optimize_printing: bool,
    /// The export format.
    #[serde(rename = "export_format")]
    pub export_format: DocExportFormat,
    /// The page order when documents with layouts that expand in horizontal and vertical directions are cut into pages.
    #[serde(rename = "page_order")]
    pub page_order: SplitOrder,
}

impl Default for DocExportPrefs {
    fn default() -> Self {
        Self {
            with_background: true,
            with_pattern: true,
            optimize_printing: false,
            export_format: DocExportFormat::default(),
            page_order: SplitOrder::default(),
        }
    }
}

impl DocExportPrefs {
    const MARGIN: f64 = 0.0;
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
#[cfg_attr(feature = "cli", derive(clap::ValueEnum))]
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
    /// Whether the background and stroke colors should be optimized for printing.
    #[serde(rename = "optimize_printing")]
    pub optimize_printing: bool,
    /// Export format
    #[serde(rename = "export_format")]
    pub export_format: DocPagesExportFormat,
    /// The page order when documents with layouts that expand in horizontal and vertical directions are cut into pages.
    #[serde(rename = "page_order")]
    pub page_order: SplitOrder,
    /// The bitmap scale-factor in relation to the actual size.
    #[serde(rename = "bitmap_scalefactor")]
    pub bitmap_scalefactor: f64,
    /// Quality when exporting as Jpeg.
    #[serde(rename = "jpg_quality")]
    pub jpeg_quality: u8,
}

impl DocPagesExportPrefs {
    const MARGIN: f64 = 0.0;
}

impl Default for DocPagesExportPrefs {
    fn default() -> Self {
        Self {
            with_background: true,
            with_pattern: true,
            optimize_printing: false,
            export_format: DocPagesExportFormat::default(),
            page_order: SplitOrder::default(),
            bitmap_scalefactor: 1.8,
            jpeg_quality: 85,
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
#[cfg_attr(feature = "cli", derive(clap::ValueEnum))]
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
    /// Whether the background and stroke colors should be optimized for printing.
    #[serde(rename = "optimize_printing")]
    pub optimize_printing: bool,
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
            optimize_printing: false,
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

impl CloneConfig for ExportPrefs {
    fn clone_config(&self) -> Self {
        *self
    }
}

impl Engine {
    /// The used image scale-factor for any strokes that are converted to bitmap images on export.
    pub const STROKE_EXPORT_IMAGE_SCALE: f64 = 1.8;

    /// Save the current document as a .rnote file.
    pub fn save_as_rnote_bytes(
        &self,
        file_name: String,
    ) -> oneshot::Receiver<anyhow::Result<Vec<u8>>> {
        let (oneshot_sender, oneshot_receiver) = oneshot::channel::<anyhow::Result<Vec<u8>>>();
        let engine_snapshot = self.take_snapshot();
        rayon::spawn(move || {
            let result = || -> anyhow::Result<Vec<u8>> {
                let rnote_file = RnoteFile {
                    engine_snapshot: ijson::to_value(&engine_snapshot)?,
                };
                rnote_file.save_as_bytes(&file_name)
            };
            if oneshot_sender.send(result()).is_err() {
                tracing::error!(
                    "Sending result to receiver failed while saving document as rnote bytes. Receiver already dropped."
                );
            }
        });
        oneshot_receiver
    }

    /// Extract the current engine configuration.
    pub fn extract_engine_config(&self) -> EngineConfig {
        EngineConfig {
            document: self.document.clone_config(),
            pens_config: self.pens_config.clone_config(),
            penholder: self.penholder.clone_config(),
            import_prefs: self.import_prefs.clone_config(),
            export_prefs: self.export_prefs.clone_config(),
            pen_sounds: self.pen_sounds(),
            optimize_epd: self.optimize_epd(),
        }
    }

    pub fn extract_document_content(&self) -> StrokeContent {
        StrokeContent::default()
            .with_strokes(
                self.store
                    .get_strokes_arc(&self.store.stroke_keys_as_rendered()),
            )
            .with_bounds(Some(
                self.bounds_w_content_extended()
                    .unwrap_or(self.document.bounds()),
            ))
            .with_background(Some(self.document.background))
    }

    pub fn extract_pages_content(&self, page_order: SplitOrder) -> Vec<StrokeContent> {
        self.pages_bounds_w_content(page_order)
            .into_iter()
            .map(|bounds| {
                StrokeContent::default()
                    .with_strokes(
                        self.store.get_strokes_arc(
                            &self
                                .store
                                .stroke_keys_as_rendered_intersecting_bounds(bounds),
                        ),
                    )
                    .with_bounds(Some(bounds))
                    .with_background(Some(self.document.background))
            })
            .collect()
    }

    pub fn extract_selection_content(&self) -> Option<StrokeContent> {
        let selection_keys = self.store.selection_keys_as_rendered();
        if selection_keys.is_empty() {
            return None;
        }
        Some(
            StrokeContent::default()
                .with_strokes(self.store.get_strokes_arc(&selection_keys))
                .with_background(Some(self.document.background)),
        )
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
        let doc_content = self.extract_document_content();

        rayon::spawn(move || {
            let result = || -> anyhow::Result<Vec<u8>> {
                let doc_svg = doc_content
                    .gen_svg(
                        doc_export_prefs.with_background,
                        doc_export_prefs.with_pattern,
                        doc_export_prefs.optimize_printing,
                        DocExportPrefs::MARGIN,
                    )?
                    .ok_or(anyhow::anyhow!("Generating doc svg failed, returned None."))?;
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

            if oneshot_sender.send(result()).is_err() {
                tracing::error!("Sending result to receiver failed while exporting document as Svg bytes. Receiver already dropped.");
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
        let pages_content = self.extract_pages_content(doc_export_prefs.page_order);
        let format_size = self.document.format.size();

        rayon::spawn(move || {
            let result = || -> anyhow::Result<Vec<u8>> {
                let target_surface =
                    cairo::PdfSurface::for_stream(format_size[0], format_size[1], Vec::<u8>::new())
                        .context("Creating Pdf target surface failed.")?;

                target_surface
                    .set_metadata(cairo::PdfMetadata::Title, title.as_str())
                    .context("Set pdf surface title metadata failed.")?;
                target_surface
                    .set_metadata(
                        cairo::PdfMetadata::CreateDate,
                        crate::utils::now_formatted_string().as_str(),
                    )
                    .context("Set pdf surface date metadata failed.")?;

                // New scope to avoid errors when flushing
                {
                    let cairo_cx = cairo::Context::new(&target_surface)
                        .context("Creating new cairo context for pdf target surface failed.")?;

                    for (i, page_content) in pages_content.into_iter().enumerate() {
                        let Some(page_bounds) = page_content.bounds() else {
                            continue;
                        };
                        cairo_cx.save()?;
                        cairo_cx.translate(-page_bounds.mins[0], -page_bounds.mins[1]);
                        page_content.draw_to_cairo(
                            &cairo_cx,
                            doc_export_prefs.with_background,
                            doc_export_prefs.with_pattern,
                            doc_export_prefs.optimize_printing,
                            DocExportPrefs::MARGIN,
                            Engine::STROKE_EXPORT_IMAGE_SCALE,
                        )?;
                        cairo_cx.show_page().map_err(|e| {
                            anyhow::anyhow!(
                                "Showing page failed while exporting page {i} as pdf, Err: {e:?}"
                            )
                        })?;
                        cairo_cx.restore()?;
                    }
                }
                let data = *target_surface
                    .finish_output_stream()
                    .map_err(|e| anyhow::anyhow!("Finishing outputstream failed, Err: {e:?}"))?
                    .downcast::<Vec<u8>>()
                    .map_err(|e| {
                        anyhow::anyhow!("Downcasting finished output stream failed, Err: {e:?}")
                    })?;

                Ok(data)
            };

            if oneshot_sender.send(result()).is_err() {
                tracing::error!("Sending result to receiver failed while exporting document as Pdf bytes. Receiver already dropped.");
            }
        });

        oneshot_receiver
    }

    /// Export the document as a Xournal++ .xopp file.
    fn export_doc_as_xopp_bytes(
        &self,
        title: String,
        doc_export_prefs_override: Option<DocExportPrefs>,
    ) -> oneshot::Receiver<Result<Vec<u8>, anyhow::Error>> {
        let (oneshot_sender, oneshot_receiver) = oneshot::channel::<anyhow::Result<Vec<u8>>>();
        let doc_export_prefs =
            doc_export_prefs_override.unwrap_or(self.export_prefs.doc_export_prefs);
        let pages_content = self.extract_pages_content(doc_export_prefs.page_order);
        let document = self.document.clone();

        rayon::spawn(move || {
            let result = || -> anyhow::Result<Vec<u8>> {
                // Only one background for all pages
                let xopp_background = xoppformat::XoppBackground {
                    name: None,
                    bg_type: xoppformat::XoppBackgroundType::Solid {
                        color: crate::utils::xoppcolor_from_color(document.background.color),
                        style: xoppformat::XoppBackgroundSolidStyle::Plain,
                    },
                };

                // xopp spec needs at least one page in vec,
                // but it is fine because pages_bounds_w_content() always produces at least one.
                let pages = pages_content
                    .into_iter()
                    .filter_map(|page_content| {
                        let page_bounds = page_content.bounds()?;
                        // Translate strokes to to page mins and convert to XoppStrokStyle
                        let xopp_strokestyles = page_content
                            .strokes
                            .into_iter()
                            .filter_map(|mut stroke| {
                                let mut stroke = Arc::make_mut(&mut stroke).clone();
                                stroke.translate(-page_bounds.mins.coords);
                                stroke.into_xopp(document.format.dpi())
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
                            document.format.dpi(),
                            xoppformat::XoppFile::DPI,
                        );

                        Some(xoppformat::XoppPage {
                            width: page_dimensions[0],
                            height: page_dimensions[1],
                            background: xopp_background.clone(),
                            layers: vec![image_layer, strokes_layer],
                        })
                    })
                    .collect::<Vec<xoppformat::XoppPage>>();

                let xopp_title = String::from(
                    "Xournal++ document - see https://github.com/xournalpp/xournalpp (exported from Rnote - see https://github.com/flxzt/rnote)"
                );

                let xopp_root = xoppformat::XoppRoot {
                    title: xopp_title,
                    fileversion: String::from("4"),
                    preview: String::from(""),
                    pages,
                };
                let xopp_file = xoppformat::XoppFile { xopp_root };

                xopp_file.save_as_bytes(&title)
            };

            if oneshot_sender.send(result()).is_err() {
                tracing::error!(
                    "Sending result to receiver failed while exporting document as xopp bytes. Receiver already dropped."
                );
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
        let pages_content = self.extract_pages_content(doc_pages_export_prefs.page_order);

        rayon::spawn(move || {
            let result = || -> anyhow::Result<Vec<Vec<u8>>> {
                pages_content
                    .into_par_iter()
                    .enumerate()
                    .map(|(i, page_content)| {
                        let page_svg = page_content
                            .gen_svg(
                                doc_pages_export_prefs.with_background,
                                doc_pages_export_prefs.with_pattern,
                                doc_pages_export_prefs.optimize_printing,
                                DocPagesExportPrefs::MARGIN,
                            )?
                            .ok_or(anyhow::anyhow!(
                                "Generating Svg for page {i} failed, returned None."
                            ))?;
                        Ok(rnote_compose::utils::add_xml_header(
                            rnote_compose::utils::wrap_svg_root(
                                page_svg.svg_data.as_str(),
                                Some(page_svg.bounds),
                                Some(page_svg.bounds),
                                false,
                            )
                            .as_str(),
                        )
                        .into_bytes())
                    })
                    .collect()
            };

            if oneshot_sender.send(result()).is_err() {
                tracing::error!(
                    "Sending result to receiver failed while exporting document pages as Svg bytes. Receiver already dropped."
                );
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
        let pages_contents = self.extract_pages_content(doc_pages_export_prefs.page_order);

        rayon::spawn(move || {
            let result = || -> Result<Vec<Vec<u8>>, anyhow::Error> {
                let image_format = match doc_pages_export_prefs.export_format {
                    DocPagesExportFormat::Svg => return Err(anyhow::anyhow!("Extracting bitmap image format from doc pages export prefs failed, not set to a bitmap format.")),
                    DocPagesExportFormat::Png => image::ImageFormat::Png,
                    DocPagesExportFormat::Jpeg => image::ImageFormat::Jpeg,
                };
                pages_contents
                    .into_par_iter()
                    .enumerate()
                    .map(|(i, page_content)| {
                        page_content
                            .gen_svg(
                                doc_pages_export_prefs.with_background,
                                doc_pages_export_prefs.with_pattern,
                                doc_pages_export_prefs.optimize_printing,
                                DocPagesExportPrefs::MARGIN,
                            )?
                            .ok_or(anyhow::anyhow!(
                                "Generating Svg for page {i} failed, returned None."
                            ))?
                            .gen_image(doc_pages_export_prefs.bitmap_scalefactor)?
                            .into_encoded_bytes(
                                image_format,
                                Some(doc_pages_export_prefs.jpeg_quality),
                            )
                    })
                    .collect()
            };
            if oneshot_sender.send(result()).is_err() {
                tracing::error!("Sending result to receiver failed while exporting document pages as bitmap bytes. Receiver already dropped.");
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
        let content = self.extract_selection_content();

        rayon::spawn(move || {
            let result = || -> Result<Option<Vec<u8>>, anyhow::Error> {
                let Some(content) = content else {
                    return Ok(None);
                };
                let Some(svg) = content.gen_svg(
                    selection_export_prefs.with_background,
                    selection_export_prefs.with_pattern,
                    selection_export_prefs.optimize_printing,
                    selection_export_prefs.margin,
                )?
                else {
                    return Ok(None);
                };

                Ok(Some(
                    rnote_compose::utils::add_xml_header(
                        rnote_compose::utils::wrap_svg_root(
                            svg.svg_data.as_str(),
                            Some(svg.bounds),
                            Some(svg.bounds),
                            false,
                        )
                        .as_str(),
                    )
                    .into_bytes(),
                ))
            };
            if oneshot_sender.send(result()).is_err() {
                tracing::error!("Sending result to receiver failed while exporting selection as Svg bytes. Receiver already dropped.");
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
        let content = self.extract_selection_content();

        rayon::spawn(move || {
            let result = || -> Result<Option<Vec<u8>>, anyhow::Error> {
                let Some(content) = content else {
                    return Ok(None);
                };
                let Some(svg) = content.gen_svg(
                    selection_export_prefs.with_background,
                    selection_export_prefs.with_pattern,
                    selection_export_prefs.optimize_printing,
                    selection_export_prefs.margin,
                )?
                else {
                    return Ok(None);
                };
                let image_format = match selection_export_prefs.export_format {
                    SelectionExportFormat::Svg => return Err(anyhow::anyhow!("Extracting bitmap image format from doc pages export prefs failed, not set to a bitmap format.")),
                    SelectionExportFormat::Png => image::ImageFormat::Png,
                    SelectionExportFormat::Jpeg => image::ImageFormat::Jpeg
                };

                Ok(Some(
                    svg.gen_image(selection_export_prefs.bitmap_scalefactor)?
                        .into_encoded_bytes(
                            image_format,
                            Some(selection_export_prefs.jpeg_quality),
                        )?,
                ))
            };
            if oneshot_sender.send(result()).is_err() {
                tracing::error!("Sending result to receiver failed while exporting selection as bitmap image bytes. Receiver already dropped");
            }
        });

        oneshot_receiver
    }
}
