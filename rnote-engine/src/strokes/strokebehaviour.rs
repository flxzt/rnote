use crate::render;
use crate::DrawBehaviour;

use rnote_compose::shapes::ShapeBehaviour;
use rnote_compose::style::composer;

/// Specifing that a type is a stroke.
/// Also needs to implement drawbehaviour, as some methods have default implementation based on it.
pub trait StrokeBehaviour: DrawBehaviour + ShapeBehaviour where Self: Sized {
    /// generates the svg elements, without the xml header or the svg root. used for export
    fn gen_svgs(&self) -> Result<Vec<render::Svg>, anyhow::Error> {
        let bounds = self.bounds();
        let mut cx = piet_svg::RenderContext::new_no_text(kurbo::Size::new(
            bounds.extents()[0],
            bounds.extents()[1],
        ));

        self.draw(&mut cx, 1.0)?;

        let svg_data = composer::piet_svg_cx_to_svg(cx)?;

        Ok(vec![render::Svg { svg_data, bounds }])
    }

    /// generates pixel images for this stroke
    /// a larger image_scale value renders them in a higher than native resolution (usually set as the camera zoom). the bounds stay the same.
    fn gen_images(&self, image_scale: f64) -> Result<Vec<render::Image>, anyhow::Error> {
        Ok(vec![render::Image::gen_image_from_piet(
            self,
            self.bounds(),
            image_scale,
        )?])
    }
}
