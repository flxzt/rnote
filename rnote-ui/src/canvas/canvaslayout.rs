use std::cell::Cell;

use gtk4::{
    glib, prelude::*, subclass::prelude::*, LayoutManager, Orientation, SizeRequestMode, Widget,
};
use p2d::bounding_volume::{Aabb, BoundingVolume};
use rnote_compose::helpers::AabbHelpers;

use crate::canvas::RnoteCanvas;
use rnote_engine::document::Layout;
use rnote_engine::{render, Document};

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

            if orientation == Orientation::Vertical {
                let natural_height = ((canvas.engine().borrow().document.height
                    + 2.0 * Document::SHADOW_WIDTH)
                    * total_zoom)
                    .ceil() as i32;

                (0, natural_height, -1, -1)
            } else {
                let natural_width = ((canvas.engine().borrow().document.width
                    + 2.0 * Document::SHADOW_WIDTH)
                    * total_zoom)
                    .ceil() as i32;

                (0, natural_width, -1, -1)
            }
        }

        fn allocate(&self, widget: &Widget, width: i32, height: i32, _baseline: i32) {
            let canvas = widget.downcast_ref::<RnoteCanvas>().unwrap();
            let hadj = canvas.hadjustment().unwrap();
            let vadj = canvas.vadjustment().unwrap();

            let engine = canvas.engine();
            let mut engine = engine.borrow_mut();
            let total_zoom = engine.camera.total_zoom();
            let doc_layout = engine.doc_layout();

            let new_size = na::vector![f64::from(width), f64::from(height)];

            // Update the adjustments
            let (h_lower, h_upper) = match doc_layout {
                Layout::FixedSize | Layout::ContinuousVertical => (
                    (engine.document.x - Document::SHADOW_WIDTH) * total_zoom,
                    (engine.document.x + engine.document.width + Document::SHADOW_WIDTH)
                        * total_zoom,
                ),
                Layout::Infinite => (
                    engine.document.x * total_zoom,
                    (engine.document.x + engine.document.width) * total_zoom,
                ),
            };

            let (v_lower, v_upper) = match doc_layout {
                Layout::FixedSize | Layout::ContinuousVertical => (
                    (engine.document.y - Document::SHADOW_WIDTH) * total_zoom,
                    (engine.document.y + engine.document.height + Document::SHADOW_WIDTH)
                        * total_zoom,
                ),
                Layout::Infinite => (
                    engine.document.y * total_zoom,
                    (engine.document.y + engine.document.height) * total_zoom,
                ),
            };

            hadj.configure(
                hadj.value(),
                h_lower,
                h_upper,
                0.1 * new_size[0],
                0.9 * new_size[0],
                new_size[0],
            );

            vadj.configure(
                vadj.value(),
                v_lower,
                v_upper,
                0.1 * new_size[1],
                0.9 * new_size[1],
                new_size[1],
            );

            // Update the camera
            engine.camera.offset = na::vector![hadj.value(), vadj.value()];
            engine.camera.size = new_size;

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
