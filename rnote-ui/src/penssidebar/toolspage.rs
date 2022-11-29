use crate::appwindow::RnoteAppWindow;
use gtk4::{glib, glib::clone, prelude::*, subclass::prelude::*, CompositeTemplate, ToggleButton};
use rnote_engine::pens::tools::ToolsStyle;

mod imp {
    use super::*;

    #[derive(Default, Debug, CompositeTemplate)]
    #[template(resource = "/com/github/flxzt/rnote/ui/penssidebar/toolspage.ui")]
    pub struct ToolsPage {
        #[template_child]
        pub toolstyle_verticalspace_toggle: TemplateChild<ToggleButton>,
        #[template_child]
        pub toolstyle_offsetcamera_toggle: TemplateChild<ToggleButton>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ToolsPage {
        const NAME: &'static str = "ToolsPage";
        type Type = super::ToolsPage;
        type ParentType = gtk4::Widget;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ToolsPage {
        fn constructed(&self) {
            self.parent_constructed();
        }

        fn dispose(&self) {
            while let Some(child) = self.instance().first_child() {
                child.unparent();
            }
        }
    }

    impl WidgetImpl for ToolsPage {}
}

glib::wrapper! {
    pub struct ToolsPage(ObjectSubclass<imp::ToolsPage>)
        @extends gtk4::Widget;
}

impl Default for ToolsPage {
    fn default() -> Self {
        Self::new()
    }
}

impl ToolsPage {
    pub fn new() -> Self {
        glib::Object::new(&[])
    }

    pub fn toolstyle_verticalspace_toggle(&self) -> ToggleButton {
        self.imp().toolstyle_verticalspace_toggle.get()
    }

    pub fn toolstyle_offsetcamera_toggle(&self) -> ToggleButton {
        self.imp().toolstyle_offsetcamera_toggle.get()
    }

    pub fn init(&self, appwindow: &RnoteAppWindow) {
        self.toolstyle_verticalspace_toggle().connect_toggled(clone!(@weak appwindow => move |toolstyle_verticalspace_toggle| {
            if toolstyle_verticalspace_toggle.is_active() {
                appwindow.canvas().engine().borrow_mut().penholder.tools.style = ToolsStyle::VerticalSpace;
            }
        }));

        self.toolstyle_offsetcamera_toggle().connect_toggled(clone!(@weak appwindow => move |toolstyle_offsetcamera_toggle| {
            if toolstyle_offsetcamera_toggle.is_active() {
                appwindow.canvas().engine().borrow_mut().penholder.tools.style = ToolsStyle::OffsetCamera;
            }
        }));
    }

    pub fn refresh_ui(&self, appwindow: &RnoteAppWindow) {
        let tools = appwindow.canvas().engine().borrow().penholder.tools.clone();

        match tools.style {
            ToolsStyle::VerticalSpace => self.toolstyle_verticalspace_toggle().set_active(true),
            ToolsStyle::OffsetCamera => self.toolstyle_offsetcamera_toggle().set_active(true),
        }
    }
}
