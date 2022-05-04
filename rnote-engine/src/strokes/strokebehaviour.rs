use crate::render;
use crate::DrawBehaviour;

use p2d::bounding_volume::AABB;
use rnote_compose::shapes::ShapeBehaviour;

#[derive(Debug, Clone)]
/// Generated stroke images. Some stroke types may only support generating (an) image(s) for the whole stroke
pub enum GeneratedStrokeImages {
    /// only part of the stroke was rendered (e.g. part of it is out of the viewport)
    Partial {
        images: Vec<render::Image>,
        viewport: AABB,
    },
    /// All stroke images were rendered
    Full(Vec<render::Image>),
}

/// Specifing that a type is a stroke.
/// Also needs to implement drawbehaviour, as some methods have default implementation based on it.
pub trait StrokeBehaviour: DrawBehaviour + ShapeBehaviour
where
    Self: Sized,
{
    /// generates the svg, without the xml header or the svg root. used for exporting.
    fn gen_svg(&self) -> Result<render::Svg, anyhow::Error>;

    /// generates pixel images for this stroke
    /// a larger image_scale value renders them in a higher than native resolution (usually set as the camera zoom). the bounds stay the same.
    fn gen_images(
        &self,
        viewport: AABB,
        image_scale: f64,
    ) -> Result<GeneratedStrokeImages, anyhow::Error>;

    /// Exporting as encoded image bytes (Png / Jpg, etc.)
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
