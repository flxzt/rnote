// Imports
use super::StrokeContent;
use crate::document::Layout;
use crate::engine_view_mut;
use crate::pens::Pen;
use crate::pens::PenStyle;
use crate::store::StrokeKey;
use crate::store::chrono_comp::StrokeLayer;
use crate::strokes::{BitmapImage, Stroke, VectorImage};
use crate::strokes::{Resize, resize::ImageSizeOption, resize::calculate_resize_ratio};
use crate::{Engine, WidgetFlags};
use futures::channel::oneshot;
use p2d::math::Vector2;
use rnote_compose::ext::Vector2Ext;
use rnote_compose::shapes::Shapeable;
use serde::{Deserialize, Serialize};
use std::ops::Range;
use std::time::Instant;
use tracing::error;

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

impl Engine {
    /// Generate a vectorimage from the bytes.
    ///
    /// The bytes are expected to be from a valid UTF-8 encoded Svg string.
    pub fn generate_vectorimage_from_bytes(
        &self,
        pos: Vector2,
        bytes: Vec<u8>,
        respect_borders: bool,
    ) -> oneshot::Receiver<anyhow::Result<VectorImage>> {
        let (oneshot_sender, oneshot_receiver) = oneshot::channel::<anyhow::Result<VectorImage>>();

        let resize_struct = Resize {
            width: self.document.config.format.width(),
            height: self.document.config.format.height(),
            layout_fixed_width: self.document.config.layout.is_fixed_width(),
            max_viewpoint: Some(self.camera.viewport().maxs),
            restrain_to_viewport: true,
            respect_borders,
        };
        rayon::spawn(move || {
            let result = || -> anyhow::Result<VectorImage> {
                let svg_str = String::from_utf8(bytes)?;

                VectorImage::from_svg_string(
                    svg_str,
                    pos,
                    ImageSizeOption::ResizeImage(resize_struct),
                )
            };

            if oneshot_sender.send(result()).is_err() {
                error!(
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
        pos: Vector2,
        bytes: Vec<u8>,
        respect_borders: bool,
    ) -> oneshot::Receiver<anyhow::Result<BitmapImage>> {
        let (oneshot_sender, oneshot_receiver) = oneshot::channel::<anyhow::Result<BitmapImage>>();

        let resize_struct = Resize {
            width: self.document.config.format.width(),
            height: self.document.config.format.height(),
            layout_fixed_width: self.document.config.layout.is_fixed_width(),
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
                error!(
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
    /// Takes ownership of the bytes: they are handed over to hayro, which keeps them alive for the
    /// whole import, so passing them by value avoids a full copy of the file.
    ///
    /// Note: `insert_pos` does not have an effect when the `adjust_document` import pref is set true.
    #[allow(clippy::type_complexity)]
    pub fn generate_pdf_pages_from_bytes(
        &self,
        bytes: Vec<u8>,
        insert_pos: Vector2,
        page_range: Option<Range<usize>>,
        password: Option<String>,
    ) -> oneshot::Receiver<anyhow::Result<Vec<(Stroke, Option<StrokeLayer>)>>> {
        let (oneshot_sender, oneshot_receiver) =
            oneshot::channel::<anyhow::Result<Vec<(Stroke, Option<StrokeLayer>)>>>();
        let pdf_import_prefs = self.config.read().import_prefs.pdf_import_prefs;
        let format = self.document.config.format;
        let insert_pos = if self
            .config
            .read()
            .import_prefs
            .pdf_import_prefs
            .adjust_document
        {
            Vector2::ZERO
        } else {
            insert_pos
        };

        rayon::spawn(move || {
            let result = || -> anyhow::Result<Vec<(Stroke, Option<StrokeLayer>)>> {
                match pdf_import_prefs.pages_type {
                    PdfImportPagesType::Bitmap => {
                        let bitmapimages = BitmapImage::from_pdf_bytes(
                            bytes,
                            pdf_import_prefs,
                            insert_pos,
                            page_range,
                            &format,
                            password,
                        )?
                        .into_iter()
                        .map(|s| (Stroke::BitmapImage(s), Some(StrokeLayer::Document)))
                        .collect::<Vec<(Stroke, Option<StrokeLayer>)>>();
                        Ok(bitmapimages)
                    }
                    PdfImportPagesType::Vector => {
                        let vectorimages = VectorImage::from_pdf_bytes(
                            bytes,
                            pdf_import_prefs,
                            insert_pos,
                            page_range,
                            &format,
                            password,
                        )?
                        .into_iter()
                        .map(|s| (Stroke::VectorImage(s), Some(StrokeLayer::Document)))
                        .collect::<Vec<(Stroke, Option<StrokeLayer>)>>();
                        Ok(vectorimages)
                    }
                }
            };

            if oneshot_sender.send(result()).is_err() {
                error!(
                    "Sending result to receiver while importing Pdf bytes failed. Receiver already dropped"
                );
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
                .fold(Vector2::ZERO, |acc, x| acc.maxs(&x));
            self.document.config.format.set_width(max_size[0]);
            self.document.config.format.set_height(max_size[1]);
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
    pub fn insert_text(&mut self, text: String, pos: Option<Vector2>) -> WidgetFlags {
        let mut widget_flags = WidgetFlags::default();

        // we need to always deselect all strokes. Even tough changing the pen style deselects too, but only when the pen is actually changed.
        let all_strokes = self.store.stroke_keys_as_rendered();
        self.store.set_selected_keys(&all_strokes, false);

        widget_flags |= self.change_pen_style(PenStyle::Typewriter);

        if let Pen::Typewriter(typewriter) = self.penholder.current_pen_mut() {
            widget_flags |= typewriter.insert_text(text, pos, &mut engine_view_mut!(self));
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
        pos: Vector2,
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

        widget_flags |= self
            .penholder
            .current_pen_update_state(&mut engine_view_mut!(self));

        widget_flags |= self.store.record(Instant::now());
        widget_flags.redraw = true;

        widget_flags
    }
}

#[cfg(test)]
mod tests {
    //! Regression tests for the Pdf import paths.
    //!
    //! The Pdf import code had no tests at all, so these pin the observable contract of the two import
    //! modes (page count, page order, rendered raster format and pixels, stored Svg) on a Pdf that is
    //! built inside the test. That keeps the test self-contained: no fixture file, no external tool.
    //!
    //! What is deliberately *not* asserted: exact pixel values or hashes of a whole page. The renderer
    //! is allowed to change its antialiasing, and a test that has to be updated for every renderer bump
    //! gets deleted instead.
    use super::*;

    use crate::document::Format;
    use crate::image::ImageMemoryFormat;

    /// Page size used by the first page of the fixtures.
    const PAGE_W: f64 = 200.0;
    const PAGE_H: f64 = 300.0;

    /// Build a minimal but valid Pdf whose pages are `(width, height)` Pdf user units and have a black
    /// rectangle on the middle 50% of the page. Uncompressed and font-free, so the bytes stay small and
    /// the rendered pixels are predictable: black in the center, white everywhere else.
    fn test_pdf(page_sizes: &[(f64, f64)]) -> Vec<u8> {
        // Object ids: 1 catalog, 2 page tree, then a content stream + page object per page.
        let content_id = |i: usize| 3 + 2 * i;
        let page_id = |i: usize| 4 + 2 * i;

        let mut objects: Vec<Vec<u8>> = vec![
            b"<< /Type /Catalog /Pages 2 0 R >>".to_vec(),
            format!(
                "<< /Type /Pages /Kids [{}] /Count {} >>",
                (0..page_sizes.len())
                    .map(|i| format!("{} 0 R", page_id(i)))
                    .collect::<Vec<_>>()
                    .join(" "),
                page_sizes.len()
            )
            .into_bytes(),
        ];

        for (i, &(w, h)) in page_sizes.iter().enumerate() {
            let content = format!(
                "0 0 0 rg {} {} {} {} re f",
                w * 0.25,
                h * 0.25,
                w * 0.5,
                h * 0.5
            );
            objects.push(
                format!(
                    "<< /Length {} >>\nstream\n{}\nendstream",
                    content.len(),
                    content
                )
                .into_bytes(),
            );
            objects.push(
            format!(
                "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {w} {h}] /Resources << >> /Contents {} 0 R >>",
                content_id(i)
            )
            .into_bytes(),
        );
        }

        let mut out: Vec<u8> = b"%PDF-1.7\n".to_vec();
        let mut offsets = Vec::with_capacity(objects.len());
        for (i, object) in objects.iter().enumerate() {
            offsets.push(out.len());
            out.extend_from_slice(format!("{} 0 obj\n", i + 1).as_bytes());
            out.extend_from_slice(object);
            out.extend_from_slice(b"\nendobj\n");
        }
        let xref_offset = out.len();
        out.extend_from_slice(format!("xref\n0 {}\n", objects.len() + 1).as_bytes());
        out.extend_from_slice(b"0000000000 65535 f \n");
        for offset in &offsets {
            out.extend_from_slice(format!("{offset:010} 00000 n \n").as_bytes());
        }
        out.extend_from_slice(
            format!(
                "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref_offset}\n%%EOF\n",
                objects.len() + 1
            )
            .as_bytes(),
        );
        out
    }

    fn three_pages() -> Vec<u8> {
        test_pdf(&[(PAGE_W, PAGE_H); 3])
    }

    fn prefs(pages_type: PdfImportPagesType) -> PdfImportPrefs {
        PdfImportPrefs {
            pages_type,
            ..Default::default()
        }
    }

    /// The premultiplied RGBA8 pixel at `(x, y)`.
    fn pixel(image: &crate::image::Image, x: u32, y: u32) -> [u8; 4] {
        let i = (y as usize * image.pixel_width as usize + x as usize) * 4;
        let data: &[u8] = &image.data;
        [data[i], data[i + 1], data[i + 2], data[i + 3]]
    }

    fn aspect_ratio(images: &[BitmapImage]) -> f64 {
        let image = &images[0].image;
        image.pixel_width as f64 / image.pixel_height as f64
    }

    #[test]
    fn bitmap_import_renders_every_page_in_order() {
        let images = BitmapImage::from_pdf_bytes(
            three_pages(),
            prefs(PdfImportPagesType::Bitmap),
            Vector2::ZERO,
            None,
            &Format::default(),
            None,
        )
        .expect("importing the test Pdf failed");

        assert_eq!(images.len(), 3, "one stroke per page");

        for image in &images {
            // The page's aspect ratio survives rendering (up to rounding to whole pixels).
            let rendered = image.image.pixel_width as f64 / image.image.pixel_height as f64;
            assert!(
                (rendered - PAGE_W / PAGE_H).abs() < 0.01,
                "page aspect ratio {rendered} does not match {}",
                PAGE_W / PAGE_H
            );
            assert!(matches!(
                image.image.memory_format,
                ImageMemoryFormat::R8g8b8a8Premultiplied
            ));
            assert_eq!(
                image.image.data.len(),
                (image.image.pixel_width * image.image.pixel_height * 4) as usize,
                "the pixel buffer must match ImageMemoryFormat"
            );

            // Pdf import renders on an opaque white background, so the buffer is fully opaque and
            // premultiplied values can be compared as plain RGB.
            let center = pixel(
                &image.image,
                image.image.pixel_width / 2,
                image.image.pixel_height / 2,
            );
            assert!(
                center == [0, 0, 0, 255],
                "the black center rectangle should render black, got {center:?}"
            );

            let corner = pixel(&image.image, 2, 2);
            assert!(
                corner == [255, 255, 255, 255],
                "the page margin should render white, got {corner:?}"
            );
        }

        // Pages are placed one after another, not on top of each other.
        let first = images[0].rectangle.bounds();
        let second = images[1].rectangle.bounds();
        assert!(
            second.mins[1] >= first.maxs[1],
            "pages overlap: {first:?} then {second:?}"
        );
    }

    #[test]
    fn bitmap_import_page_range_only_imports_the_range() {
        // Distinct page widths, so the imported page can be identified by its aspect ratio.
        let bytes = test_pdf(&[
            (PAGE_W, PAGE_H),
            (PAGE_W + 100.0, PAGE_H),
            (PAGE_W + 200.0, PAGE_H),
        ]);

        let images = BitmapImage::from_pdf_bytes(
            bytes,
            prefs(PdfImportPagesType::Bitmap),
            Vector2::ZERO,
            Some(1..2),
            &Format::default(),
            None,
        )
        .expect("importing a page range failed");

        assert_eq!(images.len(), 1, "only the selected page is imported");
        assert!(
            (aspect_ratio(&images) - (PAGE_W + 100.0) / PAGE_H).abs() < 0.01,
            "the second page was expected, got aspect ratio {}",
            aspect_ratio(&images)
        );
    }

    #[test]
    fn vector_import_keeps_a_parseable_svg_per_page() {
        let images = VectorImage::from_pdf_bytes(
            three_pages(),
            prefs(PdfImportPagesType::Vector),
            Vector2::ZERO,
            None,
            &Format::default(),
            None,
        )
        .expect("importing the test Pdf failed");

        assert_eq!(images.len(), 3, "one stroke per page");

        for image in &images {
            assert!(
                image.svg_data.contains("<svg"),
                "stored Svg data is not an Svg document"
            );
            // The stored Svg is what the renderer parses again on every draw, so it has to stay valid.
            usvg::Tree::from_str(&image.svg_data, &usvg::Options::default())
                .expect("the stored Svg data does not parse");
            assert!(
                image.intrinsic_size[0] > 0.0 && image.intrinsic_size[1] > 0.0,
                "intrinsic size was not determined"
            );
        }

        // Pages are placed one after another, not on top of each other.
        let first = images[0].rectangle.bounds();
        let second = images[1].rectangle.bounds();
        assert!(
            second.mins[1] >= first.maxs[1],
            "pages overlap: {first:?} then {second:?}"
        );
    }
}
