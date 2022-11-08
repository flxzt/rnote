use super::ShaperPage;
use crate::RnoteAppWindow;

use gtk4::{glib, glib::clone, subclass::prelude::ObjectSubclassIsExt};
use rnote_compose::builders::ConstraintRatio;
use rnote_engine::pens::Shaper;

pub fn setup(shaperpage: &ShaperPage, appwindow: &RnoteAppWindow) {
    setup_enabled_switch(shaperpage, appwindow);
    setup_one_to_one_switch(shaperpage, appwindow);
    setup_three_to_two_switch(shaperpage, appwindow);
    setup_golden_ratio_switch(shaperpage, appwindow);
}

pub fn refresh(shaperpage: &ShaperPage, shaper: &Shaper) {
    shaperpage
        .imp()
        .constraint_enabled_switch
        .set_state(shaper.constraints.enabled);

    shaperpage.imp().constraint_one_to_one_switch.set_state(
        shaper
            .constraints
            .ratios
            .get(&ConstraintRatio::OneToOne)
            .is_some(),
    );
    shaperpage.imp().constraint_three_to_two_switch.set_state(
        shaper
            .constraints
            .ratios
            .get(&ConstraintRatio::ThreeToTwo)
            .is_some(),
    );
    shaperpage.imp().constraint_golden_switch.set_state(
        shaper
            .constraints
            .ratios
            .get(&ConstraintRatio::Golden)
            .is_some(),
    );
}

fn setup_enabled_switch(shaperpage: &ShaperPage, appwindow: &RnoteAppWindow) {
    shaperpage
        .imp()
        .constraint_enabled_switch
        .get()
        .connect_state_notify(clone!(@weak appwindow => move |switch|  {
            appwindow.canvas().engine().borrow_mut().penholder.shaper.constraints.enabled = switch.state();

            if let Err(e) = appwindow.save_engine_config() {
                log::error!("saving engine config failed after changing shaper constraint enabled, Err `{}`", e);
            }
        }));
}

fn setup_one_to_one_switch(shaperpage: &ShaperPage, appwindow: &RnoteAppWindow) {
    shaperpage.imp()
            .constraint_one_to_one_switch
            .get()
            .connect_state_notify(clone!(@weak appwindow => move |switch|  {
                if switch.state() {
                    appwindow.canvas().engine().borrow_mut().penholder.shaper.constraints.ratios.insert(ConstraintRatio::OneToOne);
                } else {
                    appwindow.canvas().engine().borrow_mut().penholder.shaper.constraints.ratios.remove(&ConstraintRatio::OneToOne);
                }

                if let Err(e) = appwindow.save_engine_config() {
                    log::error!("saving engine config failed after changing shaper one to one constraint, Err `{}`", e);
                }
            }));
}

fn setup_three_to_two_switch(shaperpage: &ShaperPage, appwindow: &RnoteAppWindow) {
    shaperpage.imp()
            .constraint_three_to_two_switch
            .get()
            .connect_state_notify(clone!(@weak appwindow => move |switch|  {
                if switch.state() {
                    appwindow.canvas().engine().borrow_mut().penholder.shaper.constraints.ratios.insert(ConstraintRatio::ThreeToTwo);
                } else {
                    appwindow.canvas().engine().borrow_mut().penholder.shaper.constraints.ratios.remove(&ConstraintRatio::ThreeToTwo);
                }

                if let Err(e) = appwindow.save_engine_config() {
                    log::error!("saving engine config failed after changing shaper three to two constraint, Err `{}`", e);
                }
            }));
}

fn setup_golden_ratio_switch(shaperpage: &ShaperPage, appwindow: &RnoteAppWindow) {
    shaperpage.imp()
            .constraint_golden_switch
            .get()
            .connect_state_notify(clone!(@weak appwindow => move |switch|  {
                if switch.state() {
                    appwindow.canvas().engine().borrow_mut().penholder.shaper.constraints.ratios.insert(ConstraintRatio::Golden);
                } else {
                    appwindow.canvas().engine().borrow_mut().penholder.shaper.constraints.ratios.remove(&ConstraintRatio::Golden);
                }

                if let Err(e) = appwindow.save_engine_config() {
                    log::error!("saving engine config failed after changing shaper golden ratio constraint, Err `{}`", e);
                }
            }));
}
