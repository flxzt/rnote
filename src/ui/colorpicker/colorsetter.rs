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
                color: Cell::new(super::ColorSetter::COLOR_DEFAULT.to_gdk()),
                position: Cell::new(PositionType::Right),
            }
        }
    }

    impl ObjectImpl for ColorSetter {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            obj.set_css_classes(&["setter-button"]);
            self.css.load_from_data(
                self.generate_css_string(
                    &super::ColorSetter::COLOR_DEFAULT.to_gdk(),
                    self.position.get(),
                )
                .as_bytes(),
            );
            obj.style_context()
                .add_provider(&self.css, gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION);
        }

        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpecBoxed::new(
                        "color",
                        "color",
                        "color",
                        gdk::RGBA::static_type(),
                        glib::ParamFlags::READWRITE,
                    ),
                    glib::ParamSpecEnum::new(
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
                        self.generate_css_string(&color, self.position.get())
                            .as_bytes(),
                    );
                }
                "position" => {
                    let position = value
                        .get::<PositionType>()
                        .expect("value not of type `PositionType`");
                    let color = self.color.get();

                    self.position.replace(position);
                    self.css
                        .load_from_data(self.generate_css_string(&color, position).as_bytes());
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
    }

    impl WidgetImpl for ColorSetter {}

    impl ButtonImpl for ColorSetter {}

    impl ToggleButtonImpl for ColorSetter {}

    impl ColorSetter {
        fn generate_css_string(&self, rgba: &gdk::RGBA, position: PositionType) -> String {
            // Watch out for inverse
            let position_string: String = String::from(match position {
                PositionType::Left => "-right",
                PositionType::Right => "-left",
                PositionType::Top => "-bottom",
                PositionType::Bottom => "-top",
                _ => "",
            });
            let properties_string: String = String::from(match position {
                PositionType::Left => {
                    "
    border-top-left-radius: 0px;
    border-bottom-left-radius: 0px;
"
                }
                PositionType::Right => {
                    "
    border-top-right-radius: 0px;
    border-bottom-right-radius: 0px;
"
                }
                PositionType::Top => {
                    "
    border-top-left-radius: 0px;
    border-top-right-radius: 0px;
"
                }
                PositionType::Bottom => {
                    "
    border-bottom-left-radius: 0px;
    border-bottom-right-radius: 0px;
"
                }
                _ => "",
            });
            let properties_checked_string: String = String::from(match position {
                PositionType::Left => "border-radius: 0px 5px 5px 0px;",
                PositionType::Right => "border-radius: 5px 0px 0px 5px;",
                PositionType::Top => "border-radius: 0px 0px 5px 5px;",
                PositionType::Bottom => "border-radius: 5px 5px 0px 0px;",
                _ => "",
            });
            let parsed = format!(
                "
.setter-button {{
    padding: 0 0 0 0;
    background-blend-mode: screen;
    background-image:
        linear-gradient(45deg, #11111188 25%, transparent 25%, transparent 75%, #11111188 75%, #11111188),
        linear-gradient(45deg, #11111188 25%, transparent 25%, transparent 75%, #11111188 75%, #11111188);
    background-size: 20px 20px; background-position: 0px 0px, 10px 10px;
    border-color: @borders;
    border-style: solid;
    border-width: 1px;
    filter: brightness(100%);
    margin{0}: 10px;
    background-color: rgba({3}, {4}, {5}, {6:.3});
    transition: margin{0} 0.15s ease-out, border-radius 0.15s ease-out, filter 0.15s ease-out;
    {1}
}}

.setter-button:checked {{
    margin{0}: 0px;
    {2}
}}

.setter-button:hover {{
    filter: brightness(90%);
}}
",
                position_string,
                properties_string,
                properties_checked_string,
                (rgba.red() * 255.0) as i32,
                (rgba.green() * 255.0) as i32,
                (rgba.blue() * 255.0) as i32,
                (rgba.alpha() * 1000.0).round() / 1000.0
            );
            parsed
        }
    }
}

use gtk4::{gdk, glib, prelude::*, Button, PositionType, ToggleButton, Widget};

use crate::compose::color::Color;

glib::wrapper! {
    pub struct ColorSetter(ObjectSubclass<imp::ColorSetter>)
        @extends ToggleButton, Button, Widget;
}

impl Default for ColorSetter {
    fn default() -> Self {
        Self::new()
    }
}

impl ColorSetter {
    pub const COLOR_DEFAULT: Color = Color::BLACK;

    pub fn new() -> Self {
        glib::Object::new(&[]).expect("failed to create `ColorSetter")
    }

    pub fn position(&self) -> PositionType {
        self.property::<PositionType>("position")
    }

    pub fn set_position(&self, position: PositionType) {
        self.set_property("position", position.to_value());
    }

    pub fn color(&self) -> gdk::RGBA {
        self.property::<gdk::RGBA>("color")
    }

    pub fn set_color(&self, color: gdk::RGBA) {
        self.set_property("color", color.to_value());
    }
}
