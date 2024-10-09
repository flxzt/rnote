// Imports
use crate::RnStrokeWidthPicker;
use crate::{RnAppWindow, RnCanvasWrapper};
use adw::prelude::*;
use gtk4::{glib, glib::clone, subclass::prelude::*, CompositeTemplate, ToggleButton};
use rnote_engine::pens::pensconfig::eraserconfig::EraserStyle;
use rnote_engine::pens::pensconfig::EraserConfig;

mod imp {
    use super::*;

    #[derive(Default, Debug, CompositeTemplate)]
    #[template(resource = "/com/github/flxzt/rnote/ui/penssidebar/eraserpage.ui")]
    pub(crate) struct RnEraserPage {
        #[template_child]
        pub(crate) eraserstyle_trash_colliding_strokes_toggle: TemplateChild<ToggleButton>,
        #[template_child]
        pub(crate) eraserstyle_split_colliding_strokes_toggle: TemplateChild<ToggleButton>,
        #[template_child]
        pub(crate) stroke_width_picker: TemplateChild<RnStrokeWidthPicker>,
        #[template_child]
        pub(crate) speed_scaling_toggle: TemplateChild<ToggleButton>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for RnEraserPage {
        const NAME: &'static str = "RnEraserPage";
        type Type = super::RnEraserPage;
        type ParentType = gtk4::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for RnEraserPage {
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

    impl WidgetImpl for RnEraserPage {}
}

glib::wrapper! {
    pub(crate) struct RnEraserPage(ObjectSubclass<imp::RnEraserPage>)
        @extends gtk4::Widget;
}

impl Default for RnEraserPage {
    fn default() -> Self {
        Self::new()
    }
}

impl RnEraserPage {
    pub(crate) fn new() -> Self {
        glib::Object::new()
    }

    #[allow(unused)]
    pub(crate) fn eraser_style(&self) -> Option<EraserStyle> {
        if self
            .imp()
            .eraserstyle_trash_colliding_strokes_toggle
            .is_active()
        {
            Some(EraserStyle::TrashCollidingStrokes)
        } else if self
            .imp()
            .eraserstyle_split_colliding_strokes_toggle
            .is_active()
        {
            Some(EraserStyle::SplitCollidingStrokes)
        } else {
            None
        }
    }

    #[allow(unused)]
    pub(crate) fn set_eraser_style(&self, style: EraserStyle) {
        match style {
            EraserStyle::TrashCollidingStrokes => self
                .imp()
                .eraserstyle_trash_colliding_strokes_toggle
                .set_active(true),
            EraserStyle::SplitCollidingStrokes => self
                .imp()
                .eraserstyle_split_colliding_strokes_toggle
                .set_active(true),
        }
    }

    pub(crate) fn stroke_width_picker(&self) -> RnStrokeWidthPicker {
        self.imp().stroke_width_picker.get()
    }

    pub(crate) fn init(&self, appwindow: &RnAppWindow) {
        let imp = self.imp();

        imp.eraserstyle_trash_colliding_strokes_toggle
            .connect_toggled(clone!(
                #[weak]
                appwindow,
                move |eraserstyle_trash_colliding_strokes_toggle| {
                    let Some(canvas) = appwindow.active_tab_canvas() else {
                        return;
                    };

                    if eraserstyle_trash_colliding_strokes_toggle.is_active() {
                        canvas.engine_mut().pens_config.eraser_config.style =
                            EraserStyle::TrashCollidingStrokes;
                    }
                }
            ));

        imp.eraserstyle_split_colliding_strokes_toggle
            .connect_toggled(clone!(
                #[weak]
                appwindow,
                move |eraserstyle_split_colliding_strokes_toggle| {
                    let Some(canvas) = appwindow.active_tab_canvas() else {
                        return;
                    };

                    if eraserstyle_split_colliding_strokes_toggle.is_active() {
                        canvas.engine_mut().pens_config.eraser_config.style =
                            EraserStyle::SplitCollidingStrokes;
                    }
                }
            ));

        // width
        imp.stroke_width_picker.spinbutton().set_digits(0);
        imp.stroke_width_picker
            .spinbutton()
            .set_increments(1.0, 5.0);
        imp.stroke_width_picker
            .spinbutton()
            .set_range(EraserConfig::WIDTH_MIN, EraserConfig::WIDTH_MAX);
        // set value after the range!
        imp.stroke_width_picker
            .set_stroke_width(EraserConfig::WIDTH_DEFAULT);

        imp.stroke_width_picker.connect_notify_local(
            Some("stroke-width"),
            clone!(
                #[weak]
                appwindow,
                move |picker, _| {
                    let stroke_width = picker.stroke_width();
                    let Some(canvas) = appwindow.active_tab_canvas() else {
                        return;
                    };
                    canvas.engine_mut().pens_config.eraser_config.width = stroke_width;
                }
            ),
        );

        imp.speed_scaling_toggle.connect_toggled(clone!(
            #[weak]
            appwindow,
            move |speed_scaling_toggle| {
                let Some(canvas) = appwindow.active_tab_canvas() else {
                    return;
                };

                canvas.engine_mut().pens_config.eraser_config.speed_scaling =
                    speed_scaling_toggle.is_active();
            }
        ));
    }

    pub(crate) fn refresh_ui(&self, active_tab: &RnCanvasWrapper) {
        let imp = self.imp();

        let eraser_config = active_tab
            .canvas()
            .engine_ref()
            .pens_config
            .eraser_config
            .clone();

        imp.stroke_width_picker
            .set_stroke_width(eraser_config.width);

        self.set_eraser_style(eraser_config.style);
    }
}
