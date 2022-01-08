mod imp {
    use gtk4::{
        glib, prelude::*, subclass::prelude::*, Adjustment, Button, CompositeTemplate, SpinButton,
    };

    #[derive(Default, Debug, CompositeTemplate)]
    #[template(resource = "/com/github/flxzt/rnote/ui/penssidebar/eraserpage.ui")]
    pub struct EraserPage {
        #[template_child]
        pub width_resetbutton: TemplateChild<Button>,
        #[template_child]
        pub width_adj: TemplateChild<Adjustment>,
        #[template_child]
        pub width_spinbutton: TemplateChild<SpinButton>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for EraserPage {
        const NAME: &'static str = "EraserPage";
        type Type = super::EraserPage;
        type ParentType = gtk4::Widget;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for EraserPage {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
        }

        fn dispose(&self, obj: &Self::Type) {
            while let Some(child) = obj.first_child() {
                child.unparent();
            }
        }
    }

    impl WidgetImpl for EraserPage {}
}

use crate::pens::eraser::Eraser;
use crate::ui::appwindow::RnoteAppWindow;
use gtk4::{
    glib, glib::clone, prelude::*, subclass::prelude::*, Adjustment, Button, Orientable,
    SpinButton, Widget,
};

glib::wrapper! {
    pub struct EraserPage(ObjectSubclass<imp::EraserPage>)
        @extends Widget, @implements Orientable;
}

impl Default for EraserPage {
    fn default() -> Self {
        Self::new()
    }
}

impl EraserPage {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create EraserPage")
    }

    pub fn width_resetbutton(&self) -> Button {
        imp::EraserPage::from_instance(self).width_resetbutton.get()
    }

    pub fn width_adj(&self) -> Adjustment {
        imp::EraserPage::from_instance(self).width_adj.get()
    }

    pub fn width_spinbutton(&self) -> SpinButton {
        imp::EraserPage::from_instance(self).width_spinbutton.get()
    }

    pub fn init(&self, appwindow: &RnoteAppWindow) {
        let width_adj = self.width_adj();

        self.width_adj().set_lower(Eraser::WIDTH_MIN);

        self.width_adj().set_upper(Eraser::WIDTH_MAX);

        self.width_adj().set_value(Eraser::WIDTH_DEFAULT);

        self.width_resetbutton().connect_clicked(
            clone!(@weak width_adj, @weak appwindow => move |_| {
                appwindow.canvas().pens().borrow_mut().eraser.set_width(Eraser::WIDTH_DEFAULT);
                width_adj.set_value(Eraser::WIDTH_DEFAULT);
            }),
        );

        self.width_adj()
            .connect_value_changed(clone!(@weak appwindow => move |width_adj| {
                appwindow.canvas().pens().borrow_mut().eraser.set_width(width_adj.value());
            }));
    }
}
