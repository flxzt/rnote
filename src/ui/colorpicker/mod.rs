pub mod colorsetter;

mod imp {
    use super::colorsetter::ColorSetter;

    use std::cell::{Cell, RefCell};
    use std::rc::Rc;

    use gtk4::{
        gdk, glib, glib::clone, glib::translate::IntoGlib, prelude::*, subclass::prelude::*, Box,
        Button, ColorChooserWidget, CompositeTemplate, MenuButton, Orientation, Popover,
        PositionType, Widget,
    };
    use gtk4::{Align, BoxLayout};

    use once_cell::sync::Lazy;

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/com/github/flxzt/rnote/ui/colorpicker.ui")]
    pub struct ColorPicker {
        #[template_child]
        pub setterbox: TemplateChild<Box>,
        #[template_child]
        pub currentcolor_setter1: TemplateChild<ColorSetter>,
        #[template_child]
        pub colorpicker_button: TemplateChild<MenuButton>,
        #[template_child]
        pub colorpicker_popover: TemplateChild<Popover>,
        #[template_child]
        pub colorchooser: TemplateChild<ColorChooserWidget>,
        #[template_child]
        pub colorchooser_editor_gobackbutton: TemplateChild<Button>,
        #[template_child]
        pub colorchooser_editor_selectbutton: TemplateChild<Button>,

        pub position: Cell<PositionType>,
        pub amount_colorbuttons: Cell<u32>,
        pub currentcolor_setters: Rc<RefCell<Vec<ColorSetter>>>,
        pub current_color: Cell<gdk::RGBA>,
    }

    impl Default for ColorPicker {
        fn default() -> Self {
            Self {
                setterbox: TemplateChild::<Box>::default(),
                currentcolor_setter1: TemplateChild::<ColorSetter>::default(),
                colorpicker_button: TemplateChild::<MenuButton>::default(),
                colorpicker_popover: TemplateChild::<Popover>::default(),
                colorchooser: TemplateChild::<ColorChooserWidget>::default(),
                colorchooser_editor_gobackbutton: TemplateChild::<Button>::default(),
                colorchooser_editor_selectbutton: TemplateChild::<Button>::default(),
                position: Cell::new(PositionType::Right),
                amount_colorbuttons: Cell::new(super::ColorPicker::AMOUNT_COLORBUTTONS_DEFAULT),
                current_color: Cell::new(super::ColorPicker::COLOR_DEFAULT),
                currentcolor_setters: Rc::new(RefCell::new(Vec::with_capacity(
                    super::ColorPicker::AMOUNT_COLORBUTTONS_DEFAULT as usize,
                ))),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ColorPicker {
        const NAME: &'static str = "ColorPicker";
        type Type = super::ColorPicker;
        type ParentType = Widget;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ColorPicker {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);

            let colorchooser = self.colorchooser.get();
            let currentcolor_setter1 = self.currentcolor_setter1.get();
            let colorpicker_popover = self.colorpicker_popover.get();
            let colorchooser_editor_gobackbutton = self.colorchooser_editor_gobackbutton.get();

            self.currentcolor_setter1.connect_clicked(
                clone!(@weak obj => move |currentcolor_setter1| {
                    let color = currentcolor_setter1.property("color").unwrap().get::<gdk::RGBA>().unwrap();
                    obj.set_property("current-color", &color.to_value()).expect("settings `color` property");
                }),
            );

            self.colorpicker_button
                .set_popover(Some(&colorpicker_popover));

            self.colorchooser.connect_show_editor_notify(
                clone!(@weak colorchooser_editor_gobackbutton => move |_colorchooser| {
                    colorchooser_editor_gobackbutton.set_visible(true);
                }),
            );

            self.colorchooser_editor_selectbutton.connect_clicked(
                clone!(@weak colorpicker_popover => move |_colorchooser_editor_selectbutton| {
                    colorpicker_popover.popdown();
                }),
            );

            self.colorchooser_editor_gobackbutton.connect_clicked(
                clone!(@weak colorchooser => move |colorchooser_editor_gobackbutton| {
                    colorchooser.set_show_editor(false);
                    colorchooser_editor_gobackbutton.set_visible(false);
                }),
            );

            self.colorchooser.connect_rgba_notify(clone!(@weak obj, @weak currentcolor_setter1, @weak self.currentcolor_setters as currentcolor_setters => move |colorchooser| {
                let color = colorchooser.rgba();
                obj.set_property("current-color", &color.to_value()).expect("settings `color` property");

            }));

            obj.connect_notify_local(Some("current-color"), clone!(@weak currentcolor_setter1, @weak self.currentcolor_setters as currentcolor_setters => move |obj, _param| {
                let current_color = obj.current_color();

                // store color in the buttons
                if currentcolor_setter1.is_active() {
                    currentcolor_setter1.set_property("color", &current_color.to_value()).expect("settings `color` property");
                } else {
                    for setter_button in &*currentcolor_setters.borrow() {
                        if setter_button.is_active() {
                            setter_button.set_property("color", &current_color.to_value()).expect("settings `color` property");
                        }
                    }
                }
            }));
        }

        fn dispose(&self, obj: &Self::Type) {
            while let Some(child) = obj.first_child() {
                child.unparent();
            }
        }

        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
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
                    glib::ParamSpec::new_uint(
                        "amount-colorbuttons",
                        "amount-colorbuttons",
                        "The amount of colorbuttons shown",
                        super::ColorPicker::AMOUNT_COLORBUTTONS_MIN,
                        super::ColorPicker::AMOUNT_COLORBUTTONS_MAX,
                        super::ColorPicker::AMOUNT_COLORBUTTONS_DEFAULT,
                        glib::ParamFlags::READWRITE,
                    ),
                    glib::ParamSpec::new_boxed(
                        "current-color",
                        "current-color",
                        "current-color",
                        gdk::RGBA::static_type(),
                        glib::ParamFlags::READWRITE,
                    ),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(
            &self,
            obj: &Self::Type,
            _id: usize,
            value: &glib::Value,
            pspec: &glib::ParamSpec,
        ) {
            match pspec.name() {
                "position" => {
                    let layout_manager = obj
                        .layout_manager()
                        .unwrap()
                        .downcast::<BoxLayout>()
                        .unwrap();

                    let position = value
                        .get::<PositionType>()
                        .expect("value not of type `PositionType`");
                    self.position.replace(position);

                    self.currentcolor_setter1
                        .set_property("position", position)
                        .unwrap();
                    for setter_button in self.currentcolor_setters.borrow().iter() {
                        setter_button.set_property("position", position).unwrap();
                    }

                    match position {
                        PositionType::Left => {
                            self.colorpicker_popover.set_position(PositionType::Right);
                            layout_manager.set_orientation(Orientation::Vertical);
                            self.setterbox.set_orientation(Orientation::Vertical);
                            self.colorpicker_button.set_margin_start(0);
                            self.colorpicker_button.set_margin_end(0);
                            self.colorpicker_button.set_margin_top(6);
                            self.colorpicker_button.set_margin_bottom(0);
                        }
                        PositionType::Right => {
                            self.colorpicker_popover.set_position(PositionType::Left);
                            layout_manager.set_orientation(Orientation::Vertical);
                            self.setterbox.set_orientation(Orientation::Vertical);
                            self.colorpicker_button.set_margin_start(0);
                            self.colorpicker_button.set_margin_end(0);
                            self.colorpicker_button.set_margin_top(6);
                            self.colorpicker_button.set_margin_bottom(0);
                        }
                        PositionType::Top => {
                            self.colorpicker_popover.set_position(PositionType::Bottom);
                            layout_manager.set_orientation(Orientation::Horizontal);
                            self.setterbox.set_orientation(Orientation::Horizontal);
                            self.colorpicker_button.set_margin_start(6);
                            self.colorpicker_button.set_margin_end(0);
                            self.colorpicker_button.set_margin_top(0);
                            self.colorpicker_button.set_margin_bottom(0);
                        }
                        PositionType::Bottom => {
                            self.colorpicker_popover.set_position(PositionType::Top);
                            layout_manager.set_orientation(Orientation::Horizontal);
                            self.setterbox.set_orientation(Orientation::Horizontal);
                            self.colorpicker_button.set_margin_start(6);
                            self.colorpicker_button.set_margin_end(0);
                            self.colorpicker_button.set_margin_top(0);
                            self.colorpicker_button.set_margin_bottom(0);
                        }
                        _ => {}
                    }
                }
                "amount-colorbuttons" => {
                    self.amount_colorbuttons
                        .set(value.get::<u32>().expect("value not of type `u32`"));
                    self.init_currentcolor_setters(obj);
                }
                "current-color" => {
                    self.current_color.set(
                        value
                            .get::<gdk::RGBA>()
                            .expect("value not of type `gdk::RGBA`"),
                    );
                }
                _ => panic!("invalid property name"),
            }
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "position" => self.position.get().to_value(),
                "amount-colorbuttons" => self.amount_colorbuttons.get().to_value(),
                "current-color" => self.current_color.get().to_value(),
                _ => panic!("invalid property name"),
            }
        }
    }

    impl WidgetImpl for ColorPicker {}

    impl ColorPicker {
        fn init_currentcolor_setters(&self, obj: &<Self as ObjectSubclass>::Type) {
            let setterbox = &*self.setterbox;
            let currentcolor_setter1 = &*self.currentcolor_setter1;

            for setter_button in &*self.currentcolor_setters.borrow_mut() {
                setter_button.unparent();
            }
            self.currentcolor_setters.borrow_mut().clear();

            for _ in super::ColorPicker::AMOUNT_COLORBUTTONS_MIN..self.amount_colorbuttons.get() {
                let setter_button = ColorSetter::new();

                setter_button.set_hexpand(true);
                setter_button.set_vexpand(true);
                setter_button.set_halign(Align::Fill);
                setter_button.set_valign(Align::Fill);
                setter_button.set_position(obj.position());
                setter_button.set_group(Some(currentcolor_setter1));
                setterbox.append(&setter_button);

                setter_button.connect_clicked(clone!(@weak obj => move |setter_button| {
                    let color = setter_button.property("color").unwrap().get::<gdk::RGBA>().unwrap();
                    obj.set_property("current-color", &color.to_value()).expect("settings `color` property");
                }));

                self.currentcolor_setters.borrow_mut().push(setter_button);
            }

            self.generate_colors();
        }

        fn generate_colors(&self) {
            let color_step =
                (2.0 * std::f32::consts::PI) / ((self.amount_colorbuttons.get() - 1) as f32);
            let rgb_offset = (2.0 / 3.0) * std::f32::consts::PI;
            let color_offset = (5.0 / 4.0) * std::f32::consts::PI + 0.4;

            for (i, setter_button) in self.currentcolor_setters.borrow().iter().rev().enumerate() {
                let i = i + 1;

                let color = gdk::RGBA {
                    red: 0.5 * (i as f32 * color_step + 0.0 * rgb_offset + color_offset).sin()
                        + 0.5,
                    green: 0.5 * (i as f32 * color_step + 1.0 * rgb_offset + color_offset).sin()
                        + 0.5,
                    blue: 0.5 * (i as f32 * color_step + 2.0 * rgb_offset + color_offset).sin()
                        + 0.5,
                    alpha: 1.0,
                };
                setter_button
                    .set_property("color", &color.to_value())
                    .expect("settings `color` property");
            }
        }
    }
}

use gtk4::{gdk, glib, prelude::*, subclass::prelude::*, Orientable, PositionType, Widget};

use crate::utils;

glib::wrapper! {
    pub struct ColorPicker(ObjectSubclass<imp::ColorPicker>) @extends Widget, @implements Orientable;
}

impl Default for ColorPicker {
    fn default() -> Self {
        Self::new(utils::Color::black().to_gdk())
    }
}

impl ColorPicker {
    pub const COLOR_DEFAULT: gdk::RGBA = gdk::RGBA {
        red: 0.0,
        green: 0.0,
        blue: 0.0,
        alpha: 1.0,
    };
    pub const AMOUNT_COLORBUTTONS_MIN: u32 = 1;
    pub const AMOUNT_COLORBUTTONS_MAX: u32 = 1000;
    pub const AMOUNT_COLORBUTTONS_DEFAULT: u32 = 8;

    pub fn new(current_color: gdk::RGBA) -> Self {
        let color_picker: ColorPicker =
            glib::Object::new(&[("current-color", &current_color.to_value())])
                .expect("Failed to create ColorPicker");
        color_picker
    }

    pub fn position(&self) -> PositionType {
        self.property("position")
            .unwrap()
            .get::<PositionType>()
            .expect("value not of type `PositionType`")
    }

    pub fn set_position(&self, position: PositionType) {
        self.set_property("position", position.to_value()).unwrap();
    }

    pub fn current_color(&self) -> gdk::RGBA {
        self.property("current-color")
            .unwrap()
            .get::<gdk::RGBA>()
            .expect("value not of type `PositionType`")
    }

    pub fn set_current_color(&self, current_color: gdk::RGBA) {
        self.set_property("current-color", current_color.to_value())
            .unwrap();
    }

    pub fn amount_colorbuttons(&self) -> u32 {
        self.property("amount-colorbuttons")
            .unwrap()
            .get::<u32>()
            .expect("value not of type `u32`")
    }

    pub fn set_amount_colorbuttons(&self, amount: u32) {
        self.set_property("amount-colorbuttons", amount.to_value())
            .unwrap();
    }

    pub fn fetch_all_colors(&self) -> Vec<utils::Color> {
        let priv_ = imp::ColorPicker::from_instance(self);
        let mut all_colors = Vec::with_capacity(8);
        all_colors.push(utils::Color::from(priv_.currentcolor_setter1.get().color()));
        for colorsetter in priv_.currentcolor_setters.borrow().iter() {
            all_colors.push(utils::Color::from(colorsetter.color()));
        }

        all_colors
    }

    pub fn load_all_colors(&self, all_colors: &[utils::Color]) {
        let priv_ = imp::ColorPicker::from_instance(self);

        let mut all_colors_iter = all_colors.iter();
        if let Some(first_color) = all_colors_iter.next() {
            priv_.currentcolor_setter1.set_color(first_color.to_gdk());
        }
        for (color, colorsetter) in all_colors_iter.zip(priv_.currentcolor_setters.borrow().iter())
        {
            colorsetter.set_color(color.to_gdk());
        }
    }
}
