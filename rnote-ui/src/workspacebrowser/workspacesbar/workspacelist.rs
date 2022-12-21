use gtk4::{gio, glib, prelude::*, subclass::prelude::*};

use std::cell::{Cell, RefCell};

use super::WorkspaceListEntry;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub(crate) struct WorkspaceList {
        pub(crate) list: RefCell<Vec<WorkspaceListEntry>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for WorkspaceList {
        const NAME: &'static str = "WorkspaceList";
        type Type = super::WorkspaceList;
        type Interfaces = (gio::ListModel,);
    }

    impl ObjectImpl for WorkspaceList {}

    impl ListModelImpl for WorkspaceList {
        fn item_type(&self) -> glib::Type {
            WorkspaceListEntry::static_type()
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
    pub(crate) struct WorkspaceList(ObjectSubclass<imp::WorkspaceList>)
        @implements gio::ListModel;
}

impl Default for WorkspaceList {
    fn default() -> Self {
        Self::new()
    }
}

impl WorkspaceList {
    pub(crate) fn new() -> Self {
        glib::Object::new(&[])
    }

    pub(crate) fn from_vec(vec: Vec<WorkspaceListEntry>) -> Self {
        let list = Self::new();
        list.append(vec);

        list
    }

    pub(crate) fn replace_self(&self, list: Self) {
        self.clear();
        self.append(list.iter().collect());
    }

    pub(crate) fn iter(&self) -> Iter {
        Iter::new(self.clone())
    }

    pub(crate) fn push(&self, item: WorkspaceListEntry) {
        self.imp().list.borrow_mut().push(item);

        self.items_changed(self.n_items().saturating_sub(1), 0, 1);
    }

    #[allow(unused)]
    pub(crate) fn pop(&self) -> Option<WorkspaceListEntry> {
        let popped = self.imp().list.borrow_mut().pop();

        self.items_changed(self.n_items().saturating_sub(1), 1, 0);

        popped
    }

    /// Inserts at position i. Panics if i is OOB
    pub(crate) fn insert(&self, i: usize, el: WorkspaceListEntry) {
        self.imp().list.borrow_mut().insert(i, el);

        self.items_changed(i as u32, 0, 1);
    }

    /// Removes at position i. Panics if i is OOB
    pub(crate) fn remove(&self, i: usize) -> WorkspaceListEntry {
        let removed = self.imp().list.borrow_mut().remove(i);

        self.items_changed(i as u32, 1, 0);
        removed
    }

    /// Replaces entry at position i. Panics if i is OOB
    pub(crate) fn replace(&self, i: usize, entry: WorkspaceListEntry) {
        self.imp().list.borrow_mut()[i] = entry;

        self.items_changed(i as u32, 1, 1);
    }

    pub(crate) fn append(&self, mut items: Vec<WorkspaceListEntry>) {
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

#[derive(Debug, Clone)]
pub(crate) struct Iter {
    model: WorkspaceList,
    i: Cell<u32>,
}

impl Iter {
    const fn new(model: WorkspaceList) -> Self {
        Self {
            model,
            i: Cell::new(0),
        }
    }
}

impl Iterator for Iter {
    type Item = WorkspaceListEntry;

    fn next(&mut self) -> Option<Self::Item> {
        let index = self.i.get();

        let item = self.model.item(index);
        self.i.set(index + 1);
        item.map(|x| x.downcast::<WorkspaceListEntry>().unwrap())
    }
}

impl glib::StaticVariantType for WorkspaceList {
    fn static_variant_type() -> std::borrow::Cow<'static, glib::VariantTy> {
        let ty = WorkspaceListEntry::static_variant_type();
        let variant_type = glib::VariantType::new(format!("a({})", ty.as_str()).as_str()).unwrap();
        std::borrow::Cow::from(variant_type)
    }
}

impl glib::ToVariant for WorkspaceList {
    fn to_variant(&self) -> glib::Variant {
        self.iter()
            .collect::<Vec<WorkspaceListEntry>>()
            .to_variant()
    }
}

impl glib::FromVariant for WorkspaceList {
    fn from_variant(variant: &glib::Variant) -> Option<Self> {
        let mut vec = Vec::new();

        for ref el in variant.iter() {
            if let Some(e) = WorkspaceListEntry::from_variant(el) {
                vec.push(e);
            }
        }

        Some(Self::from_vec(vec))
    }
}
