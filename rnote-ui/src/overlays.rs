use gtk4::{
    glib, glib::clone, prelude::*, subclass::prelude::*, Button, CompositeTemplate, ProgressBar,
    Revealer, Widget,
};
use std::cell::RefCell;

use crate::canvaswrapper::RnoteCanvasWrapper;
use crate::{dialogs, RnoteAppWindow};

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
        pub(crate) quickactions_box: TemplateChild<gtk4::Box>,
        #[template_child]
        pub(crate) fixedsize_quickactions_revealer: TemplateChild<Revealer>,
        #[template_child]
        pub(crate) undo_button: TemplateChild<Button>,
        #[template_child]
        pub(crate) redo_button: TemplateChild<Button>,
        #[template_child]
        pub(crate) tabview: TemplateChild<adw::TabView>,
    }

    impl Default for RnoteOverlays {
        fn default() -> Self {
            Self {
                progresspulse_source_id: RefCell::new(None),

                toast_overlay: TemplateChild::<adw::ToastOverlay>::default(),
                progressbar: TemplateChild::<ProgressBar>::default(),
                quickactions_box: TemplateChild::<gtk4::Box>::default(),
                fixedsize_quickactions_revealer: TemplateChild::<Revealer>::default(),
                undo_button: TemplateChild::<Button>::default(),
                redo_button: TemplateChild::<Button>::default(),
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

    pub(crate) fn quickactions_box(&self) -> gtk4::Box {
        self.imp().quickactions_box.get()
    }

    pub(crate) fn fixedsize_quickactions_revealer(&self) -> Revealer {
        self.imp().fixedsize_quickactions_revealer.get()
    }

    pub(crate) fn undo_button(&self) -> Button {
        self.imp().undo_button.get()
    }

    pub(crate) fn redo_button(&self) -> Button {
        self.imp().redo_button.get()
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
        imp.tabview
            .connect_selected_page_notify(clone!(@weak appwindow => move |_tabview| {
                appwindow.clear_rendering_inactive_pages();

                appwindow.active_tab().canvas().regenerate_background_pattern();
                appwindow.active_tab().canvas().update_engine_rendering();
                adw::prelude::ActionGroupExt::activate_action(&appwindow, "refresh-ui", None);
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
