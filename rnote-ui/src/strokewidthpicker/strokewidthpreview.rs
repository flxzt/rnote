// Imports
use gtk4::{
    gdk, glib, graphene, prelude::*, subclass::prelude::*, Align, Orientation, Overflow,
    SizeRequestMode, Widget,
};
use once_cell::sync::Lazy;
use std::cell::Cell;

mod imp {
    use super::*;

    #[derive(Debug)]
    pub(crate) struct RnStrokeWidthPreview {
        stroke_width: Cell<f64>,
    }

    impl Default for RnStrokeWidthPreview {
        fn default() -> Self {
            Self {
                stroke_width: Cell::new(1.0),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for RnStrokeWidthPreview {
        const NAME: &'static str = "RnStrokeWidthPreview";
        type Type = super::RnStrokeWidthPreview;
        type ParentType = Widget;
    }

    impl ObjectImpl for RnStrokeWidthPreview {
        fn constructed(&self) {
            self.parent_constructed();
            let inst = self.instance();

            inst.set_overflow(Overflow::Hidden);
            inst.set_halign(Align::Center);
            inst.set_valign(Align::Center);
        }

        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![glib::ParamSpecDouble::new(
                    "stroke-width",
                    "stroke-width",
                    "stroke-width",
                    0.1,
                    500.0,
                    1.0,
                    glib::ParamFlags::READWRITE,
                )]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            let inst = self.instance();

            match pspec.name() {
                "stroke-width" => {
                    let stroke_width = value.get::<f64>().expect("value not of type `f64`");
                    self.stroke_width.set(stroke_width);
                    inst.queue_draw();
                }
                _ => panic!("invalid property name"),
            }
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "stroke-width" => self.stroke_width.get().to_value(),
                _ => panic!("invalid property name"),
            }
        }
    }

    impl WidgetImpl for RnStrokeWidthPreview {
        fn request_mode(&self) -> SizeRequestMode {
            SizeRequestMode::ConstantSize
        }

        fn measure(&self, orientation: Orientation, _for_size: i32) -> (i32, i32, i32, i32) {
            match orientation {
                Orientation::Horizontal => (0, 32, -1, -1),
                Orientation::Vertical => (0, 32, -1, -1),
                _ => unimplemented!(),
            }
        }

        fn snapshot(&self, snapshot: &gtk4::Snapshot) {
            let inst = self.instance();
            let size = (inst.width() as f32, inst.height() as f32);
            let center = (size.0 * 0.5, size.1 * 0.5);
            let stroke_width = self.stroke_width.get();

            let window_fg_color = inst
                .style_context()
                .lookup_color("window_fg_color")
                .unwrap_or(gdk::RGBA::BLACK);

            // Intentionally a bit larger than half widget size, indicating that it is bigger than what can be displayed
            const MAX_RADIUS: f64 = 22.0;

            // Is asymptotic to MAX_RADIUS
            let circle_radius =
                (MAX_RADIUS * stroke_width * 0.5) / (MAX_RADIUS * 0.5 + stroke_width * 0.5);

            let cairo_cx = snapshot.append_cairo(&graphene::Rect::new(0.0, 0.0, size.0, size.1));
            cairo_cx.set_source_rgba(
                window_fg_color.red() as f64,
                window_fg_color.green() as f64,
                window_fg_color.blue() as f64,
                window_fg_color.alpha() as f64,
            );
            cairo_cx.arc(
                center.0 as f64,
                center.1 as f64,
                circle_radius,
                0.0,
                std::f64::consts::PI * 2.0,
            );
            if let Err(e) = cairo_cx.fill() {
                log::error!("failed to paint stroke width preview, fill returned Err: {e:?}");
            }
        }
    }

    impl RnStrokeWidthPreview {}
}

glib::wrapper! {
    pub(crate) struct RnStrokeWidthPreview(ObjectSubclass<imp::RnStrokeWidthPreview>)
        @extends Widget,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget;
}

impl Default for RnStrokeWidthPreview {
    fn default() -> Self {
        Self::new()
    }
}

impl RnStrokeWidthPreview {
    pub(crate) fn new() -> Self {
        glib::Object::new(&[])
    }

    #[allow(unused)]
    pub(crate) fn stroke_width(&self) -> f64 {
        self.property::<f64>("stroke-width")
    }

    #[allow(unused)]
    pub(crate) fn set_stroke_width(&self, stroke_width: f64) {
        self.set_property("stroke-width", stroke_width.to_value());
    }
}
