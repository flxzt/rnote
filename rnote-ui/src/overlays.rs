use gtk4::{
    glib, glib::clone, prelude::*, subclass::prelude::*, CompositeTemplate, ProgressBar,
    ToggleButton, Widget,
};
use rnote_engine::engine::EngineViewMut;
use rnote_engine::pens::{Pen, PenStyle};
use rnote_engine::utils::GdkRGBAHelpers;
use std::cell::RefCell;

use crate::canvaswrapper::RnoteCanvasWrapper;
use crate::{dialogs, ColorPicker, RnoteAppWindow};

mod imp {
    use super::*;

    #[allow(missing_debug_implementations)]
    #[derive(CompositeTemplate)]
    #[template(resource = "/com/github/flxzt/rnote/ui/overlays.ui")]
    pub(crate) struct RnoteOverlays {
        pub(crate) progresspulse_source_id: RefCell<Option<glib::SourceId>>,

        #[template_child]
        pub(crate) toast_overlay: TemplateChild<adw::ToastOverlay>,
        #[template_child]
        pub(crate) progressbar: TemplateChild<ProgressBar>,
        #[template_child]
        pub(crate) pens_toggles_box: TemplateChild<gtk4::Box>,
        #[template_child]
        pub(crate) brush_toggle: TemplateChild<ToggleButton>,
        #[template_child]
        pub(crate) shaper_toggle: TemplateChild<ToggleButton>,
        #[template_child]
        pub(crate) typewriter_toggle: TemplateChild<ToggleButton>,
        #[template_child]
        pub(crate) eraser_toggle: TemplateChild<ToggleButton>,
        #[template_child]
        pub(crate) selector_toggle: TemplateChild<ToggleButton>,
        #[template_child]
        pub(crate) tools_toggle: TemplateChild<ToggleButton>,
        #[template_child]
        pub(crate) colorpicker: TemplateChild<ColorPicker>,
        #[template_child]
        pub(crate) tabview: TemplateChild<adw::TabView>,
    }

    impl Default for RnoteOverlays {
        fn default() -> Self {
            Self {
                progresspulse_source_id: RefCell::new(None),

                toast_overlay: TemplateChild::<adw::ToastOverlay>::default(),
                progressbar: TemplateChild::<ProgressBar>::default(),
                pens_toggles_box: TemplateChild::<gtk4::Box>::default(),
                brush_toggle: TemplateChild::<ToggleButton>::default(),
                shaper_toggle: TemplateChild::<ToggleButton>::default(),
                typewriter_toggle: TemplateChild::<ToggleButton>::default(),
                eraser_toggle: TemplateChild::<ToggleButton>::default(),
                selector_toggle: TemplateChild::<ToggleButton>::default(),
                tools_toggle: TemplateChild::<ToggleButton>::default(),
                colorpicker: TemplateChild::<ColorPicker>::default(),
                tabview: TemplateChild::<adw::TabView>::default(),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for RnoteOverlays {
        const NAME: &'static str = "RnoteOverlays";
        type Type = super::RnoteOverlays;
        type ParentType = Widget;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for RnoteOverlays {
        fn constructed(&self) {
            self.parent_constructed();
        }

        fn dispose(&self) {
            while let Some(child) = self.instance().first_child() {
                child.unparent();
            }
        }
    }

    impl WidgetImpl for RnoteOverlays {}
}

glib::wrapper! {
    pub(crate) struct RnoteOverlays(ObjectSubclass<imp::RnoteOverlays>)
    @extends Widget;
}

impl Default for RnoteOverlays {
    fn default() -> Self {
        Self::new()
    }
}

impl RnoteOverlays {
    pub(crate) fn new() -> Self {
        glib::Object::new(&[])
    }

    pub(crate) fn brush_toggle(&self) -> ToggleButton {
        self.imp().brush_toggle.get()
    }

    pub(crate) fn shaper_toggle(&self) -> ToggleButton {
        self.imp().shaper_toggle.get()
    }

    pub(crate) fn typewriter_toggle(&self) -> ToggleButton {
        self.imp().typewriter_toggle.get()
    }

    pub(crate) fn eraser_toggle(&self) -> ToggleButton {
        self.imp().eraser_toggle.get()
    }

    pub(crate) fn selector_toggle(&self) -> ToggleButton {
        self.imp().selector_toggle.get()
    }

    pub(crate) fn tools_toggle(&self) -> ToggleButton {
        self.imp().tools_toggle.get()
    }

    pub(crate) fn colorpicker(&self) -> ColorPicker {
        self.imp().colorpicker.get()
    }

    pub(crate) fn toast_overlay(&self) -> adw::ToastOverlay {
        self.imp().toast_overlay.get()
    }

    pub(crate) fn progressbar(&self) -> ProgressBar {
        self.imp().progressbar.get()
    }

    pub(crate) fn tabview(&self) -> adw::TabView {
        self.imp().tabview.get()
    }

    pub(crate) fn init(&self, appwindow: &RnoteAppWindow) {
        let imp = self.imp();

        self.setup_pens_toggles(appwindow);
        self.setup_colorpicker(appwindow);

        imp.tabview
            .connect_selected_page_notify(clone!(@weak appwindow => move |_tabview| {
                appwindow.clear_rendering_inactive_tabs();
                appwindow.active_tab().canvas().regenerate_background_pattern();
                appwindow.active_tab().canvas().update_engine_rendering();
                adw::prelude::ActionGroupExt::activate_action(&appwindow, "sync-state-active-tab", None);
            }));

        imp.tabview
            .connect_page_attached(clone!(@weak appwindow => move |_tabview, page, _| {
                let canvaswrapper = page.child().downcast::<RnoteCanvasWrapper>().unwrap();

                canvaswrapper.init_reconnect(&appwindow);
                canvaswrapper.connect_to_tab_page(&page);
                adw::prelude::ActionGroupExt::activate_action(&appwindow, "sync-state-active-tab", None);
            }));

        imp.tabview
            .connect_page_detached(clone!(@weak appwindow => move |_tabview, page, _| {
                let canvaswrapper = page.child().downcast::<RnoteCanvasWrapper>().unwrap();

                canvaswrapper.disconnect_handlers(&appwindow);
            }));

        imp.tabview.connect_close_page(
            clone!(@weak appwindow => @default-return true, move |tabview, page| {
                if !page.is_pinned() {
                    if page
                        .child()
                        .downcast::<RnoteCanvasWrapper>()
                        .unwrap()
                        .canvas()
                        .unsaved_changes()
                    {
                        // We close after showing the dialog
                        dialogs::dialog_close_tab(&appwindow, &page);
                    } else {
                        tabview.close_page_finish(&page, true);
                    }
                } else {
                    tabview.close_page_finish(&page, false);
                }

                true
            }),
        );
    }

    fn setup_pens_toggles(&self, appwindow: &RnoteAppWindow) {
        let imp = self.imp();

        imp.brush_toggle.connect_toggled(clone!(@weak appwindow => move |brush_toggle| {
                if brush_toggle.is_active() {
                    adw::prelude::ActionGroupExt::activate_action(&appwindow, "pen-style", Some(&PenStyle::Brush.to_variant()));
                }
            }));

        imp.shaper_toggle.connect_toggled(clone!(@weak appwindow => move |shaper_toggle| {
                if shaper_toggle.is_active() {
                    adw::prelude::ActionGroupExt::activate_action(&appwindow, "pen-style", Some(&PenStyle::Shaper.to_variant()));
                }
            }));

        imp.typewriter_toggle.connect_toggled(clone!(@weak appwindow => move |typewriter_toggle| {
                if typewriter_toggle.is_active() {
                    adw::prelude::ActionGroupExt::activate_action(&appwindow, "pen-style", Some(&PenStyle::Typewriter.to_variant()));
                }
            }));

        imp.eraser_toggle.get().connect_toggled(clone!(@weak appwindow => move |eraser_toggle| {
                if eraser_toggle.is_active() {
                    adw::prelude::ActionGroupExt::activate_action(&appwindow, "pen-style", Some(&PenStyle::Eraser.to_variant()));
                }
            }));

        imp.selector_toggle.get().connect_toggled(clone!(@weak appwindow => move |selector_toggle| {
                if selector_toggle.is_active() {
                    adw::prelude::ActionGroupExt::activate_action(&appwindow, "pen-style", Some(&PenStyle::Selector.to_variant()));
                }
            }));

        imp.tools_toggle.get().connect_toggled(clone!(@weak appwindow => move |tools_toggle| {
                if tools_toggle.is_active() {
                    adw::prelude::ActionGroupExt::activate_action(&appwindow, "pen-style", Some(&PenStyle::Tools.to_variant()));
                }
            }));
    }

    fn setup_colorpicker(&self, appwindow: &RnoteAppWindow) {
        let imp = self.imp();

        imp.colorpicker.connect_notify_local(
                Some("stroke-color"),
                clone!(@weak appwindow => move |colorpicker, _paramspec| {
                    let stroke_style = appwindow.active_tab().canvas().engine().borrow().penholder.current_style_w_override();
                    let stroke_color = colorpicker.stroke_color().into_compose_color();
                    let engine = appwindow.active_tab().canvas().engine();
                    let engine = &mut *engine.borrow_mut();

                    // We have a global colorpicker, so we apply it to all styles
                    engine.pens_config.brush_config.marker_options.stroke_color = Some(stroke_color);
                    engine.pens_config.brush_config.solid_options.stroke_color = Some(stroke_color);
                    engine.pens_config.brush_config.textured_options.stroke_color = Some(stroke_color);
                    engine.pens_config.shaper_config.smooth_options.stroke_color = Some(stroke_color);
                    engine.pens_config.shaper_config.rough_options.stroke_color= Some(stroke_color);
                    engine.pens_config.typewriter_config.text_style.color = stroke_color;

                    match stroke_style {
                        PenStyle::Typewriter => {
                            if let Pen::Typewriter(typewriter) = engine.penholder.current_pen_mut() {
                                let widget_flags = typewriter.change_text_style_in_modifying_stroke(
                                    |text_style| {
                                        text_style.color = stroke_color;
                                    },
                                    &mut EngineViewMut {
                                        tasks_tx: engine.tasks_tx.clone(),
                                        pens_config: &mut engine.pens_config,
                                        doc: &mut engine.document,
                                        store: &mut engine.store,
                                        camera: &mut engine.camera,
                                        audioplayer: &mut engine.audioplayer
                                });
                                appwindow.handle_widget_flags(widget_flags);
                            }
                        }
                        PenStyle::Brush | PenStyle::Shaper | PenStyle::Eraser | PenStyle::Selector | PenStyle::Tools => {}
                    }
                }),
            );

        imp.colorpicker.connect_notify_local(
            Some("fill-color"),
            clone!(@weak appwindow => move |colorpicker, _paramspec| {
                let fill_color = colorpicker.fill_color().into_compose_color();
                let engine = appwindow.active_tab().canvas().engine();
                let engine = &mut *engine.borrow_mut();

                // We have a global colorpicker, so we apply it to all styles
                engine.pens_config.brush_config.marker_options.fill_color = Some(fill_color);
                engine.pens_config.brush_config.solid_options.fill_color= Some(fill_color);
                engine.pens_config.shaper_config.smooth_options.fill_color = Some(fill_color);
                engine.pens_config.shaper_config.rough_options.fill_color= Some(fill_color);
            }),
        );
    }

    pub(crate) fn start_pulsing_progressbar(&self) {
        const PROGRESS_BAR_PULSE_INTERVAL: std::time::Duration =
            std::time::Duration::from_millis(300);

        if let Some(old_pulse_source) = self.imp().progresspulse_source_id.replace(Some(glib::source::timeout_add_local(
            PROGRESS_BAR_PULSE_INTERVAL,
            clone!(@weak self as appwindow => @default-return glib::source::Continue(false), move || {
                appwindow.progressbar().pulse();

                glib::source::Continue(true)
            })),
        )) {
            old_pulse_source.remove();
        }
    }

    pub(crate) fn finish_progressbar(&self) {
        const PROGRESS_BAR_TIMEOUT_TIME: std::time::Duration =
            std::time::Duration::from_millis(300);

        if let Some(pulse_source) = self.imp().progresspulse_source_id.take() {
            pulse_source.remove();
        }

        self.progressbar().set_fraction(1.0);

        glib::source::timeout_add_local_once(
            PROGRESS_BAR_TIMEOUT_TIME,
            clone!(@weak self as appwindow => move || {
                appwindow.progressbar().set_fraction(0.0);
            }),
        );
    }

    #[allow(unused)]
    pub(crate) fn abort_progressbar(&self) {
        if let Some(pulse_source) = self.imp().progresspulse_source_id.take() {
            pulse_source.remove();
        }

        self.progressbar().set_fraction(0.0);
    }

    pub(crate) fn dispatch_toast_w_button<F: Fn(&adw::Toast) + 'static>(
        &self,
        text: &str,
        button_label: &str,
        button_callback: F,
        timeout: u32,
    ) -> adw::Toast {
        let text_notify_toast = adw::Toast::builder()
            .title(text)
            .priority(adw::ToastPriority::High)
            .button_label(button_label)
            .timeout(timeout)
            .build();

        text_notify_toast.connect_button_clicked(button_callback);
        self.toast_overlay().add_toast(&text_notify_toast);

        text_notify_toast
    }

    /// Ensures that only one toast per `singleton_toast` is queued at the same time by dismissing the previous toast.
    ///
    /// `singleton_toast` is a mutable reference to an `Option<Toast>`. It will always hold the most recently dispatched toast
    /// and it should not be modified, because it's used to keep track of previous toasts.
    pub(crate) fn dispatch_toast_w_button_singleton<F: Fn(&adw::Toast) + 'static>(
        &self,
        text: &str,
        button_label: &str,
        button_callback: F,
        timeout: u32,
        singleton_toast: &mut Option<adw::Toast>,
    ) {
        if let Some(previous_toast) = singleton_toast {
            previous_toast.dismiss();
        }

        let text_notify_toast =
            self.dispatch_toast_w_button(text, button_label, button_callback, timeout);
        *singleton_toast = Some(text_notify_toast);
    }

    pub(crate) fn dispatch_toast_text(&self, text: &str) {
        let text_notify_toast = adw::Toast::builder()
            .title(text)
            .priority(adw::ToastPriority::High)
            .timeout(5)
            .build();

        self.toast_overlay().add_toast(&text_notify_toast);
    }

    pub(crate) fn dispatch_toast_error(&self, error: &String) {
        let text_notify_toast = adw::Toast::builder()
            .title(error.as_str())
            .priority(adw::ToastPriority::High)
            .timeout(0)
            .build();

        log::error!("{error}");

        self.toast_overlay().add_toast(&text_notify_toast);
    }
}
