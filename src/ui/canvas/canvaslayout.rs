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
    use crate::ui::canvas::Canvas;
    use crate::ui::selectionmodifier;
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
                let natural_height = ((2.0 * canvas.sheet_margin()
                    + f64::from(canvas.sheet().height()))
                    * total_zoom)
                    .round() as i32;

                (0, natural_height, -1, -1)
            } else {
                let natural_width = ((2.0 * canvas.sheet_margin()
                    + f64::from(canvas.sheet().width()))
                    * total_zoom)
                    .round() as i32;

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
            let total_zoom = canvas.total_zoom();
            let sheet_margin_zoomed = canvas.sheet_margin() * total_zoom;

            let hadj = canvas.hadjustment().unwrap();
            hadj.configure(
                hadj.value(),
                0.0,
                (2.0 * canvas.sheet_margin() + canvas.sheet().width() as f64) * total_zoom,
                0.1 * width as f64,
                0.9 * width as f64,
                width as f64,
            );

            let vadj = canvas.vadjustment().unwrap();
            vadj.configure(
                vadj.value(),
                0.0,
                (2.0 * canvas.sheet_margin() + canvas.sheet().height() as f64) * total_zoom,
                0.1 * height as f64,
                0.9 * height as f64,
                height as f64,
            );

            let child = canvas.first_child().unwrap();
            if child
                .type_()
                .is_a(selectionmodifier::SelectionModifier::static_type())
            {
                let selection_modifier = child.downcast_ref::<SelectionModifier>().unwrap();

                // update the selection_modifier internal state before allocation
                selection_modifier.update_state(canvas);

                // Allocate the selection_modifier child
                if let Some(selection_bounds) = selection_modifier.selection_bounds() {
                    let selection_bounds_zoomed =
                        geometry::aabb_scale(selection_bounds, total_zoom);

                    let selection_modifier_x =
                        (sheet_margin_zoomed + selection_bounds_zoomed.mins[0] - hadj.value())
                            .round() as i32
                            - SelectionModifier::RESIZE_NODE_SIZE;
                    let selection_modifier_y =
                        (sheet_margin_zoomed + selection_bounds_zoomed.mins[1] - vadj.value())
                            .round() as i32
                            - SelectionModifier::RESIZE_NODE_SIZE;

                    let (_, selection_modifier_width, _, _) =
                        selection_modifier.measure(Orientation::Horizontal, -1);
                    let (_, selection_modifier_height, _, _) =
                        selection_modifier.measure(Orientation::Vertical, -1);

                    selection_modifier.size_allocate(
                        &gdk::Rectangle::new(
                            selection_modifier_x,
                            selection_modifier_y,
                            selection_modifier_width,
                            selection_modifier_height,
                        ),
                        -1,
                    )
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
