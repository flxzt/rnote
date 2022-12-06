mod colorsetter;

// Re-exports
pub(crate) use colorsetter::ColorSetter;

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use gtk4::{
    gdk, glib, glib::clone, glib::translate::IntoGlib, prelude::*, subclass::prelude::*, Align,
    Box, BoxLayout, Button, ColorChooserWidget, CompositeTemplate, MenuButton, Orientation,
    Popover, PositionType, Widget,
};

use once_cell::sync::Lazy;
use rnote_compose::Color;
use rnote_engine::utils::GdkRGBAHelpers;

mod imp {
    use super::*;

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/com/github/flxzt/rnote/ui/colorpicker.ui")]
    pub(crate) struct ColorPicker {
        #[template_child]
        pub(crate) setterbox: TemplateChild<Box>,
        #[template_child]
        pub(crate) first_colorsetter: TemplateChild<ColorSetter>,
        #[template_child]
        pub(crate) colorpicker_button: TemplateChild<MenuButton>,
        #[template_child]
        pub(crate) colorpicker_popover: TemplateChild<Popover>,
        #[template_child]
        pub(crate) colorchooser: TemplateChild<ColorChooserWidget>,
        #[template_child]
        pub(crate) colorchooser_editor_gobackbutton: TemplateChild<Button>,
        #[template_child]
        pub(crate) colorchooser_editor_selectbutton: TemplateChild<Button>,

        pub(crate) position: Cell<PositionType>,
        pub(crate) selected: Cell<u32>,
        pub(crate) amount_colorbuttons: Cell<u32>,
        pub(crate) colorsetters: Rc<RefCell<Vec<ColorSetter>>>,
        pub(crate) current_color: Cell<gdk::RGBA>,
    }

    impl Default for ColorPicker {
        fn default() -> Self {
            Self {
                setterbox: TemplateChild::<Box>::default(),
                first_colorsetter: TemplateChild::<ColorSetter>::default(),
                colorpicker_button: TemplateChild::<MenuButton>::default(),
                colorpicker_popover: TemplateChild::<Popover>::default(),
                colorchooser: TemplateChild::<ColorChooserWidget>::default(),
                colorchooser_editor_gobackbutton: TemplateChild::<Button>::default(),
                colorchooser_editor_selectbutton: TemplateChild::<Button>::default(),

                position: Cell::new(PositionType::Right),
                selected: Cell::new(0),
                amount_colorbuttons: Cell::new(super::ColorPicker::AMOUNT_COLORBUTTONS_DEFAULT),
                current_color: Cell::new(gdk::RGBA::from_compose_color(
                    super::ColorPicker::COLOR_DEFAULT,
                )),
                colorsetters: Rc::new(RefCell::new(Vec::with_capacity(
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
        fn constructed(&self) {
            self.parent_constructed();
            let inst = self.instance();

            let colorchooser = self.colorchooser.get();
            let first_colorsetter = self.first_colorsetter.get();
            let colorpicker_popover = self.colorpicker_popover.get();
            let colorchooser_editor_gobackbutton = self.colorchooser_editor_gobackbutton.get();

            self.first_colorsetter.connect_clicked(
                clone!(@weak inst as colorpicker => move |first_colorsetter| {
                    let color = first_colorsetter.property::<gdk::RGBA>("color");
                    colorpicker.set_property("current-color", &color.to_value());

                    // Avoid loops
                    if colorpicker.selected() != 0_u32 {
                        colorpicker.set_selected(0_u32);
                    }
                }),
            );

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

            self.colorchooser.connect_rgba_notify(
                clone!(@weak inst as colorpicker => move |colorchooser| {
                    let color = colorchooser.rgba();
                    colorpicker.set_property("current-color", &color.to_value());
                }),
            );

            inst.connect_notify_local(Some("current-color"), clone!(@weak first_colorsetter, @weak self.colorsetters as colorsetters => move |obj, _param| {
                let current_color = obj.current_color();

                // store color in the buttons
                if first_colorsetter.is_active() {
                    first_colorsetter.set_property("color", current_color.to_value());
                } else {
                    for colorsetter in &*colorsetters.borrow() {
                        if colorsetter.is_active() {
                            colorsetter.set_property("color", current_color.to_value());
                        }
                    }
                }
            }));
        }

        fn dispose(&self) {
            while let Some(child) = self.instance().first_child() {
                child.unparent();
            }
        }

        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpecEnum::new(
                        "position",
                        "position",
                        "position",
                        PositionType::static_type(),
                        PositionType::Right.into_glib(),
                        glib::ParamFlags::READWRITE,
                    ),
                    glib::ParamSpecUInt::new(
                        "selected",
                        "selected",
                        "selected",
                        u32::MIN,
                        u32::MAX,
                        0,
                        glib::ParamFlags::READWRITE,
                    ),
                    glib::ParamSpecUInt::new(
                        "amount-colorbuttons",
                        "amount-colorbuttons",
                        "The amount of colorbuttons shown",
                        super::ColorPicker::AMOUNT_COLORBUTTONS_MIN,
                        super::ColorPicker::AMOUNT_COLORBUTTONS_MAX,
                        super::ColorPicker::AMOUNT_COLORBUTTONS_DEFAULT,
                        glib::ParamFlags::READWRITE,
                    ),
                    glib::ParamSpecBoxed::new(
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

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            let inst = self.instance();

            match pspec.name() {
                "position" => {
                    let layout_manager = inst
                        .layout_manager()
                        .unwrap()
                        .downcast::<BoxLayout>()
                        .unwrap();

                    let position = value
                        .get::<PositionType>()
                        .expect("value not of type `PositionType`");
                    self.position.replace(position);

                    self.first_colorsetter.set_property("position", position);
                    for setter_button in self.colorsetters.borrow().iter() {
                        setter_button.set_property("position", position);
                    }

                    match position {
                        PositionType::Left => {
                            self.colorpicker_popover.set_position(PositionType::Right);
                            layout_manager.set_orientation(Orientation::Vertical);
                            self.setterbox.set_orientation(Orientation::Vertical);
                            self.colorpicker_button.set_margin_start(0);
                            self.colorpicker_button.set_margin_end(0);
                            self.colorpicker_button.set_margin_top(3);
                            self.colorpicker_button.set_margin_bottom(0);
                        }
                        PositionType::Right => {
                            self.colorpicker_popover.set_position(PositionType::Left);
                            layout_manager.set_orientation(Orientation::Vertical);
                            self.setterbox.set_orientation(Orientation::Vertical);
                            self.colorpicker_button.set_margin_start(0);
                            self.colorpicker_button.set_margin_end(0);
                            self.colorpicker_button.set_margin_top(3);
                            self.colorpicker_button.set_margin_bottom(0);
                        }
                        PositionType::Top => {
                            self.colorpicker_popover.set_position(PositionType::Bottom);
                            layout_manager.set_orientation(Orientation::Horizontal);
                            self.setterbox.set_orientation(Orientation::Horizontal);
                            self.colorpicker_button.set_margin_start(3);
                            self.colorpicker_button.set_margin_end(0);
                            self.colorpicker_button.set_margin_top(0);
                            self.colorpicker_button.set_margin_bottom(0);
                        }
                        PositionType::Bottom => {
                            self.colorpicker_popover.set_position(PositionType::Top);
                            layout_manager.set_orientation(Orientation::Horizontal);
                            self.setterbox.set_orientation(Orientation::Horizontal);
                            self.colorpicker_button.set_margin_start(3);
                            self.colorpicker_button.set_margin_end(0);
                            self.colorpicker_button.set_margin_top(0);
                            self.colorpicker_button.set_margin_bottom(0);
                        }
                        _ => {}
                    }
                }
                "selected" => {
                    // Clamping to the current amount of colorbuttons
                    let index = value
                        .get::<u32>()
                        .unwrap()
                        .clamp(0, self.amount_colorbuttons.get());
                    self.selected.set(index);

                    if index == 0 {
                        self.first_colorsetter.get().set_active(true);
                    } else {
                        // index - 1, because vec of colorsetters are not including the first
                        if let Some(colorsetter) =
                            self.colorsetters.borrow().get(index as usize - 1)
                        {
                            colorsetter.set_active(true);
                        }
                    }
                }
                "amount-colorbuttons" => {
                    self.amount_colorbuttons
                        .set(value.get::<u32>().expect("value not of type `u32`"));
                    self.init_colorsetters();
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

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "position" => self.position.get().to_value(),
                "selected" => self.selected.get().to_value(),
                "amount-colorbuttons" => self.amount_colorbuttons.get().to_value(),
                "current-color" => self.current_color.get().to_value(),
                _ => panic!("invalid property name"),
            }
        }
    }

    impl WidgetImpl for ColorPicker {}

    impl ColorPicker {
        fn init_colorsetters(&self) {
            let inst = self.instance();

            let setterbox = self.setterbox.get();
            let first_colorsetter = self.first_colorsetter.get();

            // Clearing previous
            for setter_button in &*self.colorsetters.borrow_mut() {
                setter_button.unparent();
            }
            self.colorsetters.borrow_mut().clear();

            // init the colorsetters. Index starts at one to skip the first button
            for i in super::ColorPicker::AMOUNT_COLORBUTTONS_MIN..self.amount_colorbuttons.get() {
                let setter_button = ColorSetter::new();

                setter_button.set_hexpand(true);
                setter_button.set_vexpand(true);
                setter_button.set_halign(Align::Fill);
                setter_button.set_valign(Align::Fill);
                setter_button.set_position(inst.position());
                setter_button.set_group(Some(&first_colorsetter));
                setterbox.append(&setter_button);

                setter_button.connect_clicked(
                    clone!(@weak inst as colorpicker => move |setter_button| {
                        let color = setter_button.property::<gdk::RGBA>("color");
                        colorpicker.set_property("current-color", &color.to_value());

                        // Avoid loops
                        if colorpicker.selected() != i {
                            colorpicker.set_selected(i);
                        }
                    }),
                );

                self.colorsetters.borrow_mut().push(setter_button);
            }

            self.generate_colors();
        }

        fn generate_colors(&self) {
            let color_step =
                (2.0 * std::f32::consts::PI) / ((self.amount_colorbuttons.get() - 1) as f32);
            let rgb_offset = (2.0 / 3.0) * std::f32::consts::PI;
            let color_offset = (5.0 / 4.0) * std::f32::consts::PI + 0.4;

            for (i, setter_button) in self.colorsetters.borrow().iter().rev().enumerate() {
                let i = i + 1;

                let color = gdk::RGBA::new(
                    0.5 * (i as f32 * color_step + 0.0 * rgb_offset + color_offset).sin() + 0.5,
                    0.5 * (i as f32 * color_step + 1.0 * rgb_offset + color_offset).sin() + 0.5,
                    0.5 * (i as f32 * color_step + 2.0 * rgb_offset + color_offset).sin() + 0.5,
                    1.0,
                );
                setter_button.set_property("color", &color.to_value());
            }
        }
    }
}

glib::wrapper! {
    pub(crate) struct ColorPicker(ObjectSubclass<imp::ColorPicker>)
        @extends Widget;
}

impl Default for ColorPicker {
    fn default() -> Self {
        Self::new(gdk::RGBA::from_compose_color(Color::BLACK))
    }
}

impl ColorPicker {
    pub(crate) const COLOR_DEFAULT: Color = Color::BLACK;
    pub(crate) const AMOUNT_COLORBUTTONS_MIN: u32 = 1;
    pub(crate) const AMOUNT_COLORBUTTONS_MAX: u32 = 1000;
    pub(crate) const AMOUNT_COLORBUTTONS_DEFAULT: u32 = 8;

    pub(crate) fn new(current_color: gdk::RGBA) -> Self {
        glib::Object::new(&[("current-color", &current_color.to_value())])
    }

    #[allow(unused)]
    pub(crate) fn position(&self) -> PositionType {
        self.property::<PositionType>("position")
    }

    #[allow(unused)]
    pub(crate) fn set_position(&self, position: PositionType) {
        self.set_property("position", position.to_value());
    }

    #[allow(unused)]
    pub(crate) fn current_color(&self) -> gdk::RGBA {
        self.property::<gdk::RGBA>("current-color")
    }

    #[allow(unused)]
    pub(crate) fn set_current_color(&self, color: gdk::RGBA) {
        self.set_property("current-color", color.to_value());
    }

    #[allow(unused)]
    pub(crate) fn amount_colorbuttons(&self) -> u32 {
        self.property::<u32>("amount-colorbuttons")
    }

    #[allow(unused)]
    pub(crate) fn set_amount_colorbuttons(&self, amount: u32) {
        self.set_property("amount-colorbuttons", amount.to_value());
    }

    #[allow(unused)]
    pub(crate) fn selected(&self) -> u32 {
        self.property::<u32>("selected")
    }

    #[allow(unused)]
    pub(crate) fn set_selected(&self, selected: u32) {
        self.set_property("selected", selected.to_value());
    }

    /// Returns a vector of the colors
    pub(crate) fn fetch_all_colors(&self) -> Vec<Color> {
        let mut all_colors = Vec::with_capacity(8);
        all_colors.push(
            self.imp()
                .first_colorsetter
                .get()
                .color()
                .into_compose_color(),
        );
        for colorsetter in self.imp().colorsetters.borrow().iter() {
            all_colors.push(colorsetter.color().into_compose_color());
        }

        all_colors
    }

    pub(crate) fn load_colors(&self, all_colors: &[Color]) {
        let mut all_colors_iter = all_colors.iter();
        if let Some(&first_color) = all_colors_iter.next() {
            self.imp()
                .first_colorsetter
                .set_color(gdk::RGBA::from_compose_color(first_color));
        }
        for (&color, colorsetter) in all_colors_iter.zip(self.imp().colorsetters.borrow().iter()) {
            colorsetter.set_color(gdk::RGBA::from_compose_color(color));
        }
    }
}
