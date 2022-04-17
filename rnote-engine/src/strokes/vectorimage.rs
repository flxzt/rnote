use super::StrokeBehaviour;
use crate::{render, DrawBehaviour};
use rnote_compose::helpers::AABBHelpers;
use rnote_compose::shapes::Rectangle;
use rnote_compose::shapes::ShapeBehaviour;
use rnote_compose::transform::Transform;
use rnote_compose::transform::TransformBehaviour;

use p2d::bounding_volume::AABB;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename = "vectorimage")]
pub struct VectorImage {
    #[serde(rename = "svg_data")]
    pub svg_data: String,
    #[serde(rename = "intrinsic_size")]
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

impl StrokeBehaviour for VectorImage {
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
            .add(svg::node::Text::new(self.svg_data.clone()));

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

    fn gen_images(&self, image_scale: f64) -> Result<Vec<render::Image>, anyhow::Error> {
        Ok(render::Image::gen_with_piet(
            |piet_cx| self.draw(piet_cx, image_scale),
            self.bounds(),
            image_scale,
        )?)
    }
}

// Because we can't render svgs directly in piet, so we need to overwrite the gen_svgs() default implementation and call it in draw().
// There we use a svg renderer to generate pixel images. In this way we ensure to export an actual svg when calling gen_svgs(), but can also draw it onto piet.
impl DrawBehaviour for VectorImage {
    fn draw(&self, cx: &mut impl piet::RenderContext, image_scale: f64) -> anyhow::Result<()> {
        let mut image = match render::Image::join_images(
            render::Image::gen_images_from_svg(self.gen_svg()?, self.bounds(), image_scale)?,
            self.bounds(),
            image_scale,
        )? {
            Some(image) => image,
            None => return Ok(()),
        };

        // piet needs rgba8-prem. the gen_images() func might produces bgra8-prem format (when using librsvg as renderer backend), so we might need to convert the image first
        image.convert_to_rgba8pre()?;
        let piet_image_format = piet::ImageFormat::try_from(image.memory_format)?;

        let piet_image = cx
            .make_image(
                image.pixel_width as usize,
                image.pixel_height as usize,
                &image.data,
                piet_image_format,
            )
            .map_err(|e| anyhow::anyhow!("{}", e))?;

        cx.draw_image(
            &piet_image,
            image.bounds.to_kurbo_rect(),
            piet::InterpolationMode::Bilinear,
        );

        Ok(())
    }
}

impl ShapeBehaviour for VectorImage {
    fn bounds(&self) -> AABB {
        self.rectangle.bounds()
    }
}

impl TransformBehaviour for VectorImage {
    fn translate(&mut self, offset: nalgebra::Vector2<f64>) {
        self.rectangle.translate(offset);
    }

    fn rotate(&mut self, angle: f64, center: nalgebra::Point2<f64>) {
        self.rectangle.rotate(angle, center);
    }

    fn scale(&mut self, scale: na::Vector2<f64>) {
        self.rectangle.scale(scale);
    }
}

impl VectorImage {
    pub const SIZE_X_DEFAULT: f64 = 500.0;
    pub const SIZE_Y_DEFAULT: f64 = 500.0;
    pub const OFFSET_X_DEFAULT: f64 = 32.0;
    pub const OFFSET_Y_DEFAULT: f64 = 32.0;

    pub fn import_from_svg_data(
        svg_data: &str,
        pos: na::Vector2<f64>,
        size: Option<na::Vector2<f64>>,
    ) -> Result<Self, anyhow::Error> {
        let xml_options = usvg::XmlOptions {
            id_prefix: Some(rnote_compose::utils::random_id_prefix()),
            writer_opts: xmlwriter::Options {
                use_single_quote: false,
                indent: xmlwriter::Indent::None,
                attributes_indent: xmlwriter::Indent::None,
            },
        };

        let rtree = usvg::Tree::from_str(svg_data, &render::USVG_OPTIONS.to_ref())?;
        let svg_data = rtree.to_string(&xml_options);

        let svg_node = rtree.svg_node();
        let intrinsic_size = na::vector![svg_node.size.width(), svg_node.size.height()];

        let rectangle = if let Some(size) = size {
            Rectangle {
                cuboid: p2d::shape::Cuboid::new(size / 2.0),
                transform: Transform::new_w_isometry(na::Isometry2::new(pos + size / 2.0, 0.0)),
            }
        } else {
            Rectangle {
                cuboid: p2d::shape::Cuboid::new(intrinsic_size / 2.0),
                transform: Transform::new_w_isometry(na::Isometry2::new(
                    pos + intrinsic_size / 2.0,
                    0.0,
                )),
            }
        };

        Ok(Self {
            svg_data,
            intrinsic_size,
            rectangle,
        })
    }

    pub fn import_from_pdf_bytes(
        to_be_read: &[u8],
        pos: na::Vector2<f64>,
        page_width: Option<i32>,
    ) -> Result<Vec<Self>, anyhow::Error> {
        let doc = poppler::Document::from_data(to_be_read, None)?;

        let mut vector_images = Vec::new();

        for i in 0..doc.n_pages() {
            if let Some(page) = doc.page(i) {
                let intrinsic_size = page.size();

                let (width, height, _zoom) = if let Some(page_width) = page_width {
                    let zoom = f64::from(page_width) / intrinsic_size.0;

                    (f64::from(page_width), (intrinsic_size.1 * zoom), zoom)
                } else {
                    (intrinsic_size.0, intrinsic_size.1, 1.0)
                };

                let x = pos[0];
                let y = pos[1] + f64::from(i) * (height + f64::from(Self::OFFSET_Y_DEFAULT) / 2.0);

                let svg_stream: Vec<u8> = vec![];

                let surface =
                    cairo::SvgSurface::for_stream(intrinsic_size.0, intrinsic_size.1, svg_stream)
                        .map_err(|e| {
                        anyhow::anyhow!(
                            "create SvgSurface with dimensions ({}, {}) failed in vectorimage import_from_pdf_bytes with Err {}",
                            intrinsic_size.0,
                            intrinsic_size.1,
                            e
                        )
                    })?;

                {
                    let cx = cairo::Context::new(&surface).map_err(|e| {
                        anyhow::anyhow!(
                            "new cairo::Context failed in vectorimage import_from_pdf_bytes() with Err {}",
                            e
                        )
                    })?;

                    // Set margin to white
                    cx.set_source_rgba(1.0, 1.0, 1.0, 1.0);
                    cx.paint()?;

                    page.render(&cx);

                    // Draw outline around page
                    cx.set_source_rgba(0.7, 0.5, 0.5, 1.0);

                    let line_width = 1.0;
                    cx.set_line_width(line_width);
                    cx.rectangle(
                        line_width / 2.0,
                        line_width / 2.0,
                        intrinsic_size.0 - line_width,
                        intrinsic_size.1 - line_width,
                    );
                    cx.stroke()?;
                }
                let svg_data = match surface.finish_output_stream() {
                    Ok(file_content) => match file_content.downcast::<Vec<u8>>() {
                        Ok(file_content) => *file_content,
                        Err(_) => {
                            log::error!("file_content.downcast() in VectorImage::import_from_pdf_bytes() failed");
                            continue;
                        }
                    },
                    Err(e) => {
                        log::error!("surface.finish_output_stream() in VectorImage::import_from_pdf_bytes() failed with Err {}", e);
                        continue;
                    }
                };
                let svg_data = String::from_utf8(svg_data)?;

                vector_images.push(Self::import_from_svg_data(
                    svg_data.as_str(),
                    na::vector![x, y],
                    Some(na::vector![width, height]),
                )?);
            }
        }

        Ok(vector_images)
    }

    pub fn export_as_svg(&self) -> Result<String, anyhow::Error> {
        let export_bounds = self.bounds().translate(-self.bounds().mins.coords);

        let mut export_svg_data = self.gen_svg()?.svg_data;

        export_svg_data = rnote_compose::utils::wrap_svg_root(
            export_svg_data.as_str(),
            Some(export_bounds),
            Some(export_bounds),
            false,
        );

        Ok(export_svg_data)
    }

    pub fn export_as_image_bytes(
        &self,
        format: image::ImageOutputFormat,
        image_scale: f64,
    ) -> Result<Vec<u8>, anyhow::Error> {
        let bounds = self.bounds();

        let image = render::Image::gen_with_piet(
            |piet_cx| self.draw(piet_cx, image_scale),
            self.bounds(),
            image_scale,
        )?;

        match render::Image::join_images(image, bounds, image_scale)? {
            Some(image) => Ok(image.into_encoded_bytes(format)?),
            None => Ok(vec![]),
        }
    }
}
