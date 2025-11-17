// Modules
mod arrowbuilder;
/// Buildable trait.
pub mod buildable;
mod coordsystem2dbuilder;
mod coordsystem3dbuilder;
mod cubbezbuilder;
mod ellipsebuilder;
mod fociellipsebuilder;
mod gridbuilder;
mod linebuilder;
mod penpathcurvedbuilder;
mod penpathmodeledbuilder;
mod penpathsimplebuilder;
mod polygonbuilder;
mod polylinebuilder;
mod quadbezbuilder;
mod quadrantcoordsystem2dbuilder;
mod rectanglebuilder;

// Re-exports
pub use arrowbuilder::ArrowBuilder;
pub use coordsystem2dbuilder::CoordSystem2DBuilder;
pub use coordsystem3dbuilder::CoordSystem3DBuilder;
pub use cubbezbuilder::CubBezBuilder;
pub use ellipsebuilder::EllipseBuilder;
pub use fociellipsebuilder::FociEllipseBuilder;
pub use gridbuilder::GridBuilder;
pub use linebuilder::LineBuilder;
pub use penpathcurvedbuilder::PenPathCurvedBuilder;
pub use penpathmodeledbuilder::PenPathModeledBuilder;
pub use penpathsimplebuilder::PenPathSimpleBuilder;
pub use polygonbuilder::PolygonBuilder;
pub use polylinebuilder::PolylineBuilder;
pub use quadbezbuilder::QuadBezBuilder;
pub use quadrantcoordsystem2dbuilder::QuadrantCoordSystem2DBuilder;
pub use rectanglebuilder::RectangleBuilder;

// Imports
use anyhow::Context;
use serde::{Deserialize, Serialize};

#[derive(
    Copy,
    Clone,
    Debug,
    Default,
    Serialize,
    Deserialize,
    num_derive::FromPrimitive,
    num_derive::ToPrimitive,
)]
#[serde(rename = "shapebuilder_type")]
/// A choice for a shape builder type
pub enum ShapeBuilderType {
    /// A line builder
    #[serde(rename = "line")]
    #[default]
    Line = 0,
    /// An arrow builder
    #[serde(rename = "arrow")]
    Arrow,
    /// A rectangle builder
    #[serde(rename = "rectangle")]
    Rectangle,
    /// A grid
    #[serde(rename = "grid")]
    Grid,
    /// A 2D coordinate system builder
    #[serde(rename = "coord_system_2d")]
    CoordSystem2D,
    /// A 3D coordinate system builder
    #[serde(rename = "coord_system_3d")]
    CoordSystem3D,
    /// A 2D single quadrant coordinate system builder
    #[serde(rename = "quadrant_coord_system_2d")]
    QuadrantCoordSystem2D,
    /// An ellipse builder
    #[serde(rename = "ellipse")]
    Ellipse,
    /// A foci ellipse builder
    #[serde(rename = "foci_ellipse")]
    FociEllipse,
    /// A quadbez builder
    #[serde(rename = "quadbez")]
    QuadBez,
    /// A cubic bezier builder
    #[serde(rename = "cubbez")]
    CubBez,
    /// A poyline builder
    #[serde(rename = "polyline")]
    Polyline,
    /// A polygon builder
    #[serde(rename = "polygon")]
    Polygon,
}

impl ShapeBuilderType {
    /// Converts an icon name into the represented shape builder type. Returns None for invalid strings.
    pub fn from_icon_name(icon_name: &str) -> Option<Self> {
        match icon_name {
            "shapebuilder-arrow-symbolic" => Some(Self::Arrow),
            "shapebuilder-line-symbolic" => Some(Self::Line),
            "shapebuilder-rectangle-symbolic" => Some(Self::Rectangle),
            "shapebuilder-grid-symbolic" => Some(Self::Grid),
            "shapebuilder-coordsystem2d-symbolic" => Some(Self::CoordSystem2D),
            "shapebuilder-coordsystem3d-symbolic" => Some(Self::CoordSystem3D),
            "shapebuilder-quadrantcoordsystem2d-symbolic" => Some(Self::QuadrantCoordSystem2D),
            "shapebuilder-ellipse-symbolic" => Some(Self::Ellipse),
            "shapebuilder-fociellipse-symbolic" => Some(Self::FociEllipse),
            "shapebuilder-quadbez-symbolic" => Some(Self::QuadBez),
            "shapebuilder-cubbez-symbolic" => Some(Self::CubBez),
            "shapebuilder-polyline-symbolic" => Some(Self::Polyline),
            "shapebuilder-polygon-symbolic" => Some(Self::Polygon),
            _ => None,
        }
    }

    /// Converts a shape builder type into the icon name that represents it.
    pub fn to_icon_name(self) -> String {
        match self {
            Self::Arrow => String::from("shapebuilder-arrow-symbolic"),
            Self::Line => String::from("shapebuilder-line-symbolic"),
            Self::Rectangle => String::from("shapebuilder-rectangle-symbolic"),
            Self::Grid => String::from("shapebuilder-grid-symbolic"),
            Self::CoordSystem2D => String::from("shapebuilder-coordsystem2d-symbolic"),
            Self::CoordSystem3D => String::from("shapebuilder-coordsystem3d-symbolic"),
            Self::QuadrantCoordSystem2D => {
                String::from("shapebuilder-quadrantcoordsystem2d-symbolic")
            }
            Self::Ellipse => String::from("shapebuilder-ellipse-symbolic"),
            Self::FociEllipse => String::from("shapebuilder-fociellipse-symbolic"),
            Self::QuadBez => String::from("shapebuilder-quadbez-symbolic"),
            Self::CubBez => String::from("shapebuilder-cubbez-symbolic"),
            Self::Polyline => String::from("shapebuilder-polyline-symbolic"),
            Self::Polygon => String::from("shapebuilder-polygon-symbolic"),
        }
    }
}

impl TryFrom<u32> for ShapeBuilderType {
    type Error = anyhow::Error;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        num_traits::FromPrimitive::from_u32(value)
            .with_context(|| format!("ShapeBuilderType try_from::<u32>() for value {value} failed"))
    }
}

#[derive(
    Copy,
    Clone,
    Debug,
    Default,
    Serialize,
    Deserialize,
    num_derive::FromPrimitive,
    num_derive::ToPrimitive,
)]
#[serde(rename = "penpathbuilder_type")]
/// A choice for a pen path builder type
pub enum PenPathBuilderType {
    #[serde(rename = "simple")]
    /// the simple pen path builder
    Simple = 0,
    #[serde(rename = "curved")]
    /// the curved pen path builder
    Curved,
    #[serde(rename = "modeled")]
    /// the modeled pen path builder
    #[default]
    Modeled,
}

impl TryFrom<u32> for PenPathBuilderType {
    type Error = anyhow::Error;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        num_traits::FromPrimitive::from_u32(value).with_context(|| {
            format!("PenPathBuilderType try_from::<u32>() for value {value} failed",)
        })
    }
}
