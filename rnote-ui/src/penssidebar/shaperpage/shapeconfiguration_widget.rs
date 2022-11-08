use gtk4::{glib, glib::clone, subclass::prelude::ObjectSubclassIsExt};
use rnote_compose::style::rough::RoughOptions;

use crate::RnoteAppWindow;

use super::ShaperPage;

pub fn setup(shaperpage: &ShaperPage, appwindow: &RnoteAppWindow) {
    setup_roughness_spinbutton(shaperpage, appwindow);
    setup_bowing_spinbutton(shaperpage, appwindow);
    setup_curve_stepcount_spinbutton(shaperpage, appwindow);
    setup_multistroke_switch(shaperpage, appwindow);
}

fn setup_roughness_spinbutton(shaperpage: &ShaperPage, appwindow: &RnoteAppWindow) {
    let roughness_spinbutton = shaperpage.imp().roughconfig_roughness_spinbutton.get();
    roughness_spinbutton.set_increments(0.1, 2.0);
    roughness_spinbutton.set_range(RoughOptions::ROUGHNESS_MIN, RoughOptions::ROUGHNESS_MAX);
    roughness_spinbutton.set_value(RoughOptions::ROUGHNESS_DEFAULT);

    roughness_spinbutton.connect_value_changed(
            clone!(@weak appwindow => move |roughconfig_roughness_spinbutton| {
                appwindow.canvas().engine().borrow_mut().penholder.shaper.rough_options.roughness = roughconfig_roughness_spinbutton.value();

                if let Err(e) = appwindow.save_engine_config() {
                    log::error!("saving engine config failed after changing rough shape roughness, Err `{}`", e);
                }
            }),
        );
}

fn setup_bowing_spinbutton(shaperpage: &ShaperPage, appwindow: &RnoteAppWindow) {
    let bowing_spinbutton = shaperpage.imp().roughconfig_bowing_spinbutton.get();

    bowing_spinbutton.set_increments(0.1, 2.0);
    bowing_spinbutton.set_range(RoughOptions::BOWING_MIN, RoughOptions::BOWING_MAX);
    bowing_spinbutton.set_value(RoughOptions::BOWING_DEFAULT);

    bowing_spinbutton.connect_value_changed(
            clone!(@weak appwindow => move |roughconfig_bowing_spinbutton| {
                appwindow.canvas().engine().borrow_mut().penholder.shaper.rough_options.bowing = roughconfig_bowing_spinbutton.value();

                if let Err(e) = appwindow.save_engine_config() {
                    log::error!("saving engine config failed after changing rough shape bowing, Err `{}`", e);
                }
            }),
        );
}

fn setup_curve_stepcount_spinbutton(shaperpage: &ShaperPage, appwindow: &RnoteAppWindow) {
    let curve_stepcount_spinbutton = shaperpage.imp().roughconfig_curvestepcount_spinbutton.get();
    curve_stepcount_spinbutton.set_increments(1.0, 2.0);
    curve_stepcount_spinbutton.set_range(
        RoughOptions::CURVESTEPCOUNT_MIN,
        RoughOptions::CURVESTEPCOUNT_MAX,
    );
    curve_stepcount_spinbutton.set_value(RoughOptions::CURVESTEPCOUNT_DEFAULT);

    curve_stepcount_spinbutton.connect_value_changed(
            clone!(@weak appwindow => move |roughconfig_curvestepcount_spinbutton| {
                appwindow.canvas().engine().borrow_mut().penholder.shaper.rough_options.curve_stepcount = roughconfig_curvestepcount_spinbutton.value();

                if let Err(e) = appwindow.save_engine_config() {
                    log::error!("saving engine config failed after changing rough shape curve stepcount, Err `{}`", e);
                }
            }),
        );
}

fn setup_multistroke_switch(shaperpage: &ShaperPage, appwindow: &RnoteAppWindow) {
    shaperpage.imp().roughconfig_multistroke_switch.get().connect_state_notify(clone!(@weak appwindow => move |roughconfig_multistroke_switch| {
            appwindow.canvas().engine().borrow_mut().penholder.shaper.rough_options.disable_multistroke = !roughconfig_multistroke_switch.state();
        }));
}
