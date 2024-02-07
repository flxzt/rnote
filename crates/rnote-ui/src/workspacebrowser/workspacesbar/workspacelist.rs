// Imports
use super::RnWorkspaceListEntry;
use crate::utils::VecRefWrapper;
use gtk4::{gio, glib, prelude::*, subclass::prelude::*};
use std::cell::RefCell;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub(crate) struct RnWorkspaceList {
        pub(crate) list: RefCell<Vec<RnWorkspaceListEntry>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for RnWorkspaceList {
        const NAME: &'static str = "RnWorkspaceList";
        type Type = super::RnWorkspaceList;
        type Interfaces = (gio::ListModel,);
    }

    impl ObjectImpl for RnWorkspaceList {}

    impl ListModelImpl for RnWorkspaceList {
        fn item_type(&self) -> glib::Type {
            RnWorkspaceListEntry::static_type()
        }

        fn n_items(&self) -> u32 {
            self.list.borrow().len() as u32
        }

        fn item(&self, position: u32) -> Option<glib::Object> {
            self.list
                .borrow()
                .get(position as usize)
                .map(|e| e.upcast_ref::<glib::Object>())
                .cloned()
        }
    }
}

glib::wrapper! {
    pub(crate) struct RnWorkspaceList(ObjectSubclass<imp::RnWorkspaceList>)
        @implements gio::ListModel;
}

impl Default for RnWorkspaceList {
    fn default() -> Self {
        Self::new()
    }
}

impl RnWorkspaceList {
    pub(crate) fn new() -> Self {
        glib::Object::new()
    }

    pub(crate) fn to_vec(&self) -> Vec<RnWorkspaceListEntry> {
        self.imp().list.borrow().clone()
    }

    pub(crate) fn from_vec(vec: Vec<RnWorkspaceListEntry>) -> Self {
        let list = Self::new();
        list.append(vec);

        list
    }

    pub(crate) fn replace_self(&self, list: Self) {
        self.clear();
        self.append(list.to_vec());
    }

    pub(crate) fn iter(&self) -> VecRefWrapper<RnWorkspaceListEntry> {
        VecRefWrapper::new(self.imp().list.borrow())
    }

    pub(crate) fn push(&self, item: RnWorkspaceListEntry) {
        self.imp().list.borrow_mut().push(item);

        self.items_changed(self.n_items().saturating_sub(1), 0, 1);
    }

    #[allow(unused)]
    pub(crate) fn pop(&self) -> Option<RnWorkspaceListEntry> {
        let popped = self.imp().list.borrow_mut().pop();

        self.items_changed(self.n_items().saturating_sub(1), 1, 0);

        popped
    }

    /// Inserts at position i. Panics if i is OOB
    pub(crate) fn insert(&self, i: usize, el: RnWorkspaceListEntry) {
        self.imp().list.borrow_mut().insert(i, el);

        self.items_changed(i as u32, 0, 1);
    }

    /// Removes at position i. Panics if i is OOB
    pub(crate) fn remove(&self, i: usize) -> RnWorkspaceListEntry {
        let removed = self.imp().list.borrow_mut().remove(i);

        self.items_changed(i as u32, 1, 0);
        removed
    }

    /// Replaces entry at position i. Panics if i is OOB
    pub(crate) fn replace(&self, i: usize, entry: RnWorkspaceListEntry) {
        self.imp().list.borrow_mut()[i] = entry;

        self.items_changed(i as u32, 1, 1);
    }

    pub(crate) fn append(&self, mut items: Vec<RnWorkspaceListEntry>) {
        let amount = items.len() as u32;

        if amount > 0 {
            let pos = self.n_items().saturating_sub(1);
            self.imp().list.borrow_mut().append(&mut items);

            self.items_changed(pos, 0, amount);
        }
    }

    pub(crate) fn clear(&self) {
        let amount = self.n_items();
        self.imp().list.borrow_mut().clear();

        self.items_changed(0, amount, 0);
    }
}

impl glib::variant::StaticVariantType for RnWorkspaceList {
    fn static_variant_type() -> std::borrow::Cow<'static, glib::VariantTy> {
        let ty = RnWorkspaceListEntry::static_variant_type();
        let variant_type = glib::VariantType::new(format!("a({})", ty.as_str()).as_str()).unwrap();
        std::borrow::Cow::from(variant_type)
    }
}

impl glib::variant::ToVariant for RnWorkspaceList {
    fn to_variant(&self) -> glib::Variant {
        self.to_vec().to_variant()
    }
}

impl glib::variant::FromVariant for RnWorkspaceList {
    fn from_variant(variant: &glib::Variant) -> Option<Self> {
        let mut vec = Vec::new();

        for ref el in variant.iter() {
            if let Some(e) = RnWorkspaceListEntry::from_variant(el) {
                vec.push(e);
            }
        }

        Some(Self::from_vec(vec))
    }
}
