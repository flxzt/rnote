// Imports
use gettextrs::gettext;
use gtk4::{Align, StringObject};
use gtk4::{
    Image, Label, ListItem, Orientation, SignalListItemFactory, StringList, glib::prelude::*,
    prelude::*,
};
use rnote_engine::pens::PenStyle;
use std::ops::{Deref, DerefMut};
use std::str::FromStr;

/// Marker string for the extra "focus mode" entry in the pen shortcut picker.
/// Chosen to not collide with any `PenStyle` string representation.
pub(crate) const FOCUS_MODE_ENTRY: &str = "focus_mode";
pub(crate) const FOCUS_MODE_ICON_NAME: &str = "focus-mode-symbolic";

#[derive(Debug, Clone)]
pub(crate) struct PenShortcutActionListModel(StringList);

impl Default for PenShortcutActionListModel {
    fn default() -> Self {
        Self(StringList::new(&[
            &PenStyle::Brush.to_string(),
            &PenStyle::Shaper.to_string(),
            &PenStyle::Typewriter.to_string(),
            &PenStyle::Eraser.to_string(),
            &PenStyle::Selector.to_string(),
            &PenStyle::Tools.to_string(),
            FOCUS_MODE_ENTRY,
        ]))
    }
}

impl Deref for PenShortcutActionListModel {
    type Target = StringList;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for PenShortcutActionListModel {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Debug, Clone)]
pub(crate) struct PenShortcutActionListFactory(SignalListItemFactory);

impl Default for PenShortcutActionListFactory {
    fn default() -> Self {
        let factory = SignalListItemFactory::new();
        factory.connect_setup(move |_factory, list_item| {
            let list_item = list_item.downcast_ref::<ListItem>().unwrap();
            let item_box = gtk4::Box::builder()
                .orientation(Orientation::Horizontal)
                .build();
            let image = Image::builder().margin_end(12).halign(Align::Start).build();
            let label = Label::builder()
                .label("")
                .hexpand(true)
                .halign(Align::Start)
                .build();

            item_box.prepend(&image);
            item_box.append(&label);
            list_item.set_child(Some(&item_box));
        });
        factory.connect_bind(move |_factory, list_item| {
            let list_item = list_item.downcast_ref::<ListItem>().unwrap();
            let item_string = list_item
                .item()
                .unwrap()
                .downcast::<StringObject>()
                .unwrap()
                .string();
            let item_box = list_item.child().unwrap().downcast::<gtk4::Box>().unwrap();

            let (label, icon_name) = if item_string == FOCUS_MODE_ENTRY {
                (gettext("Focus mode"), String::from(FOCUS_MODE_ICON_NAME))
            } else {
                let pen_style = PenStyle::from_str(&item_string).unwrap();
                let label = match pen_style {
                    PenStyle::Brush => gettext("Brush"),
                    PenStyle::Shaper => gettext("Shaper"),
                    PenStyle::Typewriter => gettext("Typewriter"),
                    PenStyle::Eraser => gettext("Eraser"),
                    PenStyle::Selector => gettext("Selector"),
                    PenStyle::Tools => gettext("Tools"),
                };
                (label, pen_style.icon_name())
            };

            let mut child = item_box.first_child();
            while let Some(ref next_child) = child {
                if next_child.type_() == Label::static_type() {
                    next_child
                        .downcast_ref::<Label>()
                        .unwrap()
                        .set_label(&label);
                } else if next_child.type_() == Image::static_type() {
                    next_child
                        .downcast_ref::<Image>()
                        .unwrap()
                        .set_icon_name(Some(icon_name.as_str()));
                }

                child = next_child.next_sibling();
            }
        });
        Self(factory)
    }
}

impl Deref for PenShortcutActionListFactory {
    type Target = SignalListItemFactory;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for PenShortcutActionListFactory {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Debug, Clone)]
pub(crate) struct PenShortcutActionIconFactory(SignalListItemFactory);

impl Default for PenShortcutActionIconFactory {
    fn default() -> Self {
        let factory = SignalListItemFactory::new();
        factory.connect_setup(move |_factory, list_item| {
            let list_item = list_item.downcast_ref::<ListItem>().unwrap();

            let image = Image::builder().build();
            list_item.set_child(Some(&image));
        });
        factory.connect_bind(move |_factory, list_item| {
            let list_item = list_item.downcast_ref::<ListItem>().unwrap();
            let item_string = list_item
                .item()
                .unwrap()
                .downcast::<StringObject>()
                .unwrap()
                .string();
            let image = list_item.child().unwrap().downcast::<Image>().unwrap();

            let icon_name = if item_string == FOCUS_MODE_ENTRY {
                String::from(FOCUS_MODE_ICON_NAME)
            } else {
                PenStyle::from_str(&item_string).unwrap().icon_name()
            };
            image.set_icon_name(Some(icon_name.as_str()));
        });
        Self(factory)
    }
}

impl Deref for PenShortcutActionIconFactory {
    type Target = SignalListItemFactory;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for PenShortcutActionIconFactory {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
