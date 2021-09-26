mod imp {
    use crate::ui::{colorpicker::ColorPicker, templatechooser::TemplateChooser};
    use gtk4::{
        glib, prelude::*, subclass::prelude::*, Adjustment, Button, CompositeTemplate, Grid,
        SpinButton, Stack, StackPage, Widget,
    };

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/com/github/flxzt/rnote/ui/penssidebar.ui")]
    pub struct PensSideBar {
        #[template_child]
        pub sidebar_stack: TemplateChild<Stack>,
        #[template_child]
        pub marker_stackpage: TemplateChild<StackPage>,
        #[template_child]
        pub marker_grid: TemplateChild<Grid>,
        #[template_child]
        pub marker_widthreset: TemplateChild<Button>,
        #[template_child]
        pub marker_widthscale_adj: TemplateChild<Adjustment>,
        #[template_child]
        pub marker_spinbutton: TemplateChild<SpinButton>,
        #[template_child]
        pub marker_colorpicker: TemplateChild<ColorPicker>,
        #[template_child]
        pub brush_stackpage: TemplateChild<StackPage>,
        #[template_child]
        pub brush_grid: TemplateChild<Grid>,
        #[template_child]
        pub brush_widthreset: TemplateChild<Button>,
        #[template_child]
        pub brush_widthscale_adj: TemplateChild<Adjustment>,
        #[template_child]
        pub brush_spinbutton: TemplateChild<SpinButton>,
        #[template_child]
        pub brush_colorpicker: TemplateChild<ColorPicker>,
        #[template_child]
        pub brush_templatechooser: TemplateChild<TemplateChooser>,
        #[template_child]
        pub eraser_stackpage: TemplateChild<StackPage>,
        #[template_child]
        pub eraser_grid: TemplateChild<Grid>,
        #[template_child]
        pub eraser_widthreset: TemplateChild<Button>,
        #[template_child]
        pub eraser_widthscale_adj: TemplateChild<Adjustment>,
        #[template_child]
        pub eraser_spinbutton: TemplateChild<SpinButton>,
        #[template_child]
        pub selector_stackpage: TemplateChild<StackPage>,
        #[template_child]
        pub selector_grid: TemplateChild<Grid>,
        #[template_child]
        pub selector_delete_button: TemplateChild<Button>,
    }

    impl Default for PensSideBar {
        fn default() -> Self {
            Self {
                sidebar_stack: TemplateChild::<Stack>::default(),
                marker_stackpage: TemplateChild::<StackPage>::default(),
                marker_grid: TemplateChild::<Grid>::default(),
                marker_widthreset: TemplateChild::<Button>::default(),
                marker_widthscale_adj: TemplateChild::<Adjustment>::default(),
                marker_spinbutton: TemplateChild::<SpinButton>::default(),
                marker_colorpicker: TemplateChild::<ColorPicker>::default(),
                brush_stackpage: TemplateChild::<StackPage>::default(),
                brush_grid: TemplateChild::<Grid>::default(),
                brush_widthreset: TemplateChild::<Button>::default(),
                brush_widthscale_adj: TemplateChild::<Adjustment>::default(),
                brush_spinbutton: TemplateChild::<SpinButton>::default(),
                brush_colorpicker: TemplateChild::<ColorPicker>::default(),
                brush_templatechooser: TemplateChild::<TemplateChooser>::default(),
                eraser_stackpage: TemplateChild::<StackPage>::default(),
                eraser_grid: TemplateChild::<Grid>::default(),
                eraser_widthreset: TemplateChild::<Button>::default(),
                eraser_widthscale_adj: TemplateChild::<Adjustment>::default(),
                eraser_spinbutton: TemplateChild::<SpinButton>::default(),
                selector_stackpage: TemplateChild::<StackPage>::default(),
                selector_grid: TemplateChild::<Grid>::default(),
                selector_delete_button: TemplateChild::<Button>::default(),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PensSideBar {
        const NAME: &'static str = "PensSideBar";
        type Type = super::PensSideBar;
        type ParentType = Widget;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);

            ColorPicker::static_type();
            TemplateChooser::static_type();
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

use crate::{
    app::RnoteApp,
    config,
    pens::eraser::Eraser,
    pens::{brush::Brush, marker::Marker},
    strokes,
    ui::appwindow::RnoteAppWindow,
    ui::colorpicker::ColorPicker,
    ui::templatechooser::TemplateChooser,
    utils,
};

use gtk4::{
    gdk, glib, glib::clone, prelude::*, subclass::prelude::*, Adjustment, Button, Grid, SpinButton,
    Stack, StackPage, Widget,
};

glib::wrapper! {
    pub struct PensSideBar(ObjectSubclass<imp::PensSideBar>)
        @extends Widget;
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
        imp::PensSideBar::from_instance(self).sidebar_stack.get()
    }

    pub fn marker_stackpage(&self) -> StackPage {
        imp::PensSideBar::from_instance(self).marker_stackpage.get()
    }

    pub fn marker_grid(&self) -> Grid {
        imp::PensSideBar::from_instance(self).marker_grid.get()
    }

    pub fn marker_widthreset(&self) -> Button {
        imp::PensSideBar::from_instance(self)
            .marker_widthreset
            .get()
    }

    pub fn marker_widthscale_adj(&self) -> Adjustment {
        imp::PensSideBar::from_instance(self)
            .marker_widthscale_adj
            .get()
    }

    pub fn marker_spinbutton(&self) -> SpinButton {
        imp::PensSideBar::from_instance(self)
            .marker_spinbutton
            .get()
    }

    pub fn marker_colorpicker(&self) -> ColorPicker {
        imp::PensSideBar::from_instance(self)
            .marker_colorpicker
            .get()
    }

    pub fn brush_stackpage(&self) -> StackPage {
        imp::PensSideBar::from_instance(self).brush_stackpage.get()
    }

    pub fn brush_grid(&self) -> Grid {
        imp::PensSideBar::from_instance(self).brush_grid.get()
    }

    pub fn brush_widthreset(&self) -> Button {
        imp::PensSideBar::from_instance(self).brush_widthreset.get()
    }

    pub fn brush_widthscale_adj(&self) -> Adjustment {
        imp::PensSideBar::from_instance(self)
            .brush_widthscale_adj
            .get()
    }

    pub fn brush_spinbutton(&self) -> SpinButton {
        imp::PensSideBar::from_instance(self).brush_spinbutton.get()
    }

    pub fn brush_colorpicker(&self) -> ColorPicker {
        imp::PensSideBar::from_instance(self)
            .brush_colorpicker
            .get()
    }

    pub fn brush_templatechooser(&self) -> TemplateChooser {
        imp::PensSideBar::from_instance(self)
            .brush_templatechooser
            .get()
    }

    pub fn eraser_stackpage(&self) -> StackPage {
        imp::PensSideBar::from_instance(self).eraser_stackpage.get()
    }

    pub fn eraser_grid(&self) -> Grid {
        imp::PensSideBar::from_instance(self).eraser_grid.get()
    }

    pub fn eraser_widthreset(&self) -> Button {
        imp::PensSideBar::from_instance(self)
            .eraser_widthreset
            .get()
    }

    pub fn eraser_widthscale_adj(&self) -> Adjustment {
        imp::PensSideBar::from_instance(self)
            .eraser_widthscale_adj
            .get()
    }

    pub fn eraser_spinbutton(&self) -> SpinButton {
        imp::PensSideBar::from_instance(self)
            .eraser_spinbutton
            .get()
    }

    pub fn selector_stackpage(&self) -> StackPage {
        imp::PensSideBar::from_instance(self)
            .selector_stackpage
            .get()
    }

    pub fn selector_grid(&self) -> Grid {
        imp::PensSideBar::from_instance(self).selector_grid.get()
    }

    pub fn init(&self, appwindow: &RnoteAppWindow) {
        let priv_ = imp::PensSideBar::from_instance(self);

        priv_.sidebar_stack.get().connect_visible_child_name_notify(
            clone!(@weak appwindow => move |sidebar_stack| {
                if let Some(child_name) = sidebar_stack.visible_child_name() {
                    match child_name.to_value().get::<String>().unwrap().as_str() {
                        "marker_page" => {
                            appwindow.application().unwrap().activate_action("current-pen", Some(&"marker".to_variant()));
                        },
                        "brush_page" => {
                            appwindow.application().unwrap().activate_action("current-pen", Some(&"brush".to_variant()));
                        },
                        "eraser_page" => {
                            appwindow.application().unwrap().activate_action("current-pen", Some(&"eraser".to_variant()));
                        }
                        "selector_page" => {
                            appwindow.application().unwrap().activate_action("current-pen", Some(&"selector".to_variant()));
                        }
                        _ => {}
                    };
                };
            }),
        );

        self.init_marker_page(appwindow);
        self.init_brush_page(appwindow);
        self.init_eraser_page(appwindow);
        self.init_selector_page(appwindow);
    }

    fn init_marker_page(&self, appwindow: &RnoteAppWindow) {
        let priv_ = imp::PensSideBar::from_instance(self);

        priv_
            .marker_widthscale_adj
            .get()
            .set_lower(Marker::WIDTH_MIN);
        priv_
            .marker_widthscale_adj
            .get()
            .set_upper(Marker::WIDTH_MAX);
        priv_
            .marker_widthscale_adj
            .get()
            .set_value(Marker::WIDTH_DEFAULT);

        priv_.marker_colorpicker.get().connect_notify_local(Some("current-color"), clone!(@weak appwindow => move |marker_colorpicker, _paramspec| {
            let color = marker_colorpicker.property("current-color").unwrap().get::<gdk::RGBA>().unwrap();
            appwindow.canvas().pens().borrow_mut().marker.set_color(strokes::Color::from_gdk(color));
        }));

        priv_.marker_widthreset.get().connect_clicked(
            clone!(@weak appwindow => move |_marker_widthreset| {
                appwindow.canvas().pens().borrow_mut().marker.set_width(Marker::WIDTH_DEFAULT);
                appwindow.penssidebar().marker_widthscale_adj().set_value(Marker::WIDTH_DEFAULT);
            }),
        );

        priv_.marker_widthscale_adj.get().connect_value_changed(
            clone!(@weak appwindow => move |marker_widthscale_adj| {
                appwindow.canvas().pens().borrow_mut().marker.set_width(marker_widthscale_adj.value());
            }),
        );
    }

    fn init_brush_page(&self, appwindow: &RnoteAppWindow) {
        let priv_ = imp::PensSideBar::from_instance(self);

        priv_.brush_widthscale_adj.get().set_lower(Brush::WIDTH_MIN);
        priv_.brush_widthscale_adj.get().set_upper(Brush::WIDTH_MAX);
        priv_
            .brush_widthscale_adj
            .get()
            .set_value(Brush::WIDTH_DEFAULT);

        priv_.brush_colorpicker.get().connect_notify_local(Some("current-color"), clone!(@weak appwindow => move |brush_colorpicker, _paramspec| {
            let color = brush_colorpicker.property("current-color").unwrap().get::<gdk::RGBA>().unwrap();
            appwindow.canvas().pens().borrow_mut().brush.set_color(strokes::Color::from_gdk(color));
        }));

        priv_.brush_widthreset.get().connect_clicked(
            clone!(@weak appwindow => move |_brush_widthreset| {
                appwindow.canvas().pens().borrow_mut().brush.set_width(Brush::WIDTH_DEFAULT);
                appwindow.penssidebar().brush_widthscale_adj().set_value(Brush::WIDTH_DEFAULT);
            }),
        );

        priv_.brush_widthscale_adj.get().connect_value_changed(
            clone!(@weak appwindow => move |brush_widthscale_adj| {
                appwindow.canvas().pens().borrow_mut().brush.set_width(brush_widthscale_adj.value());
            }),
        );

        let brush_help_text = utils::load_string_from_resource(
            (String::from(config::APP_IDPATH) + "text/brush_filechooser-help.txt").as_str(),
        )
        .unwrap();
        priv_
            .brush_templatechooser
            .get()
            .set_help_text(brush_help_text.as_str());

        if let Some(mut templates_dirpath) = utils::app_config_base_dirpath() {
            templates_dirpath.push("brush_templates");
            if priv_
                .brush_templatechooser
                .set_templates_path(&templates_dirpath)
                .is_err()
            {
                log::error!(
                    "failed to set templates dir `{}` for templatechooser",
                    templates_dirpath.to_str().unwrap()
                )
            };
        }
    }

    fn init_eraser_page(&self, appwindow: &RnoteAppWindow) {
        let priv_ = imp::PensSideBar::from_instance(self);

        priv_
            .eraser_widthscale_adj
            .get()
            .set_lower(Eraser::WIDTH_MIN);
        priv_
            .eraser_widthscale_adj
            .get()
            .set_upper(Eraser::WIDTH_MAX);
        priv_
            .eraser_widthscale_adj
            .get()
            .set_value(Eraser::WIDTH_DEFAULT);

        priv_.eraser_widthreset.get().connect_clicked(
            clone!(@weak appwindow => move |_eraser_widthreset| {
                appwindow.canvas().pens().borrow_mut().eraser.width = Eraser::WIDTH_DEFAULT;
                appwindow.penssidebar().eraser_widthscale_adj().set_value(Eraser::WIDTH_DEFAULT);
            }),
        );

        priv_.eraser_widthscale_adj.get().connect_value_changed(
            clone!(@weak appwindow => move |eraser_widthscale_adj| {
                appwindow.canvas().pens().borrow_mut().eraser.width = eraser_widthscale_adj.value();
            }),
        );
    }

    fn init_selector_page(&self, appwindow: &RnoteAppWindow) {
        let priv_ = imp::PensSideBar::from_instance(self);

        priv_.selector_delete_button.connect_clicked(clone!(@weak appwindow => move |_selector_delete_button| {
            appwindow.application().unwrap().downcast::<RnoteApp>().unwrap().activate_action("delete-selection", None);
        }));
    }
}
