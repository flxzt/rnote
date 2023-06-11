// Imports
use gettextrs::gettext;
use gtk4::{
    glib::prelude::*, prelude::*, Image, Label, ListItem, Orientation, SignalListItemFactory,
    StringList,
};
use gtk4::{Align, StringObject};
use rnote_engine::pens::PenStyle;
use std::ops::{Deref, DerefMut};
use std::str::FromStr;

#[derive(Debug, Clone)]
pub(crate) struct ChangePenStyleListModel(StringList);

impl Default for ChangePenStyleListModel {
    fn default() -> Self {
        Self(StringList::new(&[
            &PenStyle::Brush.to_string(),
            &PenStyle::Shaper.to_string(),
            &PenStyle::Typewriter.to_string(),
            &PenStyle::Eraser.to_string(),
            &PenStyle::Selector.to_string(),
            &PenStyle::Tools.to_string(),
        ]))
    }
}

impl Deref for ChangePenStyleListModel {
    type Target = StringList;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ChangePenStyleListModel {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ChangePenStyleListFactory(SignalListItemFactory);

impl Default for ChangePenStyleListFactory {
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
            let pen_style = PenStyle::from_str(
                &list_item
                    .item()
                    .unwrap()
                    .downcast::<StringObject>()
                    .unwrap()
                    .string(),
            )
            .unwrap();
            let item_box = list_item.child().unwrap().downcast::<gtk4::Box>().unwrap();

            let mut child = item_box.first_child();
            while let Some(ref next_child) = child {
                if next_child.type_() == Label::static_type() {
                    let label = match pen_style {
                        PenStyle::Brush => gettext("Brush"),
                        PenStyle::Shaper => gettext("Shaper"),
                        PenStyle::Typewriter => gettext("Typewriter"),
                        PenStyle::Eraser => gettext("Eraser"),
                        PenStyle::Selector => gettext("Selector"),
                        PenStyle::Tools => gettext("Tools"),
                    };
                    next_child
                        .downcast_ref::<Label>()
                        .unwrap()
                        .set_label(&label);
                } else if next_child.type_() == Image::static_type() {
                    next_child
                        .downcast_ref::<Image>()
                        .unwrap()
                        .set_icon_name(Some(pen_style.icon_name().as_str()));
                }

                child = next_child.next_sibling();
            }
        });
        Self(factory)
    }
}

impl Deref for ChangePenStyleListFactory {
    type Target = SignalListItemFactory;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ChangePenStyleListFactory {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ChangePenStyleIconFactory(SignalListItemFactory);

impl Default for ChangePenStyleIconFactory {
    fn default() -> Self {
        let factory = SignalListItemFactory::new();
        factory.connect_setup(move |_factory, list_item| {
            let list_item = list_item.downcast_ref::<ListItem>().unwrap();

            let image = Image::builder().build();
            list_item.set_child(Some(&image));
        });
        factory.connect_bind(move |_factory, list_item| {
            let list_item = list_item.downcast_ref::<ListItem>().unwrap();
            let pen_style = PenStyle::from_str(
                &list_item
                    .item()
                    .unwrap()
                    .downcast::<StringObject>()
                    .unwrap()
                    .string(),
            )
            .unwrap();
            let image = list_item.child().unwrap().downcast::<Image>().unwrap();
            image
                .downcast_ref::<Image>()
                .unwrap()
                .set_icon_name(Some(pen_style.icon_name().as_str()));
        });
        Self(factory)
    }
}

impl Deref for ChangePenStyleIconFactory {
    type Target = SignalListItemFactory;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ChangePenStyleIconFactory {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
