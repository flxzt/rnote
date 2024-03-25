// Imports
use super::resize::ImageSizeOption;
use super::{Content, Stroke};
use crate::document::Format;
use crate::engine::import::{PdfImportPageSpacing, PdfImportPrefs};
use crate::render;
use crate::strokes::resize::calculate_resize_ratio;
use crate::Drawable;
use anyhow::Context;
use kurbo::Shape;
use p2d::bounding_volume::Aabb;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use rnote_compose::color;
use rnote_compose::ext::{AabbExt, Affine2Ext};
use rnote_compose::shapes::Rectangle;
use rnote_compose::shapes::Shapeable;
use rnote_compose::transform::Transform;
use rnote_compose::transform::Transformable;
use serde::{Deserialize, Serialize};
use std::ops::Range;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename = "bitmapimage")]
pub struct BitmapImage {
    /// The bitmap image.
    ///
    /// The bounds field of the image should not be used to determine the stroke bounds.
    /// Use rectangle.bounds() instead.
    #[serde(rename = "image")]
    pub image: render::Image,
    #[serde(rename = "rectangle")]
    pub rectangle: Rectangle,
}

impl Default for BitmapImage {
    fn default() -> Self {
        Self {
            image: render::Image::default(),
            rectangle: Rectangle::default(),
        }
    }
}

impl Content for BitmapImage {
    fn update_geometry(&mut self) {}
}

impl Drawable for BitmapImage {
    fn draw(&self, cx: &mut impl piet::RenderContext, _image_scale: f64) -> anyhow::Result<()> {
        let piet_image_format = piet::ImageFormat::from(self.image.memory_format);

        cx.save().map_err(|e| anyhow::anyhow!("{e:?}"))?;
        cx.transform(self.rectangle.transform.affine.to_kurbo());

        let piet_image = cx
            .make_image(
                self.image.pixel_width as usize,
                self.image.pixel_height as usize,
                &self.image.data,
                piet_image_format,
            )
            .map_err(|e| {
                anyhow::anyhow!("Make piet image in BitmapImage draw impl failed, Err: {e:?}")
            })?;
        let dest_rect = self.rectangle.cuboid.local_aabb().to_kurbo_rect();
        cx.draw_image(&piet_image, dest_rect, piet::InterpolationMode::Bilinear);
        cx.restore().map_err(|e| anyhow::anyhow!("{e:?}"))?;

        Ok(())
    }
}

impl Shapeable for BitmapImage {
    fn bounds(&self) -> Aabb {
        self.rectangle.bounds()
    }

    fn hitboxes(&self) -> Vec<Aabb> {
        vec![self.bounds()]
    }

    fn outline_path(&self) -> kurbo::BezPath {
        self.bounds().to_kurbo_rect().to_path(0.25)
    }
}

impl Transformable for BitmapImage {
    fn translate(&mut self, offset: na::Vector2<f64>) {
        self.rectangle.translate(offset);
    }

    fn rotate(&mut self, angle: f64, center: na::Point2<f64>) {
        self.rectangle.rotate(angle, center);
    }

    fn scale(&mut self, scale: na::Vector2<f64>) {
        self.rectangle.scale(scale);
    }
}

impl BitmapImage {
    pub fn from_image_bytes(
        bytes: &[u8],
        pos: na::Vector2<f64>,
        size: ImageSizeOption,
    ) -> Result<Self, anyhow::Error> {
        let image = render::Image::try_from_encoded_bytes(bytes)?;

        let initial_size = na::vector![f64::from(image.pixel_width), f64::from(image.pixel_height)];

        let (size, resize_ratio) = match size {
            ImageSizeOption::RespectOriginalSize => (initial_size, 1.0f64),
            ImageSizeOption::ImposeSize(given_size) => (given_size, 1.0f64),
            ImageSizeOption::ResizeImage(resize_struct) => (
                initial_size,
                calculate_resize_ratio(resize_struct, initial_size, pos),
            ),
        };
        tracing::debug!("the resize ratio is {resize_ratio}");

        let mut transform = Transform::default();
        transform.append_scale_mut(na::Vector2::new(resize_ratio, resize_ratio));
        transform.append_translation_mut(pos + size * resize_ratio * 0.5);
        let rectangle = Rectangle {
            cuboid: p2d::shape::Cuboid::new(size * 0.5),
            transform,
        };
        Ok(Self { image, rectangle })
    }

    pub fn from_pdf_bytes(
        to_be_read: &[u8],
        pdf_import_prefs: PdfImportPrefs,
        insert_pos: na::Vector2<f64>,
        page_range: Option<Range<u32>>,
        format: &Format,
    ) -> Result<Vec<Self>, anyhow::Error> {
        let doc = poppler::Document::from_bytes(&glib::Bytes::from(to_be_read), None)?;
        let page_range = page_range.unwrap_or(0..doc.n_pages() as u32);
        let page_width = if pdf_import_prefs.adjust_document {
            format.width()
        } else {
            format.width() * (pdf_import_prefs.page_width_perc / 100.0)
        };
        // calculate the page zoom based on the width of the first page.
        let page_zoom = if let Some(first_page) = doc.page(0) {
            page_width / first_page.size().0
        } else {
            return Ok(vec![]);
        };
        let x = insert_pos[0];
        let mut y = insert_pos[1];

        let pngs = page_range
            .map(|page_i| {
                let page = doc
                    .page(page_i as i32)
                    .ok_or_else(|| anyhow::anyhow!("no page at index '{page_i}"))?;
                let intrinsic_size = page.size();
                let width = intrinsic_size.0 * page_zoom;
                let height = intrinsic_size.1 * page_zoom;
                let surface_width = (width * pdf_import_prefs.bitmap_scalefactor).round() as i32;
                let surface_height = (height * pdf_import_prefs.bitmap_scalefactor).round() as i32;
                let surface = cairo::ImageSurface::create(
                    cairo::Format::ARgb32,
                    surface_width,
                    surface_height,
                )
                .map_err(|e| {
                    anyhow::anyhow!(
                        "Creating image surface while importing bitmapimage failed, {e:?}"
                    )
                })?;

                {
                    let cx = cairo::Context::new(&surface)
                        .context("Creating new cairo Context failed")?;

                    // Scale with the bitmap scalefactor pref
                    cx.scale(
                        page_zoom * pdf_import_prefs.bitmap_scalefactor,
                        page_zoom * pdf_import_prefs.bitmap_scalefactor,
                    );

                    // Set margin to white
                    cx.set_source_rgba(1.0, 1.0, 1.0, 1.0);
                    cx.paint()?;

                    page.render_for_printing(&cx);

                    if pdf_import_prefs.page_borders {
                        // Draw outline around page
                        cx.set_source_rgba(
                            color::GNOME_REDS[4].as_rgba().0,
                            color::GNOME_REDS[4].as_rgba().1,
                            color::GNOME_REDS[4].as_rgba().2,
                            1.0,
                        );

                        let line_width = 1.0;
                        cx.set_line_width(line_width);
                        cx.rectangle(
                            line_width * 0.5,
                            line_width * 0.5,
                            intrinsic_size.0 - line_width,
                            intrinsic_size.1 - line_width,
                        );
                        cx.stroke()?;
                    }
                }

                let mut png_data: Vec<u8> = Vec::new();
                surface.write_to_png(&mut png_data)?;
                let image_pos = na::vector![x, y];
                let image_size = na::vector![width, height];

                if pdf_import_prefs.adjust_document {
                    y += height
                } else {
                    y += match pdf_import_prefs.page_spacing {
                        PdfImportPageSpacing::Continuous => {
                            height + Stroke::IMPORT_OFFSET_DEFAULT[1] * 0.5
                        }
                        PdfImportPageSpacing::OnePerDocumentPage => format.height(),
                    };
                }

                Ok((png_data, image_pos, image_size))
            })
            .collect::<anyhow::Result<Vec<(Vec<u8>, na::Vector2<f64>, na::Vector2<f64>)>>>()?;

        pngs.into_par_iter()
            .map(|(png_data, pos, size)| {
                Self::from_image_bytes(&png_data, pos, ImageSizeOption::ImposeSize(size))
            })
            .collect()
    }
}
