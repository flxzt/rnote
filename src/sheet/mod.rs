pub mod background;
pub mod format;
pub mod selection;

use std::{cell::RefCell, error::Error, rc::Rc};

use crate::{
    pens::eraser::Eraser,
    sheet::selection::Selection,
    strokes::{self, compose, render::Renderer, Element, StrokeBehaviour, StrokeStyle},
    strokes::{bitmapimage::BitmapImage, vectorimage::VectorImage},
    utils::{self, FileType},
};

use self::{background::Background, format::Format};

use gtk4::{gio, glib, graphene, prelude::*, subclass::prelude::*, Snapshot};
use p2d::bounding_volume::BoundingVolume;
use serde::de::{self, Deserializer, Visitor};
use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize};

mod imp {
    use std::cell::Cell;
    use std::{cell::RefCell, rc::Rc};

    use gtk4::{glib, glib::clone, prelude::*, subclass::prelude::*};

    use crate::sheet::format;
    use crate::sheet::selection::Selection;
    use crate::strokes::{self, Element};

    use super::{Background, Format};

    #[derive(Debug)]
    pub struct Sheet {
        pub strokes: Rc<RefCell<Vec<strokes::StrokeStyle>>>,

        // Skipped by serde Serialize and Deserialize trait implementation
        pub strokes_trash: Rc<RefCell<Vec<strokes::StrokeStyle>>>,
        // Skipped by serde Serialize and Deserialize trait implementation
        pub elements_trash: Rc<RefCell<Vec<Element>>>,

        pub selection: Selection,
        pub format: Format,
        pub background: Rc<RefCell<Background>>,
        pub x: Cell<i32>,
        pub y: Cell<i32>,
        pub width: Cell<i32>,
        pub height: Cell<i32>,
        pub autoexpand_height: Cell<bool>,
        pub padding_bottom: Cell<i32>,
    }

    impl Default for Sheet {
        fn default() -> Self {
            Self {
                strokes: Rc::new(RefCell::new(Vec::new())),
                strokes_trash: Rc::new(RefCell::new(Vec::new())),
                elements_trash: Rc::new(RefCell::new(Vec::new())),
                selection: Selection::new(),
                format: Format::default(),
                background: Rc::new(RefCell::new(Background::default())),
                x: Cell::new(0),
                y: Cell::new(0),
                width: Cell::new(Format::default().width()),
                height: Cell::new(Format::default().height()),
                autoexpand_height: Cell::new(true),
                padding_bottom: Cell::new(Format::default().height()),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Sheet {
        const NAME: &'static str = "Sheet";
        type Type = super::Sheet;
        type ParentType = glib::Object;
    }

    impl ObjectImpl for Sheet {
        fn constructed(&self, obj: &Self::Type) {
            self.format.connect_notify_local(
                Some("dpi"),
                clone!(@weak obj => move |format, _| {
                    let new_width = format::MeasureUnit::convert_measurement(
                        f64::from(format.width()),
                        format::MeasureUnit::Px,
                        obj.format().dpi(),
                        format::MeasureUnit::Px,
                        format.dpi());

                    let new_height = format::MeasureUnit::convert_measurement(
                        f64::from(format.height()),
                        format::MeasureUnit::Px,
                        obj.format().dpi(),
                        format::MeasureUnit::Px,
                        format.dpi());

                    obj.set_width(new_width.round() as i32);
                    obj.set_height(new_height.round() as i32);

                    obj.resize_to_format();
                }),
            );
        }
    }
}

glib::wrapper! {
    pub struct Sheet(ObjectSubclass<imp::Sheet>);
}

impl Default for Sheet {
    fn default() -> Self {
        Self::new()
    }
}

impl Serialize for Sheet {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("Sheet", 12)?;
        state.serialize_field("strokes", &*self.strokes().borrow())?;
        state.serialize_field("strokes_trash", &*self.strokes_trash().borrow())?;
        state.serialize_field("selection", &self.selection())?;
        state.serialize_field("format", &self.format())?;
        state.serialize_field("background", &self.background())?;
        state.serialize_field("x", &self.x())?;
        state.serialize_field("y", &self.y())?;
        state.serialize_field("width", &self.width())?;
        state.serialize_field("height", &self.height())?;
        state.serialize_field("autoexpand_height", &self.autoexpand_height())?;
        state.serialize_field("padding_bottom", &self.padding_bottom())?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for Sheet {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "lowercase")]
        #[allow(non_camel_case_types)]
        enum Field {
            strokes,
            strokes_trash,
            selection,
            format,
            background,
            x,
            y,
            width,
            height,
            autoexpand_height,
            padding_bottom,
            unknown,
        }

        struct SheetVisitor;
        impl<'de> Visitor<'de> for SheetVisitor {
            type Value = Sheet;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct Sheet")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: de::SeqAccess<'de>,
            {
                let strokes = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let strokes_trash = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                let selection: Selection = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(2, &self))?;
                let format: Format = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(3, &self))?;
                let background = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(4, &self))?;
                let x = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(5, &self))?;
                let y = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(6, &self))?;
                let width = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(7, &self))?;
                let height = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(8, &self))?;
                let autoexpand_height = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(9, &self))?;
                let padding_bottom = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(10, &self))?;

                let sheet = Sheet::new();
                *sheet.strokes().borrow_mut() = strokes;
                *sheet.strokes_trash().borrow_mut() = strokes_trash;
                *sheet.selection().strokes().borrow_mut() = selection.strokes().borrow().clone();
                sheet.selection().set_bounds(selection.bounds());
                sheet.selection().set_shown(selection.shown());
                sheet.format().replace_fields(format);
                *sheet.background().borrow_mut() = background;
                sheet.set_x(x);
                sheet.set_y(y);
                sheet.set_width(width);
                sheet.set_height(height);
                sheet.set_autoexpand_height(autoexpand_height);
                sheet.set_padding_bottom(padding_bottom);

                Ok(sheet)
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: de::MapAccess<'de>,
            {
                let mut strokes = None;
                let mut strokes_trash = None;
                let mut selection = None;
                let mut format = None;
                let mut background = None;
                let mut x = None;
                let mut y = None;
                let mut width = None;
                let mut height = None;
                let mut autoexpand_height = None;
                let mut padding_bottom = None;

                while let Some(key) = match map.next_key() {
                    Ok(key) => key,
                    Err(e) => {
                        log::warn!("{}", e);
                        Some(Field::unknown)
                    }
                } {
                    match key {
                        Field::strokes => {
                            if strokes.is_some() {
                                return Err(de::Error::duplicate_field("strokes"));
                            }
                            strokes = Some(map.next_value()?);
                        }
                        Field::strokes_trash => {
                            if strokes_trash.is_some() {
                                return Err(de::Error::duplicate_field("strokes_trash"));
                            }
                            strokes_trash = Some(map.next_value()?);
                        }
                        Field::selection => {
                            if selection.is_some() {
                                return Err(de::Error::duplicate_field("selection"));
                            }
                            selection = Some(map.next_value()?);
                        }
                        Field::format => {
                            if format.is_some() {
                                return Err(de::Error::duplicate_field("format"));
                            }
                            format = Some(map.next_value()?);
                        }
                        Field::background => {
                            if background.is_some() {
                                return Err(de::Error::duplicate_field("background"));
                            }
                            background = Some(map.next_value()?);
                        }
                        Field::x => {
                            if x.is_some() {
                                return Err(de::Error::duplicate_field("x"));
                            }
                            x = Some(map.next_value()?);
                        }
                        Field::y => {
                            if y.is_some() {
                                return Err(de::Error::duplicate_field("y"));
                            }
                            y = Some(map.next_value()?);
                        }
                        Field::width => {
                            if width.is_some() {
                                return Err(de::Error::duplicate_field("width"));
                            }
                            width = Some(map.next_value()?);
                        }
                        Field::height => {
                            if height.is_some() {
                                return Err(de::Error::duplicate_field("height"));
                            }
                            height = Some(map.next_value()?);
                        }
                        Field::autoexpand_height => {
                            if autoexpand_height.is_some() {
                                return Err(de::Error::duplicate_field("autoexpand_height"));
                            }
                            autoexpand_height = Some(map.next_value()?);
                        }
                        Field::padding_bottom => {
                            if padding_bottom.is_some() {
                                return Err(de::Error::duplicate_field("padding_bottom"));
                            }
                            padding_bottom = Some(map.next_value()?);
                        }
                        Field::unknown => {
                            // throw away the value
                            map.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }

                let sheet_default = Sheet::default();

                let strokes = strokes.unwrap_or_else(|| {
                    let err: A::Error = de::Error::missing_field("strokes");
                    log::error!("{}", err);
                    Vec::new()
                });
                let strokes_trash = strokes_trash.unwrap_or_else(|| {
                    let err: A::Error = de::Error::missing_field("strokes_trash");
                    log::error!("{}", err);
                    Vec::new()
                });
                let selection = selection.unwrap_or_else(|| {
                    let err: A::Error = de::Error::missing_field("selection");
                    log::error!("{}", err);
                    Selection::default()
                });
                let format = format.unwrap_or_else(|| {
                    let err: A::Error = de::Error::missing_field("format");
                    log::error!("{}", err);
                    Format::default()
                });
                let background = background.unwrap_or_else(|| {
                    let err: A::Error = de::Error::missing_field("background");
                    log::error!("{}", err);
                    Background::default()
                });
                let x = x.unwrap_or_else(|| {
                    let err: A::Error = de::Error::missing_field("x");
                    log::error!("{}", err);
                    sheet_default.x()
                });
                let y = y.unwrap_or_else(|| {
                    let err: A::Error = de::Error::missing_field("y");
                    log::error!("{}", err);
                    sheet_default.y()
                });
                let width = width.unwrap_or_else(|| {
                    let err: A::Error = de::Error::missing_field("width");
                    log::error!("{}", err);
                    sheet_default.width()
                });
                let height = height.unwrap_or_else(|| {
                    let err: A::Error = de::Error::missing_field("height");
                    log::error!("{}", err);
                    sheet_default.height()
                });
                let autoexpand_height = autoexpand_height.unwrap_or_else(|| {
                    let err: A::Error = de::Error::missing_field("autoexpand_height");
                    log::error!("{}", err);
                    sheet_default.autoexpand_height()
                });
                let padding_bottom = padding_bottom.unwrap_or_else(|| {
                    let err: A::Error = de::Error::missing_field("padding_bottom");
                    log::error!("{}", err);
                    sheet_default.padding_bottom()
                });

                let sheet = Sheet::new();
                *sheet.strokes().borrow_mut() = strokes;
                *sheet.strokes_trash().borrow_mut() = strokes_trash;
                *sheet.selection().strokes().borrow_mut() = selection.strokes().borrow().clone();
                sheet.selection().set_bounds(selection.bounds());
                sheet.selection().set_shown(selection.shown());
                sheet.format().replace_fields(format);
                *sheet.background().borrow_mut() = background;
                sheet.set_x(x);
                sheet.set_y(y);
                sheet.set_width(width);
                sheet.set_height(height);
                sheet.set_autoexpand_height(autoexpand_height);
                sheet.set_padding_bottom(padding_bottom);

                Ok(sheet)
            }
        }

        const FIELDS: &[&str] = &[
            "strokes",
            "strokes_trash",
            "selection",
            "format",
            "background",
            "x",
            "y",
            "width",
            "height",
            "autoexpand_height",
            "padding_bottom",
        ];
        deserializer.deserialize_struct("Sheet", FIELDS, SheetVisitor)
    }
}

impl Sheet {
    pub fn new() -> Self {
        let sheet: Sheet = glib::Object::new(&[]).expect("Failed to create Sheet");
        sheet
    }

    pub fn strokes(&self) -> Rc<RefCell<Vec<StrokeStyle>>> {
        imp::Sheet::from_instance(self).strokes.clone()
    }

    pub fn strokes_trash(&self) -> Rc<RefCell<Vec<StrokeStyle>>> {
        imp::Sheet::from_instance(self).strokes_trash.clone()
    }

    pub fn elements_trash(&self) -> Rc<RefCell<Vec<Element>>> {
        imp::Sheet::from_instance(self).elements_trash.clone()
    }

    pub fn selection(&self) -> Selection {
        imp::Sheet::from_instance(self).selection.clone()
    }

    pub fn x(&self) -> i32 {
        imp::Sheet::from_instance(self).x.get()
    }

    pub fn set_x(&self, x: i32) {
        imp::Sheet::from_instance(self).x.set(x)
    }

    pub fn y(&self) -> i32 {
        imp::Sheet::from_instance(self).y.get()
    }

    pub fn set_y(&self, y: i32) {
        imp::Sheet::from_instance(self).y.set(y)
    }

    pub fn width(&self) -> i32 {
        imp::Sheet::from_instance(self).width.get()
    }

    pub fn set_width(&self, width: i32) {
        imp::Sheet::from_instance(self).width.set(width);
    }

    pub fn height(&self) -> i32 {
        imp::Sheet::from_instance(self).height.get()
    }

    pub fn set_height(&self, height: i32) {
        imp::Sheet::from_instance(self).height.set(height);
    }

    pub fn autoexpand_height(&self) -> bool {
        let priv_ = imp::Sheet::from_instance(self);
        priv_.autoexpand_height.get()
    }

    pub fn set_autoexpand_height(&self, autoexpand_height: bool) {
        let priv_ = imp::Sheet::from_instance(self);
        priv_.autoexpand_height.set(autoexpand_height);

        self.resize_to_format();
    }

    pub fn format(&self) -> Format {
        let priv_ = imp::Sheet::from_instance(self);
        priv_.format.clone()
    }

    pub fn padding_bottom(&self) -> i32 {
        imp::Sheet::from_instance(self).padding_bottom.get()
    }

    pub fn set_padding_bottom(&self, padding_bottom: i32) {
        imp::Sheet::from_instance(self)
            .padding_bottom
            .set(padding_bottom);
    }

    pub fn background(&self) -> Rc<RefCell<Background>> {
        imp::Sheet::from_instance(self).background.clone()
    }

    pub fn bounds(&self) -> p2d::bounding_volume::AABB {
        p2d::bounding_volume::AABB::new(
            na::point![f64::from(self.x()), f64::from(self.y())],
            na::point![f64::from(self.width()), f64::from(self.height())],
        )
    }

    /// Resize needed after calling this
    pub fn undo_last_stroke(&self) {
        let priv_ = imp::Sheet::from_instance(self);

        if let Some(removed_stroke) = priv_.strokes.borrow_mut().pop() {
            priv_.strokes_trash.borrow_mut().push(removed_stroke);
        }
        self.resize_to_format();
    }

    /// Resize needed after calling this
    pub fn redo_last_stroke(&self) {
        let priv_ = imp::Sheet::from_instance(self);

        if let Some(restored_stroke) = priv_.strokes_trash.borrow_mut().pop() {
            priv_.strokes.borrow_mut().push(restored_stroke);
        }
        self.resize_to_format();
    }

    pub fn undo_elements_last_stroke(
        &mut self,
        n_elements: usize,
        scalefactor: f64,
        renderer: &Renderer,
    ) {
        let priv_ = imp::Sheet::from_instance(self);

        if let Some(last_stroke) = priv_.strokes.borrow_mut().last_mut() {
            match last_stroke {
                StrokeStyle::MarkerStroke(markerstroke) => {
                    for _i in 1..=n_elements {
                        if let Some(element) = markerstroke.pop_elem() {
                            priv_.elements_trash.borrow_mut().push(element);

                            markerstroke.update_rendernode(scalefactor, renderer);
                        } else {
                            break;
                        }
                    }
                }
                StrokeStyle::BrushStroke(brushstroke) => {
                    for _i in 1..=n_elements {
                        if let Some(element) = brushstroke.pop_elem() {
                            priv_.elements_trash.borrow_mut().push(element);

                            brushstroke.update_rendernode(scalefactor, renderer);
                        } else {
                            break;
                        }
                    }
                }
                _ => {}
            }
        }
    }

    pub fn redo_elements_last_stroke(
        &mut self,
        n_elements: usize,
        scalefactor: f64,
        renderer: &Renderer,
    ) {
        let priv_ = imp::Sheet::from_instance(self);

        if let Some(last_stroke) = priv_.strokes.borrow_mut().last_mut() {
            match last_stroke {
                StrokeStyle::MarkerStroke(markerstroke) => {
                    for _i in 1..=n_elements {
                        if let Some(element) = priv_.elements_trash.borrow_mut().pop() {
                            markerstroke.push_elem(element);
                            markerstroke.complete_stroke();

                            markerstroke.update_rendernode(scalefactor, renderer);
                        } else {
                            break;
                        }
                    }
                }
                StrokeStyle::BrushStroke(brushstroke) => {
                    for _i in 1..=n_elements {
                        if let Some(element) = priv_.elements_trash.borrow_mut().pop() {
                            brushstroke.push_elem(element);
                            brushstroke.complete_stroke();

                            brushstroke.update_rendernode(scalefactor, renderer);
                        } else {
                            break;
                        }
                    }
                }
                _ => {}
            }
        }
    }

    /// remove any colliding stroke
    pub fn remove_colliding_strokes(
        &self,
        eraser: &Eraser,
        viewport: Option<p2d::bounding_volume::AABB>,
    ) {
        let priv_ = imp::Sheet::from_instance(self);

        if let Some(ref eraser_current_input) = eraser.current_input {
            let eraser_bounds = p2d::bounding_volume::AABB::new(
                na::Point2::from(
                    eraser_current_input.pos()
                        - na::vector![eraser.width() / 2.0, eraser.width() / 2.0],
                ),
                na::Point2::from(
                    eraser_current_input.pos()
                        + na::vector![eraser.width() / 2.0, eraser.width() / 2.0],
                ),
            );

            let mut removed_strokes: Vec<strokes::StrokeStyle> = Vec::new();

            priv_.strokes.borrow_mut().retain(|stroke| {
                if let Some(viewport) = viewport {
                    if !viewport.intersects(&stroke.bounds()) {
                        return true;
                    }
                }
                match stroke {
                    strokes::StrokeStyle::MarkerStroke(markerstroke) => {
                        // First check markerstroke bounds, then conditionally check hitbox
                        if eraser_bounds.intersects(&markerstroke.bounds) {
                            for hitbox_elem in markerstroke.hitbox.iter() {
                                if eraser_bounds.intersects(hitbox_elem) {
                                    removed_strokes.push(stroke.clone());
                                    return false;
                                }
                            }
                        }
                    }
                    strokes::StrokeStyle::BrushStroke(brushstroke) => {
                        // First check markerstroke bounds, then conditionally check hitbox
                        if eraser_bounds.intersects(&brushstroke.bounds) {
                            for hitbox_elem in brushstroke.hitbox.iter() {
                                if eraser_bounds.intersects(hitbox_elem) {
                                    removed_strokes.push(stroke.clone());
                                    return false;
                                }
                            }
                        }
                    }
                    strokes::StrokeStyle::ShapeStroke(shapestroke) => {
                        if eraser_bounds.intersects(&shapestroke.bounds) {
                            removed_strokes.push(stroke.clone());
                            return false;
                        }
                    }
                    strokes::StrokeStyle::VectorImage(vectorimage) => {
                        if eraser_bounds.intersects(&vectorimage.bounds) {
                            removed_strokes.push(stroke.clone());
                            return false;
                        }
                    }
                    strokes::StrokeStyle::BitmapImage(bitmapimage) => {
                        if eraser_bounds.intersects(&bitmapimage.bounds) {
                            removed_strokes.push(stroke.clone());
                            return false;
                        }
                    }
                }

                true
            });
            priv_
                .strokes_trash
                .borrow_mut()
                .append(&mut removed_strokes);

        }
    }

    pub fn clear(&self) {
        let priv_ = imp::Sheet::from_instance(self);

        priv_.strokes.borrow_mut().clear();
        priv_.strokes_trash.borrow_mut().clear();
        priv_.selection.strokes().borrow_mut().clear();
    }

    // Returns true if resizing is needed
    pub fn resize_autoexpand(&self) -> bool {
        let mut resizing_needed = false;
        if self.autoexpand_height() {
            let new_height = self.calc_height();

            if new_height != self.height() {
                resizing_needed = true;
            }
            self.set_height(new_height);
        }

        resizing_needed
    }

    /// Resizing needed after calling this
    pub fn resize_to_format(&self) {
        let priv_ = imp::Sheet::from_instance(self);
        if self.autoexpand_height() {
            self.set_padding_bottom(priv_.format.height());

            let new_height = self.calc_height();

            if new_height != self.height() {
                self.set_height(new_height);
            }
        } else {
            self.set_padding_bottom(0);

            let new_height = self.calc_height();
            self.set_height(
                (new_height as f64 / priv_.format.height() as f64).ceil() as i32
                    * priv_.format.height(),
            );
        }
    }

    pub fn calc_height(&self) -> i32 {
        let priv_ = imp::Sheet::from_instance(self);

        let new_height = if let Some(stroke) =
            priv_.strokes.borrow().iter().max_by_key(|&stroke| {
                stroke.bounds().maxs[1].round() as i32 + self.padding_bottom()
            }) {
            // max_by_key() returns the element, so we need to extract the height again
            stroke.bounds().maxs[1].round() as i32 + self.padding_bottom()
        } else {
            // Strokes are empty so resizing to format height
            priv_.format.height()
        };

        new_height
    }

    pub fn calc_n_pages(&self) -> i32 {
        if self.format().height() > 0 {
            self.height() / self.format().height()
        } else {
            0
        }
    }

    pub fn remove_strokes(&self, indices: Vec<usize>) {
        let priv_ = imp::Sheet::from_instance(self);

        for i in indices.iter() {
            let mut index: Option<usize> = None;
            if priv_.strokes.borrow().get(*i).is_some() {
                index = Some(*i);
            } else {
                log::error!(
                    "remove_strokes() failed at index {}, index is out of bounds",
                    i
                );
            }
            if let Some(index) = index {
                priv_.strokes.borrow_mut().remove(index);
            }
        }
    }

    pub fn draw(&self, scalefactor: f64, snapshot: &Snapshot) {
        let priv_ = imp::Sheet::from_instance(self);

        let sheet_bounds_scaled = graphene::Rect::new(
            self.x() as f32 * scalefactor as f32,
            self.y() as f32 * scalefactor as f32,
            self.width() as f32 * scalefactor as f32,
            self.height() as f32 * scalefactor as f32,
        );

        snapshot.push_clip(&sheet_bounds_scaled);

        priv_.background.borrow().draw(snapshot);

        StrokeStyle::draw_strokes(&priv_.strokes.borrow(), snapshot);

        snapshot.pop();
    }

    pub fn open_sheet(&self, file: &gio::File) -> Result<(), Box<dyn Error>> {
        let sheet: Sheet = serde_json::from_str(&utils::load_file_contents(file)?)?;

        *self.strokes().borrow_mut() = sheet.strokes().borrow().clone();
        *self.strokes_trash().borrow_mut() = sheet.strokes().borrow_mut().clone();
        *self.selection().strokes().borrow_mut() = sheet.selection().strokes().borrow().clone();
        self.selection().set_bounds(sheet.selection().bounds());
        self.selection().set_shown(sheet.selection().shown());
        self.format().replace_fields(sheet.format());
        *self.background().borrow_mut() = sheet.background().borrow().clone();
        self.set_x(sheet.x());
        self.set_y(sheet.y());
        self.set_width(sheet.width());
        self.set_height(sheet.height());
        self.set_autoexpand_height(sheet.autoexpand_height());
        self.set_padding_bottom(sheet.padding_bottom());

        StrokeStyle::complete_all_strokes(&mut *self.strokes().borrow_mut());
        StrokeStyle::complete_all_strokes(&mut *self.strokes_trash().borrow_mut());
        StrokeStyle::complete_all_strokes(&mut *self.selection().strokes().borrow_mut());
        Ok(())
    }

    pub fn save_sheet(&self, file: &gio::File) -> Result<(), Box<dyn Error>> {
        match FileType::lookup_file_type(file) {
            FileType::Rnote => {
                let json_output = serde_json::to_string(self)?;
                let output_stream = file.replace::<gio::Cancellable>(
                    None,
                    false,
                    gio::FileCreateFlags::REPLACE_DESTINATION,
                    None,
                )?;

                output_stream.write::<gio::Cancellable>(json_output.as_bytes(), None)?;
                output_stream.close::<gio::Cancellable>(None)?;
            }
            _ => {
                log::error!("invalid file type for saving sheet in native format");
            }
        }
        Ok(())
    }

    pub fn export_sheet_as_svg(&self, file: gio::File) -> Result<(), Box<dyn Error>> {
        let priv_ = imp::Sheet::from_instance(self);

        let sheet_bounds = p2d::bounding_volume::AABB::new(
            na::point![f64::from(self.x()), f64::from(self.y())],
            na::point![
                f64::from(self.x() + self.width()),
                f64::from(self.y() + self.height())
            ],
        );
        let mut data = String::new();

        data.push_str(
            self.background()
                .borrow()
                .gen_svg_data(sheet_bounds.loosened(1.0))?
                .as_str(),
        );

        for stroke in &*priv_.strokes.borrow() {
            let data_entry = stroke.gen_svg_data(na::vector![0.0, 0.0])?;

            data.push_str(&data_entry);
        }

        data = compose::wrap_svg(
            data.as_str(),
            Some(sheet_bounds),
            Some(sheet_bounds),
            true,
            true,
        );

        let output_stream = file.replace::<gio::Cancellable>(
            None,
            false,
            gio::FileCreateFlags::REPLACE_DESTINATION,
            None,
        )?;
        output_stream.write::<gio::Cancellable>(data.as_bytes(), None)?;
        output_stream.close::<gio::Cancellable>(None)?;

        Ok(())
    }

    pub fn import_file_as_svg(
        &self,
        pos: na::Vector2<f64>,
        file: &gio::File,
    ) -> Result<(), Box<dyn Error>> {
        let priv_ = imp::Sheet::from_instance(self);

        let svg = utils::load_file_contents(file)?;

        priv_
            .strokes
            .borrow_mut()
            .append(&mut priv_.selection.remove_strokes());

        let vector_image = VectorImage::import_from_svg(svg.as_str(), pos, None).unwrap();
        priv_
            .selection
            .push_to_selection(strokes::StrokeStyle::VectorImage(vector_image));

        Ok(())
    }

    pub fn import_file_as_bitmapimage(
        &self,
        pos: na::Vector2<f64>,
        file: &gio::File,
    ) -> Result<(), Box<dyn Error>> {
        let priv_ = imp::Sheet::from_instance(self);

        priv_
            .strokes
            .borrow_mut()
            .append(&mut priv_.selection.remove_strokes());

        let (file_bytes, _) = file.load_bytes::<gio::Cancellable>(None)?;
        let bitmapimage = BitmapImage::import_from_image_bytes(&file_bytes, pos).unwrap();

        priv_
            .selection
            .push_to_selection(strokes::StrokeStyle::BitmapImage(bitmapimage));

        Ok(())
    }
}
