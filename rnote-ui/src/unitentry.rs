use gtk4::{
    glib, glib::clone, prelude::*, subclass::prelude::*, Adjustment, CompositeTemplate, DropDown,
    SpinButton, Widget,
};
use once_cell::sync::Lazy;
use rnote_engine::document::format;
use std::cell::Cell;

mod imp {
    use super::*;
    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/com/github/flxzt/rnote/ui/unitentry.ui")]
    pub struct UnitEntry {
        pub value: Cell<f64>,
        pub unit: Cell<format::MeasureUnit>,
        pub dpi: Cell<f64>,
        #[template_child]
        pub value_adj: TemplateChild<Adjustment>,
        #[template_child]
        pub value_spinner: TemplateChild<SpinButton>,
        #[template_child]
        pub unit_dropdown: TemplateChild<DropDown>,
    }

    impl Default for UnitEntry {
        fn default() -> Self {
            Self {
                value: Cell::new(1.0),
                unit: Cell::new(format::MeasureUnit::Px),
                dpi: Cell::new(96.0),
                value_spinner: TemplateChild::<SpinButton>::default(),
                value_adj: TemplateChild::<Adjustment>::default(),
                unit_dropdown: TemplateChild::<DropDown>::default(),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for UnitEntry {
        const NAME: &'static str = "UnitEntry";
        type Type = super::UnitEntry;
        type ParentType = Widget;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for UnitEntry {
        fn constructed(&self) {
            let inst = self.instance();
            self.parent_constructed();

            // Spinner
            inst.bind_property("value", &self.value_spinner.get(), "value")
                .transform_to(|_, val: f64| Some(val))
                .transform_from(|_, val: f64| Some(val))
                .flags(glib::BindingFlags::BIDIRECTIONAL | glib::BindingFlags::SYNC_CREATE)
                .build();

            // DropDown
            inst.connect_notify_local(Some("unit"), |unit_entry, _pspec| {
                let unit = unit_entry.unit();

                let unit_dropdown_listmodel = unit_entry
                    .unit_dropdown()
                    .model()
                    .unwrap()
                    .downcast::<adw::EnumListModel>()
                    .unwrap();

                unit_entry
                    .unit_dropdown()
                    .set_selected(unit_dropdown_listmodel.find_position(unit as i32));
            });
            self.unit_dropdown.get().connect_selected_notify(
                clone!(@weak inst as unit_entry => move |unit_dropdown| {
                    let unit_dropdown_listmodel = unit_entry
                        .unit_dropdown()
                        .model()
                        .unwrap()
                        .downcast::<adw::EnumListModel>()
                        .unwrap();

                    let item = unit_dropdown_listmodel.item(unit_dropdown.selected());
                    if let Some(item) = item {
                        let unit = match item
                            .downcast::<adw::EnumListItem>()
                            .unwrap()
                            .nick()
                            .as_str()
                        {
                            "px" => Some(format::MeasureUnit::Px),
                            "mm" => Some(format::MeasureUnit::Mm),
                            "cm" => Some(format::MeasureUnit::Cm),
                            _ => None,
                        };

                        if let Some(unit) = unit {
                            unit_entry.set_unit(unit);
                        }
                    };
                }),
            );
        }

        fn dispose(&self) {
            while let Some(child) = self.instance().first_child() {
                child.unparent();
            }
        }

        fn signals() -> &'static [glib::subclass::Signal] {
            static SIGNALS: Lazy<Vec<glib::subclass::Signal>> =
                Lazy::new(|| vec![glib::subclass::Signal::builder("measurement-changed").build()]);
            SIGNALS.as_ref()
        }

        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpecDouble::new(
                        "value",
                        "value",
                        "value",
                        f64::MIN,
                        f64::MAX,
                        1.0,
                        glib::ParamFlags::READWRITE,
                    ),
                    glib::ParamSpecEnum::new(
                        "unit",
                        "unit",
                        "unit",
                        format::MeasureUnit::static_type(),
                        format::MeasureUnit::Px as i32,
                        glib::ParamFlags::READWRITE,
                    ),
                    glib::ParamSpecDouble::new(
                        "dpi",
                        "dpi",
                        "dpi",
                        f64::MIN,
                        f64::MAX,
                        96.0,
                        glib::ParamFlags::READWRITE,
                    ),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "value" => self.value.get().to_value(),
                "unit" => self.unit.get().to_value(),
                "dpi" => self.dpi.get().to_value(),
                _ => unimplemented!(),
            }
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            let inst = self.instance();

            match pspec.name() {
                "value" => {
                    let value_ = value.get::<f64>().expect("The value must be of type 'f64'");
                    if value_ != self.value.get() {
                        self.value.replace(value_);
                        inst.emit_by_name::<()>("measurement-changed", &[]);
                    }
                }
                "unit" => {
                    let unit = value
                        .get::<format::MeasureUnit>()
                        .expect("The value must be of type 'MeasureUnit'");

                    if unit != self.unit.get() {
                        self.unit.replace(unit);
                        inst.emit_by_name::<()>("measurement-changed", &[]);
                    }
                }
                "dpi" => {
                    let dpi = value.get::<f64>().expect("The value must be of type 'f64'");
                    if dpi != self.dpi.get() {
                        self.dpi.replace(dpi);
                        inst.emit_by_name::<()>("measurement-changed", &[]);
                    }
                }
                _ => unimplemented!(),
            }
        }
    }

    impl WidgetImpl for UnitEntry {}
}

glib::wrapper! {
    pub struct UnitEntry(ObjectSubclass<imp::UnitEntry>)
        @extends gtk4::Widget,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget;
}

impl Default for UnitEntry {
    fn default() -> Self {
        Self::new()
    }
}

impl UnitEntry {
    pub fn new() -> Self {
        glib::Object::new(&[])
    }

    pub fn value_adj(&self) -> Adjustment {
        self.imp().value_adj.get()
    }

    pub fn value_spinner(&self) -> SpinButton {
        self.imp().value_spinner.get()
    }

    pub fn unit_dropdown(&self) -> DropDown {
        self.imp().unit_dropdown.get()
    }

    pub fn value(&self) -> f64 {
        self.property::<f64>("value")
    }

    pub fn set_value(&self, value: f64) {
        self.set_property("value", value.to_value());
    }

    pub fn unit(&self) -> format::MeasureUnit {
        self.property::<format::MeasureUnit>("unit")
    }

    pub fn set_unit(&self, unit: format::MeasureUnit) {
        self.set_property("unit", unit.to_value());
    }

    pub fn dpi(&self) -> f64 {
        self.property::<f64>("dpi")
    }

    pub fn set_dpi(&self, dpi: f64) {
        self.set_property("dpi", dpi.to_value());
    }

    pub fn value_in_px(&self) -> f64 {
        format::MeasureUnit::convert_measurement(
            self.value(),
            self.unit(),
            self.dpi(),
            format::MeasureUnit::Px,
            self.dpi(),
        )
    }

    pub fn convert_current_value(&self, desired_unit: format::MeasureUnit) {
        let converted_value = format::MeasureUnit::convert_measurement(
            self.value(),
            self.unit(),
            self.dpi(),
            desired_unit,
            self.dpi(),
        );
        self.set_unit(desired_unit);
        self.set_value(converted_value);
    }
}
