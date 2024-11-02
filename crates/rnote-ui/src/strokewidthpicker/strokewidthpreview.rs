// Imports
use super::StrokeWidthPreviewStyle;
use gtk4::{
    gdk, glib, graphene, prelude::*, subclass::prelude::*, Align, Orientation, Overflow,
    SizeRequestMode, Widget,
};
use once_cell::sync::Lazy;
use std::cell::Cell;
use tracing::error;

mod imp {
    use super::*;

    #[derive(Debug)]
    pub(crate) struct RnStrokeWidthPreview {
        stroke_width: Cell<f64>,
        preview_style: Cell<StrokeWidthPreviewStyle>,
    }

    impl Default for RnStrokeWidthPreview {
        fn default() -> Self {
            Self {
                stroke_width: Cell::new(1.0),
                preview_style: Cell::new(StrokeWidthPreviewStyle::Circle),
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
            let obj = self.obj();

            obj.set_overflow(Overflow::Hidden);
            obj.set_halign(Align::Center);
            obj.set_valign(Align::Center);
        }

        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpecDouble::builder("stroke-width")
                        .minimum(0.1)
                        .maximum(500.0)
                        .default_value(1.0)
                        .build(),
                    glib::ParamSpecEnum::builder::<StrokeWidthPreviewStyle>("preview-style")
                        .default_value(StrokeWidthPreviewStyle::Circle)
                        .build(),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            let obj = self.obj();

            match pspec.name() {
                "stroke-width" => {
                    let stroke_width = value.get::<f64>().expect("value not of type `f64`");
                    self.stroke_width.set(stroke_width);
                    obj.queue_draw();
                }
                "preview-style" => {
                    let preview_style = value
                        .get::<StrokeWidthPreviewStyle>()
                        .expect("value not of type `StrokeWidthPreviewStyle`");
                    self.preview_style.set(preview_style);
                    obj.queue_draw();
                }
                _ => panic!("invalid property name"),
            }
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "stroke-width" => self.stroke_width.get().to_value(),
                "preview-style" => self.preview_style.get().to_value(),
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
            let obj = self.obj();
            let (width, height) = (obj.width() as f32, obj.height() as f32);
            let (center_x, center_y) = (width * 0.5, height * 0.5);
            let stroke_width = self.stroke_width.get();

            // accessing colors through the style context is deprecated,
            // but this needs new color API to fetch theme colors.
            #[allow(deprecated)]
            let window_fg_color = obj
                .style_context()
                .lookup_color("window_fg_color")
                .unwrap_or(gdk::RGBA::BLACK);

            match self.preview_style.get() {
                StrokeWidthPreviewStyle::Circle => {
                    // Intentionally a bit larger than half widget size, indicating that it is bigger than what can be displayed
                    const MAX_RADIUS: f64 = 22.0;

                    // Is asymptotic to MAX_RADIUS
                    let circle_radius =
                        (MAX_RADIUS * stroke_width * 0.5) / (MAX_RADIUS * 0.5 + stroke_width * 0.5);

                    let cairo_cx =
                        snapshot.append_cairo(&graphene::Rect::new(0.0, 0.0, width, height));
                    cairo_cx.set_source_rgba(
                        window_fg_color.red() as f64,
                        window_fg_color.green() as f64,
                        window_fg_color.blue() as f64,
                        window_fg_color.alpha() as f64,
                    );
                    cairo_cx.arc(
                        center_x as f64,
                        center_y as f64,
                        circle_radius,
                        0.0,
                        std::f64::consts::PI * 2.0,
                    );
                    if let Err(e) = cairo_cx.fill() {
                        error!(
                            "failed to paint stroke width preview in style `Circle`, fill returned Err: {e:?}"
                        );
                    }
                }
                StrokeWidthPreviewStyle::RoundedRect => {
                    const MAX_HALF_EXTENTS: f64 = 16.0;
                    let square_half_extents = (MAX_HALF_EXTENTS * stroke_width * 0.5)
                        / (MAX_HALF_EXTENTS * 0.5 + stroke_width * 0.5);

                    let cairo_cx =
                        snapshot.append_cairo(&graphene::Rect::new(0.0, 0.0, width, height));
                    cairo_cx.set_source_rgba(
                        window_fg_color.red() as f64,
                        window_fg_color.green() as f64,
                        window_fg_color.blue() as f64,
                        window_fg_color.alpha() as f64,
                    );
                    cairo_rounded_rect(
                        &cairo_cx,
                        width as f64 * 0.5 - square_half_extents,
                        height as f64 * 0.5 - square_half_extents,
                        square_half_extents * 2.0,
                        square_half_extents * 2.0,
                        3.0,
                    );
                    if let Err(e) = cairo_cx.fill() {
                        error!(
                            "failed to paint stroke width preview in style `RoundedRect`, fill returned Err: {e:?}"
                        );
                    }
                }
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
        glib::Object::new()
    }

    #[allow(unused)]
    pub(crate) fn stroke_width(&self) -> f64 {
        self.property::<f64>("stroke-width")
    }

    #[allow(unused)]
    pub(crate) fn set_stroke_width(&self, stroke_width: f64) {
        self.set_property("stroke-width", stroke_width.to_value());
    }

    #[allow(unused)]
    pub(crate) fn preview_style(&self) -> StrokeWidthPreviewStyle {
        self.property::<StrokeWidthPreviewStyle>("preview-style")
    }

    #[allow(unused)]
    pub(crate) fn set_preview_style(&self, preview_style: f64) {
        self.set_property("preview-style", preview_style.to_value());
    }
}

fn cairo_rounded_rect(cairo_cx: &cairo::Context, x: f64, y: f64, w: f64, h: f64, r: f64) {
    cairo_cx.new_sub_path();
    cairo_cx.arc(
        x + r,
        y + r,
        r,
        std::f64::consts::PI,
        3.0 * std::f64::consts::PI / 2.0,
    );
    cairo_cx.arc(
        x + w - r,
        y + r,
        r,
        3.0 * std::f64::consts::PI / 2.0,
        2.0 * std::f64::consts::PI,
    );
    cairo_cx.arc(x + w - r, y + h - r, r, 0.0, std::f64::consts::PI / 2.0);
    cairo_cx.arc(
        x + r,
        y + h - r,
        r,
        std::f64::consts::PI / 2.0,
        std::f64::consts::PI,
    );
    cairo_cx.close_path();
}
