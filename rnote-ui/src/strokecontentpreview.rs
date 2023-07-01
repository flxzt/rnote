// Imports
use crate::StrokeContentPaintable;
use gtk4::{
    glib, glib::clone, prelude::*, subclass::prelude::*, Button, CompositeTemplate, Overlay,
    Picture, ScrolledWindow, Widget,
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

        #[template_child]
        pub(crate) preview_overlay: TemplateChild<Overlay>,
        #[template_child]
        pub(crate) preview_scroller: TemplateChild<ScrolledWindow>,
        #[template_child]
        pub(crate) preview_picture: TemplateChild<Picture>,
        #[template_child]
        pub(crate) controls_box: TemplateChild<gtk4::Box>,
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
            Self::bind_template(klass);
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
                .set_measure_overlay(&*self.controls_box, true);

            self.prev_page_button.connect_clicked(
                clone!(@weak obj as strokecontentpreview => move |_| {
                    let current_page = strokecontentpreview.current_page();
                    strokecontentpreview.set_current_page(current_page.saturating_sub(1));
                    strokecontentpreview.imp().update_widgets();
                }),
            );

            self.next_page_button.connect_clicked(
                clone!(@weak obj as strokecontentpreview => move |_| {
                    let current_page = strokecontentpreview.current_page();
                    let n_pages = strokecontentpreview.n_pages();
                    strokecontentpreview.set_current_page(current_page.saturating_add(1).min(n_pages - 1));
                    strokecontentpreview.imp().update_widgets();
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
                    self.current_page.set(current_page);
                    self.update_widgets();
                }
                _ => unimplemented!(),
            }
        }
    }

    impl WidgetImpl for RnStrokeContentPreview {}

    impl RnStrokeContentPreview {
        pub(super) fn update_widgets(&self) {
            let current_page = self.obj().current_page();
            let n_pages = self.obj().n_pages();
            let content = if !self.contents.borrow().is_empty() {
                Some(self.contents.borrow()[current_page].clone())
            } else {
                None
            };
            self.paintable.set_stroke_content(content);
            self.prev_page_button.set_sensitive(current_page > 0);
            self.next_page_button
                .set_sensitive(current_page < n_pages - 1);
            self.obj().queue_resize()
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
        self.imp().update_widgets();
        self.imp().controls_box.set_visible(self.n_pages() > 1);
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
}
