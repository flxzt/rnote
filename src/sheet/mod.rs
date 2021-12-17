pub mod background;
pub mod format;

mod imp {
    use std::cell::Cell;
    use std::{cell::RefCell, rc::Rc};

    use gtk4::{glib, glib::clone, prelude::*, subclass::prelude::*};
    use once_cell::sync::Lazy;

    use crate::sheet::format;
    use crate::{config, strokes};

    use super::{Background, Format};

    #[derive(Debug)]
    pub struct Sheet {
        pub version: Rc<RefCell<String>>,
        pub strokes_state: Rc<RefCell<strokes::StrokesState>>,
        pub format: Format,
        pub background: Rc<RefCell<Background>>,
        pub width: Cell<i32>,
        pub height: Cell<i32>,
        pub padding_bottom: Cell<i32>,
        pub endless_sheet: Cell<bool>,
        pub format_borders: Cell<bool>,
    }

    impl Default for Sheet {
        fn default() -> Self {
            Self {
                version: Rc::new(RefCell::new(String::from(config::APP_VERSION))),
                strokes_state: Rc::new(RefCell::new(strokes::StrokesState::default())),
                format: Format::default(),
                background: Rc::new(RefCell::new(Background::default())),
                width: Cell::new(Format::default().width()),
                height: Cell::new(Format::default().height()),
                padding_bottom: Cell::new(Format::default().height()),
                endless_sheet: Cell::new(true),
                format_borders: Cell::new(true),
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

        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![glib::ParamSpec::new_boolean(
                    "endless-sheet",
                    "endless-sheet",
                    "endless-sheet",
                    false,
                    glib::ParamFlags::READWRITE,
                )]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(
            &self,
            _obj: &Self::Type,
            _id: usize,
            value: &glib::Value,
            pspec: &glib::ParamSpec,
        ) {
            match pspec.name() {
                "endless-sheet" => {
                    self.endless_sheet
                        .replace(value.get::<bool>().expect("Value not of type `bool`"));
                }
                _ => panic!("invalid property name"),
            }
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "endless-sheet" => self.endless_sheet.get().to_value(),
                _ => panic!("invalid property name"),
            }
        }
    }
}

use std::{cell::RefCell, rc::Rc};

use crate::strokes::strokestyle::StrokeStyle;
use crate::utils;
use crate::{
    compose,
    strokes::{bitmapimage::BitmapImage, vectorimage::VectorImage, StrokesState},
    utils::FileType,
};

use self::{background::Background, format::Format};

use gtk4::{gio, glib, graphene, prelude::*, subclass::prelude::*, Snapshot};
use p2d::bounding_volume::BoundingVolume;
use serde::de::{self, Deserializer, Visitor};
use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize};

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
        state.serialize_field("version", &*self.version())?;
        state.serialize_field("strokes_state", &*self.strokes_state().borrow())?;
        state.serialize_field("format", &self.format())?;
        state.serialize_field("background", &self.background())?;
        state.serialize_field("width", &self.width())?;
        state.serialize_field("height", &self.height())?;
        state.serialize_field("endless_sheet", &self.endless_sheet())?;
        state.serialize_field("padding_bottom", &self.padding_bottom())?;
        state.serialize_field("format_borders", &self.format_borders())?;
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
            version,
            strokes_state,
            format,
            background,
            width,
            height,
            padding_bottom,
            endless_sheet,
            format_borders,
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
                let version = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let strokes_state = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                let format: Format = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(2, &self))?;
                let background = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(3, &self))?;
                let width = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(4, &self))?;
                let height = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(5, &self))?;
                let padding_bottom = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(6, &self))?;
                let endless_sheet = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(7, &self))?;
                let format_borders = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(8, &self))?;

                let sheet = Sheet::new();
                sheet.set_version(version);
                *sheet.strokes_state().borrow_mut() = strokes_state;
                sheet.format().import_format(format);
                *sheet.background().borrow_mut() = background;
                sheet.set_width(width);
                sheet.set_height(height);
                sheet.set_endless_sheet(endless_sheet);
                sheet.set_padding_bottom(padding_bottom);
                sheet.set_format_borders(format_borders);

                Ok(sheet)
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: de::MapAccess<'de>,
            {
                let mut version = None;
                let mut strokes_state = None;
                let mut format = None;
                let mut background = None;
                let mut width = None;
                let mut height = None;
                let mut padding_bottom = None;
                let mut endless_sheet = None;
                let mut format_borders = None;

                while let Some(key) = match map.next_key() {
                    Ok(key) => key,
                    Err(e) => {
                        log::warn!("{}", e);
                        Some(Field::unknown)
                    }
                } {
                    match key {
                        Field::version => {
                            if version.is_some() {
                                return Err(de::Error::duplicate_field("version"));
                            }
                            version = Some(map.next_value()?);
                        }
                        Field::strokes_state => {
                            if strokes_state.is_some() {
                                return Err(de::Error::duplicate_field("strokes_state"));
                            }
                            strokes_state = Some(map.next_value()?);
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
                        Field::padding_bottom => {
                            if padding_bottom.is_some() {
                                return Err(de::Error::duplicate_field("padding_bottom"));
                            }
                            padding_bottom = Some(map.next_value()?);
                        }
                        Field::endless_sheet => {
                            if endless_sheet.is_some() {
                                return Err(de::Error::duplicate_field("endless_sheet"));
                            }
                            endless_sheet = Some(map.next_value()?);
                        }
                        Field::format_borders => {
                            if format_borders.is_some() {
                                return Err(de::Error::duplicate_field("format_borders"));
                            }
                            format_borders = Some(map.next_value()?);
                        }
                        Field::unknown => {
                            // throw away the value
                            map.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }

                let sheet_default = Sheet::default();

                let version = version.unwrap_or_else(|| {
                    let err: A::Error = de::Error::missing_field("version");
                    log::error!("{}", err);
                    sheet_default.version()
                });
                let strokes_state = strokes_state.unwrap_or_else(|| {
                    let err: A::Error = de::Error::missing_field("strokes_state");
                    log::error!("{}", err);
                    StrokesState::new()
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
                let padding_bottom = padding_bottom.unwrap_or_else(|| {
                    let err: A::Error = de::Error::missing_field("padding_bottom");
                    log::error!("{}", err);
                    sheet_default.padding_bottom()
                });
                let endless_sheet = endless_sheet.unwrap_or_else(|| {
                    let err: A::Error = de::Error::missing_field("endless_sheet");
                    log::error!("{}", err);
                    sheet_default.endless_sheet()
                });
                let format_borders = format_borders.unwrap_or_else(|| {
                    let err: A::Error = de::Error::missing_field("format_borders");
                    log::error!("{}", err);
                    sheet_default.format_borders()
                });

                let sheet = Sheet::new();
                sheet.set_version(version);
                *sheet.strokes_state().borrow_mut() = strokes_state;
                sheet.format().import_format(format);
                *sheet.background().borrow_mut() = background;
                sheet.set_width(width);
                sheet.set_height(height);
                sheet.set_padding_bottom(padding_bottom);
                sheet.set_endless_sheet(endless_sheet);
                sheet.set_format_borders(format_borders);

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
            "padding_bottom",
            "endless_sheet",
            "format_borders",
        ];
        deserializer.deserialize_struct("Sheet", FIELDS, SheetVisitor)
    }
}

impl Sheet {
    pub fn new() -> Self {
        let sheet: Sheet = glib::Object::new(&[]).expect("Failed to create Sheet");
        sheet
    }

    pub fn version(&self) -> String {
        imp::Sheet::from_instance(self).version.borrow().clone()
    }

    pub fn set_version(&self, version: String) {
        *imp::Sheet::from_instance(self).version.borrow_mut() = version;
    }

    pub fn strokes_state(&self) -> Rc<RefCell<StrokesState>> {
        imp::Sheet::from_instance(self).strokes_state.clone()
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

    pub fn padding_bottom(&self) -> i32 {
        imp::Sheet::from_instance(self).padding_bottom.get()
    }

    pub fn set_padding_bottom(&self, padding_bottom: i32) {
        imp::Sheet::from_instance(self)
            .padding_bottom
            .set(padding_bottom);
    }

    pub fn endless_sheet(&self) -> bool {
        let priv_ = imp::Sheet::from_instance(self);
        priv_.endless_sheet.get()
    }

    pub fn set_endless_sheet(&self, endless_sheet: bool) {
        let priv_ = imp::Sheet::from_instance(self);
        priv_.endless_sheet.set(endless_sheet);

        self.resize_to_format();
    }

    pub fn format_borders(&self) -> bool {
        let priv_ = imp::Sheet::from_instance(self);
        priv_.format_borders.get()
    }

    pub fn set_format_borders(&self, format_borders: bool) {
        let priv_ = imp::Sheet::from_instance(self);
        priv_.format_borders.set(format_borders);
    }

    pub fn format(&self) -> Format {
        let priv_ = imp::Sheet::from_instance(self);
        priv_.format.clone()
    }

    pub fn background(&self) -> Rc<RefCell<Background>> {
        imp::Sheet::from_instance(self).background.clone()
    }

    pub fn bounds(&self) -> p2d::bounding_volume::AABB {
        p2d::bounding_volume::AABB::new(
            na::point![0.0, 0.0],
            na::point![f64::from(self.width()), f64::from(self.height())],
        )
    }

    /// Called when any stroke could change the sheet size when "endless-sheet" is set. Returns true if resizing is needed
    pub fn resize_endless(&self) -> bool {
        let mut resizing_needed = false;
        if self.endless_sheet() {
            let new_height = self.strokes_state().borrow().calc_height() + self.padding_bottom();

            if new_height != self.height() {
                resizing_needed = true;
                self.set_height(new_height);
            }
        }

        resizing_needed
    }

    /// Called when sheet should resize to fit all strokes. Resizing needed after calling this
    pub fn resize_to_format(&self) {
        let priv_ = imp::Sheet::from_instance(self);
        if self.endless_sheet() {
            self.resize_endless();
        } else {
            // set padding_bottom to 1 because then 'fraction'.ceil() is at least 1
            self.set_padding_bottom(1);

            let new_height = self.strokes_state().borrow().calc_height() + self.padding_bottom();
            self.set_height(
                (new_height as f64 / priv_.format.height() as f64).ceil() as i32
                    * priv_.format.height(),
            );
        }
    }

    pub fn calc_n_pages(&self) -> i32 {
        if self.format().height() > 0 {
            self.height() / self.format().height()
        } else {
            0
        }
    }

    pub fn draw(&self, zoom: f64, snapshot: &Snapshot) {
        let priv_ = imp::Sheet::from_instance(self);

        let sheet_bounds_scaled = graphene::Rect::new(
            0.0,
            0.0,
            self.width() as f32 * zoom as f32,
            self.height() as f32 * zoom as f32,
        );

        snapshot.push_clip(&sheet_bounds_scaled);
        priv_.background.borrow().draw(snapshot);
        snapshot.pop();
    }

    pub fn open_sheet_from_bytes(&self, bytes: &[u8]) -> Result<(), anyhow::Error> {
        let decompressed_bytes = utils::decompress_from_gzip(bytes)?;
        let sheet: Sheet = serde_json::from_str(&String::from_utf8_lossy(&decompressed_bytes))?;

        self.set_version(sheet.version());
        self.strokes_state()
            .borrow_mut()
            .import_state(&*sheet.strokes_state().borrow());
        self.format().import_format(sheet.format());
        self.background()
            .borrow_mut()
            .import_background(&*sheet.background().borrow());
        self.set_width(sheet.width());
        self.set_height(sheet.height());
        self.set_padding_bottom(sheet.padding_bottom());
        self.set_endless_sheet(sheet.endless_sheet());

        Ok(())
    }

    pub fn save_sheet_to_file(&self, file: &gio::File) -> Result<(), anyhow::Error> {
        match FileType::lookup_file_type(file) {
            FileType::RnoteFile => {
                let json_output = serde_json::to_string(self)?;
                if let Some(file_name) = file.basename() {
                    let compressed_bytes = utils::compress_to_gzip(
                        json_output.as_bytes(),
                        &file_name.to_string_lossy(),
                    )?;
                    let output_stream = file.replace::<gio::Cancellable>(
                        None,
                        false,
                        gio::FileCreateFlags::REPLACE_DESTINATION,
                        None,
                    )?;

                    output_stream.write::<gio::Cancellable>(&compressed_bytes, None)?;
                    output_stream.close::<gio::Cancellable>(None)?;
                } else {
                    log::error!("failed to get file name while saving sheet. Invalid file");
                }
            }
            _ => {
                log::error!("invalid file type for saving sheet in native format");
            }
        }
        Ok(())
    }

    pub fn gen_svg(&self) -> Result<String, anyhow::Error> {
        let sheet_bounds = p2d::bounding_volume::AABB::new(
            na::point![0.0, 0.0],
            na::point![f64::from(self.width()), f64::from(self.height())],
        );
        let mut data = String::new();

        data.push_str(
            self.background()
                .borrow()
                .gen_svg_data(sheet_bounds.loosened(1.0))?
                .as_str(),
        );

        data.push_str(
            &self
                .strokes_state()
                .borrow()
                .gen_svg_all_strokes()?
                .as_str(),
        );

        data = compose::wrap_svg(
            data.as_str(),
            Some(sheet_bounds),
            Some(sheet_bounds),
            true,
            true,
        );

        Ok(data)
    }

    pub fn export_sheet_as_svg(&self, file: gio::File) -> Result<(), anyhow::Error> {
        let data = self.gen_svg()?;

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

    pub fn import_bytes_as_svg(
        &self,
        pos: na::Vector2<f64>,
        bytes: &[u8],
    ) -> Result<(), anyhow::Error> {
        let priv_ = imp::Sheet::from_instance(self);
        let svg = String::from_utf8_lossy(bytes);

        priv_.strokes_state.borrow_mut().deselect_all_strokes();

        let vector_image = VectorImage::import_from_svg(&svg, pos, None).unwrap();
        let inserted = priv_
            .strokes_state
            .borrow_mut()
            .insert_stroke(StrokeStyle::VectorImage(vector_image));
        priv_
            .strokes_state
            .borrow_mut()
            .set_selected(inserted, true);

        Ok(())
    }

    pub fn import_bytes_as_bitmapimage(
        &self,
        pos: na::Vector2<f64>,
        bytes: &[u8],
    ) -> Result<(), anyhow::Error> {
        let priv_ = imp::Sheet::from_instance(self);

        priv_.strokes_state.borrow_mut().deselect_all_strokes();

        let bitmapimage = BitmapImage::import_from_image_bytes(bytes, pos)?;

        let inserted = priv_
            .strokes_state
            .borrow_mut()
            .insert_stroke(StrokeStyle::BitmapImage(bitmapimage));
        priv_
            .strokes_state
            .borrow_mut()
            .set_selected(inserted, true);

        self.resize_to_format();

        Ok(())
    }

    pub fn import_bytes_as_pdf_bitmap(
        &self,
        pos: na::Vector2<f64>,
        bytes: &[u8],
        page_width: Option<i32>,
    ) -> Result<(), anyhow::Error> {
        let priv_ = imp::Sheet::from_instance(self);

        priv_.strokes_state.borrow_mut().deselect_all_strokes();

        let bitmapimages = BitmapImage::import_from_pdf_bytes(bytes, pos, page_width)?;

        for bitmapimage in bitmapimages {
            let inserted = priv_
                .strokes_state
                .borrow_mut()
                .insert_stroke(StrokeStyle::BitmapImage(bitmapimage));

            priv_
                .strokes_state
                .borrow_mut()
                .set_selected(inserted, true);
        }
        self.resize_to_format();

        Ok(())
    }

    pub fn import_bytes_as_pdf_vector(
        &self,
        pos: na::Vector2<f64>,
        bytes: &[u8],
    ) -> Result<(), anyhow::Error> {
        let priv_ = imp::Sheet::from_instance(self);

        priv_.strokes_state.borrow_mut().deselect_all_strokes();

        let pages = VectorImage::import_from_pdf_bytes(
            bytes,
            pos,
            Some(self.width() - 2 * VectorImage::OFFSET_X_DEFAULT.round() as i32),
        )?;

        for page in pages {
            let inserted = priv_
                .strokes_state
                .borrow_mut()
                .insert_stroke(StrokeStyle::VectorImage(page));

            priv_
                .strokes_state
                .borrow_mut()
                .set_selected(inserted, true);
        }
        self.resize_to_format();

        Ok(())
    }
}
