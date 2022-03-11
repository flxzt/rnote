mod imp {
    use gtk4::gdk;
    use gtk4::glib;
    use gtk4::prelude::*;
    use gtk4::subclass::prelude::*;
    use gtk4::LayoutManager;
    use gtk4::Orientation;
    use gtk4::SizeRequestMode;
    use gtk4::Widget;

    use crate::canvas::Canvas;
    use crate::selectionmodifier::SelectionModifier;
    use rnote_engine::compose::geometry::AABBHelpers;

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
            let canvas = widget.downcast_ref::<Canvas>().unwrap();
            let canvas_priv = canvas.imp();
            let total_zoom = canvas.total_zoom();

            let hadj = canvas.hadjustment().unwrap();
            let vadj = canvas.vadjustment().unwrap();

            // Update the adjustments
            canvas.update_adj_config(na::vector![f64::from(width), f64::from(height)]);
            canvas.update_background_rendernode(true);

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
                        selection_bounds.scale(na::Vector2::from_element(total_zoom));

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
