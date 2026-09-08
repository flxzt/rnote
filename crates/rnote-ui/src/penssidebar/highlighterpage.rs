// Imports
use crate::{RnAppWindow, RnStrokeWidthPicker};
use adw::prelude::*;
use gtk4::{CompositeTemplate, Widget, glib, glib::clone, subclass::prelude::*};
// Notice we removed the unused BrushStyle, PressureCurve, etc. imports!
use rnote_engine::pens::pensconfig::BrushConfig;

mod imp {
    use super::*;

    #[derive(Default, Debug, CompositeTemplate)]
    #[template(resource = "/com/github/flxzt/rnote/ui/penssidebar/highlighterpage.ui")]
    pub(crate) struct RnHighlighterPage {
        #[template_child]
        pub(crate) stroke_width_picker: TemplateChild<RnStrokeWidthPicker>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for RnHighlighterPage {
        const NAME: &'static str = "RnHighlighterPage";
        type Type = super::RnHighlighterPage;
        type ParentType = Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for RnHighlighterPage {
        fn constructed(&self) {
            self.parent_constructed();
        }

        fn dispose(&self) {
            self.dispose_template();
            while let Some(child) = self.obj().first_child() {
                child.unparent();
            }
        }
    }

    impl WidgetImpl for RnHighlighterPage {}
}

glib::wrapper! {
    pub(crate) struct RnHighlighterPage(ObjectSubclass<imp::RnHighlighterPage>)
        @extends Widget,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget;
}

impl Default for RnHighlighterPage {
    fn default() -> Self {
        Self::new()
    }
}

impl RnHighlighterPage {
    pub(crate) fn new() -> Self {
        glib::Object::new()
    }

    pub(crate) fn stroke_width_picker(&self) -> RnStrokeWidthPicker {
        self.imp().stroke_width_picker.get()
    }

    pub(crate) fn init(&self, appwindow: &RnAppWindow) {
        let imp = self.imp();

        imp.stroke_width_picker
            .spinbutton()
            .set_range(BrushConfig::STROKE_WIDTH_MIN, BrushConfig::STROKE_WIDTH_MAX);

        imp.stroke_width_picker.set_stroke_width(
            rnote_engine::pens::pensconfig::HighlighterConfig::default()
                .marker_options
                .0
                .stroke_width,
        );

        imp.stroke_width_picker.connect_notify_local(
            Some("stroke-width"),
            clone!(
                #[weak]
                appwindow,
                move |picker, _| {
                    let stroke_width = picker.stroke_width();

                    appwindow
                        .engine_config()
                        .write()
                        .pens_config
                        .highlighter_config
                        .marker_options
                        .0
                        .stroke_width = stroke_width;
                }
            ),
        );
    }

    pub(crate) fn refresh_ui(&self, appwindow: &RnAppWindow) {
        let imp = self.imp();

        let stroke_width = appwindow
            .engine_config()
            .read()
            .pens_config
            .highlighter_config
            .marker_options
            .0
            .stroke_width;
        imp.stroke_width_picker.set_stroke_width(stroke_width);
    }
}
