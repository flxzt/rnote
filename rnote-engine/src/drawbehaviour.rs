use gtk4::graphene;
use p2d::bounding_volume::AABB;
use piet::RenderContext;
use rnote_compose::helpers::{AABBHelpers, Affine2Helpers};

use crate::Camera;
use crate::utils::GrapheneRectHelpers;

/// Trait for types that can draw themselves on the sheet
/// In the coordinate space of the sheet
pub trait DrawOnSheetBehaviour {
    fn bounds_on_sheet(&self, sheet_bounds: AABB, viewport: AABB) -> Option<AABB>;
    /// draws itself on the sheet. the callers are expected to call with save / restore context
    fn draw_on_sheet(
        &self,
        cx: &mut impl piet::RenderContext,
        sheet_bounds: AABB,
        viewport: AABB,
    ) -> Result<(), anyhow::Error>;

    /// Expects snapshot untransformed in surface coordinate space.
    fn draw_on_sheet_snapshot(
        &self,
        snapshot: &gtk4::Snapshot,
        sheet_bounds: AABB,
        camera: &Camera,
    ) -> Result<(), anyhow::Error> {
        let viewport = camera.viewport();
        if let Some(bounds) = self.bounds_on_sheet(sheet_bounds, viewport) {
            // Transform the bounds into surface coords
            let mut bounds_transformed = bounds
                .scale(camera.zoom())
                .translate(-camera.offset);

            bounds_transformed.ensure_positive();
            bounds_transformed.assert_valid()?;

            let cairo_cx = snapshot.append_cairo(&graphene::Rect::from_aabb(bounds_transformed));
            let mut piet_cx = piet_cairo::CairoRenderContext::new(&cairo_cx);

            // Transform to sheet coordinate space
            piet_cx.transform(camera.transform().to_kurbo());

            self.draw_on_sheet(&mut piet_cx, sheet_bounds, viewport)?;
        }

        Ok(())
    }
}

/// Trait for types that can draw themselves on a piet RenderContext.
pub trait DrawBehaviour {
    /// draws itself. the callers are expected to call with save / restore context
    /// image_scale is the scalefactor of generated pixel images within the type. the context should not be zoomed by it!
    fn draw(
        &self,
        cx: &mut impl piet::RenderContext,
        image_scale: f64,
    ) -> Result<(), anyhow::Error>;
}
