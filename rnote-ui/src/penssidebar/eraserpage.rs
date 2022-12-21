use crate::appwindow::RnoteAppWindow;
use adw::prelude::*;
use gtk4::{glib, glib::clone, subclass::prelude::*, CompositeTemplate, SpinButton, ToggleButton};
use rnote_engine::pens::pensconfig::eraserconfig::EraserStyle;
use rnote_engine::pens::pensconfig::EraserConfig;

mod imp {
    use super::*;

    #[derive(Default, Debug, CompositeTemplate)]
    #[template(resource = "/com/github/flxzt/rnote/ui/penssidebar/eraserpage.ui")]
    pub(crate) struct EraserPage {
        #[template_child]
        pub(crate) eraserstyle_trash_colliding_strokes_toggle: TemplateChild<ToggleButton>,
        #[template_child]
        pub(crate) eraserstyle_split_colliding_strokes_toggle: TemplateChild<ToggleButton>,
        #[template_child]
        pub(crate) width_spinbutton: TemplateChild<SpinButton>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for EraserPage {
        const NAME: &'static str = "EraserPage";
        type Type = super::EraserPage;
        type ParentType = gtk4::Widget;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for EraserPage {
        fn constructed(&self) {
            self.parent_constructed();
        }

        fn dispose(&self) {
            while let Some(child) = self.instance().first_child() {
                child.unparent();
            }
        }
    }

    impl WidgetImpl for EraserPage {}
}

glib::wrapper! {
    pub(crate) struct EraserPage(ObjectSubclass<imp::EraserPage>)
        @extends gtk4::Widget;
}

impl Default for EraserPage {
    fn default() -> Self {
        Self::new()
    }
}

impl EraserPage {
    pub(crate) fn new() -> Self {
        glib::Object::new(&[])
    }

    pub(crate) fn init(&self, appwindow: &RnoteAppWindow) {
        let imp = self.imp();

        imp.eraserstyle_trash_colliding_strokes_toggle.connect_toggled(clone!(@weak appwindow => move |eraserstyle_trash_colliding_strokes_toggle| {
            if eraserstyle_trash_colliding_strokes_toggle.is_active() {
                appwindow.canvas().engine().borrow_mut().pens_config.eraser_config.style = EraserStyle::TrashCollidingStrokes;
            }
        }));

        imp.eraserstyle_split_colliding_strokes_toggle.connect_toggled(clone!(@weak appwindow => move |eraserstyle_split_colliding_strokes_toggle| {
            if eraserstyle_split_colliding_strokes_toggle.is_active() {
                appwindow.canvas().engine().borrow_mut().pens_config.eraser_config.style = EraserStyle::SplitCollidingStrokes;
            }
        }));

        imp.width_spinbutton.set_increments(1.0, 5.0);
        imp.width_spinbutton
            .set_range(EraserConfig::WIDTH_MIN, EraserConfig::WIDTH_MAX);
        imp.width_spinbutton.set_value(EraserConfig::WIDTH_DEFAULT);

        imp.width_spinbutton.connect_value_changed(
            clone!(@weak appwindow => move |width_spinbutton| {
                appwindow.canvas().engine().borrow_mut().pens_config.eraser_config.width = width_spinbutton.value();
            }),
        );
    }

    pub(crate) fn refresh_ui(&self, appwindow: &RnoteAppWindow) {
        let imp = self.imp();

        let eraser_config = appwindow
            .canvas()
            .engine()
            .borrow()
            .pens_config
            .eraser_config
            .clone();

        imp.width_spinbutton.set_value(eraser_config.width);
        match eraser_config.style {
            EraserStyle::TrashCollidingStrokes => imp
                .eraserstyle_trash_colliding_strokes_toggle
                .set_active(true),
            EraserStyle::SplitCollidingStrokes => imp
                .eraserstyle_split_colliding_strokes_toggle
                .set_active(true),
        }
    }
}
