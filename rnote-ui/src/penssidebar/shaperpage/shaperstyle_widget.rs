use crate::RnoteAppWindow;
use cairo::glib::Cast;
use gtk4::{glib, glib::clone, traits::ListBoxRowExt};
use rnote_engine::pens::{shaper::ShaperStyle, Shaper};

use super::ShaperPage;

pub fn setup(shaperpage: &ShaperPage, appwindow: &RnoteAppWindow) {
    shaperpage.shaperstyle_listbox().connect_row_selected(
            clone!(@weak shaperpage, @weak appwindow => move |_shaperstyle_listbox, selected_row| {
                if let Some(selected_row) = selected_row.map(|selected_row| {selected_row.downcast_ref::<adw::ActionRow>().unwrap()}) {
                    {
                        let engine = appwindow.canvas().engine();
                        let engine = &mut *engine.borrow_mut();

                        engine.penholder.shaper.style = ShaperStyle::try_from(selected_row.index() as u32).unwrap_or_default();
                        engine.penholder.shaper.smooth_options.stroke_width = shaperpage.width_spinbutton().value();
                        engine.penholder.shaper.smooth_options.stroke_color = Some(shaperpage.stroke_colorpicker().current_color());
                        engine.penholder.shaper.smooth_options.fill_color = Some(shaperpage.fill_colorpicker().current_color());
                        engine.penholder.shaper.rough_options.stroke_width = shaperpage.width_spinbutton().value();
                        engine.penholder.shaper.rough_options.stroke_color = Some(shaperpage.stroke_colorpicker().current_color());
                        engine.penholder.shaper.rough_options.fill_color = Some(shaperpage.fill_colorpicker().current_color());
                    }

                    if let Err(e) = appwindow.save_engine_config() {
                        log::error!("saving engine config failed after changing shaper style, Err `{}`", e);
                    }
                    // Need to refresh the whole page, because changing the style affects multiple widgets
                    shaperpage.refresh_ui(&appwindow);
                }
            }),
        );
}

pub fn refresh(shaperpage: &ShaperPage, shaper: &Shaper) {
    shaperpage
        .roughconfig_roughness_spinbutton()
        .set_value(shaper.rough_options.roughness);
    shaperpage
        .roughconfig_bowing_spinbutton()
        .set_value(shaper.rough_options.bowing);
    shaperpage
        .roughconfig_curvestepcount_spinbutton()
        .set_value(shaper.rough_options.curve_stepcount);
    shaperpage
        .roughconfig_multistroke_switch()
        .set_active(!shaper.rough_options.disable_multistroke);
}
