// Imports
use rnote_compose::eventresult::EventPropagation;

/// Extension trait for [gdk::RGBA].
#[cfg(feature = "ui")]
pub trait GdkRGBAExt
where
    Self: Sized,
{
    fn from_compose_color(color: rnote_compose::Color) -> Self;
    fn into_compose_color(self) -> rnote_compose::Color;
    fn from_piet_color(color: piet::Color) -> Self;
    fn into_piet_color(self) -> piet::Color;
}

#[cfg(feature = "ui")]
impl GdkRGBAExt for gtk4::gdk::RGBA {
    fn from_compose_color(color: rnote_compose::Color) -> Self {
        gtk4::gdk::RGBA::new(
            color.r as f32,
            color.g as f32,
            color.b as f32,
            color.a as f32,
        )
    }
    fn into_compose_color(self) -> rnote_compose::Color {
        rnote_compose::Color {
            r: f64::from(self.red()),
            g: f64::from(self.green()),
            b: f64::from(self.blue()),
            a: f64::from(self.alpha()),
        }
    }

    fn from_piet_color(color: piet::Color) -> Self {
        let (r, g, b, a) = color.as_rgba();
        gtk4::gdk::RGBA::new(r as f32, g as f32, b as f32, a as f32)
    }

    fn into_piet_color(self) -> piet::Color {
        piet::Color::rgba(
            f64::from(self.red()),
            f64::from(self.green()),
            f64::from(self.blue()),
            f64::from(self.alpha()),
        )
    }
}

/// Extension trait for [graphene::Rect].
#[cfg(feature = "ui")]
pub trait GrapheneRectExt
where
    Self: Sized,
{
    fn from_p2d_aabb(aabb: p2d::bounding_volume::Aabb) -> Self;
}

#[cfg(feature = "ui")]
impl GrapheneRectExt for gtk4::graphene::Rect {
    fn from_p2d_aabb(aabb: p2d::bounding_volume::Aabb) -> Self {
        gtk4::graphene::Rect::new(
            aabb.mins[0] as f32,
            aabb.mins[1] as f32,
            (aabb.extents()[0]) as f32,
            (aabb.extents()[1]) as f32,
        )
    }
}

pub trait EventPropagationExt {
    fn into_glib(self) -> glib::Propagation;
    fn from_glib(value: glib::Propagation) -> Self;
}

impl EventPropagationExt for EventPropagation {
    fn into_glib(self) -> glib::Propagation {
        match self {
            EventPropagation::Proceed => glib::Propagation::Proceed,
            EventPropagation::Stop => glib::Propagation::Stop,
        }
    }

    fn from_glib(value: glib::Propagation) -> Self {
        match value {
            glib::Propagation::Stop => Self::Stop,
            glib::Propagation::Proceed => Self::Proceed,
        }
    }
}
