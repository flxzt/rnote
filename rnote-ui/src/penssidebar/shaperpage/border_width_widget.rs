use rnote_engine::pens::{shaper::ShaperStyle, Shaper};

use crate::RnoteAppWindow;
use gtk4::{glib, glib::clone};

use super::ShaperPage;

pub fn setup(shaperpage: &ShaperPage, appwindow: &RnoteAppWindow) {
    let spinbutton = shaperpage.width_spinbutton();

    spinbutton.set_increments(0.1, 2.0);
    spinbutton.set_range(Shaper::STROKE_WIDTH_MIN, Shaper::STROKE_WIDTH_MAX);
    // Must be set after set_range()
    spinbutton.set_value(Shaper::STROKE_WIDTH_DEFAULT);

    spinbutton.connect_value_changed(
        clone!(@weak appwindow => move |width_spinbutton| {
            let shaper_style = appwindow.canvas().engine().borrow_mut().penholder.shaper.style;

            match shaper_style {
                ShaperStyle::Smooth => appwindow.canvas().engine().borrow_mut().penholder.shaper.smooth_options.stroke_width = width_spinbutton.value(),
                ShaperStyle::Rough => appwindow.canvas().engine().borrow_mut().penholder.shaper.rough_options.stroke_width = width_spinbutton.value(),
            }

            if let Err(e) = appwindow.save_engine_config() {
                log::error!("saving engine config failed after changing shape width, Err `{}`", e);
            }
        }),
    );
}
