mod imp {
    use crate::ui::{colorpicker::ColorPicker, templatechooser::TemplateChooser};
    use gtk4::{
        glib, prelude::*, subclass::prelude::*, Adjustment, Button, CompositeTemplate, SpinButton,
    };

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/com/github/flxzt/rnote/ui/penssidebar/brushpage.ui")]
    pub struct BrushPage {
        #[template_child]
        pub width_resetbutton: TemplateChild<Button>,
        #[template_child]
        pub width_adj: TemplateChild<Adjustment>,
        #[template_child]
        pub width_spinbutton: TemplateChild<SpinButton>,
        #[template_child]
        pub colorpicker: TemplateChild<ColorPicker>,
        #[template_child]
        pub templatechooser: TemplateChild<TemplateChooser>,
    }

    impl Default for BrushPage {
        fn default() -> Self {
            Self {
                width_resetbutton: TemplateChild::<Button>::default(),
                width_adj: TemplateChild::<Adjustment>::default(),
                width_spinbutton: TemplateChild::<SpinButton>::default(),
                colorpicker: TemplateChild::<ColorPicker>::default(),
                templatechooser: TemplateChild::<TemplateChooser>::default(),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for BrushPage {
        const NAME: &'static str = "BrushPage";
        type Type = super::BrushPage;
        type ParentType = gtk4::Widget;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for BrushPage {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
        }

        fn dispose(&self, obj: &Self::Type) {
            while let Some(child) = obj.first_child() {
                child.unparent();
            }
        }
    }

    impl WidgetImpl for BrushPage {}
}

use crate::pens::brush::Brush;
use crate::ui::{
    appwindow::RnoteAppWindow, colorpicker::ColorPicker, templatechooser::TemplateChooser,
};
use crate::{config, utils};
use gtk4::gdk;
use gtk4::{
    glib, glib::clone, prelude::*, subclass::prelude::*, Adjustment, Button, Orientable,
    SpinButton, Widget,
};

glib::wrapper! {
    pub struct BrushPage(ObjectSubclass<imp::BrushPage>)
        @extends Widget, @implements Orientable;
}

impl Default for BrushPage {
    fn default() -> Self {
        Self::new()
    }
}

impl BrushPage {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create BrushPage")
    }

    pub fn width_resetbutton(&self) -> Button {
        imp::BrushPage::from_instance(self).width_resetbutton.get()
    }

    pub fn width_adj(&self) -> Adjustment {
        imp::BrushPage::from_instance(self).width_adj.get()
    }

    pub fn width_spinbutton(&self) -> SpinButton {
        imp::BrushPage::from_instance(self).width_spinbutton.get()
    }

    pub fn colorpicker(&self) -> ColorPicker {
        imp::BrushPage::from_instance(self).colorpicker.get()
    }

    pub fn templatechooser(&self) -> TemplateChooser {
        imp::BrushPage::from_instance(self).templatechooser.get()
    }

    pub fn init(&self, appwindow: &RnoteAppWindow) {
        let width_adj = self.width_adj();

        self.width_adj().set_lower(Brush::WIDTH_MIN);
        self.width_adj().set_upper(Brush::WIDTH_MAX);
        self.width_adj().set_value(Brush::WIDTH_DEFAULT);

        self.colorpicker().connect_notify_local(Some("current-color"), clone!(@weak appwindow => move |colorpicker, _paramspec| {
            let color = colorpicker.property("current-color").unwrap().get::<gdk::RGBA>().unwrap();
            appwindow.canvas().pens().borrow_mut().brush.color = utils::Color::from(color);
        }));

        self.width_resetbutton().connect_clicked(
            clone!(@weak width_adj, @weak appwindow => move |_| {
                appwindow.canvas().pens().borrow_mut().brush.set_width(Brush::WIDTH_DEFAULT);
                width_adj.set_value(Brush::WIDTH_DEFAULT);
            }),
        );

        self.width_adj().connect_value_changed(
            clone!(@weak appwindow => move |brush_widthscale_adj| {
                appwindow.canvas().pens().borrow_mut().brush.set_width(brush_widthscale_adj.value());
            }),
        );

        let brush_help_text = utils::load_string_from_resource(
            (String::from(config::APP_IDPATH) + "text/brush_filechooser-help.txt").as_str(),
        )
        .unwrap();
        self.templatechooser()
            .set_help_text(brush_help_text.as_str());

        if let Some(mut templates_dirpath) = utils::app_config_base_dirpath() {
            templates_dirpath.push("brush_templates");
            if self
                .templatechooser()
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
}
