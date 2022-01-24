pub mod background;
pub mod format;

mod imp {
    use std::cell::Cell;
    use std::{cell::RefCell, rc::Rc};

    use gtk4::{glib, glib::clone, prelude::*, subclass::prelude::*};
    use once_cell::sync::Lazy;

    use crate::config;
    use crate::sheet::format;
    use crate::strokesstate::StrokesState;

    use super::{Background, Format};

    #[derive(Debug)]
    pub struct Sheet {
        pub version: Rc<RefCell<String>>,
        pub strokes_state: Rc<RefCell<StrokesState>>,
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
                strokes_state: Rc::new(RefCell::new(StrokesState::default())),
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
                vec![glib::ParamSpecBoolean::new(
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

use crate::compose::shapes;
use crate::pens::brush::Brush;
use crate::strokes::bitmapimage;
use crate::strokes::bitmapimage::BitmapImage;
use crate::strokes::brushstroke::BrushStroke;
use crate::strokes::strokebehaviour;
use crate::strokes::strokebehaviour::StrokeBehaviour;
use crate::strokes::strokestyle::Element;
use crate::strokes::strokestyle::InputData;
use crate::strokes::strokestyle::StrokeStyle;
use crate::{compose, strokesstate::StrokesState};
use crate::{render, utils};
use notetakingfileformats::xoppformat;
use notetakingfileformats::FileFormatLoader;
use notetakingfileformats::FileFormatSaver;

use self::{background::Background, format::Format};

use gtk4::{gio, glib, graphene, prelude::*, subclass::prelude::*, Snapshot};
use p2d::bounding_volume::{BoundingVolume, AABB};
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
        let mut state = serializer.serialize_struct("sheet", 9)?;
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
                formatter.write_str("the Sheet struct")
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
                sheet.format().import_format(&format);
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
                sheet.format().import_format(&format);
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
            "version",
            "strokes_state",
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
        deserializer.deserialize_struct("sheet", FIELDS, SheetVisitor)
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

    pub fn bounds(&self) -> AABB {
        AABB::new(
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
            // +1 because then 'fraction'.ceil() is at least 1
            let new_height = self.strokes_state().borrow().calc_height() + 1;
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

    pub fn gen_pages_bounds(&self) -> Vec<AABB> {
        let n_pages = self.calc_n_pages();
        let sheet_bounds = self.bounds();

        let page_width = f64::from(self.format().width());
        let page_height = f64::from(self.format().height());

        (0..n_pages)
            .map(|i| {
                AABB::new(
                    na::point![
                        sheet_bounds.mins[0],
                        sheet_bounds.mins[1] + page_height * f64::from(i)
                    ],
                    na::point![
                        sheet_bounds.mins[0] + page_width,
                        sheet_bounds.mins[1] + page_height * f64::from(i + 1)
                    ],
                )
            })
            .collect::<Vec<AABB>>()
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

        if self.format_borders() {
            self.format().draw(self.bounds(), snapshot, zoom);
        }

        snapshot.pop();
    }

    pub fn import_sheet(&self, sheet: &Self) {
        self.strokes_state()
            .borrow_mut()
            .import_state(&*sheet.strokes_state().borrow());
        self.format().import_format(&sheet.format());
        self.background()
            .borrow_mut()
            .import_background(&*sheet.background().borrow());
        self.set_width(sheet.width());
        self.set_height(sheet.height());
        self.set_padding_bottom(sheet.padding_bottom());
        self.set_endless_sheet(sheet.endless_sheet());
    }

    pub fn open_sheet_from_rnote_bytes(&self, bytes: glib::Bytes) -> Result<(), anyhow::Error> {
        let decompressed_bytes = utils::decompress_from_gzip(&bytes)?;
        let sheet: Sheet = serde_json::from_str(&String::from_utf8(decompressed_bytes)?)?;

        self.import_sheet(&sheet);

        Ok(())
    }

    pub fn open_from_xopp_bytes(&mut self, bytes: glib::Bytes) -> Result<(), anyhow::Error> {
        // We set the sheet dpi to the hardcoded xournal++ dpi, so no need to convert values or coordinates anywhere
        self.format().set_dpi(xoppformat::XoppFile::DPI);

        let xopp_file = xoppformat::XoppFile::load_from_bytes(&bytes)?;

        // Extract the largest width of all sheets, add together all heights
        let (sheet_width, sheet_height) = xopp_file
            .xopp_root
            .pages
            .iter()
            .map(|page| (page.width, page.height))
            .fold((0_f64, 0_f64), |prev, next| {
                (prev.0.max(next.0), prev.1 + next.1)
            });
        let no_pages = xopp_file.xopp_root.pages.len() as u32;

        let sheet = Self::default();
        let format = Format::default();
        let mut background = Background::default();

        sheet.set_width(sheet_width.round() as i32);
        sheet.set_height(sheet_height.round() as i32);

        format.set_width(sheet_width.round() as i32);
        format.set_height((sheet_height / f64::from(no_pages)).round() as i32);

        if let Some(first_page) = xopp_file.xopp_root.pages.get(0) {
            if let xoppformat::XoppBackgroundType::Solid {
                color: _color,
                style: _style,
            } = &first_page.background.bg_type
            {
                // Background styles would not align with Rnotes background patterns, so everything is plain
                background.set_pattern(background::PatternStyle::None);
            }
        }

        // Offsetting as rnote has one global coordinate space
        let mut y_offset = 0.0;

        for (_page_i, page) in xopp_file.xopp_root.pages.into_iter().enumerate() {
            for layers in page.layers.into_iter() {
                // import strokes
                for stroke in layers.strokes.into_iter() {
                    let mut brush = Brush::default();
                    brush.set_color(compose::Color::from(stroke.color));

                    let mut width_iter = stroke.width.iter();
                    // The first element is the absolute width, every following is the relative width (between 0.0 and 1.0)
                    if let Some(&width) = width_iter.next() {
                        brush.set_width(width);
                    }

                    let elements = stroke
                        .coords
                        .into_iter()
                        .map(|mut coords| {
                            coords[1] += y_offset;
                            // Defaulting to PRESSURE_DEFAULT if width iterator is shorter than the coords vec
                            let pressure =
                                width_iter.next().unwrap_or(&InputData::PRESSURE_DEFAULT);

                            Element::new(InputData::new(coords, *pressure))
                        })
                        .collect::<Vec<Element>>();

                    if let Some(new_stroke) = BrushStroke::new_w_elements(&elements, brush) {
                        sheet
                            .strokes_state()
                            .borrow_mut()
                            .insert_stroke_threaded(StrokeStyle::BrushStroke(new_stroke));
                    }
                }

                // import images
                for image in layers.images.into_iter() {
                    let bounds = AABB::new(
                        na::point![image.left, image.top],
                        na::point![image.right, image.bottom],
                    );

                    let intrinsic_size =
                        bitmapimage::extract_dimensions(&base64::decode(&image.data)?)?;

                    let rectangle = shapes::Rectangle {
                        cuboid: p2d::shape::Cuboid::new(bounds.half_extents()),
                        transform: strokebehaviour::StrokeTransform::new_w_isometry(
                            na::Isometry2::new(bounds.center().coords, 0.0),
                        ),
                    };

                    let mut bitmapimage = BitmapImage {
                        data_base64: image.data,
                        // Xopp images are always Png
                        format: bitmapimage::Format::Png,
                        intrinsic_size,
                        rectangle,
                        ..BitmapImage::default()
                    };
                    bitmapimage.update_geometry();

                    sheet
                        .strokes_state()
                        .borrow_mut()
                        .insert_stroke_threaded(StrokeStyle::BitmapImage(bitmapimage));
                }
            }

            y_offset += page.height;
        }

        *sheet.background().borrow_mut() = background;
        sheet.format().import_format(&format);

        self.import_sheet(&sheet);

        Ok(())
    }

    pub fn save_sheet_as_rnote_bytes(&self, filename: &str) -> Result<Vec<u8>, anyhow::Error> {
        let json_output = serde_json::to_string(self)?;

        let compressed_bytes = utils::compress_to_gzip(json_output.as_bytes(), filename)?;

        Ok(compressed_bytes)
    }

    pub fn export_sheet_as_xopp_bytes(&self, filename: &str) -> Result<Vec<u8>, anyhow::Error> {
        let current_dpi = self.format().dpi();

        // Only one background for all pages
        let background = xoppformat::XoppBackground {
            bg_type: xoppformat::XoppBackgroundType::Solid {
                color: self.background().borrow().color().into(),
                style: xoppformat::XoppBackgroundSolidStyle::Plain,
            },
        };

        // xopp spec needs at least one page in vec, but its fine since pages_bounds() always produces at least one
        let pages = self
            .gen_pages_bounds()
            .iter()
            .map(|&page_bounds| {
                let page_keys = self
                    .strokes_state()
                    .borrow()
                    .stroke_keys_intersect_bounds(page_bounds);

                let strokes = self
                    .strokes_state()
                    .borrow()
                    .clone_strokes_for_keys(&page_keys);

                // Translate strokes to to page mins and convert to XoppStrokStyle
                let xopp_strokestyles = strokes
                    .into_iter()
                    .filter_map(|mut stroke| {
                        stroke.translate(-page_bounds.mins.coords);
                        stroke.to_xopp(
                            current_dpi,
                            &self.strokes_state().borrow().renderer.read().unwrap(),
                        )
                    })
                    .collect::<Vec<xoppformat::XoppStrokeStyle>>();

                // Extract the strokes
                let xopp_strokes = xopp_strokestyles
                    .iter()
                    .filter_map(|stroke| {
                        if let xoppformat::XoppStrokeStyle::XoppStroke(xoppstroke) = stroke {
                            Some(xoppstroke.clone())
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<xoppformat::XoppStroke>>();

                // Extract the texts
                let xopp_texts = xopp_strokestyles
                    .iter()
                    .filter_map(|stroke| {
                        if let xoppformat::XoppStrokeStyle::XoppText(xopptext) = stroke {
                            Some(xopptext.clone())
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<xoppformat::XoppText>>();

                // Extract the images
                let xopp_images = xopp_strokestyles
                    .iter()
                    .filter_map(|stroke| {
                        if let xoppformat::XoppStrokeStyle::XoppImage(xoppstroke) = stroke {
                            Some(xoppstroke.clone())
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<xoppformat::XoppImage>>();

                let layer = xoppformat::XoppLayer {
                    strokes: xopp_strokes,
                    texts: xopp_texts,
                    images: xopp_images,
                };

                let page_dimensions = utils::convert_coord_dpi(
                    page_bounds.extents(),
                    current_dpi,
                    xoppformat::XoppFile::DPI,
                );

                xoppformat::XoppPage {
                    width: page_dimensions[0],
                    height: page_dimensions[1],
                    background: background.clone(),
                    layers: vec![layer],
                }
            })
            .collect::<Vec<xoppformat::XoppPage>>();

        let title = String::from("Xournal++ document - see https://github.com/xournalpp/xournalpp (exported from Rnote - see https://github.com/flxzt/rnote)");

        let xopp_root = xoppformat::XoppRoot {
            title,
            fileversion: String::from("4"),
            preview: String::from(""),
            pages,
        };
        let xopp_file = xoppformat::XoppFile { xopp_root };

        let xoppfile_bytes = xopp_file.save_as_bytes(filename)?;

        Ok(xoppfile_bytes)
    }

    /// Generates all containing svgs for the sheet without root or xml header.
    pub fn gen_svgs(&self) -> Result<Vec<render::Svg>, anyhow::Error> {
        let sheet_bounds = self.bounds();
        let mut svgs = vec![];

        svgs.push(
            self.background()
                .borrow()
                .gen_svg(sheet_bounds.loosened(1.0))?,
        );

        svgs.append(&mut self.strokes_state().borrow().gen_svgs_for_strokes()?);

        Ok(svgs)
    }

    pub fn export_sheet_as_svg(&self, file: &gio::File) -> Result<(), anyhow::Error> {
        let sheet_bounds = self.bounds();
        let svgs = self.gen_svgs()?;

        let mut svg_data = svgs
            .iter()
            .map(|svg| svg.svg_data.as_str())
            .collect::<Vec<&str>>()
            .join("\n");

        svg_data = compose::wrap_svg_root(
            svg_data.as_str(),
            Some(sheet_bounds),
            Some(sheet_bounds),
            true,
        );

        file.replace_async(
            None,
            false,
            gio::FileCreateFlags::REPLACE_DESTINATION,
            glib::PRIORITY_HIGH_IDLE,
            None::<&gio::Cancellable>,
            move |result| {
                let output_stream = match result {
                    Ok(output_stream) => output_stream,
                    Err(e) => {
                        log::error!(
                            "replace_async() failed in export_sheet_as_svg() with Err {}",
                            e
                        );
                        return;
                    }
                };

                if let Err(e) = output_stream.write(svg_data.as_bytes(), None::<&gio::Cancellable>)
                {
                    log::error!(
                        "output_stream().write() failed in export_sheet_as_svg() with Err {}",
                        e
                    );
                };
                if let Err(e) = output_stream.close(None::<&gio::Cancellable>) {
                    log::error!(
                        "output_stream().close() failed in export_sheet_as_svg() with Err {}",
                        e
                    );
                };
            },
        );

        Ok(())
    }
}
