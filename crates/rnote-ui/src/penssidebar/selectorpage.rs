// Imports
use crate::RnAppWindow;
use crate::RnStrokeWidthPicker;
use gtk4::{
    Button, CompositeTemplate, MenuButton, Popover, ToggleButton, Widget, glib, glib::clone,
    prelude::*, subclass::prelude::*,
};
use rnote_engine::pens::pensconfig::BrushConfig;
use rnote_engine::pens::pensconfig::brushconfig::SolidOptions;
use rnote_engine::pens::pensconfig::selectorconfig::SelectorStyle;
use std::cell::Cell;
use std::str;

mod imp {
    use super::*;

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/com/github/flxzt/rnote/ui/penssidebar/selectorpage.ui")]
    pub(crate) struct RnSelectorPage {
        #[template_child]
        pub(crate) selectorstyle_polygon_toggle: TemplateChild<ToggleButton>,
        #[template_child]
        pub(crate) selectorstyle_rect_toggle: TemplateChild<ToggleButton>,
        #[template_child]
        pub(crate) selectorstyle_single_toggle: TemplateChild<ToggleButton>,
        #[template_child]
        pub(crate) selectorstyle_intersectingpath_toggle: TemplateChild<ToggleButton>,
        #[template_child]
        pub(crate) resize_lock_aspectratio_togglebutton: TemplateChild<ToggleButton>,
        #[template_child]
        pub(crate) strokewidth_menubutton: TemplateChild<MenuButton>,
        #[template_child]
        pub(crate) strokewidth_popover: TemplateChild<Popover>,
        #[template_child]
        pub(crate) strokewidth_popover_close_button: TemplateChild<Button>,
        #[template_child]
        pub(crate) stroke_width_picker: TemplateChild<RnStrokeWidthPicker>,

        pub(crate) allow_stroke_width_changes: Cell<bool>, // Used to stop the stroke width changing immediately when the popup is opened
    }

    impl Default for RnSelectorPage {
        fn default() -> Self {
            Self {
                selectorstyle_polygon_toggle: TemplateChild::default(),
                selectorstyle_rect_toggle: TemplateChild::default(),
                selectorstyle_single_toggle: TemplateChild::default(),
                selectorstyle_intersectingpath_toggle: TemplateChild::default(),
                resize_lock_aspectratio_togglebutton: TemplateChild::default(),
                strokewidth_menubutton: TemplateChild::default(),
                strokewidth_popover: TemplateChild::default(),
                strokewidth_popover_close_button: TemplateChild::default(),
                stroke_width_picker: TemplateChild::default(),

                allow_stroke_width_changes: Cell::new(true),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for RnSelectorPage {
        const NAME: &'static str = "RnSelectorPage";
        type Type = super::RnSelectorPage;
        type ParentType = Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for RnSelectorPage {
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

    impl WidgetImpl for RnSelectorPage {}
}

glib::wrapper! {
    pub(crate) struct RnSelectorPage(ObjectSubclass<imp::RnSelectorPage>)
        @extends Widget,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget;
}

impl Default for RnSelectorPage {
    fn default() -> Self {
        Self::new()
    }
}

impl RnSelectorPage {
    pub(crate) fn new() -> Self {
        glib::Object::new()
    }

    pub(crate) fn strokewidth_menubutton(&self) -> MenuButton {
        self.imp().strokewidth_menubutton.get()
    }

    #[allow(unused)]
    pub(crate) fn selector_style(&self) -> Option<SelectorStyle> {
        if self.imp().selectorstyle_polygon_toggle.is_active() {
            Some(SelectorStyle::Polygon)
        } else if self.imp().selectorstyle_rect_toggle.is_active() {
            Some(SelectorStyle::Rectangle)
        } else if self.imp().selectorstyle_single_toggle.is_active() {
            Some(SelectorStyle::Single)
        } else if self.imp().selectorstyle_intersectingpath_toggle.is_active() {
            Some(SelectorStyle::IntersectingPath)
        } else {
            None
        }
    }

    #[allow(unused)]
    pub(crate) fn set_selector_style(&self, style: SelectorStyle) {
        match style {
            SelectorStyle::Polygon => self.imp().selectorstyle_polygon_toggle.set_active(true),
            SelectorStyle::Rectangle => self.imp().selectorstyle_rect_toggle.set_active(true),
            SelectorStyle::Single => self.imp().selectorstyle_single_toggle.set_active(true),
            SelectorStyle::IntersectingPath => self
                .imp()
                .selectorstyle_intersectingpath_toggle
                .set_active(true),
        }
    }

    pub(crate) fn stroke_width_picker(&self) -> RnStrokeWidthPicker {
        self.imp().stroke_width_picker.get()
    }

    pub(crate) fn init(&self, appwindow: &RnAppWindow) {
        let imp = self.imp();

        imp.selectorstyle_polygon_toggle.connect_toggled(clone!(
            #[weak]
            appwindow,
            move |toggle| {
                if !toggle.is_active() {
                    return;
                }
                appwindow
                    .engine_config()
                    .write()
                    .pens_config
                    .selector_config
                    .style = SelectorStyle::Polygon;
            }
        ));

        imp.selectorstyle_rect_toggle.connect_toggled(clone!(
            #[weak]
            appwindow,
            move |toggle| {
                if !toggle.is_active() {
                    return;
                }
                appwindow
                    .engine_config()
                    .write()
                    .pens_config
                    .selector_config
                    .style = SelectorStyle::Rectangle;
            }
        ));

        imp.selectorstyle_single_toggle.connect_toggled(clone!(
            #[weak]
            appwindow,
            move |toggle| {
                if !toggle.is_active() {
                    return;
                }
                appwindow
                    .engine_config()
                    .write()
                    .pens_config
                    .selector_config
                    .style = SelectorStyle::Single;
            }
        ));

        imp.selectorstyle_intersectingpath_toggle
            .connect_toggled(clone!(
                #[weak]
                appwindow,
                move |toggle| {
                    if !toggle.is_active() {
                        return;
                    }
                    appwindow
                        .engine_config()
                        .write()
                        .pens_config
                        .selector_config
                        .style = SelectorStyle::IntersectingPath;
                }
            ));

        imp.resize_lock_aspectratio_togglebutton
            .connect_toggled(clone!(
                #[weak]
                appwindow,
                move |toggle| {
                    appwindow
                        .engine_config()
                        .write()
                        .pens_config
                        .selector_config
                        .resize_lock_aspectratio = toggle.is_active();
                }
            ));

        // Stroke width

        // Close popup when close button clicked
        let strokewidth_popover = imp.strokewidth_popover.get();
        imp.strokewidth_popover_close_button.connect_clicked(clone!(
            #[weak]
            strokewidth_popover,
            move |_| {
                strokewidth_popover.popdown();
            }
        ));

        // Deselect setters when popup closed
        imp.strokewidth_popover.connect_closed(clone!(
            #[weak(rename_to=selectorpage)]
            self,
            move |_| {
                selectorpage.stroke_width_picker().deselect_setters();
                // TODO: Stop deselect animation from showing when popup opened again
            }
        ));

        // Set width picker to selected stroke width when popup opened
        imp.strokewidth_menubutton.connect_active_notify(clone!(
            #[weak]
            imp,
            #[weak]
            appwindow,
            move |btn| {
                if !btn.is_active() {
                    return;
                }

                let Some(canvas) = appwindow.active_tab_canvas() else {
                    return;
                };
                let stroke_width = canvas
                    .engine_ref()
                    .get_selection_stroke_width()
                    .unwrap_or(2.0); // Default to width of 2 if no valid strokes selected

                imp.allow_stroke_width_changes.set(false); // Stop the new picker width from applying immediately
                imp.stroke_width_picker
                    .set_stroke_width(stroke_width + 0.001); // A small value is added to allow the user to apply a width equal to 'stroke width'
            }
        ));

        imp.stroke_width_picker
            .spinbutton()
            .set_range(BrushConfig::STROKE_WIDTH_MIN, BrushConfig::STROKE_WIDTH_MAX);
        // set value after the range!
        imp.stroke_width_picker
            .set_stroke_width(SolidOptions::default().stroke_width);

        // Set stroke width when picker width changed
        imp.stroke_width_picker.connect_notify_local(
            Some("stroke-width"),
            clone!(
                #[weak]
                imp,
                #[weak]
                appwindow,
                move |picker, _| {
                    let stroke_width = picker.stroke_width();

                    // Return if no canvas found
                    let Some(canvas) = appwindow.active_tab_canvas() else {
                        return;
                    };

                    // Return if the popup has just been opened
                    if !imp.allow_stroke_width_changes.get() {
                        imp.allow_stroke_width_changes.set(true);
                        return;
                    }

                    // Change the width of the selected strokes
                    let widget_flags = canvas
                        .engine_mut()
                        .change_selection_stroke_width(stroke_width);

                    appwindow.handle_widget_flags(widget_flags, &canvas);
                }
            ),
        );
    }

    pub(crate) fn refresh_ui(&self, appwindow: &RnAppWindow) {
        let imp = self.imp();

        let selector_config = appwindow
            .engine_config()
            .read()
            .pens_config
            .selector_config
            .clone();

        self.set_selector_style(selector_config.style);

        imp.resize_lock_aspectratio_togglebutton
            .set_active(selector_config.resize_lock_aspectratio);
    }
}
