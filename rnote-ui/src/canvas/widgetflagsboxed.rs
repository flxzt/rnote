// Imports
use gtk4::glib;
use rnote_engine::WidgetFlags;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, glib::Boxed)]
#[boxed_type(name = "WidgetFlagsBoxed")]
pub(crate) struct WidgetFlagsBoxed(WidgetFlags);

impl From<WidgetFlags> for WidgetFlagsBoxed {
    fn from(value: WidgetFlags) -> Self {
        Self(value)
    }
}

impl From<WidgetFlagsBoxed> for WidgetFlags {
    fn from(value: WidgetFlagsBoxed) -> Self {
        value.0
    }
}

impl WidgetFlagsBoxed {
    pub(crate) fn inner(self) -> WidgetFlags {
        self.0
    }
}
