// Imports
use gtk4::{
    gdk, glib, prelude::*, subclass::prelude::*, Align, Button, CssProvider, ToggleButton, Widget,
};
use once_cell::sync::Lazy;
use rnote_compose::{color, Color};
use rnote_engine::ext::GdkRGBAExt;
use std::cell::Cell;

mod imp {
    use super::*;

    #[derive(Debug)]
    pub(crate) struct RnColorPad {
        pub(crate) color: Cell<gdk::RGBA>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for RnColorPad {
        const NAME: &'static str = "RnColorPad";
        type Type = super::RnColorPad;
        type ParentType = ToggleButton;
    }

    impl Default for RnColorPad {
        fn default() -> Self {
            Self {
                color: Cell::new(gdk::RGBA::from_compose_color(
                    super::RnColorPad::COLOR_DEFAULT,
                )),
            }
        }
    }

    impl ObjectImpl for RnColorPad {
        fn constructed(&self) {
            let obj = self.obj();
            self.parent_constructed();

            obj.set_hexpand(false);
            obj.set_vexpand(false);
            obj.set_halign(Align::Fill);
            obj.set_valign(Align::Center);
            obj.set_width_request(34);
            obj.set_height_request(34);
            obj.set_css_classes(&["colorpad"]);

            self.update_appearance(super::RnColorPad::COLOR_DEFAULT);
        }

        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> =
                Lazy::new(|| vec![glib::ParamSpecBoxed::builder::<gdk::RGBA>("color").build()]);
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            match pspec.name() {
                "color" => {
                    let color = value
                        .get::<gdk::RGBA>()
                        .expect("value not of type `gdk::RGBA`");
                    self.color.set(color);

                    self.update_appearance(color.into_compose_color());
                }
                _ => panic!("invalid property name"),
            }
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "color" => self.color.get().to_value(),
                _ => panic!("invalid property name"),
            }
        }
    }

    impl WidgetImpl for RnColorPad {}
    impl ButtonImpl for RnColorPad {}
    impl ToggleButtonImpl for RnColorPad {}

    impl RnColorPad {
        fn update_appearance(&self, color: Color) {
            let css = CssProvider::new();

            let colorpad_color = color.to_css_color_attr();
            let colorpad_fg_color = if color.a == 0.0 {
                String::from("@window_fg_color")
            } else if color.luma() < color::FG_LUMINANCE_THRESHOLD {
                String::from("@light_1")
            } else {
                String::from("@dark_5")
            };

            let custom_css = format!(
                "@define-color colorpad_color {colorpad_color}; @define-color colorpad_fg_color {colorpad_fg_color};",
            );
            css.load_from_string(&custom_css);

            // adding custom css is deprecated.
            // TODO: We should refactor to drawing through snapshot().
            // Doing this will also get rid of the css checkerboard glitches that appear on some devices and scaling levels.
            #[allow(deprecated)]
            self.obj()
                .style_context()
                .add_provider(&css, gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION);

            self.obj().queue_draw();
        }
    }
}

glib::wrapper! {
    pub(crate) struct RnColorPad(ObjectSubclass<imp::RnColorPad>)
        @extends ToggleButton, Button, Widget,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget;
}

impl Default for RnColorPad {
    fn default() -> Self {
        Self::new()
    }
}

impl RnColorPad {
    pub(crate) const COLOR_DEFAULT: Color = Color::BLACK;

    pub(crate) fn new() -> Self {
        glib::Object::new()
    }

    #[allow(unused)]
    pub(crate) fn color(&self) -> gdk::RGBA {
        self.property::<gdk::RGBA>("color")
    }

    #[allow(unused)]
    pub(crate) fn set_color(&self, color: gdk::RGBA) {
        self.set_property("color", color.to_value());
    }
}
