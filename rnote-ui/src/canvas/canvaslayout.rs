use std::cell::Cell;

use gtk4::{
    glib, prelude::*, subclass::prelude::*, LayoutManager, Orientation, SizeRequestMode, Widget,
};
use p2d::bounding_volume::{Aabb, BoundingVolume};
use rnote_compose::helpers::AabbHelpers;

use crate::canvas::RnoteCanvas;
use rnote_engine::{document::Layout, render};

mod imp {
    use super::*;

    #[derive(Debug)]
    pub(crate) struct CanvasLayout {
        pub(crate) old_viewport: Cell<Aabb>,
    }

    impl Default for CanvasLayout {
        fn default() -> Self {
            Self {
                old_viewport: Cell::new(Aabb::new_zero()),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for CanvasLayout {
        const NAME: &'static str = "CanvasLayout";
        type Type = super::CanvasLayout;
        type ParentType = LayoutManager;
    }

    impl ObjectImpl for CanvasLayout {}

    impl LayoutManagerImpl for CanvasLayout {
        fn request_mode(&self, _widget: &Widget) -> SizeRequestMode {
            SizeRequestMode::ConstantSize
        }

        fn measure(
            &self,
            widget: &Widget,
            orientation: Orientation,
            _for_size: i32,
        ) -> (i32, i32, i32, i32) {
            let canvas = widget.downcast_ref::<RnoteCanvas>().unwrap();
            let total_zoom = canvas.engine().borrow().camera.total_zoom();
            let document = canvas.engine().borrow().document;

            if orientation == Orientation::Horizontal {
                // let canvas_width = canvas.width() as f64;

                let natural_width = (document.width * total_zoom
                    + 2.0 * super::CanvasLayout::OVERSHOOT_HORIZONTAL)
                    .ceil() as i32;

                (0, natural_width, -1, -1)
            } else {
                // let canvas_height = canvas.height() as f64;

                let natural_height = (document.height * total_zoom
                    + 2.0 * super::CanvasLayout::OVERSHOOT_VERTICAL)
                    .ceil() as i32;

                (0, natural_height, -1, -1)
            }
        }

        fn allocate(&self, widget: &Widget, width: i32, height: i32, _baseline: i32) {
            let canvas = widget.downcast_ref::<RnoteCanvas>().unwrap();
            let hadj = canvas.hadjustment().unwrap();
            let vadj = canvas.vadjustment().unwrap();

            let engine = canvas.engine();
            let mut engine = engine.borrow_mut();
            let total_zoom = engine.camera.total_zoom();
            let document = engine.document;

            let canvas_size = na::vector![f64::from(width), f64::from(height)];

            // Update the adjustments
            let (h_lower, h_upper) = match document.layout {
                Layout::FixedSize | Layout::ContinuousVertical => (
                    document.x * total_zoom - super::CanvasLayout::OVERSHOOT_HORIZONTAL,
                    (document.x + document.width) * total_zoom
                        + super::CanvasLayout::OVERSHOOT_HORIZONTAL,
                ),
                Layout::SemiInfinite => (
                    document.x * total_zoom - super::CanvasLayout::OVERSHOOT_HORIZONTAL,
                    (document.x + document.width) * total_zoom,
                ),
                Layout::Infinite => (
                    document.x * total_zoom,
                    (document.x + document.width) * total_zoom,
                ),
            };

            let (v_lower, v_upper) = match document.layout {
                Layout::FixedSize | Layout::ContinuousVertical => (
                    document.y * total_zoom - super::CanvasLayout::OVERSHOOT_VERTICAL,
                    (document.y + document.height) * total_zoom
                        + super::CanvasLayout::OVERSHOOT_VERTICAL,
                ),
                Layout::SemiInfinite => (
                    document.y * total_zoom - super::CanvasLayout::OVERSHOOT_VERTICAL,
                    (document.y + document.height) * total_zoom,
                ),
                Layout::Infinite => (
                    document.y * total_zoom,
                    (document.y + document.height) * total_zoom,
                ),
            };

            let hadj_val = hadj.value().clamp(h_lower, h_upper);
            let vadj_val = vadj.value().clamp(v_lower, v_upper);

            hadj.configure(
                hadj_val,
                h_lower,
                h_upper,
                0.1 * canvas_size[0],
                0.9 * canvas_size[0],
                canvas_size[0],
            );

            vadj.configure(
                vadj_val,
                v_lower,
                v_upper,
                0.1 * canvas_size[1],
                0.9 * canvas_size[1],
                canvas_size[1],
            );

            // Update the camera
            engine.camera.offset = na::vector![hadj_val, vadj_val];
            engine.camera.size = canvas_size;

            // always update the background rendering
            if let Err(e) = engine.update_background_rendering_current_viewport() {
                log::error!("failed to update background rendering for current viewport in canvas layout allocate, Err: {e:?}");
            }

            let viewport = engine.camera.viewport();
            let old_viewport = self.old_viewport.get();

            // We only extend the viewport by a (tweakable) fraction of the margin, because we want to trigger rendering before we reach it.
            // This has two advantages: Strokes that might take longer to render have a head start while still being out of view,
            // And the rendering gets triggered more often, so not that many strokes start to get rendered. This avoids stutters,
            // because while the rendering itself is on worker threads, we still have to `integrate` the resulted textures,
            // which can also take up quite some time on the main UI thread.
            let old_viewport_extended = old_viewport
                .extend_by(old_viewport.extents() * render::VIEWPORT_EXTENTS_MARGIN_FACTOR * 0.8);
            /*
                       log::debug!(
                           "viewport: {:#?}\nold_viewport_extended: {:#?}",
                           viewport,
                           old_viewport_extended
                       );
            */

            // On zoom outs or viewport translations this will evaluate true, so we render the strokes that are coming into view.
            // But after zoom ins we need to update old_viewport with layout_manager.update_state()
            if !old_viewport_extended.contains(&viewport) {
                // Because we don't set the rendering of strokes that are already in the view dirty, we only render those that may come into the view.
                if let Err(e) = engine.update_rendering_current_viewport() {
                    log::error!("failed to update engine rendering for current viewport in canvas layout allocate, Err: {e:?}");
                }

                self.old_viewport.set(viewport);
            }
        }
    }
}

glib::wrapper! {
    pub(crate) struct CanvasLayout(ObjectSubclass<imp::CanvasLayout>)
        @extends LayoutManager;
}

impl Default for CanvasLayout {
    fn default() -> Self {
        Self::new()
    }
}

impl CanvasLayout {
    pub(crate) const OVERSHOOT_VERTICAL: f64 = 96.0;
    pub(crate) const OVERSHOOT_HORIZONTAL: f64 = 32.0;

    pub(crate) fn new() -> Self {
        glib::Object::new(&[])
    }

    // needs to be called after zooming
    pub(crate) fn update_state(&self, canvas: &RnoteCanvas) {
        self.imp()
            .old_viewport
            .set(canvas.engine().borrow().camera.viewport());
    }
}
