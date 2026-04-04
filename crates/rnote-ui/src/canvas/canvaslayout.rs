// Imports
use crate::canvas::RnCanvas;
use gtk4::{
    LayoutManager, Orientation, SizeRequestMode, Widget, glib, prelude::*, subclass::prelude::*,
};
use p2d::bounding_volume::{Aabb, BoundingVolume};
use rnote_compose::ext::AabbExt;
use rnote_engine::image;
use std::cell::Cell;

mod imp {
    use super::*;

    #[derive(Debug)]
    pub(crate) struct RnCanvasLayout {
        pub(crate) old_viewport: Cell<Aabb>,
    }

    impl Default for RnCanvasLayout {
        fn default() -> Self {
            Self {
                old_viewport: Cell::new(Aabb::new_zero()),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for RnCanvasLayout {
        const NAME: &'static str = "RnCanvasLayout";
        type Type = super::RnCanvasLayout;
        type ParentType = LayoutManager;
    }

    impl ObjectImpl for RnCanvasLayout {}

    impl LayoutManagerImpl for RnCanvasLayout {
        fn request_mode(&self, _widget: &Widget) -> SizeRequestMode {
            SizeRequestMode::ConstantSize
        }

        fn measure(
            &self,
            widget: &Widget,
            orientation: Orientation,
            _for_size: i32,
        ) -> (i32, i32, i32, i32) {
            let canvas = widget.downcast_ref::<RnCanvas>().unwrap();

            let (surface_mins, surface_maxs) = canvas.engine_ref().camera_surface_mins_maxs();
            let surface_size = surface_maxs - surface_mins;

            if orientation == Orientation::Horizontal {
                let natural_width = surface_size.x.ceil() as i32;
                (0, natural_width, -1, -1)
            } else {
                let natural_height = surface_size.y.ceil() as i32;
                (0, natural_height, -1, -1)
            }
        }

        fn allocate(&self, widget: &Widget, width: i32, height: i32, _baseline: i32) {
            let canvas = widget.downcast_ref::<RnCanvas>().unwrap();

            let new_size = na::vector![width as f64, height as f64];
            let offset = canvas.engine_ref().camera.offset();

            // Configure adjustments using new size
            let (surface_mins, surface_maxs) = canvas.engine_ref().camera_surface_mins_maxs();

            let adjustment_maxs = RnCanvas::surface_to_adjustment(surface_maxs, surface_mins);
            let adjustment_value = RnCanvas::surface_to_adjustment(offset, surface_mins);

            canvas.configure_adjustments(new_size, adjustment_maxs, adjustment_value);

            // Update the camera size and re-clamp the offset to the new size
            let _ = canvas.engine_mut().camera_set_size(new_size);
            let _ = canvas.engine_mut().camera_set_offset(offset);

            // Calculate new viewport from the updated camera state
            let old_viewport = self.old_viewport.get();
            let new_viewport = canvas.engine_ref().camera.viewport();

            // We only extend the viewport by a (tweakable) fraction of the margin, because we want to trigger rendering
            // before we reach it. This has two advantages: Strokes that might take longer to render have a head start
            // while still being out of view, and the rendering gets triggered more often, so not that many strokes
            // start to get rendered. This avoids stutters, because while the rendering itself is on worker threads, we
            // still have to `integrate` the resulted textures, which can also take up quite some time on the main UI
            // thread.
            let old_viewport_extended = old_viewport
                .extend_by(old_viewport.extents() * image::VIEWPORT_EXTENTS_MARGIN_FACTOR * 0.8);

            // always update the background rendering
            let _ = canvas
                .engine_mut()
                .update_background_rendering_current_viewport();

            // On zoom outs or viewport translations this will evaluate true, so we render the strokes that are coming
            // into view. But after zoom ins we need to update old_viewport with layout_manager.update_state()
            if !old_viewport_extended.contains(&new_viewport) {
                self.old_viewport.set(new_viewport);

                // Because we don't set the rendering of strokes that are already in the view dirty, we only rerender
                // those that may come into the view and are flagged dirty.
                let _ = canvas
                    .engine_mut()
                    .update_content_rendering_current_viewport();
            }
        }
    }
}

glib::wrapper! {
    pub(crate) struct RnCanvasLayout(ObjectSubclass<imp::RnCanvasLayout>)
        @extends LayoutManager;
}

impl Default for RnCanvasLayout {
    fn default() -> Self {
        Self::new()
    }
}

impl RnCanvasLayout {
    pub(crate) fn new() -> Self {
        glib::Object::new()
    }

    // needs to be called after zooming
    pub(crate) fn update_old_viewport(&self, viewport: Aabb) {
        self.imp().old_viewport.set(viewport);
    }
}
