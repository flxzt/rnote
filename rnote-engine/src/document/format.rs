use gtk4::{prelude::*, gdk, glib, graphene, gsk, Snapshot};
use p2d::bounding_volume::{BoundingVolume, AABB};
use piet::RenderContext;
use serde::{Deserialize, Serialize};

use rnote_compose::helpers::AABBHelpers;
use rnote_compose::{Color, color};

use crate::utils::{GdkRGBAHelpers, GrapheneRectHelpers};
use crate::Camera;

#[derive(Debug, Eq, PartialEq, Clone, Copy, glib::Enum, Serialize, Deserialize)]
#[repr(u32)]
#[enum_type(name = "PredefinedFormat")]
#[serde(rename = "predefined_format")]
pub enum PredefinedFormat {
    #[enum_value(name = "A6", nick = "a6")]
    #[serde(rename = "a6")]
    A6 = 0,
    #[enum_value(name = "A5", nick = "a5")]
    #[serde(rename = "a5")]
    A5,
    #[enum_value(name = "A4", nick = "a4")]
    #[serde(rename = "a4")]
    A4,
    #[enum_value(name = "A3", nick = "a3")]
    #[serde(rename = "a3")]
    A3,
    #[enum_value(name = "A2", nick = "a2")]
    #[serde(rename = "a2")]
    A2,
    #[enum_value(name = "US Letter", nick = "us-letter")]
    #[serde(rename = "us_letter")]
    UsLetter,
    #[enum_value(name = "US Legal", nick = "us-legal")]
    #[serde(rename = "us_legal")]
    UsLegal,
    #[enum_value(name = "Custom", nick = "custom")]
    #[serde(rename = "custom")]
    Custom,
}

impl Default for PredefinedFormat {
    fn default() -> Self {
        Self::A3
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Copy, glib::Enum, Serialize, Deserialize)]
#[repr(u32)]
#[enum_type(name = "MeasureUnit")]
#[serde(rename = "measure_unit")]
pub enum MeasureUnit {
    #[enum_value(name = "Pixel", nick = "px")]
    #[serde(rename = "px")]
    Px = 0,
    #[enum_value(name = "Millimeter", nick = "mm")]
    #[serde(rename = "mm")]
    Mm,
    #[enum_value(name = "Centimeter", nick = "cm")]
    #[serde(rename = "cm")]
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

#[derive(Debug, Eq, PartialEq, Clone, Copy, glib::Enum, Serialize, Deserialize)]
#[repr(u32)]
#[enum_type(name = "FormatOrientation")]
#[serde(rename = "orientation")]
pub enum Orientation {
    #[enum_value(name = "Portrait", nick = "portrait")]
    #[serde(rename = "portrait")]
    Portrait = 0,
    #[enum_value(name = "Landscape", nick = "landscape")]
    #[serde(rename = "landscape")]
    Landscape,
}

impl Default for Orientation {
    fn default() -> Self {
        Self::Portrait
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
#[serde(default, rename = "format")]
pub struct Format {
    #[serde(rename = "width")]
    pub width: f64,
    #[serde(rename = "height")]
    pub height: f64,
    #[serde(rename = "dpi")]
    pub dpi: f64,
    #[serde(rename = "orientation")]
    pub orientation: Orientation,
    #[serde(rename = "border_color")]
    pub border_color: Color,
    #[serde(rename = "show_borders")]
    pub show_borders: bool,
}

impl Default for Format {
    fn default() -> Self {
        Self {
            width: Self::WIDTH_DEFAULT,
            height: Self::HEIGHT_DEFAULT,
            dpi: Self::DPI_DEFAULT,
            orientation: Orientation::default(),
            border_color: Color::from(color::GNOME_REDS[3]),
            show_borders: true,
        }
    }
}

impl Format {
    pub const WIDTH_MIN: f64 = 1.0;
    pub const WIDTH_MAX: f64 = 30000.0;
    pub const WIDTH_DEFAULT: f64 = 1123.0;

    pub const HEIGHT_MIN: f64 = 1.0;
    pub const HEIGHT_MAX: f64 = 30000.0;
    pub const HEIGHT_DEFAULT: f64 = 1587.0;

    pub const DPI_MIN: f64 = 1.0;
    pub const DPI_MAX: f64 = 5000.0;
    pub const DPI_DEFAULT: f64 = 96.0;

    fn draw_origin_indicator(camera: &Camera) -> anyhow::Result<gsk::RenderNode> {
        const PATH_COLOR: piet::Color = color::GNOME_GREENS[4];
        let path_width: f64 = 1.0 / camera.total_zoom();

        let indicator_bounds = AABB::from_half_extents(
            na::point![0.0, 0.0],
            na::Vector2::repeat(6.0 / camera.total_zoom()),
        );

        let cairo_node = gsk::CairoNode::new(&graphene::Rect::from_p2d_aabb(indicator_bounds));
        let cairo_cx = cairo_node.draw_context();
        let mut piet_cx = piet_cairo::CairoRenderContext::new(&cairo_cx);

        let mut indicator_path = kurbo::BezPath::new();
        indicator_path.move_to(kurbo::Point::new(
            indicator_bounds.mins[0],
            indicator_bounds.mins[1],
        ));
        indicator_path.line_to(kurbo::Point::new(
            indicator_bounds.maxs[0],
            indicator_bounds.maxs[1],
        ));
        indicator_path.move_to(kurbo::Point::new(
            indicator_bounds.mins[0],
            indicator_bounds.maxs[1],
        ));
        indicator_path.line_to(kurbo::Point::new(
            indicator_bounds.maxs[0],
            indicator_bounds.mins[1],
        ));

        piet_cx.stroke(indicator_path, &PATH_COLOR, path_width);

        piet_cx.finish().map_err(|e| anyhow::anyhow!("{}", e))?;

        Ok(cairo_node.upcast())
    }

    pub fn draw(
        &self,
        snapshot: &Snapshot,
        doc_bounds: AABB,
        camera: &Camera,
    ) -> anyhow::Result<()> {
        if self.show_borders {
            let total_zoom = camera.total_zoom();
            let border_width = 1.0 / total_zoom;
            let viewport = camera.viewport();

            snapshot.push_clip(&graphene::Rect::from_p2d_aabb(doc_bounds.loosened(2.0)));

            for page_bounds in
                doc_bounds.split_extended_origin_aligned(na::vector![self.width, self.height])
            {
                if !page_bounds.intersects(&viewport) {
                    continue;
                }

                let rounded_rect = gsk::RoundedRect::new(
                    graphene::Rect::from_p2d_aabb(page_bounds),
                    graphene::Size::zero(),
                    graphene::Size::zero(),
                    graphene::Size::zero(),
                    graphene::Size::zero(),
                );

                snapshot.append_border(
                    &rounded_rect,
                    &[
                        border_width as f32,
                        border_width as f32,
                        border_width as f32,
                        border_width as f32,
                    ],
                    &[
                        gdk::RGBA::from_compose_color(self.border_color),
                        gdk::RGBA::from_compose_color(self.border_color),
                        gdk::RGBA::from_compose_color(self.border_color),
                        gdk::RGBA::from_compose_color(self.border_color),
                    ],
                )
            }

            snapshot.pop();
        }

        // Draw an indicator at the origin
        snapshot.append_node(&Self::draw_origin_indicator(camera)?);

        Ok(())
    }
}
