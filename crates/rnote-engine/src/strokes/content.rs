// Imports
use crate::{render, Drawable};
use p2d::bounding_volume::{Aabb, BoundingVolume};
use rnote_compose::ext::AabbExt;
use rnote_compose::{color, shapes::Shapeable};

#[derive(Debug, Clone)]
/// Generated content images.
///
/// Some `Content` trait implementors only support generating image(s) for the entire content (Full).
pub enum GeneratedContentImages {
    /// Only part of the content was rendered (for example when part of it is out of the current viewport).
    Partial {
        images: Vec<render::Image>,
        viewport: Aabb,
    },
    /// All content image(s) were rendered.
    Full(Vec<render::Image>),
}

pub(crate) const CONTENT_HIGHLIGHT_COLOR: piet::Color = color::GNOME_BLUES[1].with_a8(96);

/// Types that are content.
pub trait Content: Drawable + Shapeable
where
    Self: Sized,
{
    /// Generate Svg from the content, without the Xml header or the Svg root.
    ///
    /// Used for exporting.
    fn gen_svg(&self) -> Result<render::Svg, anyhow::Error> {
        let bounds = self.bounds();
        render::Svg::gen_with_cairo(|cx| self.draw_to_cairo(cx, 1.0), bounds)
    }

    /// Generate bitmap images for rendering in the app.
    ///
    /// A larger `image_scale` value renders them in a higher than native resolution (usually set as the camera zoom).
    /// The bounds are not scaled by it.
    fn gen_images(
        &self,
        viewport: Aabb,
        image_scale: f64,
    ) -> Result<GeneratedContentImages, anyhow::Error> {
        let bounds = self.bounds();

        if viewport.contains(&bounds) {
            Ok(GeneratedContentImages::Full(vec![
                render::Image::gen_with_piet(
                    |piet_cx| self.draw(piet_cx, image_scale),
                    bounds,
                    image_scale,
                )?,
            ]))
        } else if let Some(intersection_bounds) = viewport.intersection(&bounds) {
            Ok(GeneratedContentImages::Partial {
                images: vec![render::Image::gen_with_piet(
                    |piet_cx| self.draw(piet_cx, image_scale),
                    intersection_bounds,
                    image_scale,
                )?],
                viewport,
            })
        } else {
            Ok(GeneratedContentImages::Partial {
                images: vec![],
                viewport,
            })
        }
    }

    /// Draw the content highlight. Used when indicating a selection.
    ///
    /// The implementors are expected to save/restore the drawing context.
    ///
    /// `total_zoom` is the zoom-factor of the surface that the highlight gets drawn on.
    fn draw_highlight(
        &self,
        cx: &mut impl piet::RenderContext,
        total_zoom: f64,
    ) -> anyhow::Result<()> {
        const HIGHLIGHT_STROKE_WIDTH: f64 = 1.5;
        const DRAW_BOUNDS_THRESHOLD_AREA: f64 = 10_u32.pow(2) as f64;
        let bounds = self.bounds();
        let bez_path = self.outline_path();

        if bounds.scale(total_zoom).volume() < DRAW_BOUNDS_THRESHOLD_AREA {
            cx.fill(bounds.to_kurbo_rect(), &CONTENT_HIGHLIGHT_COLOR);
        } else {
            cx.stroke_styled(
                bez_path,
                &CONTENT_HIGHLIGHT_COLOR,
                HIGHLIGHT_STROKE_WIDTH / total_zoom,
                &piet::StrokeStyle::new()
                    .line_join(piet::LineJoin::Round)
                    .line_cap(piet::LineCap::Round),
            );
        }
        Ok(())
    }

    /// Update the content geometry, possibly regenerating internally stored state.
    ///
    /// Must be called after the stroke has been (geometrically) modified or transformed.
    fn update_geometry(&mut self);

    /// Export to encoded bitmap image (Png/Jpeg/..).
    fn export_to_bitmap_image_bytes(
        &self,
        format: image::ImageFormat,
        image_scale: f64,
    ) -> Result<Vec<u8>, anyhow::Error> {
        render::Image::gen_with_piet(
            |piet_cx| self.draw(piet_cx, image_scale),
            self.bounds(),
            image_scale,
        )?
        .into_encoded_bytes(format, None)
    }
}
