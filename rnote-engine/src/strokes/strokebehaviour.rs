use crate::render;
use crate::DrawBehaviour;

use rnote_compose::shapes::ShapeBehaviour;

/// Specifing that a type is a stroke.
/// Also needs to implement drawbehaviour, as some methods have default implementation based on it.
pub trait StrokeBehaviour: DrawBehaviour + ShapeBehaviour
where
    Self: Sized,
{
    /// generates the svg, without the xml header or the svg root. used for export
    fn gen_svg(&self) -> Result<render::Svg, anyhow::Error>;

    /// generates pixel images for this stroke
    /// a larger image_scale value renders them in a higher than native resolution (usually set as the camera zoom). the bounds stay the same.
    fn gen_images(&self, image_scale: f64) -> Result<Vec<render::Image>, anyhow::Error>;

    fn export_as_image_bytes(
        &self,
        format: image::ImageOutputFormat,
        image_scale: f64,
    ) -> Result<Vec<u8>, anyhow::Error> {
        let image = render::Image::gen_with_piet(
            |piet_cx| self.draw(piet_cx, image_scale),
            self.bounds(),
            image_scale,
        )?;

        Ok(image.into_encoded_bytes(format)?)
    }
}
