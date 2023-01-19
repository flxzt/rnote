use std::cell::Cell;

use gtk4::{
    gdk, glib, prelude::*, subclass::prelude::*, Align, Button, CssProvider, ToggleButton, Widget,
};
use once_cell::sync::Lazy;
use rnote_compose::{color, Color};
use rnote_engine::utils::GdkRGBAHelpers;

mod imp {
    use super::*;

    #[derive(Debug)]
    pub(crate) struct ColorPad {
        pub(crate) css: CssProvider,
        pub(crate) color: Cell<gdk::RGBA>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ColorPad {
        const NAME: &'static str = "ColorPad";
        type Type = super::ColorPad;
        type ParentType = ToggleButton;
    }

    impl Default for ColorPad {
        fn default() -> Self {
            Self {
                css: CssProvider::new(),
                color: Cell::new(gdk::RGBA::from_compose_color(
                    super::ColorPad::COLOR_DEFAULT,
                )),
            }
        }
    }

    impl ObjectImpl for ColorPad {
        fn constructed(&self) {
            let inst = self.instance();
            self.parent_constructed();

            inst.set_hexpand(false);
            inst.set_vexpand(false);
            inst.set_halign(Align::Fill);
            inst.set_valign(Align::Center);
            inst.set_width_request(34);
            inst.set_height_request(34);
            inst.set_css_classes(&["colorpad"]);

            self.update_appearance(super::ColorPad::COLOR_DEFAULT);
            inst.style_context()
                .add_provider(&self.css, gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION);
        }

        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![glib::ParamSpecBoxed::new(
                    "color",
                    "color",
                    "color",
                    gdk::RGBA::static_type(),
                    glib::ParamFlags::READWRITE,
                )]
            });
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

    impl WidgetImpl for ColorPad {}
    impl ButtonImpl for ColorPad {}
    impl ToggleButtonImpl for ColorPad {}

    impl ColorPad {
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
            css.load_from_data(custom_css.as_bytes());

            self.instance()
                .style_context()
                .add_provider(&css, gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION);

            self.instance().queue_draw();
        }
    }
}

glib::wrapper! {
    pub(crate) struct ColorPad(ObjectSubclass<imp::ColorPad>)
        @extends ToggleButton, Button, Widget,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget;
}

impl Default for ColorPad {
    fn default() -> Self {
        Self::new()
    }
}

impl ColorPad {
    pub(crate) const COLOR_DEFAULT: Color = Color::BLACK;

    pub(crate) fn new() -> Self {
        glib::Object::new(&[])
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
