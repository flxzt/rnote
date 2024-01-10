// Imports
use gtk4::glib;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, glib::Enum)]
#[enum_type(name = "RnStrokeWidthPreviewType")]
#[repr(i32)]
pub(crate) enum StrokeWidthPreviewStyle {
    #[enum_value(name = "Circle")]
    Circle,
    #[enum_value(name = "RoundedRect")]
    RoundedRect,
}

impl Default for StrokeWidthPreviewStyle {
    fn default() -> Self {
        Self::Circle
    }
}
