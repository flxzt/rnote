// Imports
use gtk4::glib;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, glib::Enum)]
#[enum_type(name = "RnStrokeWidthPreviewType")]
#[repr(i32)]
pub(crate) enum StrokeWidthPreviewStyle {
    #[default]
    #[enum_value(name = "Circle")]
    Circle,
    #[enum_value(name = "RoundedRect")]
    RoundedRect,
}
