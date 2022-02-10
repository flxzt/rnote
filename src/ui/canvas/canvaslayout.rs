mod imp {
    use gtk4::gdk;
    use gtk4::glib;
    use gtk4::prelude::*;
    use gtk4::subclass::prelude::*;
    use gtk4::LayoutManager;
    use gtk4::Orientation;
    use gtk4::SizeRequestMode;
    use gtk4::Widget;

    use crate::compose::geometry;
    use crate::ui::canvas::{Canvas, ExpandMode};
    use crate::ui::selectionmodifier::SelectionModifier;

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
            let canvas = widget.downcast_ref::<Canvas>().unwrap();
            let total_zoom = canvas.zoom() * canvas.temporary_zoom();

            if orientation == Orientation::Vertical {
                let natural_height = ((canvas.sheet().borrow().height + 2.0 * Canvas::SHADOW_WIDTH)
                    * total_zoom)
                    .ceil() as i32;

                (0, natural_height, -1, -1)
            } else {
                let natural_width = ((canvas.sheet().borrow().width + 2.0 * Canvas::SHADOW_WIDTH)
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
            let width = f64::from(width);
            let height = f64::from(height);
            let canvas = widget.downcast_ref::<Canvas>().unwrap();
            let canvas_priv = canvas.imp();
            let total_zoom = canvas.total_zoom();

            let hadj = canvas.hadjustment().unwrap();

            let (h_lower, h_upper) = match canvas.expand_mode() {
                ExpandMode::FixedSize => (
                    (canvas.sheet().borrow().x - Canvas::SHADOW_WIDTH) * total_zoom,
                    (canvas.sheet().borrow().x
                        + canvas.sheet().borrow().width
                        + Canvas::SHADOW_WIDTH)
                        * total_zoom,
                ),
                ExpandMode::EndlessVertical => (
                    (canvas.sheet().borrow().x - Canvas::SHADOW_WIDTH) * total_zoom,
                    (canvas.sheet().borrow().x
                        + canvas.sheet().borrow().width
                        + Canvas::SHADOW_WIDTH)
                        * total_zoom,
                ),
                ExpandMode::Infinite => (
                    canvas.sheet().borrow().x * total_zoom,
                    (canvas.sheet().borrow().x + canvas.sheet().borrow().width) * total_zoom,
                ),
            };

            let vadj = canvas.vadjustment().unwrap();

            let (v_lower, v_upper) = match canvas.expand_mode() {
                ExpandMode::FixedSize => (
                    (canvas.sheet().borrow().y - Canvas::SHADOW_WIDTH) * total_zoom,
                    (canvas.sheet().borrow().y
                        + canvas.sheet().borrow().height
                        + Canvas::SHADOW_WIDTH)
                        * total_zoom,
                ),
                ExpandMode::EndlessVertical => (
                    (canvas.sheet().borrow().y - Canvas::SHADOW_WIDTH) * total_zoom,
                    (canvas.sheet().borrow().y
                        + canvas.sheet().borrow().height
                        + Canvas::SHADOW_WIDTH)
                        * total_zoom,
                ),
                ExpandMode::Infinite => (
                    canvas.sheet().borrow().y * total_zoom,
                    (canvas.sheet().borrow().y + canvas.sheet().borrow().height) * total_zoom,
                ),
            };

            hadj.configure(
                hadj.value(),
                h_lower,
                h_upper,
                0.1 * width,
                0.9 * width,
                width,
            );

            vadj.configure(
                vadj.value(),
                v_lower,
                v_upper,
                0.1 * height,
                0.9 * height,
                height,
            );

            canvas.update_size_autoexpand();

            // Allocate the selection_modifier child
            {
                canvas_priv
                    .selection_modifier
                    .update_translate_node_size_request(&canvas);

                let (_, selection_modifier_width, _, _) = canvas_priv
                    .selection_modifier
                    .measure(Orientation::Horizontal, -1);
                let (_, selection_modifier_height, _, _) = canvas_priv
                    .selection_modifier
                    .measure(Orientation::Vertical, -1);

                let (selection_modifier_x, selection_modifier_y) = if let Some(selection_bounds) =
                    canvas_priv.selection_modifier.selection_bounds()
                {
                    let selection_bounds_zoomed =
                        geometry::aabb_scale(selection_bounds, total_zoom);

                    (
                        (selection_bounds_zoomed.mins[0] - hadj.value()).ceil() as i32
                            - SelectionModifier::RESIZE_NODE_SIZE,
                        (selection_bounds_zoomed.mins[1] - vadj.value()).ceil() as i32
                            - SelectionModifier::RESIZE_NODE_SIZE,
                    )
                } else {
                    (0, 0)
                };

                canvas_priv.selection_modifier.size_allocate(
                    &gdk::Rectangle::new(
                        selection_modifier_x,
                        selection_modifier_y,
                        selection_modifier_width,
                        selection_modifier_height,
                    ),
                    -1,
                );
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
