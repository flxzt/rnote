// Imports
use gtk4::{
    glib, glib::clone, prelude::*, subclass::prelude::*, CompositeTemplate, DropDown,
    EventControllerScroll, PropagationPhase, SpinButton, Widget,
};
use num_traits::ToPrimitive;
use once_cell::sync::Lazy;
use rnote_engine::document::format::MeasureUnit;
use std::cell::Cell;

mod imp {
    use super::*;

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/com/github/flxzt/rnote/ui/unitentry.ui")]
    pub(crate) struct RnUnitEntry {
        pub(crate) value: Cell<f64>,
        pub(crate) unit: Cell<MeasureUnit>,
        pub(crate) dpi: Cell<f64>,

        #[template_child]
        pub(crate) value_spinner: TemplateChild<SpinButton>,
        #[template_child]
        pub(crate) unit_dropdown: TemplateChild<DropDown>,
    }

    impl Default for RnUnitEntry {
        fn default() -> Self {
            Self {
                value: Cell::new(1.0),
                unit: Cell::new(MeasureUnit::Px),
                dpi: Cell::new(96.0),
                value_spinner: TemplateChild::<SpinButton>::default(),
                unit_dropdown: TemplateChild::<DropDown>::default(),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for RnUnitEntry {
        const NAME: &'static str = "RnUnitEntry";
        type Type = super::RnUnitEntry;
        type ParentType = Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for RnUnitEntry {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();

            self.configure_spinner(self.unit.get(), self.dpi.get());
            self.value_spinner.set_value(10.0);

            // Disable scrolling entirely,
            // fixing unexpected scrolling issue when the unit entry is inside a scrolled window
            //
            // taken from the implementation libadwaita's SpinRow
            // see: https://gitlab.gnome.org/GNOME/libadwaita/-/blob/44543013d76c37da6f8bbf558ea2d6b57f0bd692/src/adw-spin-row.c#L483
            if let Some(scroll_controller) = self.value_spinner.observe_controllers().into_iter().find(|controller| {
                let Ok(controller) = controller else {

                        tracing::error!("Unable to get scroll controller in RnUnitEntry, controller ListModel mutated while fetching");
                        return false;
                };
                controller.downcast_ref::<EventControllerScroll>().is_some()
            }) {
                let scroll_controller = scroll_controller.unwrap().downcast::<EventControllerScroll>().unwrap();
                scroll_controller.set_propagation_phase(PropagationPhase::None);
            }

            obj.bind_property("value", &self.value_spinner.get(), "value")
                .transform_to(|_, val: f64| Some(val))
                .transform_from(|_, val: f64| Some(val))
                .sync_create()
                .bidirectional()
                .build();

            obj.connect_notify_local(Some("unit"), |unit_entry, _pspec| {
                unit_entry
                    .imp()
                    .unit_dropdown
                    .set_selected(unit_entry.unit().to_u32().unwrap());
            });

            self.unit_dropdown.get().connect_selected_notify(
                clone!(@weak obj as unit_entry => move |unit_dropdown| {
                    unit_entry.set_unit(MeasureUnit::try_from(unit_dropdown.selected()).unwrap());
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
                    glib::ParamSpecDouble::builder("value")
                        .minimum(f64::MIN)
                        .maximum(f64::MAX)
                        .default_value(1.0)
                        .build(),
                    glib::ParamSpecUInt::builder("unit")
                        .default_value(MeasureUnit::Px.to_u32().unwrap())
                        .build(),
                    glib::ParamSpecDouble::builder("dpi")
                        .minimum(f64::MIN)
                        .maximum(f64::MAX)
                        .default_value(96.0)
                        .build(),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "value" => self.value.get().to_value(),
                "unit" => self.unit.get().to_u32().unwrap().to_value(),
                "dpi" => self.dpi.get().to_value(),
                _ => unimplemented!(),
            }
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            let obj = self.obj();

            match pspec.name() {
                "value" => {
                    let value = value.get::<f64>().expect("The value must be of type 'f64'");
                    if value != self.value.get() {
                        self.value.replace(value);
                    }
                }
                "unit" => {
                    let unit = MeasureUnit::try_from(
                        value.get::<u32>().expect("The value must be of type 'u32'"),
                    )
                    .expect("Could not convert u32 to MeasureUnit.");
                    if unit != self.unit.get() {
                        self.configure_spinner(unit, self.dpi.get());
                        obj.set_value(MeasureUnit::convert_measurement(
                            self.value.get(),
                            self.unit.get(),
                            self.dpi.get(),
                            unit,
                            self.dpi.get(),
                        ));
                        self.unit.replace(unit);
                    }
                }
                "dpi" => {
                    let dpi = value.get::<f64>().expect("The value must be of type 'f64'");
                    if dpi != self.dpi.get() {
                        self.configure_spinner(self.unit.get(), dpi);
                        obj.set_value(MeasureUnit::convert_measurement(
                            self.value.get(),
                            self.unit.get(),
                            self.dpi.get(),
                            self.unit.get(),
                            dpi,
                        ));
                        self.dpi.replace(dpi);
                    }
                }
                _ => unimplemented!(),
            }
        }
    }

    impl WidgetImpl for RnUnitEntry {}

    impl RnUnitEntry {
        const MIN_VAL_IN_PX: f64 = 1.0;
        const MAX_VAL_IN_PX: f64 = 100_000.0;

        const STEP_INCREMENT_PX: f64 = 1.0;
        const CLIMB_RATE_PX: f64 = 2.0;
        const DIGITS_PX: u32 = 0;

        const STEP_INCREMENT_MM: f64 = 1.0;
        const CLIMB_RATE_MM: f64 = 2.0;
        const DIGITS_MM: u32 = 1;

        const STEP_INCREMENT_CM: f64 = 0.1;
        const CLIMB_RATE_CM: f64 = 0.2;
        const DIGITS_CM: u32 = 2;

        fn configure_spinner(&self, unit: MeasureUnit, dpi: f64) {
            let min_val = MeasureUnit::convert_measurement(
                Self::MIN_VAL_IN_PX,
                MeasureUnit::Px,
                dpi,
                unit,
                dpi,
            );
            let max_val = MeasureUnit::convert_measurement(
                Self::MAX_VAL_IN_PX,
                MeasureUnit::Px,
                dpi,
                unit,
                dpi,
            );

            let (step_increment, climb_rate, digits) = match unit {
                MeasureUnit::Px => (
                    Self::STEP_INCREMENT_PX,
                    Self::CLIMB_RATE_PX,
                    Self::DIGITS_PX,
                ),
                MeasureUnit::Mm => (
                    Self::STEP_INCREMENT_MM,
                    Self::CLIMB_RATE_MM,
                    Self::DIGITS_MM,
                ),
                MeasureUnit::Cm => (
                    Self::STEP_INCREMENT_CM,
                    Self::CLIMB_RATE_CM,
                    Self::DIGITS_CM,
                ),
            };

            self.value_spinner.set_range(min_val, max_val);
            self.value_spinner
                .set_increments(step_increment, 2.0 * step_increment);
            self.value_spinner.set_climb_rate(climb_rate);
            self.value_spinner.set_digits(digits);
        }
    }
}

glib::wrapper! {
    pub(crate) struct RnUnitEntry(ObjectSubclass<imp::RnUnitEntry>)
        @extends gtk4::Widget,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget;
}

impl Default for RnUnitEntry {
    fn default() -> Self {
        Self::new()
    }
}

impl RnUnitEntry {
    pub(crate) fn new() -> Self {
        glib::Object::new()
    }

    #[allow(unused)]
    pub(crate) fn value(&self) -> f64 {
        self.property::<f64>("value")
    }

    #[allow(unused)]
    pub(crate) fn set_value(&self, value: f64) {
        self.set_property("value", value.to_value());
    }

    #[allow(unused)]
    pub(crate) fn unit(&self) -> MeasureUnit {
        MeasureUnit::try_from(self.property::<u32>("unit"))
            .expect("Could not convert u32 to `MeasureUnit`")
    }

    #[allow(unused)]
    pub(crate) fn set_unit(&self, unit: MeasureUnit) {
        self.set_property("unit", unit.to_u32().unwrap().to_value());
    }

    #[allow(unused)]
    pub(crate) fn dpi(&self) -> f64 {
        self.property::<f64>("dpi")
    }

    #[allow(unused)]
    pub(crate) fn set_dpi(&self, dpi: f64) {
        self.set_property("dpi", dpi.to_value());
    }

    pub(crate) fn value_in_px(&self) -> f64 {
        MeasureUnit::convert_measurement(
            self.value(),
            self.unit(),
            self.dpi(),
            MeasureUnit::Px,
            self.dpi(),
        )
    }

    pub(crate) fn set_value_in_px(&self, val_px: f64) {
        self.set_value(MeasureUnit::convert_measurement(
            val_px,
            MeasureUnit::Px,
            self.dpi(),
            self.unit(),
            self.dpi(),
        ));
    }

    pub(crate) fn set_dpi_keep_value(&self, dpi: f64) {
        let value = self.value();
        self.set_dpi(dpi);
        self.set_value(value);
    }
}
