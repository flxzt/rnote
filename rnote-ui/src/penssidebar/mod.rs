mod brushpage;
mod eraserpage;
mod selectorpage;
mod shaperpage;
mod toolspage;

// Re-exports
pub use brushpage::BrushPage;
pub use eraserpage::EraserPage;
use rnote_engine::pens::penholder::PenStyle;
pub use selectorpage::SelectorPage;
pub use shaperpage::ShaperPage;
pub use toolspage::ToolsPage;

use gtk4::{
    glib, glib::clone, prelude::*, subclass::prelude::*, CompositeTemplate, Stack, StackPage,
    Widget,
};

use crate::RnoteAppWindow;

mod imp {
    use super::*;

    #[derive(Default, Debug, CompositeTemplate)]
    #[template(resource = "/com/github/flxzt/rnote/ui/penssidebar/penssidebar.ui")]
    pub struct PensSideBar {
        #[template_child]
        pub sidebar_stack: TemplateChild<Stack>,
        #[template_child]
        pub brush_stackpage: TemplateChild<StackPage>,
        #[template_child]
        pub brush_page: TemplateChild<BrushPage>,
        #[template_child]
        pub shaper_stackpage: TemplateChild<StackPage>,
        #[template_child]
        pub shaper_page: TemplateChild<ShaperPage>,
        #[template_child]
        pub eraser_stackpage: TemplateChild<StackPage>,
        #[template_child]
        pub eraser_page: TemplateChild<EraserPage>,
        #[template_child]
        pub selector_stackpage: TemplateChild<StackPage>,
        #[template_child]
        pub selector_page: TemplateChild<SelectorPage>,
        #[template_child]
        pub tools_stackpage: TemplateChild<StackPage>,
        #[template_child]
        pub tools_page: TemplateChild<ToolsPage>,
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
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
        }

        fn dispose(&self, obj: &Self::Type) {
            while let Some(child) = obj.first_child() {
                child.unparent();
            }
        }
    }
    impl WidgetImpl for PensSideBar {}
}

glib::wrapper! {
    pub struct PensSideBar(ObjectSubclass<imp::PensSideBar>)
        @extends gtk4::Widget;
}

impl Default for PensSideBar {
    fn default() -> Self {
        Self::new()
    }
}

impl PensSideBar {
    pub fn new() -> Self {
        let penssidebar: PensSideBar =
            glib::Object::new(&[]).expect("Failed to create PensSideBar");
        penssidebar
    }

    pub fn sidebar_stack(&self) -> Stack {
        self.imp().sidebar_stack.get()
    }

    pub fn brush_stackpage(&self) -> StackPage {
        self.imp().brush_stackpage.get()
    }

    pub fn brush_page(&self) -> BrushPage {
        self.imp().brush_page.get()
    }

    pub fn shaper_page(&self) -> ShaperPage {
        self.imp().shaper_page.get()
    }

    pub fn eraser_stackpage(&self) -> StackPage {
        self.imp().eraser_stackpage.get()
    }

    pub fn eraser_page(&self) -> EraserPage {
        self.imp().eraser_page.get()
    }

    pub fn selector_stackpage(&self) -> StackPage {
        self.imp().selector_stackpage.get()
    }

    pub fn selector_page(&self) -> SelectorPage {
        self.imp().selector_page.get()
    }

    pub fn tools_stackpage(&self) -> StackPage {
        self.imp().tools_stackpage.get()
    }

    pub fn tools_page(&self) -> ToolsPage {
        self.imp().tools_page.get()
    }

    pub fn init(&self, appwindow: &RnoteAppWindow) {
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
