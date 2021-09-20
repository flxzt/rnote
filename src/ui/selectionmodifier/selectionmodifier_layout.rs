mod imp {
    use gtk4::{glib, subclass::prelude::*, LayoutManager};

    #[derive(Default)]
    pub struct SelectionModifierLayout {}

    #[glib::object_subclass]
    impl ObjectSubclass for SelectionModifierLayout {
        const NAME: &'static str = "SelectionModifierLayout";
        type Type = super::SelectionModifierLayout;
        type ParentType = LayoutManager;
    }

    impl ObjectImpl for SelectionModifierLayout {}
    impl LayoutManagerImpl for SelectionModifierLayout {}
}

use gtk4::{glib, LayoutManager};

glib::wrapper! {
    pub struct SelectionModifierLayout(ObjectSubclass<imp::SelectionModifierLayout>)
    @extends LayoutManager;
}

impl SelectionModifierLayout {}
