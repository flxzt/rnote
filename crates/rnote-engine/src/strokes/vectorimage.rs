// Imports
use super::content::GeneratedContentImages;
use super::resize::ImageSizeOption;
use super::{Content, Stroke};
use crate::document::Format;
use crate::engine::import::{PdfImportPageSpacing, PdfImportPrefs};
use crate::strokes::resize::calculate_resize_ratio;
use crate::{render, Drawable};
use kurbo::Shape;
use na;
use p2d::bounding_volume::Aabb;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use rnote_compose::color;
use rnote_compose::ext::AabbExt;
use rnote_compose::shapes::Rectangle;
use rnote_compose::shapes::Shapeable;
use rnote_compose::transform::Transform;
use rnote_compose::transform::Transformable;
use serde::{Deserialize, Serialize};
use std::ops::Range;

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
}

impl Default for VectorImage {
    fn default() -> Self {
        Self {
            svg_data: String::default(),
            intrinsic_size: na::Vector2::zeros(),
            rectangle: Rectangle::default(),
        }
    }
}

impl Content for VectorImage {
    fn gen_svg(&self) -> Result<render::Svg, anyhow::Error> {
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
        let svg = render::Svg {
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
        Ok(GeneratedContentImages::Full(vec![
            render::Image::gen_with_piet(
                |piet_cx| self.draw(piet_cx, image_scale),
                bounds,
                image_scale,
            )?,
        ]))
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
        size: ImageSizeOption,
    ) -> Result<Self, anyhow::Error> {
        const COORDINATES_PREC: u8 = 3;
        const TRANSFORMS_PREC: u8 = 4;

        let xml_options = usvg::WriteOptions {
            id_prefix: Some(rnote_compose::utils::svg_random_id_prefix()),
            preserve_text: true,
            coordinates_precision: COORDINATES_PREC,
            transforms_precision: TRANSFORMS_PREC,
            use_single_quote: false,
            indent: xmlwriter::Indent::None,
            attributes_indent: xmlwriter::Indent::None,
        };
        let svg_tree =
            usvg::Tree::from_str(svg_data, &usvg::Options::default(), &render::USVG_FONTDB)?;

        let intrinsic_size = na::vector![
            svg_tree.size().width() as f64,
            svg_tree.size().height() as f64
        ];
        let svg_data = svg_tree.to_string(&xml_options);

        let mut transform = Transform::default();
        let rectangle = match size {
            ImageSizeOption::RespectOriginalSize => {
                // Size not given : use the intrisic size
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
        })
    }

    pub fn from_pdf_bytes(
        bytes: &[u8],
        pdf_import_prefs: PdfImportPrefs,
        insert_pos: na::Vector2<f64>,
        page_range: Option<Range<u32>>,
        format: &Format,
    ) -> Result<Vec<Self>, anyhow::Error> {
        let doc = poppler::Document::from_bytes(&glib::Bytes::from(bytes), None)?;
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

        let svgs = page_range
            .filter_map(|page_i| {
                let page = doc.page(page_i as i32)?;
                let intrinsic_size = page.size();
                let width = intrinsic_size.0 * page_zoom;
                let height = intrinsic_size.1 * page_zoom;

                let res = move || -> anyhow::Result<String> {
                    let svg_stream: Vec<u8> = vec![];

                    let mut svg_surface = cairo::SvgSurface::for_stream(
                        intrinsic_size.0,
                        intrinsic_size.1,
                        svg_stream,
                    )
                    .map_err(|e| {
                        anyhow::anyhow!(
                            "Creating SvgSurface with dimensions ({}, {}) failed, Err: {e:?}",
                            intrinsic_size.0,
                            intrinsic_size.1
                        )
                    })?;

                    // Popplers page units are in points ( equals 1/72 inch )
                    svg_surface.set_document_unit(cairo::SvgUnit::Pt);

                    {
                        let cx = cairo::Context::new(&svg_surface).map_err(|e| {
                            anyhow::anyhow!("Creating new cairo context failed, Err: {e:?}")
                        })?;

                        // Set margin to white
                        cx.set_source_rgba(1.0, 1.0, 1.0, 1.0);
                        cx.paint()?;

                        // Render the poppler page
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

                    let svg_content = String::from_utf8(
                        *svg_surface
                            .finish_output_stream()
                            .map_err(|e| {
                                anyhow::anyhow!(
                                    "Failed to finish Pdf page surface output stream, Err: {e:?}"
                                )
                            })?
                            .downcast::<Vec<u8>>()
                            .map_err(|e| {
                                anyhow::anyhow!(
                                    "Failed to downcast Pdf page surface content, Err: {e:?}"
                                )
                            })?,
                    )?;

                    Ok(svg_content)
                };

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

                match res() {
                    Ok(svg_data) => Some(render::Svg { svg_data, bounds }),
                    Err(e) => {
                        tracing::error!("Importing page {page_i} from pdf failed, Err: {e:?}");
                        None
                    }
                }
            })
            .collect::<Vec<render::Svg>>();

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
