use crate::{compose, geometry, render};

use p2d::bounding_volume::BoundingVolume;

/// Specifing that a type can be drawn
pub trait DrawBehaviour {
    /// returns the current bounds of this stroke
    fn bounds(&self) -> p2d::bounding_volume::AABB;
    /// sets the bounds of this stroke
    fn set_bounds(&mut self, bounds: p2d::bounding_volume::AABB);
    /// generates the bounds of this stroke
    fn gen_bounds(&self) -> Option<p2d::bounding_volume::AABB> {
        if let Ok(svgs) = self.gen_svgs(na::vector![0.0, 0.0]) {
            let mut svgs_iter = svgs.iter();
            if let Some(first) = svgs_iter.next() {
                let mut new_bounds = first.bounds;

                svgs_iter.for_each(|svg| {
                    new_bounds.merge(&svg.bounds);
                });
                new_bounds = geometry::aabb_ceil(new_bounds);

                return Some(new_bounds);
            }
        }

        None
    }
    /// generates the svg elements, without the xml header or the svg root.
    fn gen_svgs(&self, offset: na::Vector2<f64>) -> Result<Vec<render::Svg>, anyhow::Error>;
    /// generates the image for this stroke
    fn gen_image(
        &self,
        zoom: f64,
        renderer: &render::Renderer,
    ) -> Result<render::Image, anyhow::Error> {
        let offset = na::vector![0.0, 0.0];
        let mut svgs = self.gen_svgs(offset)?;

        for svg in svgs.iter_mut() {
            svg.svg_data = compose::wrap_svg(
                svg.svg_data.as_str(),
                Some(self.bounds()),
                Some(self.bounds()),
                true,
                false,
            );
        }

        renderer.gen_image(zoom, &svgs, self.bounds())
    }
}
