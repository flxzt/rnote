use gtk4::graphene;
use p2d::bounding_volume::AABB;
use piet::RenderContext;
use rnote_compose::helpers::{AABBHelpers, Affine2Helpers};

use crate::utils::GrapheneRectHelpers;
use crate::Camera;

/// Trait for types that can draw themselves on the document.
/// In the coordinate space of the document
pub trait DrawOnDocBehaviour {
    fn bounds_on_doc(&self, doc_bounds: AABB, camera: &Camera) -> Option<AABB>;
    /// draws itself on the document. the implementors are expected save / restore context
    fn draw_on_doc(
        &self,
        cx: &mut piet_cairo::CairoRenderContext,
        doc_bounds: AABB,
        camera: &Camera,
    ) -> anyhow::Result<()>;

    /// Expects snapshot untransformed in surface coordinate space.
    fn draw_on_doc_snapshot(
        &self,
        snapshot: &gtk4::Snapshot,
        doc_bounds: AABB,
        camera: &Camera,
    ) -> anyhow::Result<()> {
        snapshot.save();

        if let Some(bounds) = self.bounds_on_doc(doc_bounds, camera) {
            let viewport = camera.viewport();

            // Restrict to viewport as maximum bounds. Else cairo will panic for very large bounds
            let bounds = bounds.clamp(None, Some(viewport));
            // Transform the bounds into surface coords
            let mut bounds_transformed = bounds
                .scale(camera.total_zoom())
                .translate(-camera.offset)
                .ceil();

            bounds_transformed.ensure_positive();
            bounds_transformed.assert_valid()?;

            let cairo_cx =
                snapshot.append_cairo(&graphene::Rect::from_p2d_aabb(bounds_transformed));
            let mut piet_cx = piet_cairo::CairoRenderContext::new(&cairo_cx);

            // Transform to doc coordinate space
            piet_cx.transform(camera.transform().to_kurbo());

            self.draw_on_doc(&mut piet_cx, doc_bounds, camera)?;
        }

        snapshot.restore();
        Ok(())
    }
}

/// Trait for types that can draw themselves on a piet RenderContext.
pub trait DrawBehaviour {
    /// draws itself. the implementors are expected save / restore context
    /// image_scale is the scalefactor of generated pixel images within the type. the content should not be zoomed by it!
    fn draw(&self, cx: &mut impl piet::RenderContext, image_scale: f64) -> anyhow::Result<()>;
}
