// Imports
use rnote_compose::builders::ShapeBuilderType;
use rnote_engine::pens::pensconfig::brushconfig::BrushStyle;
use rnote_engine::pens::pensconfig::eraserconfig::EraserStyle;
use rnote_engine::pens::pensconfig::selectorconfig::SelectorStyle;
use rnote_engine::pens::pensconfig::toolsconfig::ToolStyle;

pub(crate) const APP_LICENSE: gtk4::License = gtk4::License::Gpl30;

/// Step size when adjusting the stroke width of the active pen with the keyboard.
pub(crate) const STROKE_WIDTH_STEP: f64 = 1.0;

/// Sub-styles of the brush pen, in the order they are reachable through the `sub-style` action.
pub(crate) const SUB_STYLES_BRUSH: &[BrushStyle] =
    &[BrushStyle::Marker, BrushStyle::Solid, BrushStyle::Textured];
/// Sub-styles of the shaper pen, in the order they are reachable through the `sub-style` action.
/// Only the most frequently used shapes are listed, the remaining ones stay sidebar-only.
pub(crate) const SUB_STYLES_SHAPER: &[ShapeBuilderType] = &[
    ShapeBuilderType::Line,
    ShapeBuilderType::Arrow,
    ShapeBuilderType::Rectangle,
    ShapeBuilderType::Ellipse,
    ShapeBuilderType::Polyline,
    ShapeBuilderType::Polygon,
    ShapeBuilderType::Grid,
    ShapeBuilderType::QuadBez,
    ShapeBuilderType::CubBez,
];
/// Sub-styles of the eraser pen, in the order they are reachable through the `sub-style` action.
pub(crate) const SUB_STYLES_ERASER: &[EraserStyle] = &[
    EraserStyle::TrashCollidingStrokes,
    EraserStyle::SplitCollidingStrokes,
];
/// Sub-styles of the selector pen, in the order they are reachable through the `sub-style` action.
pub(crate) const SUB_STYLES_SELECTOR: &[SelectorStyle] = &[
    SelectorStyle::Polygon,
    SelectorStyle::Rectangle,
    SelectorStyle::Single,
    SelectorStyle::IntersectingPath,
];
/// Sub-styles of the tools pen, in the order they are reachable through the `sub-style` action.
pub(crate) const SUB_STYLES_TOOLS: &[ToolStyle] = &[
    ToolStyle::VerticalSpace,
    ToolStyle::OffsetCamera,
    ToolStyle::Zoom,
    ToolStyle::Laser,
];

/// The accelerator label shown to the user for the n-th (0-based) sub-style of a pen.
pub(crate) fn sub_style_accel_label(index: usize) -> String {
    format!("Alt+{}", index + 1)
}
