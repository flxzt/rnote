// Imports
use crate::engine::EngineView;
use p2d::bounding_volume::Aabb;
use vello_cpu::RenderContext;

/// Trait for types that can draw a preview on the document.
///
/// In the coordinate space of the document.
pub trait Previewable {
    /// Preview bounds on the document.
    fn preview_bounds(&self, engine_view: &EngineView) -> Option<Aabb>;

    /// Draw the preview on the document.
    ///
    /// The implementers are expected to save/restore the drawing context state themselves.
    fn preview_draw(&self, cx: &mut RenderContext, engine_view: &EngineView) -> anyhow::Result<()>;

    /// Draw the preview to a [gtk4::Snapshot].
    ///
    /// The snapshot is expected to be untransformed in surface coordinate space.
    #[cfg(feature = "ui")]
    fn preview_draw_to_gtk_snapshot(
        &self,
        snapshot: &gtk4::Snapshot,
        engine_view: &EngineView,
    ) -> anyhow::Result<()> {
        use crate::ext::GrapheneRectExt;
        use gtk4::{gdk, graphene, prelude::*};
        use rnote_compose::ext::{AabbExt, Vector2Ext};

        let Some(bounds) = self.preview_bounds(engine_view) else {
            return Ok(());
        };

        let viewport = engine_view.camera.viewport();
        // Restrict to viewport as maximum bounds, else cairo is very unperformant
        // and will even crash for very large bounds
        let bounds = bounds.clamp(None, Some(viewport));

        let mut bounds_on_surface = bounds
            .scale(engine_view.camera.total_zoom())
            .translate(-engine_view.camera.offset())
            .ceil();
        bounds_on_surface.ensure_positive();
        bounds_on_surface.assert_valid()?;
        let target_width = bounds_on_surface.extents().x.ceil() as u16;
        let target_height = bounds_on_surface.extents().y.ceil() as u16;

        let mut vello_target = vello_cpu::Pixmap::new(target_width, target_height);
        let mut vello_resources = vello_cpu::Resources::new();
        let mut vello_cx = RenderContext::new(target_width, target_height);
        vello_cx.set_transform(
            kurbo::Affine::IDENTITY
                .then_translate(-bounds.mins.to_kurbo_vec())
                .then_scale(engine_view.camera.total_zoom()),
        );
        self.preview_draw(&mut vello_cx, engine_view)?;
        vello_cx.flush();
        vello_cx.render(&mut vello_target, &mut vello_resources);

        let memory_texture = gdk::MemoryTexture::new(
            target_width as i32,
            target_height as i32,
            gdk::MemoryFormat::R8g8b8a8Premultiplied,
            &glib::Bytes::from(vello_target.data_as_u8_slice()),
            4 * target_width as usize,
        );
        snapshot.append_texture(
            &memory_texture,
            &graphene::Rect::from_p2d_aabb(bounds_on_surface),
        );

        Ok(())
    }
}
