use crate::appwindow::RnoteAppWindow;
use gtk4::{glib, glib::clone, prelude::*, subclass::prelude::*, CompositeTemplate, ToggleButton};
use rnote_engine::pens::pensconfig::toolsconfig::ToolsStyle;

mod imp {
    use super::*;

    #[derive(Default, Debug, CompositeTemplate)]
    #[template(resource = "/com/github/flxzt/rnote/ui/penssidebar/toolspage.ui")]
    pub(crate) struct ToolsPage {
        #[template_child]
        pub(crate) toolstyle_verticalspace_toggle: TemplateChild<ToggleButton>,
        #[template_child]
        pub(crate) toolstyle_offsetcamera_toggle: TemplateChild<ToggleButton>,
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
    pub(crate) struct ToolsPage(ObjectSubclass<imp::ToolsPage>)
        @extends gtk4::Widget;
}

impl Default for ToolsPage {
    fn default() -> Self {
        Self::new()
    }
}

impl ToolsPage {
    pub(crate) fn new() -> Self {
        glib::Object::new(&[])
    }

    pub(crate) fn init(&self, appwindow: &RnoteAppWindow) {
        let imp = self.imp();

        imp.toolstyle_verticalspace_toggle.connect_toggled(clone!(@weak appwindow => move |toolstyle_verticalspace_toggle| {
            if toolstyle_verticalspace_toggle.is_active() {
                appwindow.canvas().engine().borrow_mut().pens_config.tools_config.style = ToolsStyle::VerticalSpace;
            }
        }));

        imp.toolstyle_offsetcamera_toggle.connect_toggled(clone!(@weak appwindow => move |toolstyle_offsetcamera_toggle| {
            if toolstyle_offsetcamera_toggle.is_active() {
                appwindow.canvas().engine().borrow_mut().pens_config.tools_config.style = ToolsStyle::OffsetCamera;
            }
        }));
    }

    pub(crate) fn refresh_ui(&self, appwindow: &RnoteAppWindow) {
        let imp = self.imp();

        let tools_config = appwindow
            .canvas()
            .engine()
            .borrow()
            .pens_config
            .tools_config
            .clone();

        match tools_config.style {
            ToolsStyle::VerticalSpace => imp.toolstyle_verticalspace_toggle.set_active(true),
            ToolsStyle::OffsetCamera => imp.toolstyle_offsetcamera_toggle.set_active(true),
        }
    }
}
