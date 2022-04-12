mod imp {
    use gtk4::glib;
    use gtk4::prelude::*;
    use gtk4::subclass::prelude::*;
    use gtk4::LayoutManager;
    use gtk4::Orientation;
    use gtk4::SizeRequestMode;
    use gtk4::Widget;
    use p2d::bounding_volume::{BoundingVolume, AABB};
    use rnote_engine::engine::ExpandMode;
    use rnote_engine::Sheet;

    use crate::canvas::RnoteCanvas;
    use rnote_compose::helpers::AABBHelpers;

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

            // Update content and background
            canvas.update_background_rendernodes(false);
            canvas.regenerate_content(false, true);

            let viewport = canvas.engine().borrow().camera.viewport();
            match expand_mode {
                ExpandMode::FixedSize | ExpandMode::EndlessVertical => {}
                ExpandMode::Infinite => {
                    // Show "return to center" toast when far away in infinite mode
                    let threshold_bounds = AABB::new(
                        na::point![0.0, 0.0],
                        na::point![
                            canvas.engine().borrow().sheet.format.width,
                            canvas.engine().borrow().sheet.format.height
                        ],
                    )
                    .extend_by(na::vector![
                        2.0 * canvas.engine().borrow().sheet.format.width,
                        2.0 * canvas.engine().borrow().sheet.format.height
                    ]);

                    if !viewport.intersects(&threshold_bounds) {
                        canvas.show_return_to_center_toast()
                    } else {
                        canvas.dismiss_return_to_center_toast();
                    }
                }
            }
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
