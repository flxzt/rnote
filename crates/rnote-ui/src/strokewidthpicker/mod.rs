// Modules
mod previewstyle;
mod strokewidthpreview;
mod strokewidthsetter;

// Re-exports
pub(crate) use previewstyle::StrokeWidthPreviewStyle;
pub(crate) use strokewidthpreview::RnStrokeWidthPreview;
pub(crate) use strokewidthsetter::RnStrokeWidthSetter;

// Imports
use gtk4::{
    glib, glib::clone, prelude::*, subclass::prelude::*, BoxLayout, CompositeTemplate, Orientation,
    PositionType, SpinButton, Widget,
};
use once_cell::sync::Lazy;
use std::cell::Cell;

mod imp {
    use super::*;

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/com/github/flxzt/rnote/ui/strokewidthpicker.ui")]
    pub(crate) struct RnStrokeWidthPicker {
        pub(crate) position: Cell<PositionType>,
        pub(crate) stroke_width: Cell<f64>,
        pub(crate) preview_style: Cell<StrokeWidthPreviewStyle>,

        #[template_child]
        pub(crate) spinbutton: TemplateChild<SpinButton>,
        #[template_child]
        pub(crate) setter_box: TemplateChild<gtk4::Box>,
        #[template_child]
        pub(crate) setter_1: TemplateChild<RnStrokeWidthSetter>,
        #[template_child]
        pub(crate) setter_2: TemplateChild<RnStrokeWidthSetter>,
        #[template_child]
        pub(crate) setter_3: TemplateChild<RnStrokeWidthSetter>,
    }

    impl Default for RnStrokeWidthPicker {
        fn default() -> Self {
            Self {
                position: Cell::new(PositionType::Right),
                stroke_width: Cell::new(1.0),
                preview_style: Cell::new(StrokeWidthPreviewStyle::Circle),

                spinbutton: TemplateChild::default(),
                setter_box: TemplateChild::default(),
                setter_1: TemplateChild::default(),
                setter_2: TemplateChild::default(),
                setter_3: TemplateChild::default(),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for RnStrokeWidthPicker {
        const NAME: &'static str = "RnStrokeWidthPicker";
        type Type = super::RnStrokeWidthPicker;
        type ParentType = Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for RnStrokeWidthPicker {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();

            self.spinbutton.set_increments(0.1, 2.0);

            obj.bind_property("stroke-width", &*self.spinbutton, "value")
                .sync_create()
                .bidirectional()
                .build();
            obj.bind_property("preview-style", &self.setter_1.preview(), "preview-style")
                .sync_create()
                .build();
            obj.bind_property("preview-style", &self.setter_2.preview(), "preview-style")
                .sync_create()
                .build();
            obj.bind_property("preview-style", &self.setter_3.preview(), "preview-style")
                .sync_create()
                .build();

            self.setter_1.set_stroke_width(2.0);
            self.setter_2.set_stroke_width(8.0);
            self.setter_3.set_stroke_width(16.0);

            self.setter_1.connect_active_notify(clone!(
                #[weak(rename_to=strokewidthpicker)]
                obj,
                move |setter| {
                    if setter.is_active() {
                        strokewidthpicker.setter_2().set_active(false);
                        strokewidthpicker.setter_3().set_active(false);
                        // Must come after setting the other toggles inactive
                        strokewidthpicker.set_stroke_width(setter.stroke_width());
                    }
                }
            ));

            self.setter_2.connect_active_notify(clone!(
                #[weak(rename_to=strokewidthpicker)]
                obj,
                move |setter| {
                    if setter.is_active() {
                        strokewidthpicker.setter_1().set_active(false);
                        strokewidthpicker.setter_3().set_active(false);
                        strokewidthpicker.set_stroke_width(setter.stroke_width());
                    }
                }
            ));

            self.setter_3.connect_active_notify(clone!(
                #[weak(rename_to=strokewidthpicker)]
                obj,
                move |setter| {
                    if setter.is_active() {
                        strokewidthpicker.setter_1().set_active(false);
                        strokewidthpicker.setter_2().set_active(false);
                        strokewidthpicker.set_stroke_width(setter.stroke_width());
                    }
                }
            ));

            self.spinbutton.connect_value_changed(clone!(
                #[weak(rename_to=strokewidthpicker)]
                obj,
                move |spinbutton| {
                    strokewidthpicker.set_active_setter_stroke_width(spinbutton.value());
                }
            ));
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
                    glib::ParamSpecEnum::builder_with_default("position", PositionType::Right)
                        .build(),
                    glib::ParamSpecDouble::builder("stroke-width")
                        .minimum(0.0)
                        .maximum(500.0)
                        .default_value(1.0)
                        .build(),
                    glib::ParamSpecEnum::builder::<StrokeWidthPreviewStyle>("preview-style")
                        .default_value(StrokeWidthPreviewStyle::Circle)
                        .build(),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "position" => self.position.get().to_value(),
                "stroke-width" => self.stroke_width.get().to_value(),
                "preview-style" => self.preview_style.get().to_value(),
                _ => panic!("invalid property name"),
            }
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            let obj = self.obj();

            match pspec.name() {
                "position" => {
                    let position = value
                        .get::<PositionType>()
                        .expect("value not of type `PositionType`");
                    self.position.replace(position);

                    let layout_manager = obj
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
                "preview-style" => {
                    let preview_style = value
                        .get::<StrokeWidthPreviewStyle>()
                        .expect("value not of type `StrokeWidthPreviewStyle`");
                    self.preview_style.set(preview_style);
                    obj.queue_draw();
                }
                _ => panic!("invalid property name"),
            }
        }
    }

    impl WidgetImpl for RnStrokeWidthPicker {}

    impl RnStrokeWidthPicker {}
}

glib::wrapper! {
    pub(crate) struct RnStrokeWidthPicker(ObjectSubclass<imp::RnStrokeWidthPicker>)
        @extends Widget,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget;
}

impl Default for RnStrokeWidthPicker {
    fn default() -> Self {
        Self::new()
    }
}

impl RnStrokeWidthPicker {
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
    pub(crate) fn stroke_width(&self) -> f64 {
        self.property::<f64>("stroke-width")
    }

    #[allow(unused)]
    pub(crate) fn set_stroke_width(&self, stroke_width: f64) {
        self.set_property("stroke-width", stroke_width.to_value());
    }

    #[allow(unused)]
    pub(crate) fn preview_style(&self) -> StrokeWidthPreviewStyle {
        self.property::<StrokeWidthPreviewStyle>("preview-style")
    }

    #[allow(unused)]
    pub(crate) fn set_preview_style(&self, preview_style: f64) {
        self.set_property("preview-style", preview_style.to_value());
    }

    pub(crate) fn spinbutton(&self) -> SpinButton {
        self.imp().spinbutton.get()
    }

    pub(crate) fn setter_1(&self) -> RnStrokeWidthSetter {
        self.imp().setter_1.get()
    }

    pub(crate) fn setter_2(&self) -> RnStrokeWidthSetter {
        self.imp().setter_2.get()
    }

    pub(crate) fn setter_3(&self) -> RnStrokeWidthSetter {
        self.imp().setter_3.get()
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
