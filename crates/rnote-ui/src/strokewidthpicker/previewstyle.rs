// Imports
use gtk4::glib;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, glib::Enum)]
#[enum_type(name = "RnStrokeWidthPreviewType")]
#[repr(i32)]
pub(crate) enum StrokeWidthPreviewStyle {
    #[enum_value(name = "Circle")]
    #[default]
    Circle,
    #[enum_value(name = "RoundedRect")]
    RoundedRect,
}
