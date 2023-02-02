// Imports
use super::RnStrokeWidthPreview;
use gtk4::{glib, prelude::*, subclass::prelude::*, Button, Overflow, ToggleButton, Widget};
use once_cell::sync::Lazy;
use std::cell::Cell;

mod imp {
    use super::*;

    #[derive(Debug)]
    pub(crate) struct RnStrokeWidthSetter {
        preview: RnStrokeWidthPreview,
        stroke_width: Cell<f64>,
    }

    impl Default for RnStrokeWidthSetter {
        fn default() -> Self {
            Self {
                preview: RnStrokeWidthPreview::default(),
                stroke_width: Cell::new(1.0),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for RnStrokeWidthSetter {
        const NAME: &'static str = "RnStrokeWidthSetter";
        type Type = super::RnStrokeWidthSetter;
        type ParentType = ToggleButton;
    }

    impl ObjectImpl for RnStrokeWidthSetter {
        fn constructed(&self) {
            self.parent_constructed();
            let inst = self.instance();

            inst.set_overflow(Overflow::Hidden);
            inst.set_css_classes(&["strokewidthsetter"]);
            inst.set_child(Some(&self.preview));
            inst.bind_property("stroke-width", &self.preview, "stroke-width")
                .sync_create()
                .build();
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

    impl WidgetImpl for RnStrokeWidthSetter {}
    impl ButtonImpl for RnStrokeWidthSetter {}
    impl ToggleButtonImpl for RnStrokeWidthSetter {}

    impl RnStrokeWidthSetter {
        fn update_appearance(&self, stroke_width: f64) {
            let inst = self.instance();

            inst.set_tooltip_text(Some(&format!("{stroke_width:.1}")));
        }
    }
}

glib::wrapper! {
    pub(crate) struct RnStrokeWidthSetter(ObjectSubclass<imp::RnStrokeWidthSetter>)
        @extends ToggleButton, Button, Widget,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget;
}

impl Default for RnStrokeWidthSetter {
    fn default() -> Self {
        Self::new()
    }
}

impl RnStrokeWidthSetter {
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
