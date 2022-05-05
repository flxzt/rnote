mod imp {
    use gtk4::{
        glib, prelude::*, subclass::prelude::*, LayoutManager, Orientation, SizeRequestMode, Widget,
    };

    use crate::canvas::RnoteCanvas;
    use rnote_engine::sheet::ExpandMode;
    use rnote_engine::Sheet;

    #[derive(Debug, Default)]
    pub struct CanvasLayout {}

    #[glib::object_subclass]
    impl ObjectSubclass for CanvasLayout {
        const NAME: &'static str = "CanvasLayout";
        type Type = super::CanvasLayout;
        type ParentType = LayoutManager;
    }

    impl ObjectImpl for CanvasLayout {}
    impl LayoutManagerImpl for CanvasLayout {
        fn request_mode(&self, _layout_manager: &Self::Type, _widget: &Widget) -> SizeRequestMode {
            SizeRequestMode::ConstantSize
        }

        fn measure(
            &self,
            _layout_manager: &Self::Type,
            widget: &Widget,
            orientation: Orientation,
            _for_size: i32,
        ) -> (i32, i32, i32, i32) {
            let canvas = widget.downcast_ref::<RnoteCanvas>().unwrap();
            let total_zoom = canvas.engine().borrow().camera.total_zoom();

            if orientation == Orientation::Vertical {
                let natural_height = ((canvas.engine().borrow().sheet.height
                    + 2.0 * Sheet::SHADOW_WIDTH)
                    * total_zoom)
                    .ceil() as i32;

                (0, natural_height, -1, -1)
            } else {
                let natural_width = ((canvas.engine().borrow().sheet.width
                    + 2.0 * Sheet::SHADOW_WIDTH)
                    * total_zoom)
                    .ceil() as i32;

                (0, natural_width, -1, -1)
            }
        }

        fn allocate(
            &self,
            _layout_manager: &Self::Type,
            widget: &Widget,
            width: i32,
            height: i32,
            _baseline: i32,
        ) {
            let canvas = widget.downcast_ref::<RnoteCanvas>().unwrap();
            let total_zoom = canvas.engine().borrow().camera.total_zoom();
            let expand_mode = canvas.engine().borrow().expand_mode();

            let hadj = canvas.hadjustment().unwrap();
            let vadj = canvas.vadjustment().unwrap();
            let new_size = na::vector![f64::from(width), f64::from(height)];

            // Update the adjustments
            let (h_lower, h_upper) = match expand_mode {
                ExpandMode::FixedSize | ExpandMode::EndlessVertical => (
                    (canvas.engine().borrow().sheet.x - Sheet::SHADOW_WIDTH) * total_zoom,
                    (canvas.engine().borrow().sheet.x
                        + canvas.engine().borrow().sheet.width
                        + Sheet::SHADOW_WIDTH)
                        * total_zoom,
                ),
                ExpandMode::Infinite => (
                    canvas.engine().borrow().sheet.x * total_zoom,
                    (canvas.engine().borrow().sheet.x + canvas.engine().borrow().sheet.width)
                        * total_zoom,
                ),
            };

            let (v_lower, v_upper) = match canvas.engine().borrow().expand_mode() {
                ExpandMode::FixedSize | ExpandMode::EndlessVertical => (
                    (canvas.engine().borrow().sheet.y - Sheet::SHADOW_WIDTH) * total_zoom,
                    (canvas.engine().borrow().sheet.y
                        + canvas.engine().borrow().sheet.height
                        + Sheet::SHADOW_WIDTH)
                        * total_zoom,
                ),
                ExpandMode::Infinite => (
                    canvas.engine().borrow().sheet.y * total_zoom,
                    (canvas.engine().borrow().sheet.y + canvas.engine().borrow().sheet.height)
                        * total_zoom,
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
            canvas.engine().borrow_mut().camera.offset = na::vector![hadj.value(), vadj.value()];
            canvas.engine().borrow_mut().camera.size = new_size;

            // Update the background rendering
            canvas.engine().borrow_mut().update_background_rendering_current_viewport();
        }
    }
}

use gtk4::{glib, LayoutManager};

glib::wrapper! {
    pub struct CanvasLayout(ObjectSubclass<imp::CanvasLayout>)
        @extends LayoutManager;
}

impl Default for CanvasLayout {
    fn default() -> Self {
        Self::new()
    }
}

impl CanvasLayout {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create CanvasLayout")
    }
}
