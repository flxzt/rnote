// Imports
use crate::StrokeContentPaintable;
use gtk4::{
    glib, glib::clone, prelude::*, subclass::prelude::*, Button, CompositeTemplate, Entry, Overlay,
    Picture, ProgressBar, ScrolledWindow, Widget,
};
use once_cell::sync::Lazy;
use rnote_engine::engine::StrokeContent;
use std::cell::{Cell, RefCell};

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/github/flxzt/rnote/ui/strokecontentpreview.ui")]
    pub(crate) struct RnStrokeContentPreview {
        pub(crate) contents: RefCell<Vec<StrokeContent>>,
        pub(crate) paintable: StrokeContentPaintable,
        pub(crate) current_page: Cell<usize>,
        pub(crate) progresspulse_id: RefCell<Option<glib::SourceId>>,

        #[template_child]
        pub(crate) preview_overlay: TemplateChild<Overlay>,
        #[template_child]
        pub(crate) preview_scroller: TemplateChild<ScrolledWindow>,
        #[template_child]
        pub(crate) preview_picture: TemplateChild<Picture>,
        #[template_child]
        pub(crate) progressbar: TemplateChild<ProgressBar>,
        #[template_child]
        pub(crate) pages_controls_box: TemplateChild<gtk4::Box>,
        #[template_child]
        pub(crate) page_entry: TemplateChild<Entry>,
        #[template_child]
        pub(crate) n_pages_button: TemplateChild<Button>,
        #[template_child]
        pub(crate) prev_page_button: TemplateChild<Button>,
        #[template_child]
        pub(crate) next_page_button: TemplateChild<Button>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for RnStrokeContentPreview {
        const NAME: &'static str = "RnStrokeContentPreview";
        type Type = super::RnStrokeContentPreview;
        type ParentType = Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for RnStrokeContentPreview {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();

            self.preview_picture.set_paintable(Some(&self.paintable));
            self.preview_overlay
                .set_measure_overlay(&*self.pages_controls_box, true);

            self.paintable.connect_local(
                "repaint-in-progress",
                false,
                clone!(@weak obj as strokecontentpreview => @default-return None, move |vals| {
                    let in_progress = vals[1].get::<bool>().unwrap();
                    let current_page = strokecontentpreview.current_page();
                    let n_pages = strokecontentpreview.n_pages();

                    if in_progress {
                        strokecontentpreview.progressbar_start_pulsing();
                        strokecontentpreview.imp().prev_page_button.set_sensitive(false);
                        strokecontentpreview.imp().next_page_button.set_sensitive(false);
                    } else {
                        strokecontentpreview.progressbar_finish();
                        strokecontentpreview.imp().prev_page_button.set_sensitive(current_page > 0);
                        strokecontentpreview.imp().next_page_button
                            .set_sensitive(current_page < n_pages.saturating_sub(1));
                    }
                    None
                }),
            );

            self.page_entry.connect_changed(
                clone!(@weak obj as stroke_content_preview => move |entry| {
                    let n_pages = stroke_content_preview.n_pages();
                    match parse_page_text(&entry.text(), n_pages) {
                        Ok(page) => {
                            entry.remove_css_class("error");
                            stroke_content_preview.set_current_page(page);
                        }
                        _ => {
                            entry.add_css_class("error");
                        }
                    }
                }),
            );

            self.prev_page_button.connect_clicked(
                clone!(@weak obj as strokecontentpreview => move |_| {
                    let current_page = strokecontentpreview.current_page();
                    strokecontentpreview.set_current_page(current_page.saturating_sub(1));
                }),
            );

            self.next_page_button.connect_clicked(
                clone!(@weak obj as strokecontentpreview => move |_| {
                    let current_page = strokecontentpreview.current_page();
                    let n_pages = strokecontentpreview.n_pages();
                    strokecontentpreview.set_current_page(current_page.saturating_add(1).min(n_pages - 1));
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
                vec![glib::ParamSpecUInt::builder("current-page")
                    .default_value(0)
                    .build()]
            });
            PROPERTIES.as_ref()
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "current-page" => (self.current_page.get() as u32).to_value(),
                _ => unimplemented!(),
            }
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            match pspec.name() {
                "current-page" => {
                    let current_page = value.get::<u32>().unwrap() as usize;
                    let n_pages = self.obj().n_pages();
                    self.current_page
                        .set(current_page.min(n_pages.saturating_sub(1)));
                    self.update_paintable_content();
                }
                _ => unimplemented!(),
            }
        }
    }

    impl WidgetImpl for RnStrokeContentPreview {}

    impl RnStrokeContentPreview {
        pub(super) fn update_paintable_content(&self) {
            let current_page = self.obj().current_page();
            let n_pages = self.obj().n_pages();
            self.paintable.set_stroke_content(
                self.contents
                    .borrow()
                    .get(current_page)
                    .cloned()
                    .unwrap_or_default(),
            );

            self.pages_controls_box.set_visible(n_pages > 1);
            // the prev/next page buttons sensitivity get updated in the paintable `repaint-in-progress` signal handler.
            match parse_page_text(&self.page_entry.text(), n_pages) {
                Ok(page) if page == current_page => {
                    // Don't update entry if it is already the current page
                }
                Ok(_) | Err(_) => {
                    // user facing page number is 1 indexed
                    self.page_entry.set_text(&(current_page + 1).to_string());
                }
            }
            self.n_pages_button.set_label(&n_pages.to_string());
        }
    }

    fn parse_page_text(text: &str, n_pages: usize) -> anyhow::Result<usize> {
        // user facing page number is 1 indexed
        let page_range = 1..=n_pages;
        match text.parse::<usize>() {
            Ok(page) if page_range.contains(&page) => Ok(page - 1),
            Ok(page) => Err(anyhow::anyhow!(
                "Could not parse text as page number, '{page}' outside valid range '{page_range:?}'.",
            )),
            Err(e) => Err(anyhow::anyhow!(
                "Could not parse text as page number, parsing error: {e:?}"
            )),
        }
    }
}

glib::wrapper! {
    pub(crate) struct RnStrokeContentPreview(ObjectSubclass<imp::RnStrokeContentPreview>)
        @extends gtk4::Widget,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget;
}

impl Default for RnStrokeContentPreview {
    fn default() -> Self {
        Self::new()
    }
}

impl RnStrokeContentPreview {
    pub(crate) fn new() -> Self {
        glib::Object::new()
    }

    #[allow(unused)]
    pub(crate) fn current_page(&self) -> usize {
        self.property::<u32>("current-page") as usize
    }

    #[allow(unused)]
    pub(crate) fn set_current_page(&self, current_page: usize) {
        if self.imp().current_page.get() != current_page {
            self.set_property("current-page", (current_page as u32).to_value());
        }
    }

    pub(crate) fn set_contents(&self, contents: Vec<StrokeContent>) {
        self.imp().contents.replace(contents);
        let n_pages = self.n_pages();
        self.set_current_page(self.current_page().min(n_pages.saturating_sub(1)));
        self.imp().update_paintable_content();
    }

    pub(crate) fn n_pages(&self) -> usize {
        self.imp().contents.borrow().len()
    }

    #[allow(unused)]
    pub(crate) fn draw_background(&self) -> bool {
        self.imp().paintable.draw_background()
    }

    #[allow(unused)]
    pub(crate) fn set_draw_background(&self, draw_background: bool) {
        self.imp().paintable.set_draw_background(draw_background);
    }

    #[allow(unused)]
    pub(crate) fn draw_pattern(&self) -> bool {
        self.imp().paintable.draw_pattern()
    }

    #[allow(unused)]
    pub(crate) fn set_draw_pattern(&self, draw_pattern: bool) {
        self.imp().paintable.set_draw_pattern(draw_pattern);
    }

    #[allow(unused)]
    pub(crate) fn optimize_printing(&self) -> bool {
        self.imp().paintable.optimize_printing()
    }

    #[allow(unused)]
    pub(crate) fn set_optimize_printing(&self, optimize_printing: bool) {
        self.imp()
            .paintable
            .set_optimize_printing(optimize_printing);
    }

    #[allow(unused)]
    pub(crate) fn margin(&self) -> f64 {
        self.imp().paintable.margin()
    }

    #[allow(unused)]
    pub(crate) fn set_margin(&self, margin: f64) {
        self.imp().paintable.set_margin(margin);
    }

    pub(crate) fn progressbar_start_pulsing(&self) {
        const PULSE_INTERVAL: std::time::Duration = std::time::Duration::from_millis(100);
        if let Some(src) = self.imp().progresspulse_id.replace(Some(glib::source::timeout_add_local(
            PULSE_INTERVAL,
            clone!(@weak self as strokecontentpreview => @default-return glib::ControlFlow::Break, move || {
                strokecontentpreview.imp().progressbar.pulse();

                glib::ControlFlow::Continue
            })),
        )) {
            src.remove();
        }
    }

    pub(crate) fn progressbar_finish(&self) {
        const FINISH_TIMEOUT: std::time::Duration = std::time::Duration::from_millis(200);
        if let Some(pulse_source) = self.imp().progresspulse_id.take() {
            pulse_source.remove();
        }
        self.imp().progressbar.set_fraction(1.);
        glib::source::timeout_add_local_once(
            FINISH_TIMEOUT,
            clone!(@weak self as strokecontentpreview => move || {
                strokecontentpreview.imp().progressbar.set_fraction(0.);
            }),
        );
    }

    #[allow(unused)]
    pub(crate) fn progressbar_abort(&self) {
        if let Some(src) = self.imp().progresspulse_id.take() {
            src.remove();
        }
        self.imp().progressbar.set_fraction(0.);
    }
}
