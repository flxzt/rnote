use crate::RnoteAppWindow;
use cairo::glib::ObjectExt;
use gtk4::{gdk, glib, glib::clone};
use rnote_engine::{pens::shaper::ShaperStyle, utils::GdkRGBAHelpers};

use super::ShaperPage;

pub fn setup(shaperpage: &ShaperPage, appwindow: &RnoteAppWindow) {
    shaperpage.fill_colorpicker().connect_notify_local(
        Some("current-color"),
        clone!(@weak shaperpage ,@weak appwindow => move |fill_colorpicker, _paramspec| {
            let color = fill_colorpicker.property::<gdk::RGBA>("current-color").into_compose_color();
            let shaper_style = appwindow.canvas().engine().borrow_mut().penholder.shaper.style;

            match shaper_style {
                ShaperStyle::Smooth => appwindow.canvas().engine().borrow_mut().penholder.shaper.smooth_options.fill_color = Some(color),
                ShaperStyle::Rough => appwindow.canvas().engine().borrow_mut().penholder.shaper.rough_options.fill_color= Some(color),
            }

            if let Err(e) = appwindow.save_engine_config() {
                log::error!("saving engine config failed after changing shaper fill color, Err `{}`", e);
            }
        }),
    );
}
