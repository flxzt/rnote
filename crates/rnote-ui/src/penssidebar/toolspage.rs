// Imports
use crate::{RnAppWindow, RnCanvasWrapper};
use gtk4::{
    Button, CompositeTemplate, MenuButton, Popover, ToggleButton, glib, glib::clone, prelude::*,
    subclass::prelude::*,
};
use rnote_engine::pens::pensconfig::toolsconfig::ToolStyle;

mod imp {
    use super::*;

    #[derive(Default, Debug, CompositeTemplate)]
    #[template(resource = "/com/github/flxzt/rnote/ui/penssidebar/toolspage.ui")]
    pub(crate) struct RnToolsPage {
        #[template_child]
        pub(crate) toolstyle_verticalspace_toggle: TemplateChild<ToggleButton>,
        #[template_child]
        pub(crate) toolstyle_offsetcamera_toggle: TemplateChild<ToggleButton>,
        #[template_child]
        pub(crate) toolstyle_zoom_toggle: TemplateChild<ToggleButton>,
        #[template_child]
        pub(crate) toolstyle_laser_toggle: TemplateChild<ToggleButton>,
        #[template_child]
        pub(crate) verticalspace_menubutton: TemplateChild<MenuButton>,
        #[template_child]
        pub(crate) verticalspace_popover: TemplateChild<Popover>,
        #[template_child]
        pub(crate) verticalspace_popover_close_button: TemplateChild<Button>,
        #[template_child]
        pub(crate) verticalspace_limit_movement_vertical_bordersrow: TemplateChild<adw::SwitchRow>,
        #[template_child]
        pub(crate) verticalspace_limit_movement_horizontal_bordersrow:
            TemplateChild<adw::SwitchRow>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for RnToolsPage {
        const NAME: &'static str = "RnToolsPage";
        type Type = super::RnToolsPage;
        type ParentType = gtk4::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for RnToolsPage {
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

    impl WidgetImpl for RnToolsPage {}
}

glib::wrapper! {
    pub(crate) struct RnToolsPage(ObjectSubclass<imp::RnToolsPage>)
        @extends gtk4::Widget;
}

impl Default for RnToolsPage {
    fn default() -> Self {
        Self::new()
    }
}

impl RnToolsPage {
    pub(crate) fn new() -> Self {
        glib::Object::new()
    }

    #[allow(unused)]
    pub(crate) fn tool_style(&self) -> Option<ToolStyle> {
        let imp = self.imp();

        if imp.toolstyle_verticalspace_toggle.is_active() {
            Some(ToolStyle::VerticalSpace)
        } else if imp.toolstyle_offsetcamera_toggle.is_active() {
            Some(ToolStyle::OffsetCamera)
        } else if imp.toolstyle_zoom_toggle.is_active() {
            Some(ToolStyle::Zoom)
        } else if imp.toolstyle_laser_toggle.is_active() {
            Some(ToolStyle::Laser)
        } else {
            None
        }
    }

    #[allow(unused)]
    pub(crate) fn verticalspace_menubutton(&self) -> MenuButton {
        self.imp().verticalspace_menubutton.get()
    }

    #[allow(unused)]
    pub(crate) fn set_tool_style(&self, style: ToolStyle) {
        let imp = self.imp();

        match style {
            ToolStyle::VerticalSpace => imp.toolstyle_verticalspace_toggle.set_active(true),
            ToolStyle::OffsetCamera => imp.toolstyle_offsetcamera_toggle.set_active(true),
            ToolStyle::Zoom => imp.toolstyle_zoom_toggle.set_active(true),
            ToolStyle::Laser => imp.toolstyle_laser_toggle.set_active(true),
        }
    }

    pub(crate) fn init(&self, appwindow: &RnAppWindow) {
        let imp = self.imp();
        // for now doesn't do anything but for the close button later
        let verticalspace_popover = imp.verticalspace_popover.get();

        imp.toolstyle_verticalspace_toggle.connect_toggled(clone!(
            #[weak]
            appwindow,
            move |toggle| {
                let Some(canvas) = appwindow.active_tab_canvas() else {
                    return;
                };

                if toggle.is_active() {
                    canvas.engine_mut().pens_config.tools_config.style = ToolStyle::VerticalSpace;
                    let widget_flags = canvas.engine_mut().reinstall_pen_current_style();
                    canvas.emit_handle_widget_flags(widget_flags);
                }
            }
        ));

        imp.toolstyle_offsetcamera_toggle.connect_toggled(clone!(
            #[weak]
            appwindow,
            move |toggle| {
                let Some(canvas) = appwindow.active_tab_canvas() else {
                    return;
                };

                if toggle.is_active() {
                    canvas.engine_mut().pens_config.tools_config.style = ToolStyle::OffsetCamera;
                    let widget_flags = canvas.engine_mut().reinstall_pen_current_style();
                    canvas.emit_handle_widget_flags(widget_flags);
                }
            }
        ));

        imp.toolstyle_zoom_toggle.connect_toggled(clone!(
            #[weak]
            appwindow,
            move |toggle| {
                let Some(canvas) = appwindow.active_tab_canvas() else {
                    return;
                };

                if toggle.is_active() {
                    canvas.engine_mut().pens_config.tools_config.style = ToolStyle::Zoom;
                    let widget_flags = canvas.engine_mut().reinstall_pen_current_style();
                    canvas.emit_handle_widget_flags(widget_flags);
                }
            }
        ));

        imp.toolstyle_laser_toggle.connect_toggled(clone!(
            #[weak]
            appwindow,
            move |toggle| {
                let Some(canvas) = appwindow.active_tab_canvas() else {
                    return;
                };

                if toggle.is_active() {
                    canvas.engine_mut().pens_config.tools_config.style = ToolStyle::Laser;
                    let widget_flags = canvas.engine_mut().reinstall_pen_current_style();
                    canvas.emit_handle_widget_flags(widget_flags);
                }
            }
        ));

        imp.verticalspace_menubutton.connect_active_notify(clone!(
            #[weak(rename_to=toolspage)]
            self,
            move |menubutton| {
                if menubutton.is_active() {
                    toolspage.set_tool_style(ToolStyle::VerticalSpace);
                }
            }
        ));

        imp.verticalspace_popover_close_button
            .connect_clicked(clone!(
                #[weak]
                verticalspace_popover,
                move |_| {
                    verticalspace_popover.popdown();
                }
            ));

        imp.verticalspace_limit_movement_vertical_bordersrow
            .get()
            .connect_active_notify(clone!(
                #[weak]
                appwindow,
                move |row| {
                    let Some(canvas) = appwindow.active_tab_canvas() else {
                        return;
                    };

                    canvas
                        .engine_mut()
                        .pens_config
                        .tools_config
                        .verticalspace_tool_config
                        .limit_movement_vertical_borders = row.is_active();
                }
            ));
        imp.verticalspace_limit_movement_horizontal_bordersrow
            .get()
            .connect_active_notify(clone!(
                #[weak]
                appwindow,
                move |row| {
                    let Some(canvas) = appwindow.active_tab_canvas() else {
                        return;
                    };

                    canvas
                        .engine_mut()
                        .pens_config
                        .tools_config
                        .verticalspace_tool_config
                        .limit_movement_horizontal_borders = row.is_active();
                }
            ));
    }

    pub(crate) fn refresh_ui(&self, active_tab: &RnCanvasWrapper) {
        let tools_config = active_tab
            .canvas()
            .engine_ref()
            .pens_config
            .tools_config
            .clone();

        self.set_tool_style(tools_config.style);

        let imp = self.imp();
        imp.verticalspace_limit_movement_vertical_bordersrow
            .set_active(
                tools_config
                    .verticalspace_tool_config
                    .limit_movement_horizontal_borders,
            );
        imp.verticalspace_limit_movement_horizontal_bordersrow
            .set_active(
                tools_config
                    .verticalspace_tool_config
                    .limit_movement_vertical_borders,
            );
    }
}
