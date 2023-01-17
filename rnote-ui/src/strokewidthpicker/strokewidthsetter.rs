// Imports
use gtk4::{
    glib, prelude::*, subclass::prelude::*, Align, Button, Image, Overflow, ToggleButton, Widget,
};
use once_cell::sync::Lazy;
use std::cell::Cell;

mod imp {
    use super::*;

    #[derive(Debug)]
    pub(crate) struct StrokeWidthSetter {
        image: Image,
        stroke_width: Cell<f64>,
    }

    impl Default for StrokeWidthSetter {
        fn default() -> Self {
            Self {
                image: Image::default(),
                stroke_width: Cell::new(1.0),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for StrokeWidthSetter {
        const NAME: &'static str = "StrokeWidthSetter";
        type Type = super::StrokeWidthSetter;
        type ParentType = ToggleButton;
    }

    impl ObjectImpl for StrokeWidthSetter {
        fn constructed(&self) {
            self.parent_constructed();
            let inst = self.instance();

            inst.set_overflow(Overflow::Hidden);
            inst.set_halign(Align::Center);
            inst.set_valign(Align::Center);
            inst.set_hexpand(false);
            inst.set_vexpand(false);
            inst.set_css_classes(&["strokewidthsetter"]);

            self.image.add_css_class("strokewidthsetterimage");
            self.image.set_icon_name(Some("strokewidthsetter-symbolic"));
            inst.set_child(Some(&self.image));

            self.update_appearance(self.stroke_width.get());
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
            match pspec.name() {
                "stroke-width" => {
                    let stroke_width = value.get::<f64>().expect("value not of type `f64`");
                    self.stroke_width.set(stroke_width);
                    self.update_appearance(stroke_width);
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

    impl WidgetImpl for StrokeWidthSetter {}
    impl ButtonImpl for StrokeWidthSetter {}
    impl ToggleButtonImpl for StrokeWidthSetter {}

    impl StrokeWidthSetter {
        fn update_appearance(&self, stroke_width: f64) {
            let inst = self.instance();

            inst.set_tooltip_text(Some(&format!("{stroke_width:.1}")));

            // The max size is 24
            const IMAGE_MAX_SIZE: f64 = 24.0;

            // Is asymptotic to IMAGE_MAX_SIZE
            let pixel_size =
                (IMAGE_MAX_SIZE * stroke_width) / (IMAGE_MAX_SIZE * 0.5 + stroke_width);
            self.image
                .set_pixel_size(pixel_size as i32 + pixel_size as i32 % 2);
        }
    }
}

glib::wrapper! {
    pub(crate) struct StrokeWidthSetter(ObjectSubclass<imp::StrokeWidthSetter>)
        @extends ToggleButton, Button, Widget,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget;
}

impl Default for StrokeWidthSetter {
    fn default() -> Self {
        Self::new()
    }
}

impl StrokeWidthSetter {
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
