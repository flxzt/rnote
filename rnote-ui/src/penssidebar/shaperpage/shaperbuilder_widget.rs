use crate::RnoteAppWindow;
use cairo::glib::Cast;
use gtk4::{glib, glib::clone, traits::ListBoxRowExt};
use rnote_compose::builders::ShapeBuilderType;
use rnote_engine::pens::Shaper;

use super::ShaperPage;

pub fn setup(shaperpage: &ShaperPage, appwindow: &RnoteAppWindow) {
    shaperpage.shapebuildertype_listbox().connect_row_selected(
        clone!(@weak shaperpage, @weak appwindow => move |_shapetype_listbox, selected_row| {
            if let Some(selected_row) = selected_row.map(|selected_row| {selected_row.downcast_ref::<adw::ActionRow>().unwrap()}) {
                appwindow.canvas().engine().borrow_mut().penholder.shaper.builder_type = ShapeBuilderType::try_from(selected_row.index() as u32).unwrap_or_default();

                if let Err(e) = appwindow.save_engine_config() {
                    log::error!("saving engine config failed after changing shape builder type, Err `{}`", e);
                }
                // Need to refresh the whole page, because changing the builder type affects multiple widgets
                shaperpage.refresh_ui(&appwindow);
            }
        }),
    );
}

pub fn refresh(shaperpage: &ShaperPage, appwindow: &RnoteAppWindow, shaper: &Shaper) {
    match shaper.builder_type {
        ShapeBuilderType::Line => {
            shaperpage.shapebuildertype_listbox().select_row(Some(
                &appwindow
                    .penssidebar()
                    .shaper_page()
                    .shapebuildertype_line_row(),
            ));
            shaperpage
                .shapebuildertype_image()
                .set_icon_name(Some("shape-line-symbolic"));
        }
        ShapeBuilderType::Rectangle => {
            shaperpage.shapebuildertype_listbox().select_row(Some(
                &appwindow
                    .penssidebar()
                    .shaper_page()
                    .shapebuildertype_rectangle_row(),
            ));
            shaperpage
                .shapebuildertype_image()
                .set_icon_name(Some("shape-rectangle-symbolic"));
        }
        ShapeBuilderType::Ellipse => {
            shaperpage
                .shapebuildertype_listbox()
                .select_row(Some(&shaperpage.shapebuildertype_ellipse_row()));
            shaperpage
                .shapebuildertype_image()
                .set_icon_name(Some("shape-ellipse-symbolic"));
        }
        ShapeBuilderType::FociEllipse => {
            shaperpage
                .shapebuildertype_listbox()
                .select_row(Some(&shaperpage.shapebuildertype_fociellipse_row()));
            shaperpage
                .shapebuildertype_image()
                .set_icon_name(Some("shape-fociellipse-symbolic"));
        }
        ShapeBuilderType::QuadBez => {
            shaperpage.shapebuildertype_listbox().select_row(Some(
                &appwindow
                    .penssidebar()
                    .shaper_page()
                    .shapebuildertype_quadbez_row(),
            ));
            shaperpage
                .shapebuildertype_image()
                .set_icon_name(Some("shape-quadbez-symbolic"));
        }
        ShapeBuilderType::CubBez => {
            shaperpage.shapebuildertype_listbox().select_row(Some(
                &appwindow
                    .penssidebar()
                    .shaper_page()
                    .shapebuildertype_cubbez_row(),
            ));
            shaperpage
                .shapebuildertype_image()
                .set_icon_name(Some("shape-cubbez-symbolic"));
        }
    }
}
