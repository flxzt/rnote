use gtk4::Align;
use gtk4::{glib::prelude::*, prelude::*, Image, Label, Orientation, SignalListItemFactory};
use rnote_engine::pens::penholder::PenStyle;
use std::ops::{Deref, DerefMut};

#[derive(Debug, Clone)]
pub struct ChangePenStyleListModel(adw::EnumListModel);

impl Default for ChangePenStyleListModel {
    fn default() -> Self {
        Self(adw::EnumListModel::new(PenStyle::static_type()))
    }
}

impl Deref for ChangePenStyleListModel {
    type Target = adw::EnumListModel;

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
pub struct ChangePenStyleListFactory(SignalListItemFactory);

impl Default for ChangePenStyleListFactory {
    fn default() -> Self {
        let factory = SignalListItemFactory::new();
        factory.connect_setup(move |_factory, item| {
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
            item.set_child(Some(&item_box));
        });
        factory.connect_bind(move |_factory, item| {
            let pen_style = item
                .item()
                .expect("ChangePenStyleListFactory bind() failed, item is None")
                .downcast::<adw::EnumListItem>()
                .expect("ChangePenStyleListFactory bind() failed, item has to be of type `PenStyle`");
            let pen_style =
                PenStyle::try_from(pen_style.value() as u32).expect("PenStyle try_from() failed");

            let item_box = item
                .child()
                .expect("ChangePenStyleListFactory bind() failed, item child is None")
                .downcast::<gtk4::Box>()
                .expect(
                    "ChangePenStyleListFactory bind() failed, item child is not of type `gtk4::Box`",
                );

            let mut child = item_box.first_child();
            while let Some(ref next_child) = child {
                if next_child.type_() == Label::static_type() {
                    next_child
                        .downcast_ref::<Label>()
                        .unwrap()
                        .set_label(pen_style.name().as_str());
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
pub struct ChangePenStyleIconFactory(SignalListItemFactory);

impl Default for ChangePenStyleIconFactory {
    fn default() -> Self {
        let factory = SignalListItemFactory::new();
        factory.connect_setup(move |_factory, item| {
            let image = Image::builder().build();

            item.set_child(Some(&image));
        });
        factory.connect_bind(move |_factory, item| {
            let pen_style = item
                .item()
                .expect("ChangePenStyleIconFactory bind() failed, item is None")
                .downcast::<adw::EnumListItem>()
                .expect(
                    "ChangePenStyleIconFactory bind() failed, item has to be of type `PenStyle`",
                );
            let pen_style =
                PenStyle::try_from(pen_style.value() as u32).expect("PenStyle try_from() failed");

            let image = item
                .child()
                .expect("ChangePenStyleIconFactory bind() failed, item child is None")
                .downcast::<Image>()
                .expect(
                    "ChangePenStyleIconFactory bind() failed, item child is not of type `Image`",
                );

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
