// Imports
use crate::render;
use crate::DrawBehaviour;
use once_cell::sync::Lazy;
use p2d::bounding_volume::Aabb;
use rnote_compose::{color, shapes::ShapeBehaviour};

#[derive(Debug, Clone)]
/// Generated stroke images.
///
/// Some stroke types may only support generating image(s) for the whole stroke.
pub enum GeneratedStrokeImages {
    /// Only part of the stroke was rendered (for example when part of it is out of the current viewport).
    Partial {
        images: Vec<render::Image>,
        viewport: Aabb,
    },
    /// All stroke images were rendered.
    Full(Vec<render::Image>),
}

pub(crate) static STROKE_HIGHLIGHT_COLOR: Lazy<piet::Color> =
    Lazy::new(|| color::GNOME_BLUES[1].with_alpha(0.376));

/// Types that are strokes.
pub trait StrokeBehaviour: DrawBehaviour + ShapeBehaviour
where
    Self: Sized,
{
    /// Generate Svg, without the xml header or the svg root. Used when exporting.
    ///
    /// Implementors should translate the stroke drawing so that the svg has origin (0.0, 0.0).
    fn gen_svg(&self) -> Result<render::Svg, anyhow::Error>;

    /// Generates bitmap images.
    ///
    /// A larger `image_scale` value renders them in a higher than native resolution (usually set as the camera zoom).
    /// The bounds are not scaled by it.
    fn gen_images(
        &self,
        viewport: Aabb,
        image_scale: f64,
    ) -> Result<GeneratedStrokeImages, anyhow::Error>;

    /// Draw it's highlight.
    /// The implementors are expected to save/restore the drawing context.
    ///
    /// `total_zoom` is the zoom-factor of the surface that draws the highlight.
    fn draw_highlight(
        &self,
        cx: &mut impl piet::RenderContext,
        total_zoom: f64,
    ) -> anyhow::Result<()>;

    /// Update the geometry, possibly regenerating internally stored state.
    ///
    /// Must be called after the stroke has been (geometrically) modified or transformed.
    fn update_geometry(&mut self);

    /// Export as encoded bitmap image (Png/Jpg/..).
    fn export_as_bitmapimage_bytes(
        &self,
        format: image::ImageOutputFormat,
        image_scale: f64,
    ) -> Result<Vec<u8>, anyhow::Error> {
        let image = render::Image::gen_with_piet(
            |piet_cx| self.draw(piet_cx, image_scale),
            self.bounds(),
            image_scale,
        )?;

        image.into_encoded_bytes(format)
    }
}
