// Imports
use crate::{
    RnColorPicker, RnPenPicker, appmenu::RnAppMenu, appwindow::RnAppWindow,
    canvasmenu::RnCanvasMenu,
};
use gtk4::{
    Box, CompositeTemplate, EventControllerLegacy, Label, ToggleButton, Widget, glib, glib::clone,
    prelude::*, subclass::prelude::*,
};
use rnote_engine::ext::GdkRGBAExt;
use rnote_engine::pens::PenStyle;

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/github/flxzt/rnote/ui/mainheader.ui")]
    pub(crate) struct RnMainHeader {
        #[template_child]
        pub(crate) headerbar: TemplateChild<adw::HeaderBar>,
        #[template_child]
        pub(crate) colorpicker: TemplateChild<RnColorPicker>,
        #[template_child]
        pub(crate) penpicker: TemplateChild<RnPenPicker>,
        #[template_child]
        pub(crate) left_sidebar_reveal_toggle: TemplateChild<ToggleButton>,
        #[template_child]
        pub(crate) right_sidebar_reveal_toggle: TemplateChild<ToggleButton>,
        #[template_child]
        pub(crate) canvasmenu: TemplateChild<RnCanvasMenu>,
        #[template_child]
        pub(crate) appmenu: TemplateChild<RnAppMenu>,
        #[template_child]
        pub(crate) quickactions_box: TemplateChild<Box>,
        #[template_child]
        pub(crate) right_buttons_box: TemplateChild<Box>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for RnMainHeader {
        const NAME: &'static str = "RnMainHeader";
        type Type = super::RnMainHeader;
        type ParentType = Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for RnMainHeader {
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
    impl WidgetImpl for RnMainHeader {}
}

glib::wrapper! {
    pub(crate) struct RnMainHeader(ObjectSubclass<imp::RnMainHeader>)
        @extends Widget,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget;
}

impl Default for RnMainHeader {
    fn default() -> Self {
        Self::new()
    }
}

impl RnMainHeader {
    pub(crate) fn new() -> Self {
        glib::Object::new()
    }

    pub(crate) fn headerbar(&self) -> adw::HeaderBar {
        self.imp().headerbar.get()
    }

    pub(crate) fn colorpicker(&self) -> RnColorPicker {
        self.imp().colorpicker.get()
    }

    pub(crate) fn penpicker(&self) -> RnPenPicker {
        self.imp().penpicker.get()
    }

    pub(crate) fn left_sidebar_reveal_toggle(&self) -> ToggleButton {
        self.imp().left_sidebar_reveal_toggle.get()
    }

    pub(crate) fn right_sidebar_reveal_toggle(&self) -> ToggleButton {
        self.imp().right_sidebar_reveal_toggle.get()
    }

    pub(crate) fn canvasmenu(&self) -> RnCanvasMenu {
        self.imp().canvasmenu.get()
    }

    pub(crate) fn appmenu(&self) -> RnAppMenu {
        self.imp().appmenu.get()
    }

    pub(crate) fn init(&self, appwindow: &RnAppWindow) {
        let imp = self.imp();

        imp.canvasmenu.get().init(appwindow);
        imp.appmenu.get().init(appwindow);
        imp.colorpicker.get().init(appwindow);
        imp.penpicker.get().init(appwindow);

        self.setup_colorpicker(appwindow);

        // add controllers to elements to prevent accidental resizes: left buttons
        let capture_left = EventControllerLegacy::builder()
            .name("capture_event_left")
            .propagation_phase(gtk4::PropagationPhase::Bubble)
            .build();

        capture_left.connect_event(|_, _| glib::Propagation::Stop);
        imp.quickactions_box.add_controller(capture_left);

        // add controllers to elements to prevent accidental resizes: right buttons
        let capture_right = EventControllerLegacy::builder()
            .name("capture_event_right")
            .propagation_phase(gtk4::PropagationPhase::Bubble)
            .build();

        capture_right.connect_event(|_, _| glib::Propagation::Stop);
        imp.right_buttons_box.add_controller(capture_right);
    }

    fn setup_colorpicker(&self, appwindow: &RnAppWindow) {
        let imp = self.imp();

        imp.colorpicker.connect_notify_local(
            Some("stroke-color"),
            clone!(
                #[weak]
                appwindow,
                move |colorpicker, _paramspec| {
                    let Some(canvas) = appwindow.active_tab_canvas() else {
                        return;
                    };
                    let stroke_color = colorpicker.stroke_color().into_compose_color();
                    let current_pen_style = canvas.engine_ref().current_pen_style_w_override();

                    match current_pen_style {
                        PenStyle::Typewriter => {
                            let widget_flags = canvas.engine_mut().text_change_color(stroke_color);
                            appwindow.handle_widget_flags(widget_flags, &canvas);
                        }
                        PenStyle::Selector => {
                            let widget_flags = canvas
                                .engine_mut()
                                .change_selection_stroke_colors(stroke_color);
                            appwindow.handle_widget_flags(widget_flags, &canvas);
                        }
                        PenStyle::Brush | PenStyle::Shaper | PenStyle::Eraser | PenStyle::Tools => {
                        }
                    }

                    // We have a global colorpicker, so we apply it to all styles
                    appwindow
                        .engine_config()
                        .write()
                        .pens_config
                        .set_all_stroke_colors(stroke_color);
                }
            ),
        );

        imp.colorpicker.connect_notify_local(
            Some("fill-color"),
            clone!(
                #[weak]
                appwindow,
                move |colorpicker, _paramspec| {
                    let Some(canvas) = appwindow.active_tab_canvas() else {
                        return;
                    };
                    let fill_color = colorpicker.fill_color().into_compose_color();
                    let stroke_style = canvas.engine_ref().current_pen_style_w_override();

                    match stroke_style {
                        PenStyle::Selector => {
                            let widget_flags =
                                canvas.engine_mut().change_selection_fill_colors(fill_color);
                            appwindow.handle_widget_flags(widget_flags, &canvas);
                        }
                        PenStyle::Typewriter
                        | PenStyle::Brush
                        | PenStyle::Shaper
                        | PenStyle::Eraser
                        | PenStyle::Tools => {}
                    }

                    // We have a global colorpicker, so we apply it to all styles
                    appwindow
                        .engine_config()
                        .write()
                        .pens_config
                        .set_all_fill_colors(fill_color);
                }
            ),
        );
    }
}
