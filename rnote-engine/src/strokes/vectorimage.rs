use std::sync::{Arc, RwLock};

use crate::compose;
use crate::compose::geometry::AABBHelpers;
use crate::compose::shapes;
use crate::compose::transformable::{Transform, Transformable};
use crate::drawbehaviour::DrawBehaviour;
use crate::render;
use crate::render::Renderer;

use anyhow::Context;
use p2d::bounding_volume::AABB;
use rand::Rng;
use serde::{Deserialize, Serialize};
use svg::node::{self, element};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename = "vectorimage")]
pub struct VectorImage {
    #[serde(rename = "svg_data")]
    pub svg_data: String,
    #[serde(rename = "intrinsic_size")]
    pub intrinsic_size: na::Vector2<f64>,
    #[serde(rename = "rectangle")]
    pub rectangle: shapes::Rectangle,
    #[serde(rename = "bounds")]
    pub bounds: AABB,
}

impl Default for VectorImage {
    fn default() -> Self {
        Self {
            svg_data: String::default(),
            intrinsic_size: na::Vector2::zeros(),
            rectangle: shapes::Rectangle::default(),
            bounds: AABB::new_zero(),
        }
    }
}

impl DrawBehaviour for VectorImage {
    fn bounds(&self) -> AABB {
        self.bounds
    }

    fn set_bounds(&mut self, bounds: AABB) {
        self.bounds = bounds;
    }

    fn gen_bounds(&self) -> Option<AABB> {
        Some(self.rectangle.global_aabb())
    }

    fn gen_svgs(&self, offset: na::Vector2<f64>) -> Result<Vec<render::Svg>, anyhow::Error> {
        let mut rectangle = self.rectangle.clone();
        rectangle.transform.append_translation_mut(offset);

        let transform_string = rectangle.transform.to_svg_transform_attr_str();

        let svg_root = element::SVG::new()
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
            .add(node::Text::new(self.svg_data.clone()));

        let group = element::Group::new()
            .set("transform", transform_string)
            .add(svg_root);

        let svg_data = compose::svg_node_to_string(&group)?;
        let svg = render::Svg {
            bounds: self.bounds.translate(offset),
            svg_data,
        };

        Ok(vec![svg])
    }
}

impl Transformable for VectorImage {
    fn translate(&mut self, offset: nalgebra::Vector2<f64>) {
        self.rectangle.translate(offset);
        self.update_geometry();
    }

    fn rotate(&mut self, angle: f64, center: nalgebra::Point2<f64>) {
        self.rectangle.rotate(angle, center);
        self.update_geometry();
    }

    fn scale(&mut self, scale: na::Vector2<f64>) {
        self.rectangle.scale(scale);
        self.update_geometry();
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
        renderer: Arc<RwLock<Renderer>>,
    ) -> Result<Self, anyhow::Error> {
        // Random prefix to ensure uniqueness
        let rand_prefix = rand::thread_rng()
            .sample_iter(&rand::distributions::Alphanumeric)
            .take(8)
            .map(char::from)
            .collect::<String>();
        let mut xml_options = renderer.read().unwrap().usvg_xml_options.clone();
        xml_options.id_prefix = Some(rand_prefix);

        let rtree =
            usvg::Tree::from_str(svg_data, &renderer.read().unwrap().usvg_options.to_ref())?;
        let svg_data = rtree.to_string(&xml_options);

        let svg_node = rtree.svg_node();
        let intrinsic_size = na::vector![svg_node.size.width(), svg_node.size.height()];

        let rectangle = if let Some(size) = size {
            shapes::Rectangle {
                cuboid: p2d::shape::Cuboid::new(size / 2.0),
                transform: Transform::new_w_isometry(na::Isometry2::new(pos + size / 2.0, 0.0)),
            }
        } else {
            shapes::Rectangle {
                cuboid: p2d::shape::Cuboid::new(intrinsic_size / 2.0),
                transform: Transform::new_w_isometry(na::Isometry2::new(
                    pos + intrinsic_size / 2.0,
                    0.0,
                )),
            }
        };

        let mut vector_image = Self {
            svg_data,
            intrinsic_size,
            rectangle,
            bounds: AABB::new_zero(),
        };
        vector_image.update_geometry();

        Ok(vector_image)
    }

    pub fn import_from_pdf_bytes(
        to_be_read: &[u8],
        pos: na::Vector2<f64>,
        page_width: Option<i32>,
        renderer: Arc<RwLock<Renderer>>,
    ) -> Result<Vec<Self>, anyhow::Error> {
        let doc = poppler::Document::from_data(to_be_read, None)?;

        let mut images = Vec::new();

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

                images.push(Self::import_from_svg_data(
                    svg_data.as_str(),
                    na::vector![x, y],
                    Some(na::vector![width, height]),
                    Arc::clone(&renderer),
                )?);
            }
        }

        Ok(images)
    }

    pub fn update_geometry(&mut self) {
        if let Some(new_bounds) = self.gen_bounds() {
            self.set_bounds(new_bounds);
        }
    }

    pub fn export_as_svg(&self) -> Result<String, anyhow::Error> {
        let export_bounds = self.bounds.translate(-self.bounds().mins.coords);
        let mut export_svg_data = self
            .gen_svgs(-self.bounds().mins.coords)?
            .iter()
            .map(|svg| svg.svg_data.clone())
            .collect::<Vec<String>>()
            .join("\n");

        export_svg_data = compose::wrap_svg_root(
            export_svg_data.as_str(),
            Some(export_bounds),
            Some(export_bounds),
            false,
        );

        Ok(export_svg_data)
    }

    pub fn export_as_image_bytes(
        &self,
        zoom: f64,
        format: image::ImageOutputFormat,
        renderer: Arc<RwLock<Renderer>>,
    ) -> Result<Vec<u8>, anyhow::Error> {
        let export_bounds = self.bounds.translate(-self.bounds().mins.coords);
        let mut export_svg_data = self
            .gen_svgs(-self.bounds().mins.coords)
            .context("gen_svgs() failed in VectorImage export_as_bytes()")?
            .iter()
            .map(|svg| svg.svg_data.clone())
            .collect::<Vec<String>>()
            .join("\n");
        export_svg_data = compose::wrap_svg_root(
            export_svg_data.as_str(),
            Some(export_bounds),
            Some(export_bounds),
            false,
        );
        let export_svg = render::Svg {
            bounds: export_bounds,
            svg_data: export_svg_data,
        };

        let image_raw = render::concat_images(
            renderer
                .read()
                .unwrap()
                .gen_images(zoom, vec![export_svg], export_bounds)?,
            export_bounds,
            zoom,
        )?;

        Ok(render::image_into_encoded_bytes(image_raw, format)?)
    }
}
