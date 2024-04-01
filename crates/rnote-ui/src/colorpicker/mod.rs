// Modules
mod colorpad;
mod colorsetter;

// Re-exports
pub(crate) use colorpad::RnColorPad;
pub(crate) use colorsetter::RnColorSetter;

// Imports
use crate::RnAppWindow;
use gtk4::{
    gdk, glib, glib::clone, prelude::*, subclass::prelude::*, BoxLayout, Button, ColorDialog,
    CompositeTemplate, Orientation, PositionType, Widget,
};
use once_cell::sync::Lazy;
use rnote_compose::{color, Color};
use rnote_engine::ext::GdkRGBAExt;
use std::cell::{Cell, RefCell};

mod imp {
    use super::*;

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/com/github/flxzt/rnote/ui/colorpicker.ui")]
    pub(crate) struct RnColorPicker {
        pub(crate) stroke_color: RefCell<gdk::RGBA>,
        pub(crate) fill_color: RefCell<gdk::RGBA>,
        pub(crate) position: Cell<PositionType>,
        pub(crate) color_dialog: glib::WeakRef<ColorDialog>,

        #[template_child]
        pub(crate) active_colors_box: TemplateChild<gtk4::Box>,
        #[template_child]
        pub(crate) stroke_color_pad: TemplateChild<RnColorPad>,
        #[template_child]
        pub(crate) fill_color_pad: TemplateChild<RnColorPad>,
        #[template_child]
        pub(crate) setter_box: TemplateChild<gtk4::Box>,
        #[template_child]
        pub(crate) setter_1: TemplateChild<RnColorSetter>,
        #[template_child]
        pub(crate) setter_2: TemplateChild<RnColorSetter>,
        #[template_child]
        pub(crate) setter_3: TemplateChild<RnColorSetter>,
        #[template_child]
        pub(crate) setter_4: TemplateChild<RnColorSetter>,
        #[template_child]
        pub(crate) setter_5: TemplateChild<RnColorSetter>,
        #[template_child]
        pub(crate) setter_6: TemplateChild<RnColorSetter>,
        #[template_child]
        pub(crate) setter_7: TemplateChild<RnColorSetter>,
        #[template_child]
        pub(crate) setter_8: TemplateChild<RnColorSetter>,
        #[template_child]
        pub(crate) setter_9: TemplateChild<RnColorSetter>,
        #[template_child]
        pub(crate) colordialog_button: TemplateChild<Button>,
    }

    impl Default for RnColorPicker {
        fn default() -> Self {
            Self {
                stroke_color: RefCell::new(gdk::RGBA::from_compose_color(
                    *super::STROKE_COLOR_DEFAULT,
                )),
                fill_color: RefCell::new(gdk::RGBA::from_compose_color(*super::FILL_COLOR_DEFAULT)),
                position: Cell::new(PositionType::Right),
                color_dialog: glib::WeakRef::new(),

                active_colors_box: TemplateChild::default(),
                stroke_color_pad: TemplateChild::default(),
                fill_color_pad: TemplateChild::default(),
                setter_box: TemplateChild::default(),
                setter_1: TemplateChild::default(),
                setter_2: TemplateChild::default(),
                setter_3: TemplateChild::default(),
                setter_4: TemplateChild::default(),
                setter_5: TemplateChild::default(),
                setter_6: TemplateChild::default(),
                setter_7: TemplateChild::default(),
                setter_8: TemplateChild::default(),
                setter_9: TemplateChild::default(),
                colordialog_button: TemplateChild::default(),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for RnColorPicker {
        const NAME: &'static str = "RnColorPicker";
        type Type = super::RnColorPicker;
        type ParentType = Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for RnColorPicker {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();

            self.setup_setters();

            self.stroke_color_pad
                .bind_property("color", &*obj, "stroke-color")
                .sync_create()
                .bidirectional()
                .build();

            self.stroke_color_pad.connect_active_notify(
                clone!(@weak obj as colorpicker => move |_| {
                    colorpicker.deselect_setters();
                }),
            );

            self.fill_color_pad
                .bind_property("color", &*obj, "fill-color")
                .sync_create()
                .bidirectional()
                .build();

            self.fill_color_pad.connect_active_notify(
                clone!(@weak obj as colorpicker => move |_| {
                    colorpicker.deselect_setters();
                }),
            );
        }

        fn dispose(&self) {
            self.dispose_template();
            while let Some(child) = self.obj().first_child() {
                child.unparent();
            }
        }

        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpecEnum::builder_with_default::<PositionType>(
                        "position",
                        PositionType::Right,
                    )
                    .build(),
                    glib::ParamSpecBoxed::builder::<gdk::RGBA>("stroke-color").build(),
                    glib::ParamSpecBoxed::builder::<gdk::RGBA>("fill-color").build(),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            let obj = self.obj();

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

                    self.setter_1.set_position(position);
                    self.setter_2.set_position(position);
                    self.setter_3.set_position(position);
                    self.setter_4.set_position(position);
                    self.setter_5.set_position(position);
                    self.setter_6.set_position(position);
                    self.setter_7.set_position(position);
                    self.setter_8.set_position(position);
                    self.setter_9.set_position(position);

                    match position {
                        PositionType::Left => {
                            layout_manager.set_orientation(Orientation::Vertical);
                            self.active_colors_box
                                .set_orientation(Orientation::Vertical);
                            self.setter_box.set_orientation(Orientation::Vertical);
                        }
                        PositionType::Right => {
                            layout_manager.set_orientation(Orientation::Vertical);
                            self.active_colors_box
                                .set_orientation(Orientation::Vertical);
                            self.setter_box.set_orientation(Orientation::Vertical);
                        }
                        PositionType::Top => {
                            layout_manager.set_orientation(Orientation::Horizontal);
                            self.active_colors_box
                                .set_orientation(Orientation::Horizontal);
                            self.setter_box.set_orientation(Orientation::Horizontal);
                        }
                        PositionType::Bottom => {
                            layout_manager.set_orientation(Orientation::Horizontal);
                            self.active_colors_box
                                .set_orientation(Orientation::Horizontal);
                            self.setter_box.set_orientation(Orientation::Horizontal);
                        }
                        _ => {}
                    }
                }
                "stroke-color" => {
                    self.stroke_color.replace(
                        value
                            .get::<gdk::RGBA>()
                            .expect("value not of type `gdk::RGBA`"),
                    );
                }
                "fill-color" => {
                    self.fill_color.replace(
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
                "stroke-color" => self.stroke_color.borrow().to_value(),
                "fill-color" => self.fill_color.borrow().to_value(),
                _ => panic!("invalid property name"),
            }
        }
    }

    impl WidgetImpl for RnColorPicker {}

    impl RnColorPicker {
        pub fn reset_colors(&self) {
            self.setter_1.set_color(Self::default_color(0));
            self.setter_2.set_color(Self::default_color(1));
            self.setter_3.set_color(Self::default_color(2));
            self.setter_4.set_color(Self::default_color(3));
            self.setter_5.set_color(Self::default_color(4));
            self.setter_6.set_color(Self::default_color(5));
            self.setter_7.set_color(Self::default_color(6));
            self.setter_8.set_color(Self::default_color(7));
            self.setter_9.set_color(Self::default_color(8));
        }

        fn setup_setters(&self) {
            let obj = self.obj();

            self.reset_colors();

            self.setter_1
                .connect_active_notify(clone!(@weak obj as colorpicker => move |setter| {
                    if setter.is_active() {
                        colorpicker.setter_2().set_active(false);
                        colorpicker.setter_3().set_active(false);
                        colorpicker.setter_4().set_active(false);
                        colorpicker.setter_5().set_active(false);
                        colorpicker.setter_6().set_active(false);
                        colorpicker.setter_7().set_active(false);
                        colorpicker.setter_8().set_active(false);
                        colorpicker.setter_9().set_active(false);
                        // Must come after setting the other setters inactive
                        colorpicker.set_color_active_pad(setter.color());
                    }
                }));

            self.setter_2
                .connect_active_notify(clone!(@weak obj as colorpicker => move |setter| {
                    if setter.is_active() {
                        colorpicker.setter_1().set_active(false);
                        colorpicker.setter_3().set_active(false);
                        colorpicker.setter_4().set_active(false);
                        colorpicker.setter_5().set_active(false);
                        colorpicker.setter_6().set_active(false);
                        colorpicker.setter_7().set_active(false);
                        colorpicker.setter_8().set_active(false);
                        colorpicker.setter_9().set_active(false);
                        colorpicker.set_color_active_pad(setter.color());
                    }
                }));

            self.setter_3
                .connect_active_notify(clone!(@weak obj as colorpicker => move |setter| {
                    if setter.is_active() {
                        colorpicker.setter_1().set_active(false);
                        colorpicker.setter_2().set_active(false);
                        colorpicker.setter_4().set_active(false);
                        colorpicker.setter_5().set_active(false);
                        colorpicker.setter_6().set_active(false);
                        colorpicker.setter_7().set_active(false);
                        colorpicker.setter_8().set_active(false);
                        colorpicker.setter_9().set_active(false);
                        colorpicker.set_color_active_pad(setter.color());
                    }
                }));

            self.setter_4
                .connect_active_notify(clone!(@weak obj as colorpicker => move |setter| {
                    if setter.is_active() {
                        colorpicker.setter_1().set_active(false);
                        colorpicker.setter_2().set_active(false);
                        colorpicker.setter_3().set_active(false);
                        colorpicker.setter_5().set_active(false);
                        colorpicker.setter_6().set_active(false);
                        colorpicker.setter_7().set_active(false);
                        colorpicker.setter_8().set_active(false);
                        colorpicker.setter_9().set_active(false);
                        colorpicker.set_color_active_pad(setter.color());
                    }
                }));

            self.setter_5
                .connect_active_notify(clone!(@weak obj as colorpicker => move |setter| {
                    if setter.is_active() {
                        colorpicker.setter_1().set_active(false);
                        colorpicker.setter_2().set_active(false);
                        colorpicker.setter_3().set_active(false);
                        colorpicker.setter_4().set_active(false);
                        colorpicker.setter_6().set_active(false);
                        colorpicker.setter_7().set_active(false);
                        colorpicker.setter_8().set_active(false);
                        colorpicker.setter_9().set_active(false);
                        colorpicker.set_color_active_pad(setter.color());
                    }
                }));

            self.setter_6
                .connect_active_notify(clone!(@weak obj as colorpicker => move |setter| {
                    if setter.is_active() {
                        colorpicker.setter_1().set_active(false);
                        colorpicker.setter_2().set_active(false);
                        colorpicker.setter_3().set_active(false);
                        colorpicker.setter_4().set_active(false);
                        colorpicker.setter_5().set_active(false);
                        colorpicker.setter_7().set_active(false);
                        colorpicker.setter_8().set_active(false);
                        colorpicker.setter_9().set_active(false);
                        colorpicker.set_color_active_pad(setter.color());
                    }
                }));

            self.setter_7
                .connect_active_notify(clone!(@weak obj as colorpicker => move |setter| {
                    if setter.is_active() {
                        colorpicker.setter_1().set_active(false);
                        colorpicker.setter_2().set_active(false);
                        colorpicker.setter_3().set_active(false);
                        colorpicker.setter_4().set_active(false);
                        colorpicker.setter_5().set_active(false);
                        colorpicker.setter_6().set_active(false);
                        colorpicker.setter_8().set_active(false);
                        colorpicker.set_color_active_pad(setter.color());
                    }
                }));

            self.setter_8
                .connect_active_notify(clone!(@weak obj as colorpicker => move |setter| {
                    if setter.is_active() {
                        colorpicker.setter_1().set_active(false);
                        colorpicker.setter_2().set_active(false);
                        colorpicker.setter_3().set_active(false);
                        colorpicker.setter_4().set_active(false);
                        colorpicker.setter_5().set_active(false);
                        colorpicker.setter_6().set_active(false);
                        colorpicker.setter_7().set_active(false);
                        colorpicker.setter_9().set_active(false);
                        colorpicker.set_color_active_pad(setter.color());
                    }
                }));

            self.setter_9
                .connect_active_notify(clone!(@weak obj as colorpicker => move |setter| {
                    if setter.is_active() {
                        colorpicker.setter_1().set_active(false);
                        colorpicker.setter_2().set_active(false);
                        colorpicker.setter_3().set_active(false);
                        colorpicker.setter_4().set_active(false);
                        colorpicker.setter_5().set_active(false);
                        colorpicker.setter_6().set_active(false);
                        colorpicker.setter_7().set_active(false);
                        colorpicker.setter_8().set_active(false);
                        colorpicker.set_color_active_pad(setter.color());
                    }
                }));
        }

        fn default_color(i: usize) -> gdk::RGBA {
            match i {
                0 => gdk::RGBA::new(0.0, 0.0, 0.0, 1.0),
                1 => gdk::RGBA::new(1.0, 1.0, 1.0, 1.0),
                2 => gdk::RGBA::new(0.0, 0.0, 0.0, 0.0),
                3 => gdk::RGBA::new(0.597, 0.753, 0.941, 1.0),
                4 => gdk::RGBA::new(0.101, 0.371, 0.703, 1.0),
                5 => gdk::RGBA::new(0.148, 0.632, 0.410, 1.0),
                6 => gdk::RGBA::new(0.957, 0.757, 0.066, 1.0),
                7 => gdk::RGBA::new(0.898, 0.378, 0.0, 1.0),
                8 => gdk::RGBA::new(0.644, 0.113, 0.175, 1.0),
                _ => gdk::RGBA::new(0.0, 0.0, 0.0, 1.0),
            }
        }
    }
}

glib::wrapper! {
    pub(crate) struct RnColorPicker(ObjectSubclass<imp::RnColorPicker>)
        @extends Widget,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget;
}

impl Default for RnColorPicker {
    fn default() -> Self {
        Self::new()
    }
}

pub(crate) static STROKE_COLOR_DEFAULT: Lazy<Color> =
    Lazy::new(|| Color::from(color::GNOME_DARKS[4]));
pub(crate) static FILL_COLOR_DEFAULT: Lazy<Color> =
    Lazy::new(|| Color::from(color::GNOME_BLUES[1]));

impl RnColorPicker {
    pub(crate) fn new() -> Self {
        glib::Object::new()
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
    pub(crate) fn stroke_color(&self) -> gdk::RGBA {
        self.property::<gdk::RGBA>("stroke-color")
    }

    #[allow(unused)]
    pub(crate) fn set_stroke_color(&self, color: gdk::RGBA) {
        self.set_property("stroke-color", color.to_value());
    }

    #[allow(unused)]
    pub(crate) fn fill_color(&self) -> gdk::RGBA {
        self.property::<gdk::RGBA>("fill-color")
    }

    #[allow(unused)]
    pub(crate) fn set_fill_color(&self, color: gdk::RGBA) {
        self.set_property("fill-color", color.to_value());
    }

    pub(crate) fn setter_1(&self) -> RnColorSetter {
        self.imp().setter_1.get()
    }

    pub(crate) fn setter_2(&self) -> RnColorSetter {
        self.imp().setter_2.get()
    }

    pub(crate) fn setter_3(&self) -> RnColorSetter {
        self.imp().setter_3.get()
    }

    pub(crate) fn setter_4(&self) -> RnColorSetter {
        self.imp().setter_4.get()
    }

    pub(crate) fn setter_5(&self) -> RnColorSetter {
        self.imp().setter_5.get()
    }

    pub(crate) fn setter_6(&self) -> RnColorSetter {
        self.imp().setter_6.get()
    }

    pub(crate) fn setter_7(&self) -> RnColorSetter {
        self.imp().setter_7.get()
    }

    pub(crate) fn setter_8(&self) -> RnColorSetter {
        self.imp().setter_8.get()
    }

    pub(crate) fn setter_9(&self) -> RnColorSetter {
        self.imp().setter_9.get()
    }

    pub(crate) fn init(&self, appwindow: &RnAppWindow) {
        self.imp().colordialog_button.connect_clicked(
            clone!(@weak self as colorpicker, @weak appwindow => move |_| {
                if colorpicker.imp().color_dialog.upgrade().is_some() {
                    // Unfortunately Gtk currently does not have API to make the dialog the active window.
                } else {
                    glib::spawn_future_local(clone!(@weak colorpicker, @weak appwindow => async move {
                        let dialog = ColorDialog::builder().modal(false).with_alpha(true).build();
                        colorpicker.imp().color_dialog.set(Some(&dialog));

                        let active_color = if colorpicker.stroke_color_pad_active() {
                            colorpicker.stroke_color()
                        } else {
                            colorpicker.fill_color()
                        };
                        match dialog.choose_rgba_future(Some(&appwindow), Some(&active_color)).await {
                            Ok(new_color) => {
                                colorpicker.set_color_active_pad(new_color);
                                colorpicker.set_color_active_setter(new_color);
                            },
                            // this reports as error if the dialog is dismissed by the user.
                            // The API is a bit odd, expected would be Result<Option<RGBA>>
                            Err(e) => tracing::debug!("Did not choose new color (Error or dialog dismissed by user), Err: {e:?}"),
                        }

                        colorpicker.imp().color_dialog.set(None);
                    }));
                }
            }),
        );
    }

    fn set_color_active_setter(&self, color: gdk::RGBA) {
        let imp = self.imp();

        if imp.setter_1.is_active() {
            imp.setter_1.set_color(color);
        } else if imp.setter_2.is_active() {
            imp.setter_2.set_color(color);
        } else if imp.setter_3.is_active() {
            imp.setter_3.set_color(color);
        } else if imp.setter_4.is_active() {
            imp.setter_4.set_color(color);
        } else if imp.setter_5.is_active() {
            imp.setter_5.set_color(color);
        } else if imp.setter_6.is_active() {
            imp.setter_6.set_color(color);
        } else if imp.setter_7.is_active() {
            imp.setter_7.set_color(color);
        } else if imp.setter_8.is_active() {
            imp.setter_8.set_color(color);
        } else if imp.setter_9.is_active() {
            imp.setter_9.set_color(color);
        }
    }

    #[allow(unused)]
    pub(crate) fn stroke_color_pad_active(&self) -> bool {
        self.imp().stroke_color_pad.is_active()
    }

    #[allow(unused)]
    pub(crate) fn fill_color_pad_active(&self) -> bool {
        self.imp().fill_color_pad.is_active()
    }

    fn set_color_active_pad(&self, color: gdk::RGBA) {
        if self.imp().stroke_color_pad.is_active() {
            self.set_stroke_color(color);
        } else if self.imp().fill_color_pad.is_active() {
            self.set_fill_color(color);
        }
    }

    pub(crate) fn deselect_setters(&self) {
        let imp = self.imp();

        imp.setter_1.set_active(false);
        imp.setter_2.set_active(false);
        imp.setter_3.set_active(false);
        imp.setter_4.set_active(false);
        imp.setter_5.set_active(false);
        imp.setter_6.set_active(false);
        imp.setter_7.set_active(false);
        imp.setter_8.set_active(false);
        imp.setter_9.set_active(false);
    }

    pub(crate) fn reset_colors(&self) {
        self.imp().reset_colors();
    }
}
