use std::error::Error;

use crate::strokes::{compose, render};

use gtk4::gsk;
use serde::{Deserialize, Serialize};

use super::StrokeBehaviour;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorImage {
    pub bounds: p2d::bounding_volume::AABB,
    pub intrinsic_size: na::Vector2<f64>,
    pub svg_data: String,
    #[serde(skip, default = "render::default_rendernode")]
    pub rendernode: gsk::RenderNode,
}

impl StrokeBehaviour for VectorImage {
    fn bounds(&self) -> p2d::bounding_volume::AABB {
        self.bounds
    }

    fn translate(&mut self, offset: na::Vector2<f64>) {
        self.bounds = self
            .bounds
            .transform_by(&na::geometry::Isometry2::new(offset, 0.0));
    }

    fn resize(&mut self, new_bounds: p2d::bounding_volume::AABB) {
        self.bounds = new_bounds;
    }

    fn gen_svg_data(&self, offset: na::Vector2<f64>) -> Result<String, Box<dyn Error>> {
        let bounds = p2d::bounding_volume::AABB::new(
            na::point![
                self.bounds.mins[0] + offset[0],
                self.bounds.mins[1] + offset[1]
            ],
            na::point![
                self.bounds.maxs[0] + offset[0],
                self.bounds.maxs[1] + offset[1]
            ],
        );

        let intrinsic_bounds = p2d::bounding_volume::AABB::new(
            na::point![0.0, 0.0],
            na::point![self.intrinsic_size[0], self.intrinsic_size[1]],
        );

        let svg = compose::wrap_svg(
            self.svg_data.as_str(),
            Some(bounds),
            Some(intrinsic_bounds),
            false,
            false,
        );
        Ok(svg)
    }

    fn update_rendernode(&mut self, scalefactor: f64, renderer: &render::Renderer) {
        if let Ok(rendernode) = self.gen_rendernode(scalefactor, renderer) {
            self.rendernode = rendernode;
        } else {
            log::error!("failed to gen_rendernode() in update_rendernode() of vectorimage");
        }
    }

    fn gen_rendernode(
        &self,
        scalefactor: f64,
        renderer: &render::Renderer,
    ) -> Result<gsk::RenderNode, Box<dyn Error>> {
        renderer.gen_rendernode(
            self.bounds,
            scalefactor,
            compose::add_xml_header(self.gen_svg_data(na::vector![0.0, 0.0])?.as_str()).as_str(),
        )
    }
}

impl VectorImage {
    pub const SIZE_X_DEFAULT: f64 = 500.0;
    pub const SIZE_Y_DEFAULT: f64 = 500.0;
    pub const OFFSET_X_DEFAULT: f64 = 28.0;
    pub const OFFSET_Y_DEFAULT: f64 = 28.0;

    pub fn import_from_svg(svg: &str, pos: na::Vector2<f64>) -> Result<Self, Box<dyn Error>> {
        let svg_data = compose::remove_xml_header(svg);
        let intrinsic_size = compose::svg_intrinsic_size(svg).unwrap_or(na::vector![
            VectorImage::SIZE_X_DEFAULT,
            VectorImage::SIZE_Y_DEFAULT
        ]);

        let bounds = p2d::bounding_volume::AABB::new(
            na::Point2::from(pos),
            na::Point2::from(intrinsic_size + pos),
        );

        let vector_image = Self {
            bounds,
            intrinsic_size,
            svg_data,
            rendernode: render::default_rendernode(),
        };

        Ok(vector_image)
    }
}
