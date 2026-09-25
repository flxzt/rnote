// Imports
use super::content::GeneratedContentImages;
use super::resize::{ImageSizeOption, calculate_resize_ratio};
use super::{Content, Stroke};
use crate::Image;
use crate::document::Format;
use crate::engine::import::{PdfImportPageSpacing, PdfImportPrefs};
use crate::svg::USVG_FONTDB;
use crate::{Drawable, Svg};
use anyhow::anyhow;
use hayro::{hayro_interpret, hayro_syntax};
use kurbo::Shape;
use p2d::bounding_volume::Aabb;
use p2d::glamx::DAffine2;
use p2d::math::Vector2;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use rnote_compose::Transformable;
use rnote_compose::ext::{AabbExt, DAffine2Ext};
use rnote_compose::shapes::Rectangle;
use rnote_compose::shapes::Shapeable;
use serde::{Deserialize, Serialize};
use std::ops::Range;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename = "vectorimage")]
pub struct VectorImage {
    #[serde(rename = "svg_data")]
    pub svg_data: String,
    #[serde(
        rename = "intrinsic_size",
        with = "rnote_compose::serialize::glam_vector2_dp3"
    )]
    pub intrinsic_size: Vector2,
    #[serde(rename = "rectangle")]
    pub rectangle: Rectangle,
}

impl Default for VectorImage {
    fn default() -> Self {
        Self {
            svg_data: String::default(),
            intrinsic_size: Vector2::ZERO,
            rectangle: Rectangle::default(),
        }
    }
}

impl Content for VectorImage {
    fn gen_svg(&self) -> Result<Svg, anyhow::Error> {
        let svg_root = svg::node::element::SVG::new()
            .set("x", -self.rectangle.cuboid.half_extents[0])
            .set("y", -self.rectangle.cuboid.half_extents[1])
            .set("width", 2.0 * self.rectangle.cuboid.half_extents[0])
            .set("height", 2.0 * self.rectangle.cuboid.half_extents[1])
            .set(
                "viewBox",
                format!(
                    "{:.3} {:.3} {:.3} {:.3}",
                    0.0, 0.0, self.intrinsic_size[0], self.intrinsic_size[1]
                ),
            )
            .set("preserveAspectRatio", "none")
            .add(svg::node::Blob::new(self.svg_data.clone()));
        let group = svg::node::element::Group::new()
            .set(
                "transform",
                self.rectangle.affine.to_svg_transform_attr_str(),
            )
            .add(svg_root);
        let svg_data = rnote_compose::utils::svg_node_to_string(&group)?;
        let svg = Svg {
            bounds: self.rectangle.bounds(),
            svg_data,
        };
        Ok(svg)
    }

    fn gen_images(
        &self,
        _viewport: Aabb,
        image_scale: f64,
    ) -> Result<GeneratedContentImages, anyhow::Error> {
        let bounds = self.bounds();
        // always generate full stroke images for vectorimages, they are too expensive to be repeatedly rendered
        Ok(GeneratedContentImages::Full(vec![Image::gen_with_piet(
            |piet_cx| self.draw(piet_cx, image_scale),
            bounds,
            image_scale,
        )?]))
    }

    fn update_geometry(&mut self) {}
}

// Because it is currently not possible to render SVGs directly with piet, the default gen_svg() implementation is
// overwritten and called in `draw()` and `draw_to_cairo()`. There the rsvg renderer is used to generate bitmap
// images. This way it is ensured that an actual Svg is generated when calling `gen_svg()`, but it is also possible to
// to be drawn to piet.
impl Drawable for VectorImage {
    fn draw(&self, cx: &mut impl piet::RenderContext, image_scale: f64) -> anyhow::Result<()> {
        let image = self.gen_svg()?.gen_image(image_scale)?;
        // image_scale does not have a meaning here
        image.draw(cx, image_scale)
    }

    fn draw_to_cairo(&self, cx: &cairo::Context, _image_scale: f64) -> anyhow::Result<()> {
        self.gen_svg()?.draw_to_cairo(cx)
    }
}

impl Shapeable for VectorImage {
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

impl Transformable for VectorImage {
    fn translate(&mut self, offset: Vector2) {
        self.rectangle.translate(offset);
    }

    fn rotate(&mut self, angle: f64, center: Vector2) {
        self.rectangle.rotate(angle, center);
    }

    fn scale(&mut self, scale: Vector2) {
        self.rectangle.scale(scale);
    }
}

impl VectorImage {
    /// Construct a vector image from an Svg string.
    ///
    /// Takes ownership of the Svg data because the parsed [usvg::Tree] is self-contained: the data
    /// can be released before the tree is serialized again (see below), which halves the transient
    /// for the large Svg data of imported Pdf pages.
    pub fn from_svg_string(
        svg_data: String,
        pos: Vector2,
        size_option: ImageSizeOption,
    ) -> Result<Self, anyhow::Error> {
        const COORDINATES_PREC: u8 = 3;
        const TRANSFORMS_PREC: u8 = 8;

        let xml_options = usvg::WriteOptions {
            id_prefix: Some(rnote_compose::utils::svg_random_id_prefix()),
            preserve_text: true,
            coordinates_precision: COORDINATES_PREC,
            transforms_precision: TRANSFORMS_PREC,
            use_single_quote: false,
            indent: xmlwriter::Indent::None,
            attributes_indent: xmlwriter::Indent::None,
        };
        let svg_tree = usvg::Tree::from_str(
            &svg_data,
            &usvg::Options {
                fontdb: Arc::clone(&USVG_FONTDB),
                ..Default::default()
            },
        )?;

        let intrinsic_size = Vector2::new(
            svg_tree.size().width() as f64,
            svg_tree.size().height() as f64,
        );

        // The tree owns everything it needs, so the source data is not needed anymore. For a scanned
        // Pdf page the tree holds the decoded page image while the source data holds it encoded, so
        // releasing one of the two before the tree is serialized avoids holding both at once.
        drop(svg_data);

        let svg_data = svg_tree.to_string(&xml_options);

        let mut affine = DAffine2::IDENTITY;
        let rectangle = match size_option {
            ImageSizeOption::RespectOriginalSize => {
                // Size not given : use the intrinsic size
                affine.append_translation_mut(pos + intrinsic_size * 0.5);
                Rectangle {
                    cuboid: p2d::shape::Cuboid::new(intrinsic_size * 0.5),
                    affine,
                }
            }
            ImageSizeOption::ImposeSize(given_size) => {
                // Size given : use the given size
                affine.append_translation_mut(pos + given_size * 0.5);
                Rectangle {
                    cuboid: p2d::shape::Cuboid::new(given_size * 0.5),
                    affine,
                }
            }
            ImageSizeOption::ResizeImage(resize_struct) => {
                // Resize : calculate the ratio
                let resize_ratio = calculate_resize_ratio(resize_struct, intrinsic_size, pos);
                affine.append_translation_mut(pos + intrinsic_size * resize_ratio * 0.5);
                Rectangle {
                    cuboid: p2d::shape::Cuboid::new(intrinsic_size * resize_ratio * 0.5),
                    affine,
                }
            }
        };

        Ok(Self {
            svg_data,
            intrinsic_size,
            rectangle,
        })
    }

    /// Generate vector image strokes from the pages of a Pdf.
    ///
    /// Takes ownership of the Pdf bytes: hayro parses Pdf objects lazily and keeps the file bytes
    /// alive for as long as the [hayro_syntax::Pdf] exists, so the buffer is handed over instead of
    /// being copied. Copying it kept a second, equally large copy of the whole file resident for the
    /// entire import.
    pub fn from_pdf_bytes(
        to_be_read: Vec<u8>,
        pdf_import_prefs: PdfImportPrefs,
        insert_pos: Vector2,
        page_range: Option<Range<usize>>,
        format: &Format,
        password: Option<String>,
    ) -> Result<Vec<Self>, anyhow::Error> {
        let pdf = if let Some(password) = password {
            hayro_syntax::Pdf::new_with_password(to_be_read, &password)
                .map_err(|err| anyhow!("Creating Pdf instance failed, Err: {err:?}"))?
        } else {
            hayro_syntax::Pdf::new(to_be_read)
                .map_err(|err| anyhow!("Creating Pdf instance failed, Err: {err:?}"))?
        };
        let interpreter_settings = hayro_interpret::InterpreterSettings::default();
        let render_settings = hayro_svg::SvgRenderSettings {
            bg_color: [255, 255, 255, 255],
        };
        let pages = pdf.pages();
        let page_range = page_range.unwrap_or(0..pages.len());
        let page_width = if pdf_import_prefs.adjust_document {
            format.width()
        } else {
            format.width() * (pdf_import_prefs.page_width_perc / 100.0)
        };

        // calculate the page zoom based on the width of the first page.
        let page_zoom = if let Some(first_page) = pages.first() {
            page_width / first_page.render_dimensions().0 as f64
        } else {
            return Ok(vec![]);
        };
        let x = insert_pos[0];
        let mut y = insert_pos[1];

        // The pages are converted one after another and only parsed in parallel below.
        //
        // `hayro_svg::convert()` is the memory-hungry step: it renders the page and re-encodes
        // its image, which costs a transient of roughly 35 MB for a small page and ~95 MB for a
        // page of a scanned textbook (dominated by decoding and re-encoding the page image).
        // Parallelizing it multiplies that transient by the number of pages in flight, which
        // measured *worse* than converting sequentially on real documents: on a 128 MB scanned
        // textbook importing 5 pages took 424 MB of peak allocation here versus 606 MB when the
        // conversion was moved into the parallel pass, while on a 60-page scan of small pages the
        // same change went the other way (967 MB -> 765 MB) because holding the converted Svg of
        // every page is what dominates there. Sequential conversion is the variant that never
        // regresses; parallelizing it needs a bound on the number of pages in flight.
        // As in the bitmap path, the render cache is created once for the document and reused
        // across the pages of this import.
        let render_cache = hayro_svg::RenderCache::new();
        let svgs = page_range
            .filter_map(|page_i| {
                let page = pages.get(page_i)?;
                let (intrinsic_width, intrinsic_height) = {
                    let dimensions = page.render_dimensions();
                    (dimensions.0 as f64, dimensions.1 as f64)
                };
                let width = intrinsic_width * page_zoom;
                let height = intrinsic_height * page_zoom;
                let bounds = Aabb::new(Vector2::new(x, y), Vector2::new(x + width, y + height));

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
                let svg_data = hayro_svg::convert(
                    page,
                    &render_cache,
                    &interpreter_settings,
                    &render_settings,
                );
                let svg = Svg { svg_data, bounds };

                Some(svg)
            })
            .collect::<Vec<Svg>>();

        svgs.into_par_iter()
            .map(|svg| {
                Self::from_svg_string(
                    svg.svg_data,
                    svg.bounds.mins,
                    ImageSizeOption::ImposeSize(svg.bounds.extents()),
                )
            })
            .collect()
    }
}
