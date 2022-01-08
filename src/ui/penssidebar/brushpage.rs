mod imp {
    use crate::ui::colorpicker::ColorPicker;
    use gtk4::{ListBox, MenuButton};
    use gtk4::{
        glib, prelude::*, subclass::prelude::*, Adjustment, Button, CompositeTemplate, SpinButton,
    };

    #[derive(Default, Debug, CompositeTemplate)]
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
        pub brushstyle_menubutton: TemplateChild<MenuButton>,
        #[template_child]
        pub brushstyle_listbox: TemplateChild<ListBox>,
        #[template_child]
        pub brushstyle_solid_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub brushstyle_textured_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub brushstyle_experimental_row: TemplateChild<adw::ActionRow>,
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
use crate::ui::{appwindow::RnoteAppWindow, colorpicker::ColorPicker};
use crate::utils;
use gtk4::{gdk, Accessible, Actionable, Buildable, ConstraintTarget, ListBox, MenuButton};
use gtk4::{
    glib, glib::clone, prelude::*, subclass::prelude::*, Adjustment, Button, Orientable,
    SpinButton, Widget,
};

glib::wrapper! {
    pub struct BrushPage(ObjectSubclass<imp::BrushPage>)
        @extends Widget,
        @implements Orientable, Accessible, Actionable, Buildable, ConstraintTarget;
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

    pub fn brushstyle_menubutton(&self) -> MenuButton {
        imp::BrushPage::from_instance(self).brushstyle_menubutton.get()
    }

    pub fn brushstyle_listbox(&self) -> ListBox {
        imp::BrushPage::from_instance(self).brushstyle_listbox.get()
    }

    pub fn brushstyle_solid_row(&self) -> adw::ActionRow {
        imp::BrushPage::from_instance(self)
            .brushstyle_solid_row
            .get()
    }

    pub fn brushstyle_textured_row(&self) -> adw::ActionRow {
        imp::BrushPage::from_instance(self)
            .brushstyle_textured_row
            .get()
    }

    pub fn brushstyle_experimental_row(&self) -> adw::ActionRow {
        imp::BrushPage::from_instance(self)
            .brushstyle_experimental_row
            .get()
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

        self.brushstyle_listbox().connect_row_selected(
            clone!(@weak appwindow => move |_brushstyle_listbox, selected_row| {
                if let Some(selected_row) = selected_row.map(|selected_row| {selected_row.downcast_ref::<adw::ActionRow>().unwrap()}) {
                    match selected_row.index() {
                        // Solid
                        0 => {
                            adw::prelude::ActionGroupExt::activate_action(&appwindow, "brush-style", Some(&"solid".to_variant()));
                        }
                        // Textured
                        1 => {
                            adw::prelude::ActionGroupExt::activate_action(&appwindow, "brush-style", Some(&"textured".to_variant()));
                        }
                        // Experimental
                        2 => {
                            adw::prelude::ActionGroupExt::activate_action(&appwindow, "brush-style", Some(&"experimental".to_variant()));
                        }
                        _ => {}
                    }
                }
            }),
        );
    }
}
