use gtk4::{gdk, glib, Button, ToggleButton, Widget};

mod imp {
    use std::cell::Cell;

    use gtk4::{
        gdk, glib, glib::translate::IntoGlib, prelude::*, subclass::prelude::*, CssProvider,
        PositionType, ToggleButton,
    };
    use once_cell::sync::Lazy;

    #[derive(Debug)]
    pub struct ColorSetter {
        pub css: CssProvider,
        pub color: Cell<gdk::RGBA>,
        pub position: Cell<PositionType>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ColorSetter {
        const NAME: &'static str = "ColorSetter";
        type Type = super::ColorSetter;
        type ParentType = ToggleButton;
    }

    impl Default for ColorSetter {
        fn default() -> Self {
            Self {
                css: CssProvider::new(),
                color: Cell::new(super::ColorSetter::COLOR_DEFAULT),
                position: Cell::new(PositionType::Right),
            }
        }
    }

    impl ObjectImpl for ColorSetter {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpec::new_boxed(
                        "color",
                        "color",
                        "color",
                        gdk::RGBA::static_type().into(),
                        glib::ParamFlags::READWRITE,
                    ),
                    glib::ParamSpec::new_enum(
                        // Name
                        "position",
                        // Nickname
                        "position",
                        // Short description
                        "position",
                        // Enum type
                        PositionType::static_type(),
                        // Default value
                        PositionType::Right.into_glib(),
                        // The property can be read and written to
                        glib::ParamFlags::READWRITE,
                    ),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(
            &self,
            _obj: &Self::Type,
            _id: usize,
            value: &glib::Value,
            pspec: &glib::ParamSpec,
        ) {
            match pspec.name() {
                "color" => {
                    let color = value
                        .get::<gdk::RGBA>()
                        .expect("value not of type `gdk::RGBA`");
                    self.color.set(color);
                    self.css.load_from_data(
                        self.parse_gdk_rgba(&color, self.position.get()).as_bytes(),
                    );
                }
                "position" => {
                    let position = value
                        .get::<PositionType>()
                        .expect("value not of type `PositionType`");
                    let color = self.color.get();

                    self.position.replace(position);
                    self.css
                        .load_from_data(self.parse_gdk_rgba(&color, position).as_bytes());
                }
                _ => panic!("invalid property name"),
            }
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "color" => self.color.get().to_value(),
                "position" => self.position.get().to_value(),
                _ => panic!("invalid property name"),
            }
        }

        fn constructed(&self, obj: &Self::Type) {
            obj.set_css_classes(&["setter-button"]);
            self.css.load_from_data(
                self.parse_gdk_rgba(&super::ColorSetter::COLOR_DEFAULT, self.position.get())
                    .as_bytes(),
            );
            obj.style_context()
                .add_provider(&self.css, gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION);
        }
    }

    impl WidgetImpl for ColorSetter {}

    impl ButtonImpl for ColorSetter {}

    impl ToggleButtonImpl for ColorSetter {}

    impl ColorSetter {
        fn parse_gdk_rgba(&self, rgba: &gdk::RGBA, position: PositionType) -> String {
            let pos_string: String = String::from(match position {
                PositionType::Left => "right",
                PositionType::Right => "left",
                PositionType::Top => "bottom",
                PositionType::Bottom => "top",
                _ => "left",
            });
            let parsed = format!(
                "
.setter-button {{
    padding: 0 0 0 0;
    margin-{0}: 10px;
    background: rgba({1}, {2}, {3}, {4:.2});

    transition: margin-{0} 0.3s;
}}

.setter-button:checked {{
    margin-{0}: 0px;
}}
",
                pos_string,
                (rgba.red * 255.0) as i32,
                (rgba.green * 255.0) as i32,
                (rgba.blue * 255.0) as i32,
                (rgba.alpha * 1000.0).round() / 1000.0
            );
            parsed
        }
    }
}

glib::wrapper! {
    pub struct ColorSetter(ObjectSubclass<imp::ColorSetter>)
        @extends ToggleButton, Button, Widget;
}

impl ColorSetter {
    pub const COLOR_DEFAULT: gdk::RGBA = gdk::RGBA {
        red: 0.0,
        green: 0.0,
        blue: 0.0,
        alpha: 1.0,
    };
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("failed to create `ColorSetter")
    }
}

impl Default for ColorSetter {
    fn default() -> Self {
        Self::new()
    }
}
