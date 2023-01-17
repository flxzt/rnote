mod strokewidthsetter;

// Re-exports
pub(crate) use strokewidthsetter::StrokeWidthSetter;

// Imports
use gtk4::{
    glib, glib::clone, glib::translate::IntoGlib, prelude::*, subclass::prelude::*, BoxLayout,
    CompositeTemplate, Orientation, PositionType, SpinButton, Widget,
};
use once_cell::sync::Lazy;
use std::cell::Cell;

mod imp {
    use super::*;

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/com/github/flxzt/rnote/ui/strokewidthpicker.ui")]
    pub(crate) struct StrokeWidthPicker {
        pub(crate) position: Cell<PositionType>,
        pub(crate) stroke_width: Cell<f64>,

        #[template_child]
        pub(crate) spinbutton: TemplateChild<SpinButton>,
        #[template_child]
        pub(crate) setter_box: TemplateChild<gtk4::Box>,
        #[template_child]
        pub(crate) setter_1: TemplateChild<StrokeWidthSetter>,
        #[template_child]
        pub(crate) setter_2: TemplateChild<StrokeWidthSetter>,
        #[template_child]
        pub(crate) setter_3: TemplateChild<StrokeWidthSetter>,
    }

    impl Default for StrokeWidthPicker {
        fn default() -> Self {
            Self {
                position: Cell::new(PositionType::Right),
                stroke_width: Cell::new(1.0),

                spinbutton: TemplateChild::default(),
                setter_box: TemplateChild::default(),
                setter_1: TemplateChild::default(),
                setter_2: TemplateChild::default(),
                setter_3: TemplateChild::default(),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for StrokeWidthPicker {
        const NAME: &'static str = "StrokeWidthPicker";
        type Type = super::StrokeWidthPicker;
        type ParentType = Widget;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for StrokeWidthPicker {
        fn constructed(&self) {
            self.parent_constructed();
            let inst = self.instance();

            self.spinbutton.set_increments(0.1, 2.0);

            inst.bind_property("stroke-width", &*self.spinbutton, "value")
                .sync_create()
                .bidirectional()
                .build();

            self.setter_1.set_stroke_width(2.0);
            self.setter_2.set_stroke_width(6.0);
            self.setter_3.set_stroke_width(32.0);

            self.setter_1.connect_clicked(
                clone!(@weak inst as strokewidthpicker => move |setter| {
                    strokewidthpicker.set_stroke_width(setter.stroke_width());
                }),
            );

            self.setter_2.connect_clicked(
                clone!(@weak inst as strokewidthpicker => move |setter| {
                    strokewidthpicker.set_stroke_width(setter.stroke_width());
                }),
            );

            self.setter_3.connect_clicked(
                clone!(@weak inst as strokewidthpicker => move |setter| {
                    strokewidthpicker.set_stroke_width(setter.stroke_width());
                }),
            );

            self.spinbutton.connect_value_changed(
                clone!(@weak inst as strokewidthpicker => move |spinbutton| {
                    strokewidthpicker.set_active_setter_stroke_width(spinbutton.value());
                }),
            );
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
                    glib::ParamSpecDouble::new(
                        "stroke-width",
                        "stroke-width",
                        "stroke-width",
                        0.0,
                        500.0,
                        1.0,
                        glib::ParamFlags::READWRITE,
                    ),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "position" => self.position.get().to_value(),
                "stroke-width" => self.stroke_width.get().to_value(),
                _ => panic!("invalid property name"),
            }
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            let inst = self.instance();

            match pspec.name() {
                "position" => {
                    let position = value
                        .get::<PositionType>()
                        .expect("value not of type `PositionType`");
                    self.position.replace(position);

                    let layout_manager = inst
                        .layout_manager()
                        .unwrap()
                        .downcast::<BoxLayout>()
                        .unwrap();

                    match position {
                        PositionType::Left => {
                            layout_manager.set_orientation(Orientation::Vertical);
                            self.setter_box.set_orientation(Orientation::Vertical);
                            self.spinbutton.set_orientation(Orientation::Vertical);
                        }
                        PositionType::Right => {
                            layout_manager.set_orientation(Orientation::Vertical);
                            self.setter_box.set_orientation(Orientation::Vertical);
                            self.spinbutton.set_orientation(Orientation::Vertical);
                        }
                        PositionType::Top => {
                            layout_manager.set_orientation(Orientation::Horizontal);
                            self.setter_box.set_orientation(Orientation::Horizontal);
                            self.spinbutton.set_orientation(Orientation::Horizontal);
                        }
                        PositionType::Bottom => {
                            layout_manager.set_orientation(Orientation::Horizontal);
                            self.setter_box.set_orientation(Orientation::Horizontal);
                            self.spinbutton.set_orientation(Orientation::Horizontal);
                        }
                        _ => {}
                    }
                }
                "stroke-width" => {
                    self.stroke_width
                        .set(value.get::<f64>().expect("value not of type `f64`"));
                }
                _ => panic!("invalid property name"),
            }
        }
    }

    impl WidgetImpl for StrokeWidthPicker {}

    impl StrokeWidthPicker {}
}

glib::wrapper! {
    pub(crate) struct StrokeWidthPicker(ObjectSubclass<imp::StrokeWidthPicker>)
        @extends Widget,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget;
}

impl Default for StrokeWidthPicker {
    fn default() -> Self {
        Self::new()
    }
}

impl StrokeWidthPicker {
    pub(crate) fn new() -> Self {
        glib::Object::new(&[])
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
    pub(crate) fn stroke_width(&self) -> f64 {
        self.property::<f64>("stroke-width")
    }

    #[allow(unused)]
    pub(crate) fn set_stroke_width(&self, stroke_width: f64) {
        self.set_property("stroke-width", stroke_width.to_value());
    }

    pub(crate) fn spinbutton(&self) -> SpinButton {
        self.imp().spinbutton.get()
    }

    pub(crate) fn set_active_setter_stroke_width(&self, stroke_width: f64) {
        let imp = self.imp();

        if imp.setter_1.is_active() {
            imp.setter_1.set_stroke_width(stroke_width);
        } else if imp.setter_2.is_active() {
            imp.setter_2.set_stroke_width(stroke_width);
        } else if imp.setter_3.is_active() {
            imp.setter_3.set_stroke_width(stroke_width);
        }
    }

    pub(crate) fn deselect_setters(&self) {
        let imp = self.imp();

        imp.setter_1.set_active(false);
        imp.setter_2.set_active(false);
        imp.setter_3.set_active(false);
    }
}
