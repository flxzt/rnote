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
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use rnote_compose::ext::AabbExt;
use rnote_compose::shapes::Rectangle;
use rnote_compose::shapes::Shapeable;
use rnote_compose::transform::Transform;
use rnote_compose::transform::Transformable;
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
        with = "rnote_compose::serialize::na_vector2_f64_dp3"
    )]
    pub intrinsic_size: na::Vector2<f64>,
    #[serde(rename = "rectangle")]
    pub rectangle: Rectangle,
    /// Optional Typst source code, if this image was generated from Typst
    #[serde(
        rename = "typst_source",
        skip_serializing_if = "Option::is_none",
        default
    )]
    pub typst_source: Option<String>,
}

impl Default for VectorImage {
    fn default() -> Self {
        Self {
            svg_data: String::default(),
            intrinsic_size: na::Vector2::zeros(),
            rectangle: Rectangle::default(),
            typst_source: None,
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
                self.rectangle.transform.to_svg_transform_attr_str(),
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

impl VectorImage {
    pub fn from_svg_str(
        svg_data: &str,
        pos: na::Vector2<f64>,
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
            svg_data,
            &usvg::Options {
                fontdb: Arc::clone(&USVG_FONTDB),
                ..Default::default()
            },
        )?;

        let intrinsic_size = na::vector![
            svg_tree.size().width() as f64,
            svg_tree.size().height() as f64
        ];
        let svg_data = svg_tree.to_string(&xml_options);

        let mut transform = Transform::default();
        let rectangle = match size_option {
            ImageSizeOption::RespectOriginalSize => {
                // Size not given : use the intrinsic size
                transform.append_translation_mut(pos + intrinsic_size * 0.5);
                Rectangle {
                    cuboid: p2d::shape::Cuboid::new(intrinsic_size * 0.5),
                    transform,
                }
            }
            ImageSizeOption::ImposeSize(given_size) => {
                // Size given : use the given size
                transform.append_translation_mut(pos + given_size * 0.5);
                Rectangle {
                    cuboid: p2d::shape::Cuboid::new(given_size * 0.5),
                    transform,
                }
            }
            ImageSizeOption::ResizeImage(resize_struct) => {
                // Resize : calculate the ratio
                let resize_ratio = calculate_resize_ratio(resize_struct, intrinsic_size, pos);
                transform.append_translation_mut(pos + intrinsic_size * resize_ratio * 0.5);
                Rectangle {
                    cuboid: p2d::shape::Cuboid::new(intrinsic_size * resize_ratio * 0.5),
                    transform,
                }
            }
        };

        Ok(Self {
            svg_data,
            intrinsic_size,
            rectangle,
            typst_source: None,
        })
    }

    pub fn from_pdf_bytes(
        to_be_read: &[u8],
        pdf_import_prefs: PdfImportPrefs,
        insert_pos: na::Vector2<f64>,
        page_range: Option<Range<usize>>,
        format: &Format,
        password: Option<String>,
    ) -> Result<Vec<Self>, anyhow::Error> {
        // TODO: how to avoid this allocation without lifetime issues?
        let data = Arc::new(to_be_read.to_vec());
        let pdf = if let Some(password) = password {
            hayro_syntax::Pdf::new_with_password(data, &password)
                .map_err(|err| anyhow!("Creating Pdf instance failed, Err: {err:?}"))?
        } else {
            hayro_syntax::Pdf::new(data)
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

        // TODO: investigate if this can be parallelized with rayon's `par_iter()`
        let svgs = page_range
            .filter_map(|page_i| {
                let page = pages.get(page_i)?;
                let (intrinsic_width, intrinsic_height) = {
                    let dimensions = page.render_dimensions();
                    (dimensions.0 as f64, dimensions.1 as f64)
                };
                let width = intrinsic_width * page_zoom;
                let height = intrinsic_height * page_zoom;
                let bounds = Aabb::new(na::point![x, y], na::point![x + width, y + height]);

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
                let svg_data = hayro_svg::convert(page, &interpreter_settings, &render_settings);
                let svg = Svg { svg_data, bounds };

                Some(svg)
            })
            .collect::<Vec<Svg>>();

        svgs.into_par_iter()
            .map(|svg| {
                Self::from_svg_str(
                    svg.svg_data.as_str(),
                    svg.bounds.mins.coords,
                    ImageSizeOption::ImposeSize(svg.bounds.extents()),
                )
            })
            .collect()
    }
}
