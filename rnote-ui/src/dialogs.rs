use adw::prelude::*;
use gettextrs::gettext;
use gtk4::MenuButton;
use gtk4::{
    gio, glib, glib::clone, Builder, Button, ColorButton, Dialog, FileChooserAction,
    FileChooserNative, FileFilter, Label, ResponseType, ShortcutsWindow, SpinButton, StringList,
    ToggleButton,
};
use num_traits::ToPrimitive;
use rnote_engine::import::{PdfImportPageSpacing, PdfImportPagesType, PdfImportPrefs};

use crate::appwindow::RnoteAppWindow;
use crate::workspacebrowser::WorkspaceRow;
use crate::{app::RnoteApp, config};
use crate::{globals, IconPicker};

// About Dialog
pub fn dialog_about(appwindow: &RnoteAppWindow) {
    let aboutdialog = adw::AboutWindow::builder()
        .modal(true)
        .transient_for(appwindow)
        .application_name(config::APP_NAME_CAPITALIZED)
        .application_icon(config::APP_ID)
        .comments(&gettext("Sketch and take handwritten notes"))
        .website(config::APP_WEBSITE)
        .developers(
            config::APP_AUTHORS
                .iter()
                .map(|&s| String::from(s))
                .collect(),
        )
        // TRANSLATORS: 'Name <email@domain.com>' or 'Name https://website.example'
        .translator_credits(&gettext("translator-credits"))
        .license_type(globals::APP_LICENSE)
        .version((String::from(config::APP_VERSION) + config::APP_VERSION_SUFFIX).as_str())
        .build();

    if config::PROFILE == "devel" {
        aboutdialog.add_css_class("devel");
    }

    aboutdialog.show();
}

// Message Dialogs

pub fn dialog_keyboard_shortcuts(appwindow: &RnoteAppWindow) {
    let builder =
        Builder::from_resource((String::from(config::APP_IDPATH) + "ui/shortcuts.ui").as_str());
    let dialog_shortcuts: ShortcutsWindow = builder.object("shortcuts_window").unwrap();

    if config::PROFILE == "devel" {
        dialog_shortcuts.add_css_class("devel");
    }

    dialog_shortcuts.set_transient_for(Some(appwindow));
    dialog_shortcuts.show();
}

pub fn dialog_clear_doc(appwindow: &RnoteAppWindow) {
    let builder =
        Builder::from_resource((String::from(config::APP_IDPATH) + "ui/dialogs.ui").as_str());
    let dialog_clear_doc: adw::MessageDialog = builder.object("dialog_clear_doc").unwrap();

    dialog_clear_doc.set_transient_for(Some(appwindow));

    dialog_clear_doc.connect_response(
        None,
        clone!(@weak appwindow => move |_dialog_clear_doc, response| {
            match response {
                "clear" => {
                    appwindow.canvas().engine().borrow_mut().clear();

                    appwindow.canvas().return_to_origin_page();
                    appwindow.canvas().engine().borrow_mut().resize_autoexpand();
                    appwindow.canvas().update_engine_rendering();

                    appwindow.canvas().set_unsaved_changes(false);
                    appwindow.canvas().set_empty(true);
                },
                _ => {
                }
            }
        }),
    );

    dialog_clear_doc.show();
}

pub fn dialog_new_doc(appwindow: &RnoteAppWindow) {
    let builder =
        Builder::from_resource((String::from(config::APP_IDPATH) + "ui/dialogs.ui").as_str());
    let dialog_new_doc: adw::MessageDialog = builder.object("dialog_new_doc").unwrap();

    dialog_new_doc.set_transient_for(Some(appwindow));
    dialog_new_doc.connect_response(None, clone!(@weak appwindow => move |_dialog_new_doc, response| {
        match response{
            "save_current_doc" => {
                dialog_save_doc_as(&appwindow);
            },
            "new" => {
                appwindow.canvas().engine().borrow_mut().clear();

                appwindow.canvas().return_to_origin_page();
                appwindow.canvas().engine().borrow_mut().resize_autoexpand();
                appwindow.canvas().update_engine_rendering();

                appwindow.canvas().set_unsaved_changes(false);
                appwindow.canvas().set_empty(true);
                appwindow.application().unwrap().downcast::<RnoteApp>().unwrap().set_input_file(None);
                appwindow.canvas().set_output_file(None);
            },
            _ => {
            }
        }
    }));

    dialog_new_doc.show();
}

pub fn dialog_quit_save(appwindow: &RnoteAppWindow) {
    let builder =
        Builder::from_resource((String::from(config::APP_IDPATH) + "ui/dialogs.ui").as_str());
    let dialog_quit_save: adw::MessageDialog = builder.object("dialog_quit_save").unwrap();

    dialog_quit_save.set_transient_for(Some(appwindow));

    dialog_quit_save.connect_response(
        None,
        clone!(@weak appwindow => move |_dialog_quit_save, response| {
            match response {
                "save_current_doc" => {
                    dialog_save_doc_as(&appwindow);
                },
                "quit" => {
                    appwindow.close_force();
                },
                _ => {
                }
            }
        }),
    );

    dialog_quit_save.show();
}

pub fn dialog_open_overwrite(appwindow: &RnoteAppWindow) {
    let builder =
        Builder::from_resource((String::from(config::APP_IDPATH) + "ui/dialogs.ui").as_str());
    let dialog_open_input_file: adw::MessageDialog =
        builder.object("dialog_open_overwrite").unwrap();

    dialog_open_input_file.set_transient_for(Some(appwindow));

    dialog_open_input_file.connect_response(
        None,
        clone!(@weak appwindow => move |_dialog_open_input_file, response| {
            match response {
                "save_current_doc" => {
                    dialog_save_doc_as(&appwindow);
                },
                "open" => {
                    if let Some(input_file) = appwindow.application().unwrap().downcast::<RnoteApp>().unwrap().input_file().as_ref() {
                        if let Err(e) = appwindow.load_in_file(input_file, None) {
                            log::error!("failed to load in input file, {}", e);
                            adw::prelude::ActionGroupExt::activate_action(&appwindow, "error-toast", Some(&gettext("Opening file failed.").to_variant()));
                        }
                        }
                    },
                _ => {
                }
            }
        }),
    );

    dialog_open_input_file.show();
}

pub fn dialog_import_pdf_w_prefs(appwindow: &RnoteAppWindow, target_pos: Option<na::Vector2<f64>>) {
    let builder =
        Builder::from_resource((String::from(config::APP_IDPATH) + "ui/dialogs.ui").as_str());
    let dialog_import_pdf: Dialog = builder.object("dialog_import_pdf_w_prefs").unwrap();
    let pdf_page_start_spinbutton: SpinButton =
        builder.object("pdf_page_start_spinbutton").unwrap();
    let pdf_page_end_spinbutton: SpinButton = builder.object("pdf_page_end_spinbutton").unwrap();
    let pdf_info_label: Label = builder.object("pdf_info_label").unwrap();
    let pdf_import_width_perc_spinbutton: SpinButton =
        builder.object("pdf_import_width_perc_spinbutton").unwrap();
    let pdf_import_as_bitmap_toggle: ToggleButton =
        builder.object("pdf_import_as_bitmap_toggle").unwrap();
    let pdf_import_as_vector_toggle: ToggleButton =
        builder.object("pdf_import_as_vector_toggle").unwrap();
    let pdf_import_page_spacing_row: adw::ComboRow =
        builder.object("pdf_import_page_spacing_row").unwrap();

    let pdf_import_prefs = appwindow.canvas().engine().borrow().pdf_import_prefs;

    // Set the widget state from the pdf import prefs
    pdf_import_width_perc_spinbutton.set_value(pdf_import_prefs.page_width_perc);
    match pdf_import_prefs.pages_type {
        PdfImportPagesType::Bitmap => pdf_import_as_bitmap_toggle.set_active(true),
        PdfImportPagesType::Vector => pdf_import_as_vector_toggle.set_active(true),
    }
    pdf_import_page_spacing_row.set_selected(pdf_import_prefs.page_spacing.to_u32().unwrap());

    pdf_page_start_spinbutton.set_increments(1.0, 2.0);
    pdf_page_end_spinbutton.set_increments(1.0, 2.0);

    pdf_page_start_spinbutton
        .bind_property("value", &pdf_page_end_spinbutton.adjustment(), "lower")
        .flags(glib::BindingFlags::SYNC_CREATE | glib::BindingFlags::DEFAULT)
        .build();
    pdf_page_end_spinbutton
        .bind_property("value", &pdf_page_start_spinbutton.adjustment(), "upper")
        .flags(glib::BindingFlags::SYNC_CREATE | glib::BindingFlags::DEFAULT)
        .build();

    dialog_import_pdf.set_transient_for(Some(appwindow));

    if let Some(input_file) = appwindow
        .application()
        .unwrap()
        .downcast::<RnoteApp>()
        .unwrap()
        .input_file()
    {
        if let Ok(poppler_doc) =
            poppler::Document::from_gfile(&input_file, None, None::<&gio::Cancellable>)
        {
            let file_name = input_file.basename().map_or_else(
                || gettext("- no file name -"),
                |s| s.to_string_lossy().to_string(),
            );
            let title = poppler_doc
                .title()
                .map_or_else(|| gettext("- no title -"), |s| s.to_string());
            let author = poppler_doc
                .author()
                .map_or_else(|| gettext("- no author -"), |s| s.to_string());
            let mod_date = poppler_doc
                .mod_datetime()
                .and_then(|dt| dt.format("%F").ok())
                .map_or_else(|| gettext("- no date -"), |s| s.to_string());
            let n_pages = poppler_doc.n_pages();

            // pdf info
            pdf_info_label.set_label(
                (String::from("")
                    + "<b>"
                    + &gettext("File name:")
                    + "  </b>"
                    + &format!("{file_name}\n")
                    + "<b>"
                    + &gettext("Title:")
                    + "  </b>"
                    + &format!("{title}\n")
                    + "<b>"
                    + &gettext("Author:")
                    + "  </b>"
                    + &format!("{author}\n")
                    + "<b>"
                    + &gettext("Modification date:")
                    + "  </b>"
                    + &format!("{mod_date}\n")
                    + "<b>"
                    + &gettext("Pages:")
                    + "  </b>"
                    + &format!("{n_pages}\n"))
                    .as_str(),
            );

            // Configure pages spinners
            pdf_page_start_spinbutton.set_range(1.into(), n_pages.into());
            pdf_page_start_spinbutton.set_value(1.into());

            pdf_page_end_spinbutton.set_range(1.into(), n_pages.into());
            pdf_page_end_spinbutton.set_value(n_pages.into());
        }

        dialog_import_pdf.connect_response(
        clone!(@weak appwindow => move |dialog_import_pdf, responsetype| {
            match responsetype {
                ResponseType::Apply => {
                    dialog_import_pdf.close();

                    let page_range = (pdf_page_start_spinbutton.value() as u32 - 1)..pdf_page_end_spinbutton.value() as u32;

                    // Save the preferences into the engine before loading the file
                    let pages_type = if pdf_import_as_bitmap_toggle.is_active() {
                        PdfImportPagesType::Bitmap
                    } else {
                        PdfImportPagesType::Vector
                    };
                    let page_spacing = PdfImportPageSpacing::try_from(pdf_import_page_spacing_row.selected()).unwrap();

                    appwindow.canvas().engine().borrow_mut().pdf_import_prefs = PdfImportPrefs {
                        page_width_perc: pdf_import_width_perc_spinbutton.value(),
                        pages_type,
                        page_spacing,
                    };

                    glib::MainContext::default().spawn_local(clone!(@strong input_file, @strong appwindow => async move {
                        appwindow.start_pulsing_canvas_progressbar();

                        let result = input_file.load_bytes_future().await;

                        if let Ok((file_bytes, _)) = result {
                            if let Err(e) = appwindow.load_in_pdf_bytes(file_bytes.to_vec(), target_pos, Some(page_range)).await {
                                adw::prelude::ActionGroupExt::activate_action(&appwindow, "error-toast", Some(&gettext("Opening PDF file failed.").to_variant()));
                                log::error!(
                                    "load_in_rnote_bytes() failed in dialog import pdf with Err {}",
                                    e
                                );
                            }
                        }

                        appwindow.finish_canvas_progressbar();
                    }));
                }
                ResponseType::Cancel => {
                    dialog_import_pdf.close();

                    appwindow.application().unwrap().downcast::<RnoteApp>().unwrap().set_input_file(None);
                }
                _ => {
                    dialog_import_pdf.close();

                    appwindow.application().unwrap().downcast::<RnoteApp>().unwrap().set_input_file(None);
                }
            }
        }),
    );

        dialog_import_pdf.show();
    }
}

pub fn dialog_edit_workspace(appwindow: &RnoteAppWindow) {
    let builder =
        Builder::from_resource((String::from(config::APP_IDPATH) + "ui/dialogs.ui").as_str());
    let dialog_edit_workspace: Dialog = builder.object("dialog_edit_workspace").unwrap();
    let edit_workspace_preview_row: WorkspaceRow =
        builder.object("edit_workspace_preview_row").unwrap();
    let change_workspace_name_entryrow: adw::EntryRow =
        builder.object("change_workspace_name_entryrow").unwrap();
    let change_workspace_color_button: ColorButton =
        builder.object("change_workspace_color_button").unwrap();
    let change_workspace_dir_button: Button =
        builder.object("change_workspace_dir_button").unwrap();
    let change_workspace_dir_label: Label = builder.object("change_workspace_dir_label").unwrap();
    let change_workspace_icon_menubutton: MenuButton =
        builder.object("change_workspace_icon_menubutton").unwrap();
    let change_workspace_icon_picker: IconPicker =
        builder.object("change_workspace_icon_picker").unwrap();

    edit_workspace_preview_row.init(appwindow);
    dialog_edit_workspace.set_transient_for(Some(appwindow));

    // Sets the icons
    change_workspace_icon_picker.set_list(StringList::new(globals::WORKSPACELISTENTRY_ICONS_LIST));

    let dialog_change_workspace_dir: FileChooserNative = FileChooserNative::builder()
        .title(&gettext("Change workspace directory"))
        .modal(true)
        .transient_for(appwindow)
        .accept_label(&gettext("Select"))
        .cancel_label(&gettext("Cancel"))
        .action(FileChooserAction::SelectFolder)
        .select_multiple(false)
        .build();

    if let Some(row) = appwindow
        .workspacebrowser()
        .current_selected_workspace_row()
    {
        if let Err(e) =
            dialog_change_workspace_dir.set_file(&gio::File::for_path(&row.entry().dir()))
        {
            log::error!("set file in change workspace dialog failed with Err {}", e);
        }

        // set initial dialog UI on popup
        edit_workspace_preview_row
            .entry()
            .replace_data(&row.entry());
        change_workspace_name_entryrow.set_text(row.entry().name().as_str());
        change_workspace_icon_menubutton.set_icon_name(row.entry().icon().as_str());
        change_workspace_color_button.set_rgba(&row.entry().color());
        change_workspace_dir_label.set_label(&row.entry().dir().as_str());
    }

    change_workspace_name_entryrow.connect_apply(
        clone!(@weak edit_workspace_preview_row => move |entry| {
            let text = entry.text().to_string();
            edit_workspace_preview_row.entry().set_name(text);
        }),
    );

    change_workspace_icon_picker.connect_local(
        "icon-picked",
        false,
        clone!(@weak change_workspace_icon_menubutton, @weak edit_workspace_preview_row, @weak appwindow =>@default-return None, move |args| {
            let picked = args[1].get::<String>().unwrap();

            change_workspace_icon_menubutton.set_icon_name(&picked);
            edit_workspace_preview_row.entry().set_icon(picked);
            None
        }),
    );

    change_workspace_color_button.connect_color_set(
        clone!(@weak edit_workspace_preview_row => move |button| {
            let color = button.rgba();
            edit_workspace_preview_row.entry().set_color(color);
        }),
    );

    dialog_change_workspace_dir.connect_response(
        clone!(@weak edit_workspace_preview_row, @weak change_workspace_dir_label, @weak dialog_edit_workspace, @weak appwindow => move |dialog_change_workspace_dir, responsetype| {
            match responsetype {
                ResponseType::Accept => {
                    if let Some(p) = dialog_change_workspace_dir.file().and_then(|f| f.path()) {
                        let path_string = p.to_string_lossy().to_string();
                        change_workspace_dir_label.set_label(&path_string);
                        edit_workspace_preview_row.entry().set_dir(path_string);
                    } else {
                        change_workspace_dir_label.set_label(&gettext("- no directory selected -"));
                    }
                }
                _ => {}
            }

            dialog_change_workspace_dir.hide();
            dialog_edit_workspace.show();
        }),
    );

    dialog_edit_workspace.connect_response(
        clone!(@weak edit_workspace_preview_row, @weak appwindow => move |dialog_modify_workspace, responsetype| {
            match responsetype {
                ResponseType::Apply => {
                    // update the actual row
                    if let Some(current_row) = appwindow.workspacebrowser().current_selected_workspace_row() {
                        current_row.entry().replace_data(&edit_workspace_preview_row.entry());
                    }
                }
                _ => {}
            }

            dialog_modify_workspace.close();
        }));

    change_workspace_dir_button.connect_clicked(
        clone!(@weak dialog_edit_workspace, @weak dialog_change_workspace_dir, @weak appwindow => move |_| {
            dialog_edit_workspace.hide();
            dialog_change_workspace_dir.show();
        }),
    );

    dialog_edit_workspace.show();
    *appwindow.filechoosernative().borrow_mut() = Some(dialog_change_workspace_dir);
}

// FileChooserNative Dialogs

pub fn dialog_open_doc(appwindow: &RnoteAppWindow) {
    let filter = FileFilter::new();
    filter.add_mime_type("application/rnote");
    filter.add_mime_type("application/x-xopp");
    filter.add_pattern("*.rnote");
    filter.set_name(Some(&gettext(".rnote / .xopp File")));

    let dialog_open_file: FileChooserNative = FileChooserNative::builder()
        .title(&gettext("Open file"))
        .modal(true)
        .transient_for(appwindow)
        .accept_label(&gettext("Open"))
        .cancel_label(&gettext("Cancel"))
        .action(FileChooserAction::Open)
        .select_multiple(false)
        .build();

    dialog_open_file.add_filter(&filter);

    dialog_open_file.connect_response(clone!(@weak appwindow => move |dialog_open_file, responsetype| {
            match responsetype {
                ResponseType::Accept => {
                    if let Some(file) = dialog_open_file.file() {
                        appwindow.application().unwrap().downcast::<RnoteApp>().unwrap().set_input_file(Some(file));
                        appwindow.canvas().set_unsaved_changes(false);

                        dialog_open_overwrite(&appwindow);
                    }
                },
                _ => {
                }
            }

        }));

    dialog_open_file.show();

    // keeping the filechooser around because otherwise GTK won't keep it alive
    *appwindow.filechoosernative().borrow_mut() = Some(dialog_open_file);
}

pub fn dialog_save_doc_as(appwindow: &RnoteAppWindow) {
    let filter = FileFilter::new();
    filter.add_mime_type("application/rnote");
    filter.add_pattern("*.rnote");
    filter.set_name(Some(&gettext(".rnote file")));

    let dialog_save_doc_as: FileChooserNative = FileChooserNative::builder()
        .title(&gettext("Save document as"))
        .modal(true)
        .transient_for(appwindow)
        .accept_label(&gettext("Save as"))
        .cancel_label(&gettext("Cancel"))
        .action(FileChooserAction::Save)
        .select_multiple(false)
        .build();

    dialog_save_doc_as.add_filter(&filter);

    dialog_save_doc_as.set_current_name(
        format!("{}_doc.rnote", rnote_engine::utils::now_formatted_string()).as_str(),
    );

    dialog_save_doc_as.connect_response(
        clone!(@weak appwindow => move |dialog_export_doc, responsetype| {
            match responsetype {
                ResponseType::Accept => {
                    if let Some(file) = dialog_export_doc.file() {
                        glib::MainContext::default().spawn_local(clone!(@strong appwindow => async move {
                            appwindow.start_pulsing_canvas_progressbar();

                            if let Err(e) = appwindow.save_document_to_file(&file).await {
                                appwindow.canvas().set_output_file(None);

                                log::error!("saving document failed with error `{}`", e);
                                adw::prelude::ActionGroupExt::activate_action(&appwindow, "error-toast", Some(&gettext("Saving document failed.").to_variant()));
                            } else {
                                adw::prelude::ActionGroupExt::activate_action(&appwindow, "text-toast", Some(&gettext("Saved document successfully.").to_variant()));
                            }

                            appwindow.finish_canvas_progressbar();
                        }));
                    }
                }
                _ => {
                }
            }
        }),
    );

    dialog_save_doc_as.show();
    // keeping the filechooser around because otherwise GTK won't keep it alive
    *appwindow.filechoosernative().borrow_mut() = Some(dialog_save_doc_as);
}

pub fn dialog_import_file(appwindow: &RnoteAppWindow) {
    let filter = FileFilter::new();
    filter.add_mime_type("image/svg+xml");
    filter.add_mime_type("image/png");
    filter.add_mime_type("image/jpeg");
    filter.add_mime_type("application/pdf");
    filter.add_pattern("*.svg");
    filter.add_pattern("*.png");
    filter.add_pattern("*.jpg");
    filter.add_pattern("*.pdf");
    filter.set_name(Some(&gettext("PNG / SVG / JPG / PDF file")));

    let dialog_import_file: FileChooserNative = FileChooserNative::builder()
        .title(&gettext("Import file"))
        .modal(true)
        .transient_for(appwindow)
        .accept_label(&gettext("Import"))
        .cancel_label(&gettext("Cancel"))
        .action(FileChooserAction::Open)
        .select_multiple(false)
        .build();

    dialog_import_file.add_filter(&filter);

    dialog_import_file.connect_response(
        clone!(@weak appwindow => move |dialog_import_file, responsetype| {
            match responsetype {
                ResponseType::Accept => {
                    if let Some(file) = dialog_import_file.file() {
                        appwindow.open_file_w_dialogs(&file, None);
                    }
                }
                _ => {
                }
            }
        }),
    );

    dialog_import_file.show();
    // keeping the filechooser around because otherwise GTK won't keep it alive
    *appwindow.filechoosernative().borrow_mut() = Some(dialog_import_file);
}

pub fn dialog_export_selection_as_svg(appwindow: &RnoteAppWindow) {
    let filter = FileFilter::new();
    filter.add_mime_type("image/svg+xml");
    filter.add_pattern("*.svg");
    filter.set_name(Some(&gettext("SVG file")));

    let dialog_export_selection_as_svg: FileChooserNative = FileChooserNative::builder()
        .title(&gettext("Export Selection"))
        .modal(true)
        .transient_for(appwindow)
        .accept_label(&gettext("Export"))
        .cancel_label(&gettext("Cancel"))
        .action(FileChooserAction::Save)
        .select_multiple(false)
        .build();
    dialog_export_selection_as_svg.add_filter(&filter);

    dialog_export_selection_as_svg.set_current_name(
        format!(
            "{}_selection.svg",
            rnote_engine::utils::now_formatted_string()
        )
        .as_str(),
    );

    dialog_export_selection_as_svg.connect_response(clone!(@weak appwindow => move |dialog_export_selection_as_svg, responsetype| {
            match responsetype {
                ResponseType::Accept => {
                    if let Some(file) = dialog_export_selection_as_svg.file() {
                        glib::MainContext::default().spawn_local(clone!(@strong appwindow => async move {
                            appwindow.start_pulsing_canvas_progressbar();

                            if let Err(e) = appwindow.export_selection_as_svg(&file, false).await {
                                log::error!("exporting selection as svg failed with error `{}`", e);
                                adw::prelude::ActionGroupExt::activate_action(&appwindow, "error-toast", Some(&gettext("Export selection as SVG failed.").to_variant()));
                            } else {
                                adw::prelude::ActionGroupExt::activate_action(&appwindow, "text-toast", Some(&gettext("Exported selection as SVG successfully.").to_variant()));
                            }

                            appwindow.finish_canvas_progressbar();
                        }));
                    }
                }
                _ => {}
            }
        }));

    dialog_export_selection_as_svg.show();
    // keeping the filechooser around because otherwise GTK won't keep it alive
    *appwindow.filechoosernative().borrow_mut() = Some(dialog_export_selection_as_svg);
}

pub fn dialog_export_selection_as_png(appwindow: &RnoteAppWindow) {
    let filter = FileFilter::new();
    filter.add_mime_type("image/png");
    filter.add_pattern("*.png");
    filter.set_name(Some(&gettext("PNG file")));

    let dialog_export_selection_as_png: FileChooserNative = FileChooserNative::builder()
        .title(&gettext("Export Selection"))
        .modal(true)
        .transient_for(appwindow)
        .accept_label(&gettext("Export"))
        .cancel_label(&gettext("Cancel"))
        .action(FileChooserAction::Save)
        .select_multiple(false)
        .build();
    dialog_export_selection_as_png.add_filter(&filter);

    dialog_export_selection_as_png.set_current_name(
        format!(
            "{}_selection.png",
            rnote_engine::utils::now_formatted_string()
        )
        .as_str(),
    );

    dialog_export_selection_as_png.connect_response(clone!(@weak appwindow => move |dialog_export_selection_as_png, responsetype| {
            match responsetype {
                ResponseType::Accept => {
                    if let Some(file) = dialog_export_selection_as_png.file() {
                        glib::MainContext::default().spawn_local(clone!(@strong appwindow => async move {
                            appwindow.start_pulsing_canvas_progressbar();

                            if let Err(e) = appwindow.export_selection_as_bitmapimage(&file, image::ImageOutputFormat::Png, false).await {
                                log::error!("exporting selection as png failed with error `{}`", e);
                                adw::prelude::ActionGroupExt::activate_action(&appwindow, "error-toast", Some(&gettext("Export selection as PNG failed.").to_variant()));
                            } else {
                                adw::prelude::ActionGroupExt::activate_action(&appwindow, "text-toast", Some(&gettext("Exported selection as PNG successfully.").to_variant()));
                            }

                            appwindow.finish_canvas_progressbar();
                        }));
                    }
                }
                _ => {}
            }
        }));

    dialog_export_selection_as_png.show();
    // keeping the filechooser around because otherwise GTK won't keep it alive
    *appwindow.filechoosernative().borrow_mut() = Some(dialog_export_selection_as_png);
}

pub fn dialog_export_doc_as_svg(appwindow: &RnoteAppWindow) {
    let filter = FileFilter::new();
    filter.add_mime_type("image/svg+xml");
    filter.add_pattern("*.svg");
    filter.set_name(Some(&gettext("SVG file")));

    let dialog_export_doc_as_svg: FileChooserNative = FileChooserNative::builder()
        .title(&gettext("Export document"))
        .modal(true)
        .transient_for(appwindow)
        .accept_label(&gettext("Export"))
        .cancel_label(&gettext("Cancel"))
        .action(FileChooserAction::Save)
        .select_multiple(false)
        .build();
    dialog_export_doc_as_svg.add_filter(&filter);

    dialog_export_doc_as_svg.set_current_name(
        format!("{}_doc.svg", rnote_engine::utils::now_formatted_string()).as_str(),
    );

    dialog_export_doc_as_svg.connect_response(
        clone!(@weak appwindow => move |dialog_export_doc, responsetype| {
            match responsetype {
                ResponseType::Accept => {
                    if let Some(file) = dialog_export_doc.file() {
                        glib::MainContext::default().spawn_local(clone!(@strong appwindow => async move {
                            appwindow.start_pulsing_canvas_progressbar();

                            if let Err(e) = appwindow.export_doc_as_svg(&file, true).await {
                                log::error!("exporting document failed with error `{}`", e);
                                adw::prelude::ActionGroupExt::activate_action(&appwindow, "error-toast", Some(&gettext("Export document as SVG failed.").to_variant()));
                            } else {
                                adw::prelude::ActionGroupExt::activate_action(&appwindow, "text-toast", Some(&gettext("Exported document as SVG successfully.").to_variant()));
                            }

                            appwindow.finish_canvas_progressbar();
                        }));
                    }
                }
                _ => {
                }
            }
        }),
    );

    dialog_export_doc_as_svg.show();
    // keeping the filechooser around because otherwise GTK won't keep it alive
    *appwindow.filechoosernative().borrow_mut() = Some(dialog_export_doc_as_svg);
}

pub fn dialog_export_doc_as_pdf(appwindow: &RnoteAppWindow) {
    let filter = FileFilter::new();
    filter.add_mime_type("application/pdf");
    filter.add_pattern("*.pdf");
    filter.set_name(Some(&gettext("PDF file")));

    let dialog_export_doc_as_pdf: FileChooserNative = FileChooserNative::builder()
        .title(&gettext("Export document"))
        .modal(true)
        .transient_for(appwindow)
        .accept_label(&gettext("Export"))
        .cancel_label(&gettext("Cancel"))
        .action(FileChooserAction::Save)
        .select_multiple(false)
        .build();
    dialog_export_doc_as_pdf.add_filter(&filter);

    dialog_export_doc_as_pdf.set_current_name(
        format!("{}_doc.pdf", rnote_engine::utils::now_formatted_string()).as_str(),
    );

    dialog_export_doc_as_pdf.connect_response(
        clone!(@weak appwindow => move |dialog_export_doc, responsetype| {
            match responsetype {
                ResponseType::Accept => {
                    if let Some(file) = dialog_export_doc.file() {
                        glib::MainContext::default().spawn_local(clone!(@strong appwindow, @strong file => async move {
                            appwindow.start_pulsing_canvas_progressbar();

                            if let Err(e) = appwindow.export_doc_as_pdf(&file, true).await {
                                log::error!("export_doc_as_pdf() failed in export dialog with Err {}", e);
                                adw::prelude::ActionGroupExt::activate_action(&appwindow, "error-toast", Some(&gettext("Export document as PDF failed.").to_variant()));
                            } else {
                                adw::prelude::ActionGroupExt::activate_action(&appwindow, "text-toast", Some(&gettext("Exported document as PDF successfully.").to_variant()));
                            };

                            appwindow.finish_canvas_progressbar();
                        }));
                    }
                }
                _ => {
                }
            }
        }),
    );

    dialog_export_doc_as_pdf.show();
    // keeping the filechooser around because otherwise GTK won't keep it alive
    *appwindow.filechoosernative().borrow_mut() = Some(dialog_export_doc_as_pdf);
}

pub fn dialog_export_doc_as_xopp(appwindow: &RnoteAppWindow) {
    let filter = FileFilter::new();
    filter.add_mime_type("application/x-xopp");
    filter.add_pattern("*.xopp");
    filter.set_name(Some(&gettext(".xopp file")));

    let dialog_export_doc_as_xopp: FileChooserNative = FileChooserNative::builder()
        .title(&gettext("Export document"))
        .modal(true)
        .transient_for(appwindow)
        .accept_label(&gettext("Export"))
        .cancel_label(&gettext("Cancel"))
        .action(FileChooserAction::Save)
        .select_multiple(false)
        .build();
    dialog_export_doc_as_xopp.add_filter(&filter);

    dialog_export_doc_as_xopp.set_current_name(
        format!("{}_doc.xopp", rnote_engine::utils::now_formatted_string()).as_str(),
    );

    dialog_export_doc_as_xopp.connect_response(
        clone!(@weak appwindow => move |dialog_export_doc, responsetype| {
            match responsetype {
                ResponseType::Accept => {
                    if let Some(file) = dialog_export_doc.file() {
                        glib::MainContext::default().spawn_local(clone!(@strong appwindow => async move {
                            appwindow.start_pulsing_canvas_progressbar();

                            if let Err(e) = appwindow.export_doc_as_xopp(&file).await {
                                log::error!("exporting document as .xopp failed, replace_file_async failed with Err {}", e);
                                adw::prelude::ActionGroupExt::activate_action(&appwindow, "error-toast", Some(&gettext("Exporting document as .xopp failed.").to_variant()));
                            } else {
                                adw::prelude::ActionGroupExt::activate_action(&appwindow, "text-toast", Some(&gettext("Exported document as .xopp successfully.").to_variant()));
                            }

                            appwindow.finish_canvas_progressbar();
                        }));
                    }
                }
                _ => {}
            }
        }),
    );

    dialog_export_doc_as_xopp.show();
    // keeping the filechooser around because otherwise GTK won't keep it alive
    *appwindow.filechoosernative().borrow_mut() = Some(dialog_export_doc_as_xopp);
}

pub fn dialog_export_engine_state(appwindow: &RnoteAppWindow) {
    let filter = FileFilter::new();
    filter.add_mime_type("application/json");
    filter.add_pattern("*.json");
    filter.set_name(Some(&gettext("JSON file")));

    let dialog_export_engine_state: FileChooserNative = FileChooserNative::builder()
        .title(&gettext("Export engine state"))
        .modal(true)
        .transient_for(appwindow)
        .accept_label(&gettext("Export"))
        .cancel_label(&gettext("Cancel"))
        .action(FileChooserAction::Save)
        .select_multiple(false)
        .build();
    dialog_export_engine_state.add_filter(&filter);

    dialog_export_engine_state.set_current_name(
        format!(
            "{}_engine_state.json",
            rnote_engine::utils::now_formatted_string()
        )
        .as_str(),
    );

    dialog_export_engine_state.connect_response(
        clone!(@weak appwindow => move |dialog_export_engine_state, responsetype| {
            match responsetype {
                ResponseType::Accept => {
                    if let Some(file) = dialog_export_engine_state.file() {
                        glib::MainContext::default().spawn_local(clone!(@strong appwindow => async move {
                            appwindow.start_pulsing_canvas_progressbar();

                            if let Err(e) = appwindow.export_engine_state(&file).await {
                                log::error!("exporting engine state failed with error `{}`", e);
                                adw::prelude::ActionGroupExt::activate_action(&appwindow, "error-toast", Some(&gettext("Export engine state failed.").to_variant()));
                            } else {
                                adw::prelude::ActionGroupExt::activate_action(&appwindow, "text-toast", Some(&gettext("Exported engine state successfully.").to_variant()));
                            }

                            appwindow.finish_canvas_progressbar();
                        }));
                    }
                }
                _ => {}
            }
        }),
    );

    dialog_export_engine_state.show();
    // keeping the filechooser around because otherwise GTK won't keep it alive
    *appwindow.filechoosernative().borrow_mut() = Some(dialog_export_engine_state);
}

pub fn dialog_export_engine_config(appwindow: &RnoteAppWindow) {
    let filter = FileFilter::new();
    filter.add_mime_type("application/json");
    filter.add_pattern("*.json");
    filter.set_name(Some(&gettext("JSON file")));

    let dialog_export_engine_config: FileChooserNative = FileChooserNative::builder()
        .title(&gettext("Export engine config"))
        .modal(true)
        .transient_for(appwindow)
        .accept_label(&gettext("Export"))
        .cancel_label(&gettext("Cancel"))
        .action(FileChooserAction::Save)
        .select_multiple(false)
        .build();
    dialog_export_engine_config.add_filter(&filter);

    dialog_export_engine_config.set_current_name(
        format!(
            "{}_engine_config.json",
            rnote_engine::utils::now_formatted_string()
        )
        .as_str(),
    );

    dialog_export_engine_config.connect_response(
        clone!(@weak appwindow => move |dialog_export_engine_config, responsetype| {
            match responsetype {
                ResponseType::Accept => {
                    if let Some(file) = dialog_export_engine_config.file() {
                        glib::MainContext::default().spawn_local(clone!(@strong appwindow => async move {
                            appwindow.start_pulsing_canvas_progressbar();

                            if let Err(e) = appwindow.export_engine_config(&file).await {
                                log::error!("exporting engine state failed with error `{}`", e);
                                adw::prelude::ActionGroupExt::activate_action(&appwindow, "error-toast", Some(&gettext("Export engine config failed.").to_variant()));
                            } else {
                                adw::prelude::ActionGroupExt::activate_action(&appwindow, "text-toast", Some(&gettext("Exported engine config successfully.").to_variant()));
                            }

                            appwindow.finish_canvas_progressbar();
                        }));
                    }
                }
                _ => {}
            }
        }),
    );

    dialog_export_engine_config.show();
    // keeping the filechooser around because otherwise GTK won't keep it alive
    *appwindow.filechoosernative().borrow_mut() = Some(dialog_export_engine_config);
}
