// Imports
use crate::canvaswrapper::RnCanvasWrapper;
use crate::RnPensSideBar;
use crate::{dialogs, RnAppWindow, RnColorPicker, RnPenPicker};
use core::time::Duration;
use gtk4::{
    gio, glib, glib::clone, prelude::*, subclass::prelude::*, CompositeTemplate, Overlay,
    ProgressBar, ScrolledWindow, Widget,
};
use rnote_engine::ext::GdkRGBAExt;
use rnote_engine::pens::PenStyle;
use std::cell::{Cell, RefCell};

mod imp {
    use super::*;

    #[derive(Default, Debug, CompositeTemplate)]
    #[template(resource = "/com/github/flxzt/rnote/ui/overlays.ui")]
    pub(crate) struct RnOverlays {
        pub(crate) progresspulses_active: Cell<usize>,
        pub(crate) progresspulse_id: RefCell<Option<glib::SourceId>>,
        pub(super) prev_active_tab_page: glib::WeakRef<adw::TabPage>,

        #[template_child]
        pub(crate) toolbar_overlay: TemplateChild<Overlay>,
        #[template_child]
        pub(crate) toast_overlay: TemplateChild<adw::ToastOverlay>,
        #[template_child]
        pub(crate) progressbar: TemplateChild<ProgressBar>,
        #[template_child]
        pub(crate) penpicker: TemplateChild<RnPenPicker>,
        #[template_child]
        pub(crate) colorpicker: TemplateChild<RnColorPicker>,
        #[template_child]
        pub(crate) tabview: TemplateChild<adw::TabView>,
        #[template_child]
        pub(crate) sidebar_box: TemplateChild<gtk4::Box>,
        #[template_child]
        pub(crate) sidebar_scroller: TemplateChild<ScrolledWindow>,
        #[template_child]
        pub(crate) penssidebar: TemplateChild<RnPensSideBar>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for RnOverlays {
        const NAME: &'static str = "RnOverlays";
        type Type = super::RnOverlays;
        type ParentType = Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for RnOverlays {
        fn constructed(&self) {
            self.parent_constructed();

            self.setup_toolbar_overlay();
        }

        fn dispose(&self) {
            self.dispose_template();
            while let Some(child) = self.obj().first_child() {
                child.unparent();
            }
        }
    }

    impl WidgetImpl for RnOverlays {}
    impl RnOverlays {
        fn setup_toolbar_overlay(&self) {
            self.toolbar_overlay
                .set_measure_overlay(&*self.colorpicker, true);
            self.toolbar_overlay
                .set_measure_overlay(&*self.penpicker, true);
            self.toolbar_overlay
                .set_measure_overlay(&*self.sidebar_box, true);
        }
    }
}

/// The default timeout for regular text toasts.
pub(crate) const TEXT_TOAST_TIMEOUT_DEFAULT: Option<Duration> = Some(Duration::from_secs(5));

glib::wrapper! {
    pub(crate) struct RnOverlays(ObjectSubclass<imp::RnOverlays>)
    @extends Widget;
}

impl Default for RnOverlays {
    fn default() -> Self {
        Self::new()
    }
}

impl RnOverlays {
    pub(crate) fn new() -> Self {
        glib::Object::new()
    }

    pub(crate) fn penpicker(&self) -> RnPenPicker {
        self.imp().penpicker.get()
    }

    pub(crate) fn colorpicker(&self) -> RnColorPicker {
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

    pub(crate) fn sidebar_box(&self) -> gtk4::Box {
        self.imp().sidebar_box.get()
    }

    pub(crate) fn sidebar_scroller(&self) -> ScrolledWindow {
        self.imp().sidebar_scroller.get()
    }

    pub(crate) fn penssidebar(&self) -> RnPensSideBar {
        self.imp().penssidebar.get()
    }

    pub(crate) fn init(&self, appwindow: &RnAppWindow) {
        let imp = self.imp();
        imp.colorpicker.get().init(appwindow);
        imp.penssidebar.get().init(appwindow);
        imp.penpicker.get().init(appwindow);
        imp.penssidebar.get().brush_page().init(appwindow);
        imp.penssidebar.get().shaper_page().init(appwindow);
        imp.penssidebar.get().typewriter_page().init(appwindow);
        imp.penssidebar.get().eraser_page().init(appwindow);
        imp.penssidebar.get().selector_page().init(appwindow);
        imp.penssidebar.get().tools_page().init(appwindow);

        self.setup_colorpicker(appwindow);
        self.setup_tabview(appwindow);
    }

    fn setup_colorpicker(&self, appwindow: &RnAppWindow) {
        let imp = self.imp();

        imp.colorpicker.connect_notify_local(
                Some("stroke-color"),
                clone!(@weak appwindow => move |colorpicker, _paramspec| {
                    let stroke_color = colorpicker.stroke_color().into_compose_color();
                    let canvas = appwindow.active_tab_wrapper().canvas();
                    let current_pen_style = canvas.engine_ref().penholder.current_pen_style_w_override();

                    match current_pen_style {
                        PenStyle::Typewriter => {
                            let widget_flags = canvas.engine_mut().text_change_color(stroke_color);
                            appwindow.handle_widget_flags(widget_flags, &canvas);
                        }
                        PenStyle::Selector => {
                            let widget_flags = canvas.engine_mut().change_selection_stroke_colors(stroke_color);
                            appwindow.handle_widget_flags(widget_flags, &canvas);
                        }
                        PenStyle::Brush | PenStyle::Shaper | PenStyle::Eraser | PenStyle::Tools => {}
                    }

                    // We have a global colorpicker, so we apply it to all styles
                    canvas.engine_mut().pens_config.set_all_stroke_colors(stroke_color);
                }),
            );

        imp.colorpicker.connect_notify_local(
            Some("fill-color"),
            clone!(@weak appwindow => move |colorpicker, _paramspec| {
                let fill_color = colorpicker.fill_color().into_compose_color();
                let canvas = appwindow.active_tab_wrapper().canvas();
                let stroke_style = canvas.engine_ref().penholder.current_pen_style_w_override();

                match stroke_style {
                    PenStyle::Selector => {
                        let widget_flags = canvas.engine_mut().change_selection_fill_colors(fill_color);
                        appwindow.handle_widget_flags(widget_flags, &canvas);
                    }
                    PenStyle::Typewriter | PenStyle::Brush | PenStyle::Shaper | PenStyle::Eraser | PenStyle::Tools => {}
                }

                // We have a global colorpicker, so we apply it to all styles
                canvas.engine_mut().pens_config.set_all_fill_colors(fill_color);
            }),
        );
    }

    fn setup_tabview(&self, appwindow: &RnAppWindow) {
        let imp = self.imp();

        imp.tabview
            .connect_selected_page_notify(clone!(@weak self as overlays, @weak appwindow => move |_| {
                let active_tab_page = appwindow.active_tab_page();
                let active_canvaswrapper = active_tab_page.child().downcast::<RnCanvasWrapper>().unwrap();
                appwindow.tabs_set_unselected_inactive();

                if let Some(prev_active_tab_page) = overlays.imp().prev_active_tab_page.upgrade() {
                        if prev_active_tab_page != active_tab_page {
                            appwindow.sync_state_between_tabs(&prev_active_tab_page, &active_tab_page);
                        }
                }
                overlays.imp().prev_active_tab_page.set(Some(&active_tab_page));

                let widget_flags = active_canvaswrapper.canvas().engine_mut().set_active(true);
                appwindow.handle_widget_flags(widget_flags, &active_canvaswrapper.canvas());
                appwindow.refresh_ui_from_engine(&active_canvaswrapper);
            }));

        imp.tabview.connect_page_attached(
            clone!(@weak self as overlays, @weak appwindow => move |_tabview, page, _| {
                let canvaswrapper = page.child().downcast::<RnCanvasWrapper>().unwrap();
                canvaswrapper.init_reconnect(&appwindow);
                canvaswrapper.connect_to_tab_page(page);
                let widget_flags = canvaswrapper.canvas().engine_mut().set_active(true);
                appwindow.handle_widget_flags(widget_flags, &canvaswrapper.canvas());
            }),
        );

        imp.tabview.connect_page_detached(
            clone!(@weak self as overlays, @weak appwindow => move |_, page, _| {
                let canvaswrapper = page.child().downcast::<RnCanvasWrapper>().unwrap();

                // if the to be detached page was the active (selected), remove it.
                if overlays.imp().prev_active_tab_page.upgrade().map_or(true, |prev| prev == *page) {
                    overlays.imp().prev_active_tab_page.set(None);
                }

                let _ = canvaswrapper.canvas().engine_mut().set_active(false);
                canvaswrapper.disconnect_connections();
            }),
        );

        imp.tabview.connect_close_page(
            clone!(@weak self as overlays, @weak appwindow => @default-return true, move |_, page| {
                    glib::spawn_future_local(clone!(@weak overlays, @weak appwindow, @weak page => async move {
                    let close_finish_confirm = if page
                        .child()
                        .downcast::<RnCanvasWrapper>()
                        .unwrap()
                        .canvas()
                        .unsaved_changes()
                    {
                        dialogs::dialog_close_tab(&appwindow, &page).await
                    } else {
                        true
                    };

                    appwindow.close_tab_finish(&page, close_finish_confirm);
                }));

                true
            }),
        );

        imp.tabview.connect_setup_menu(clone!(@weak appwindow => move |tabview, page| {
            if let Some(page) = page {
                let action_active_tab_move_left = appwindow.lookup_action("active-tab-move-left").unwrap().downcast::<gio::SimpleAction>().unwrap();
                let action_active_tab_move_right = appwindow.lookup_action("active-tab-move-right").unwrap().downcast::<gio::SimpleAction>().unwrap();
                let action_active_tab_close = appwindow.lookup_action("active-tab-close").unwrap().downcast::<gio::SimpleAction>().unwrap();

                tabview.set_selected_page(page);

                let n_pages = tabview.n_pages();
                let pos = tabview.page_position(page);
                action_active_tab_move_left.set_enabled(pos > 0);
                action_active_tab_move_right.set_enabled(pos + 1 < n_pages);
                action_active_tab_close.set_enabled(n_pages > 1);
            }
        }));
    }

    pub(crate) fn progressbar_start_pulsing(&self) {
        const PULSE_INTERVAL: std::time::Duration = std::time::Duration::from_millis(300);

        self.imp()
            .progresspulses_active
            .set(self.imp().progresspulses_active.get().saturating_add(1));

        if let Some(src) = self.imp().progresspulse_id.replace(Some(glib::source::timeout_add_local(
            PULSE_INTERVAL,
            clone!(@weak self as appwindow => @default-return glib::ControlFlow::Break, move || {
                appwindow.progressbar().pulse();

                glib::ControlFlow::Continue
            })),
        )) {
            src.remove();
        }
    }

    pub(crate) fn progressbar_finish(&self) {
        const FINISH_TIMEOUT: std::time::Duration = std::time::Duration::from_millis(300);

        self.progressbar().set_fraction(1.);
        self.imp()
            .progresspulses_active
            .set(self.imp().progresspulses_active.get().saturating_sub(1));

        if self.imp().progresspulses_active.get() == 0 {
            if let Some(src) = self.imp().progresspulse_id.take() {
                src.remove();
            }
            glib::source::timeout_add_local_once(
                FINISH_TIMEOUT,
                clone!(@weak self as appwindow => move || {
                    appwindow.progressbar().set_fraction(0.);
                }),
            );
        }
    }

    #[allow(unused)]
    pub(crate) fn progressbar_abort(&self) {
        self.imp()
            .progresspulses_active
            .set(self.imp().progresspulses_active.get().saturating_sub(1));

        if self.imp().progresspulses_active.get() == 0 {
            if let Some(src) = self.imp().progresspulse_id.take() {
                src.remove();
            }
            self.progressbar().set_fraction(0.);
        }
    }

    pub(crate) fn dispatch_toast_w_button<F: Fn(&adw::Toast) + 'static>(
        &self,
        text: &str,
        button_label: &str,
        button_callback: F,
        timeout: Option<Duration>,
    ) -> adw::Toast {
        let toast = adw::Toast::builder()
            .title(text)
            .priority(adw::ToastPriority::High)
            .button_label(button_label)
            .timeout(timeout.map(|t| t.as_secs() as u32).unwrap_or(0))
            .build();
        toast.connect_button_clicked(button_callback);
        self.toast_overlay().add_toast(toast.clone());
        toast
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
        timeout: Option<Duration>,
        singleton_toast: &mut Option<adw::Toast>,
    ) {
        if let Some(previous_toast) = singleton_toast {
            previous_toast.dismiss();
        }
        *singleton_toast =
            Some(self.dispatch_toast_w_button(text, button_label, button_callback, timeout));
    }

    pub(crate) fn dispatch_toast_text(&self, text: &str, timeout: Option<Duration>) -> adw::Toast {
        let toast = adw::Toast::builder()
            .title(text)
            .priority(adw::ToastPriority::High)
            .timeout(timeout.map(|t| t.as_secs() as u32).unwrap_or(0))
            .build();
        self.toast_overlay().add_toast(toast.clone());
        toast
    }

    pub(crate) fn dispatch_toast_text_singleton(
        &self,
        text: &str,
        timeout: Option<Duration>,
        singleton_toast: &mut Option<adw::Toast>,
    ) {
        if let Some(previous_toast) = singleton_toast {
            previous_toast.dismiss();
        }
        *singleton_toast = Some(self.dispatch_toast_text(text, timeout));
    }

    pub(crate) fn dispatch_toast_error(&self, error: &str) -> adw::Toast {
        let toast = adw::Toast::builder()
            .title(error)
            .priority(adw::ToastPriority::High)
            .timeout(0)
            .build();
        self.toast_overlay().add_toast(toast.clone());
        tracing::error!("{error}");
        toast
    }
}
