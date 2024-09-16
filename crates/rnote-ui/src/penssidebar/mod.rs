// Modules
mod brushpage;
mod eraserpage;
mod selectorpage;
mod shaperpage;
mod toolspage;
mod typewriterpage;

// Re-exports
pub(crate) use brushpage::RnBrushPage;
pub(crate) use eraserpage::RnEraserPage;
use rnote_engine::pens::PenStyle;
pub(crate) use selectorpage::RnSelectorPage;
pub(crate) use shaperpage::RnShaperPage;
pub(crate) use toolspage::RnToolsPage;
pub(crate) use typewriterpage::RnTypewriterPage;

// Imports
use crate::RnAppWindow;
use gtk4::{
    glib, glib::clone, prelude::*, subclass::prelude::*, CompositeTemplate, Stack, StackPage,
    Widget,
};

mod imp {
    use super::*;

    #[derive(Default, Debug, CompositeTemplate)]
    #[template(resource = "/com/github/flxzt/rnote/ui/penssidebar/penssidebar.ui")]
    pub(crate) struct RnPensSideBar {
        #[template_child]
        pub(crate) sidebar_stack: TemplateChild<Stack>,
        #[template_child]
        pub(crate) brush_stackpage: TemplateChild<StackPage>,
        #[template_child]
        pub(crate) brush_page: TemplateChild<RnBrushPage>,
        #[template_child]
        pub(crate) shaper_stackpage: TemplateChild<StackPage>,
        #[template_child]
        pub(crate) shaper_page: TemplateChild<RnShaperPage>,
        #[template_child]
        pub(crate) typewriter_stackpage: TemplateChild<StackPage>,
        #[template_child]
        pub(crate) typewriter_page: TemplateChild<RnTypewriterPage>,
        #[template_child]
        pub(crate) eraser_stackpage: TemplateChild<StackPage>,
        #[template_child]
        pub(crate) eraser_page: TemplateChild<RnEraserPage>,
        #[template_child]
        pub(crate) selector_stackpage: TemplateChild<StackPage>,
        #[template_child]
        pub(crate) selector_page: TemplateChild<RnSelectorPage>,
        #[template_child]
        pub(crate) tools_stackpage: TemplateChild<StackPage>,
        #[template_child]
        pub(crate) tools_page: TemplateChild<RnToolsPage>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for RnPensSideBar {
        const NAME: &'static str = "RnPensSideBar";
        type Type = super::RnPensSideBar;
        type ParentType = Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for RnPensSideBar {
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
    impl WidgetImpl for RnPensSideBar {}
}

glib::wrapper! {
    pub(crate) struct RnPensSideBar(ObjectSubclass<imp::RnPensSideBar>)
        @extends gtk4::Widget;
}

impl Default for RnPensSideBar {
    fn default() -> Self {
        Self::new()
    }
}

impl RnPensSideBar {
    pub(crate) fn new() -> Self {
        glib::Object::new()
    }

    pub(crate) fn sidebar_stack(&self) -> Stack {
        self.imp().sidebar_stack.get()
    }

    pub(crate) fn brush_page(&self) -> RnBrushPage {
        self.imp().brush_page.get()
    }

    pub(crate) fn shaper_page(&self) -> RnShaperPage {
        self.imp().shaper_page.get()
    }

    pub(crate) fn typewriter_page(&self) -> RnTypewriterPage {
        self.imp().typewriter_page.get()
    }

    pub(crate) fn eraser_page(&self) -> RnEraserPage {
        self.imp().eraser_page.get()
    }

    pub(crate) fn selector_page(&self) -> RnSelectorPage {
        self.imp().selector_page.get()
    }

    pub(crate) fn tools_page(&self) -> RnToolsPage {
        self.imp().tools_page.get()
    }

    pub(crate) fn init(&self, appwindow: &RnAppWindow) {
        self.imp()
            .sidebar_stack
            .get()
            .connect_visible_child_name_notify(clone!(
                #[weak]
                appwindow,
                move |sidebar_stack| {
                    if let Some(child_name) = sidebar_stack.visible_child_name() {
                        match child_name.to_value().get::<String>().unwrap().as_str() {
                            "brush_page" => {
                                adw::prelude::ActionGroupExt::activate_action(
                                    &appwindow,
                                    "pen-style",
                                    Some(&PenStyle::Brush.to_string().to_variant()),
                                );
                            }
                            "shaper_page" => {
                                adw::prelude::ActionGroupExt::activate_action(
                                    &appwindow,
                                    "pen-style",
                                    Some(&PenStyle::Shaper.to_string().to_variant()),
                                );
                            }
                            "typewriter_page" => {
                                adw::prelude::ActionGroupExt::activate_action(
                                    &appwindow,
                                    "pen-style",
                                    Some(&PenStyle::Typewriter.to_string().to_variant()),
                                );
                            }
                            "eraser_page" => {
                                adw::prelude::ActionGroupExt::activate_action(
                                    &appwindow,
                                    "pen-style",
                                    Some(&PenStyle::Eraser.to_string().to_variant()),
                                );
                            }
                            "selector_page" => {
                                adw::prelude::ActionGroupExt::activate_action(
                                    &appwindow,
                                    "pen-style",
                                    Some(&PenStyle::Selector.to_string().to_variant()),
                                );
                            }
                            "tools_page" => {
                                adw::prelude::ActionGroupExt::activate_action(
                                    &appwindow,
                                    "pen-style",
                                    Some(&PenStyle::Tools.to_string().to_variant()),
                                );
                            }
                            _ => {}
                        };
                    };
                }
            ));
    }
}
