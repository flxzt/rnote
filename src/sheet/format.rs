mod imp {
    use std::cell::Cell;

    use gtk4::{glib, prelude::*, subclass::prelude::*};
    use once_cell::sync::Lazy;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(default)]
    pub struct Format {
        width: Cell<i32>,
        height: Cell<i32>,
        dpi: Cell<f64>,
        orientation: Cell<super::Orientation>,
    }

    impl Default for Format {
        fn default() -> Self {
            Self {
                width: Cell::new(super::Format::WIDTH_DEFAULT),
                height: Cell::new(super::Format::HEIGHT_DEFAULT),
                dpi: Cell::new(super::Format::DPI_DEFAULT),
                orientation: Cell::new(super::Orientation::Portrait),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Format {
        const NAME: &'static str = "Format";
        type Type = super::Format;
        type ParentType = glib::Object;
    }

    impl ObjectImpl for Format {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpec::new_int(
                        // Name
                        "width",
                        // Nickname
                        "width",
                        // Short description
                        "width",
                        // Minimum
                        super::Format::WIDTH_MIN,
                        // Maximum
                        super::Format::WIDTH_MAX,
                        // Default value
                        super::Format::WIDTH_DEFAULT,
                        // The property can be read and written to
                        glib::ParamFlags::READWRITE,
                    ),
                    glib::ParamSpec::new_int(
                        // Name
                        "height",
                        // Nickname
                        "height",
                        // Short description
                        "height",
                        // Minimum
                        super::Format::HEIGHT_MIN,
                        // Maximum
                        super::Format::HEIGHT_MAX,
                        // Default value
                        super::Format::HEIGHT_DEFAULT,
                        // The property can be read and written to
                        glib::ParamFlags::READWRITE,
                    ),
                    glib::ParamSpec::new_double(
                        // Name
                        "dpi",
                        // Nickname
                        "dpi",
                        // Short description
                        "dpi",
                        // Minimum
                        super::Format::DPI_MIN,
                        // Maximum
                        super::Format::DPI_MAX,
                        // Default value
                        super::Format::DPI_DEFAULT,
                        // The property can be read and written to
                        glib::ParamFlags::READWRITE,
                    ),
                    glib::ParamSpec::new_enum(
                        // Name
                        "orientation",
                        // Nickname
                        "orientation",
                        // Short description
                        "orientation",
                        // Type
                        super::Orientation::static_type(),
                        // Default value
                        super::Orientation::Portrait as i32,
                        // The property can be read and written to
                        glib::ParamFlags::READWRITE,
                    ),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "width" => self.width.get().to_value(),
                "height" => self.height.get().to_value(),
                "dpi" => self.dpi.get().to_value(),
                "orientation" => self.orientation.get().to_value(),
                _ => unimplemented!(),
            }
        }

        fn set_property(
            &self,
            _obj: &Self::Type,
            _id: usize,
            value: &glib::Value,
            pspec: &glib::ParamSpec,
        ) {
            match pspec.name() {
                "width" => {
                    let width: i32 = value
                        .get::<i32>()
                        .expect("The value needs to be of type `i32`.");
                    self.width.replace(width);
                }
                "height" => {
                    let height: i32 = value
                        .get::<i32>()
                        .expect("The value needs to be of type `i32`.");
                    self.height.replace(height);
                }
                "dpi" => {
                    let dpi: f64 = value
                        .get::<f64>()
                        .expect("The value needs to be of type `f64`.");
                    self.dpi.replace(dpi);
                }
                "orientation" => {
                    let orientation: super::Orientation = value
                        .get::<super::Orientation>()
                        .expect("The value needs to be of type `Orientation`.");
                    self.orientation.replace(orientation);
                }
                _ => unimplemented!(),
            }
        }
    }
}

use gtk4::{gdk, glib, glib::clone, graphene, gsk, prelude::*, Snapshot};
use serde::de::{self, Deserializer, Visitor};
use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize};

use crate::geometry;
use crate::ui::appwindow::RnoteAppWindow;

glib::wrapper! {
    pub struct Format(ObjectSubclass<imp::Format>);
}

impl Serialize for Format {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("Format", 4)?;
        state.serialize_field("width", &self.width())?;
        state.serialize_field("height", &self.height())?;
        state.serialize_field("dpi", &self.dpi())?;
        state.serialize_field("orientation", &self.orientation())?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for Format {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "lowercase")]
        #[allow(non_camel_case_types)]
        enum Field {
            width,
            height,
            dpi,
            orientation,
            unknown,
        }

        struct FormatVisitor;
        impl<'de> Visitor<'de> for FormatVisitor {
            type Value = Format;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct Format")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: de::SeqAccess<'de>,
            {
                let width = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let height = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                let dpi = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(2, &self))?;
                let orientation = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(3, &self))?;

                let format = Format::new();
                format.set_width(width);
                format.set_height(height);
                format.set_dpi(dpi);
                format.set_orientation(orientation);

                Ok(format)
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: de::MapAccess<'de>,
            {
                let mut width = None;
                let mut height = None;
                let mut dpi = None;
                let mut orientation = None;

                while let Some(key) = match map.next_key() {
                    Ok(key) => key,
                    Err(e) => {
                        log::warn!("{}", e);
                        Some(Field::unknown)
                    }
                } {
                    match key {
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
                        Field::dpi => {
                            if dpi.is_some() {
                                return Err(de::Error::duplicate_field("dpi"));
                            }
                            dpi = Some(map.next_value()?);
                        }
                        Field::orientation => {
                            if orientation.is_some() {
                                return Err(de::Error::duplicate_field("orientation"));
                            }
                            orientation = Some(map.next_value()?);
                        }
                        Field::unknown => {
                            // throw away the value
                            map.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                let format_default = Format::default();

                let width = width.unwrap_or_else(|| {
                    let err: A::Error = de::Error::missing_field("width");
                    log::error!("{}", err);
                    format_default.width()
                });
                let height = height.unwrap_or_else(|| {
                    let err: A::Error = de::Error::missing_field("height");
                    log::error!("{}", err);
                    format_default.height()
                });
                let dpi = dpi.unwrap_or_else(|| {
                    let err: A::Error = de::Error::missing_field("dpi");
                    log::error!("{}", err);
                    format_default.dpi()
                });
                let orientation = orientation.unwrap_or_else(|| {
                    let err: A::Error = de::Error::missing_field("orientation");
                    log::error!("{}", err);
                    format_default.orientation()
                });

                let format = Format::new();
                format.set_width(width);
                format.set_height(height);
                format.set_dpi(dpi);
                format.set_orientation(orientation);

                Ok(format)
            }
        }

        const FIELDS: &[&str] = &["width", "height", "dpi", "orientation"];
        deserializer.deserialize_struct("Format", FIELDS, FormatVisitor)
    }
}

impl Default for Format {
    fn default() -> Self {
        Self::new()
    }
}

impl Format {
    pub const WIDTH_MIN: i32 = 0;
    pub const WIDTH_MAX: i32 = 30000;
    pub const WIDTH_DEFAULT: i32 = 1240;

    pub const HEIGHT_MIN: i32 = 0;
    pub const HEIGHT_MAX: i32 = 30000;
    pub const HEIGHT_DEFAULT: i32 = 1754;

    pub const DPI_MIN: f64 = 1.0;
    pub const DPI_MAX: f64 = 5000.0;
    pub const DPI_DEFAULT: f64 = 96.0;

    pub const FORMAT_BORDER_COLOR: gdk::RGBA = gdk::RGBA {
        red: 0.6,
        green: 0.0,
        blue: 0.0,
        alpha: 1.0,
    };

    pub fn new() -> Self {
        glib::Object::new(&[
            ("width", &Self::WIDTH_DEFAULT),
            ("height", &Self::HEIGHT_DEFAULT),
            ("dpi", &Self::DPI_DEFAULT),
            ("orientation", &Orientation::Portrait),
        ])
        .expect("Failed to create Format")
    }

    pub fn width(&self) -> i32 {
        self.property("width").unwrap().get::<i32>().unwrap()
    }

    pub fn set_width(&self, width: i32) {
        self.set_property("width", width.to_value()).unwrap();
    }

    pub fn height(&self) -> i32 {
        self.property("height").unwrap().get::<i32>().unwrap()
    }

    pub fn set_height(&self, height: i32) {
        self.set_property("height", height.to_value()).unwrap();
    }

    pub fn dpi(&self) -> f64 {
        self.property("dpi").unwrap().get::<f64>().unwrap()
    }

    pub fn set_dpi(&self, dpi: f64) {
        self.set_property("dpi", dpi.to_value()).unwrap();
    }

    /// Width and height are independent of the orientation and should be updated when the orientation changes. Its use is mainly for printing and selecting predefined formats
    pub fn orientation(&self) -> Orientation {
        self.property("orientation")
            .unwrap()
            .get::<Orientation>()
            .unwrap()
    }

    /// Width and height are independent of the orientation and should be updated when the orientation changes. Its use is mainly for printing and selecting predefined formats
    pub fn set_orientation(&self, orientation: Orientation) {
        self.set_property("orientation", orientation.to_value())
            .unwrap();
    }

    pub fn import_format(&self, format: Self) {
        self.set_width(format.width());
        self.set_height(format.height());
        self.set_dpi(format.dpi());
        self.set_orientation(format.orientation());
    }

    pub fn draw(
        &self,
        sheet_bounds: p2d::bounding_volume::AABB,
        snapshot: &Snapshot,
        scalefactor: f64,
    ) {
        let border_radius = graphene::Size::new(0.0, 0.0);
        let border_width = 2.0;

        let mut offset_y = sheet_bounds.mins[1];

        snapshot.push_clip(&geometry::aabb_to_graphene_rect(geometry::aabb_scale(
            sheet_bounds,
            scalefactor,
        )));

        while offset_y < sheet_bounds.maxs[1] {
            let border_bounds = graphene::Rect::new(
                (sheet_bounds.mins[0] * scalefactor) as f32,
                (offset_y * scalefactor) as f32 - border_width / 2.0,
                (f64::from(self.width()) * scalefactor) as f32,
                ((offset_y + f64::from(self.height())) * scalefactor) as f32 + border_width / 2.0,
            );

            let rounded_rect = gsk::RoundedRect::new(
                border_bounds.clone(),
                border_radius.clone(),
                border_radius.clone(),
                border_radius.clone(),
                border_radius.clone(),
            );
            snapshot.append_border(
                &rounded_rect,
                &[border_width, border_width, border_width, border_width],
                &[
                    Self::FORMAT_BORDER_COLOR,
                    Self::FORMAT_BORDER_COLOR,
                    Self::FORMAT_BORDER_COLOR,
                    Self::FORMAT_BORDER_COLOR,
                ],
            );
            offset_y += f64::from(self.height());
        }

        snapshot.pop();
    }

    pub fn init(&self, appwindow: &RnoteAppWindow) {
        self.connect_notify_local(Some("dpi"), clone!(@weak appwindow => move |format, _pspec| {
            appwindow.settings_panel().background_pattern_width_unitentry().set_dpi(format.dpi());
            appwindow.settings_panel().background_pattern_height_unitentry().set_dpi(format.dpi());
        }));
    }
}

#[derive(
    Debug,
    Eq,
    PartialEq,
    Clone,
    Copy,
    glib::GEnum,
    Serialize,
    Deserialize,
    num_derive::FromPrimitive,
)]
#[repr(u32)]
#[genum(type_name = "PredefinedFormats")]
pub enum PredefinedFormat {
    #[genum(name = "A6", nick = "a6")]
    A6 = 0,
    #[genum(name = "A5", nick = "a5")]
    A5,
    #[genum(name = "A4", nick = "a4")]
    A4,
    #[genum(name = "A3", nick = "a3")]
    A3,
    #[genum(name = "A2", nick = "a2")]
    A2,
    #[genum(name = "US Letter", nick = "us-letter")]
    UsLetter,
    #[genum(name = "US Legal", nick = "us-legal")]
    UsLegal,
    #[genum(name = "Custom", nick = "custom")]
    Custom,
}

impl Default for PredefinedFormat {
    fn default() -> Self {
        Self::A4
    }
}

#[derive(
    Debug,
    Eq,
    PartialEq,
    Clone,
    Copy,
    glib::GEnum,
    Serialize,
    Deserialize,
    num_derive::FromPrimitive,
)]
#[repr(u32)]
#[genum(type_name = "MeasureUnits")]
pub enum MeasureUnit {
    #[genum(name = "Pixel", nick = "px")]
    Px = 0,
    #[genum(name = "Millimeter", nick = "mm")]
    Mm,
    #[genum(name = "Centimeter", nick = "cm")]
    Cm,
}

impl Default for MeasureUnit {
    fn default() -> Self {
        Self::Px
    }
}

impl MeasureUnit {
    pub const AMOUNT_MM_IN_INCH: f64 = 25.4;

    pub fn convert_measurement(
        value: f64,
        value_unit: MeasureUnit,
        value_dpi: f64,
        desired_unit: MeasureUnit,
        desired_dpi: f64,
    ) -> f64 {
        let value_in_px = match value_unit {
            MeasureUnit::Px => value,
            MeasureUnit::Mm => (value / Self::AMOUNT_MM_IN_INCH) * value_dpi,
            MeasureUnit::Cm => ((value * 10.0) / Self::AMOUNT_MM_IN_INCH) * value_dpi,
        };

        match desired_unit {
            MeasureUnit::Px => value_in_px,
            MeasureUnit::Mm => (value_in_px / desired_dpi) * Self::AMOUNT_MM_IN_INCH,
            MeasureUnit::Cm => (value_in_px / desired_dpi) * Self::AMOUNT_MM_IN_INCH * 10.0,
        }
    }
}

#[derive(
    Debug,
    Eq,
    PartialEq,
    Clone,
    Copy,
    glib::GEnum,
    Serialize,
    Deserialize,
    num_derive::FromPrimitive,
)]
#[repr(u32)]
#[genum(type_name = "FormatOrientation")]
pub enum Orientation {
    #[genum(name = "Portrait", nick = "portrait")]
    Portrait = 0,
    #[genum(name = "Landscape", nick = "landscape")]
    Landscape,
}

impl Default for Orientation {
    fn default() -> Self {
        Self::Portrait
    }
}
