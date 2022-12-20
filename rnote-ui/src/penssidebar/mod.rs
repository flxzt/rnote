mod brushpage;
mod eraserpage;
mod selectorpage;
mod shaperpage;
mod toolspage;
mod typewriterpage;

// Re-exports
pub(crate) use brushpage::BrushPage;
pub(crate) use eraserpage::EraserPage;
use rnote_engine::pens::PenStyle;
pub(crate) use selectorpage::SelectorPage;
pub(crate) use shaperpage::ShaperPage;
pub(crate) use toolspage::ToolsPage;
pub(crate) use typewriterpage::TypewriterPage;

use gtk4::{
    glib, glib::clone, prelude::*, subclass::prelude::*, CompositeTemplate, Stack, StackPage,
    Widget,
};

use crate::RnoteAppWindow;

mod imp {
    use super::*;

    #[derive(Default, Debug, CompositeTemplate)]
    #[template(resource = "/com/github/flxzt/rnote/ui/penssidebar/penssidebar.ui")]
    pub(crate) struct PensSideBar {
        #[template_child]
        pub(crate) sidebar_stack: TemplateChild<Stack>,
        #[template_child]
        pub(crate) brush_stackpage: TemplateChild<StackPage>,
        #[template_child]
        pub(crate) brush_page: TemplateChild<BrushPage>,
        #[template_child]
        pub(crate) shaper_stackpage: TemplateChild<StackPage>,
        #[template_child]
        pub(crate) shaper_page: TemplateChild<ShaperPage>,
        #[template_child]
        pub(crate) typewriter_stackpage: TemplateChild<StackPage>,
        #[template_child]
        pub(crate) typewriter_page: TemplateChild<TypewriterPage>,
        #[template_child]
        pub(crate) eraser_stackpage: TemplateChild<StackPage>,
        #[template_child]
        pub(crate) eraser_page: TemplateChild<EraserPage>,
        #[template_child]
        pub(crate) selector_stackpage: TemplateChild<StackPage>,
        #[template_child]
        pub(crate) selector_page: TemplateChild<SelectorPage>,
        #[template_child]
        pub(crate) tools_stackpage: TemplateChild<StackPage>,
        #[template_child]
        pub(crate) tools_page: TemplateChild<ToolsPage>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PensSideBar {
        const NAME: &'static str = "PensSideBar";
        type Type = super::PensSideBar;
        type ParentType = Widget;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for PensSideBar {
        fn constructed(&self) {
            self.parent_constructed();
        }

        fn dispose(&self) {
            while let Some(child) = self.instance().first_child() {
                child.unparent();
            }
        }
    }
    impl WidgetImpl for PensSideBar {}
}

glib::wrapper! {
    pub(crate) struct PensSideBar(ObjectSubclass<imp::PensSideBar>)
        @extends gtk4::Widget;
}

impl Default for PensSideBar {
    fn default() -> Self {
        Self::new()
    }
}

impl PensSideBar {
    pub(crate) fn new() -> Self {
        glib::Object::new(&[])
    }

    pub(crate) fn sidebar_stack(&self) -> Stack {
        self.imp().sidebar_stack.get()
    }

    pub(crate) fn brush_page(&self) -> BrushPage {
        self.imp().brush_page.get()
    }

    pub(crate) fn shaper_page(&self) -> ShaperPage {
        self.imp().shaper_page.get()
    }

    pub(crate) fn typewriter_page(&self) -> TypewriterPage {
        self.imp().typewriter_page.get()
    }

    pub(crate) fn eraser_page(&self) -> EraserPage {
        self.imp().eraser_page.get()
    }

    pub(crate) fn selector_page(&self) -> SelectorPage {
        self.imp().selector_page.get()
    }

    pub(crate) fn tools_page(&self) -> ToolsPage {
        self.imp().tools_page.get()
    }

    pub(crate) fn init(&self, appwindow: &RnoteAppWindow) {
        self.imp().sidebar_stack.get().connect_visible_child_name_notify(
            clone!(@weak appwindow => move |sidebar_stack| {
                if let Some(child_name) = sidebar_stack.visible_child_name() {
                    match child_name.to_value().get::<String>().unwrap().as_str() {
                        "brush_page" => {
                            adw::prelude::ActionGroupExt::activate_action(&appwindow, "pen-style", Some(&PenStyle::Brush.nick().to_variant()));
                        },
                        "shaper_page" => {
                            adw::prelude::ActionGroupExt::activate_action(&appwindow, "pen-style", Some(&PenStyle::Shaper.nick().to_variant()));
                        },
                        "typewriter_page" => {
                            adw::prelude::ActionGroupExt::activate_action(&appwindow, "pen-style", Some(&PenStyle::Typewriter.nick().to_variant()));
                        },
                        "eraser_page" => {
                            adw::prelude::ActionGroupExt::activate_action(&appwindow, "pen-style", Some(&PenStyle::Eraser.nick().to_variant()));
                        }
                        "selector_page" => {
                            adw::prelude::ActionGroupExt::activate_action(&appwindow, "pen-style", Some(&PenStyle::Selector.nick().to_variant()));
                        }
                        "tools_page" => {
                            adw::prelude::ActionGroupExt::activate_action(&appwindow, "pen-style", Some(&PenStyle::Tools.nick().to_variant()));
                        }
                        _ => {}
                    };
                };
            }),
        );
    }
}
