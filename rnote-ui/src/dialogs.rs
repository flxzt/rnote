use adw::prelude::*;
use gettextrs::gettext;
use gtk4::{
    gio, glib, glib::clone, Builder, Button, ColorButton, Dialog, FileChooserAction,
    FileChooserNative, FileFilter, Label, ResponseType, ShortcutsWindow, SpinButton, StringList,
    ToggleButton,
};
use gtk4::{MenuButton, Switch};
use num_traits::ToPrimitive;
use rnote_engine::engine::export::{SelectionExportFormat, SelectionExportPrefs};
use rnote_engine::engine::import::{PdfImportPageSpacing, PdfImportPagesType, PdfImportPrefs};

use crate::appwindow::{self, RnoteAppWindow};
use crate::config;
use crate::workspacebrowser::WorkspaceRow;
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
        .issue_url(config::APP_ISSUES_URL)
        .support_url(config::APP_SUPPORT_URL)
        .developer_name(config::APP_AUTHOR_NAME)
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
                // Cancel
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
    dialog_new_doc.connect_response(
        None,
        clone!(@weak appwindow => move |_dialog_new_doc, response| {
        let new_doc = |appwindow: &RnoteAppWindow| {
            appwindow.canvas().engine().borrow_mut().clear();

            appwindow.canvas().return_to_origin_page();
            appwindow.canvas().engine().borrow_mut().resize_autoexpand();
            appwindow.canvas().update_engine_rendering();

            appwindow.canvas().set_unsaved_changes(false);
            appwindow.canvas().set_empty(true);

            appwindow.app().set_input_file(None);
            appwindow.canvas().set_output_file(None);
        };

        match response{
            "discard" => {
                new_doc(&appwindow)
            },
            "save" => {
                glib::MainContext::default().spawn_local(clone!(@strong appwindow => async move {
                    if let Some(output_file) = appwindow.canvas().output_file() {
                        appwindow.start_pulsing_canvas_progressbar();

                        if let Err(e) = appwindow.save_document_to_file(&output_file).await {
                            appwindow.canvas().set_output_file(None);

                            log::error!("saving document failed with error `{}`", e);
                            adw::prelude::ActionGroupExt::activate_action(&appwindow, "error-toast", Some(&gettext("Saving document failed.").to_variant()));
                        }

                        appwindow.finish_canvas_progressbar();
                        // No success toast on saving without dialog, success is already indicated in the header title
                    } else {
                        // Open a dialog to choose a save location
                        filechooser_save_doc_as(&appwindow);
                    }

                    // only create new document if saving was successful
                    if !appwindow.unsaved_changes() {
                        new_doc(&appwindow)
                    }
                }));
            },
            _ => {
                // Cancel
            }
        }
        }),
    );

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
                "discard" => {
                    appwindow.close_force();
                },
                "save" => {
                    glib::MainContext::default().spawn_local(clone!(@strong appwindow => async move {
                        if let Some(output_file) = appwindow.canvas().output_file() {
                            appwindow.start_pulsing_canvas_progressbar();

                            if let Err(e) = appwindow.save_document_to_file(&output_file).await {
                                appwindow.canvas().set_output_file(None);

                                log::error!("saving document failed with error `{}`", e);
                                adw::prelude::ActionGroupExt::activate_action(&appwindow, "error-toast", Some(&gettext("Saving document failed.").to_variant()));
                            }

                            appwindow.finish_canvas_progressbar();
                            // No success toast on saving without dialog, success is already indicated in the header title
                        } else {
                            // Open a dialog to choose a save location
                            filechooser_save_doc_as(&appwindow);
                        }

                        // only close if saving was successful
                        if !appwindow.unsaved_changes() {
                            appwindow.close_force();
                        }
                    }));
                },
                _ => {
                // Cancel
                }
            }
        }),
    );

    dialog_quit_save.show();
}

/// Asks to open the document from the app `input-file` property and overwrites the current document.
pub fn dialog_open_overwrite(appwindow: &RnoteAppWindow) {
    let builder =
        Builder::from_resource((String::from(config::APP_IDPATH) + "ui/dialogs.ui").as_str());
    let dialog_open_input_file: adw::MessageDialog =
        builder.object("dialog_open_overwrite").unwrap();

    dialog_open_input_file.set_transient_for(Some(appwindow));

    dialog_open_input_file.connect_response(
        None,
        clone!(@weak appwindow => move |_dialog_open_input_file, response| {
            let open_overwrite = |appwindow: &RnoteAppWindow| {
                if let Some(input_file) = appwindow.app().input_file().as_ref() {
                    if let Err(e) = appwindow.load_in_file(input_file, None) {
                        log::error!("failed to load in input file, {}", e);
                        adw::prelude::ActionGroupExt::activate_action(appwindow, "error-toast", Some(&gettext("Opening file failed.").to_variant()));
                    }
                }
            };

            match response {
                "discard" => {
                    open_overwrite(&appwindow);
                }
                "save" => {
                    glib::MainContext::default().spawn_local(clone!(@strong appwindow => async move {
                        if let Some(output_file) = appwindow.canvas().output_file() {
                            appwindow.start_pulsing_canvas_progressbar();

                            if let Err(e) = appwindow.save_document_to_file(&output_file).await {
                                appwindow.canvas().set_output_file(None);

                                log::error!("saving document failed with error `{}`", e);
                                adw::prelude::ActionGroupExt::activate_action(&appwindow, "error-toast", Some(&gettext("Saving document failed.").to_variant()));
                            }

                            appwindow.finish_canvas_progressbar();
                            // No success toast on saving without dialog, success is already indicated in the header title
                        } else {
                            // Open a dialog to choose a save location
                            filechooser_save_doc_as(&appwindow);
                        }

                        // only open and overwrite document if saving was successful
                        if !appwindow.unsaved_changes() {
                            open_overwrite(&appwindow);
                        }
                    }));
                },
                _ => {
                // Cancel
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

    let pdf_import_prefs = appwindow
        .canvas()
        .engine()
        .borrow()
        .import_prefs
        .pdf_import_prefs;

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

    if let Some(input_file) = appwindow.app().input_file() {
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

                    appwindow.canvas().engine().borrow_mut().import_prefs.pdf_import_prefs = PdfImportPrefs {
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

                    appwindow.app().set_input_file(None);
                }
                _ => {
                    dialog_import_pdf.close();

                    appwindow.app().set_input_file(None);
                }
            }
        }),
    );

        dialog_import_pdf.show();
    }
}

pub fn dialog_export_selection_w_prefs(appwindow: &RnoteAppWindow) {
    let builder =
        Builder::from_resource((String::from(config::APP_IDPATH) + "ui/dialogs.ui").as_str());
    let dialog: Dialog = builder.object("dialog_export_selection_w_prefs").unwrap();
    let with_background_switch: Switch = builder
        .object("export_selection_with_background_switch")
        .unwrap();
    let export_format_row: adw::ComboRow = builder
        .object("export_selection_export_format_row")
        .unwrap();
    let export_file_label: Label = builder
        .object("export_selection_export_file_label")
        .unwrap();
    let export_file_button: Button = builder
        .object("export_selection_export_file_button")
        .unwrap();
    let jpeg_quality_spinbutton: SpinButton = builder
        .object("export_selection_jpeg_quality_spinbutton")
        .unwrap();

    let selection_export_prefs = appwindow
        .canvas()
        .engine()
        .borrow_mut()
        .export_prefs
        .selection_export_prefs;

    dialog.set_transient_for(Some(appwindow));

    // initial widget state with the preferences

    let filechooser = create_filechooser_export_selection(appwindow);
    with_background_switch.set_active(selection_export_prefs.with_background);
    export_format_row.set_selected(selection_export_prefs.export_format.to_u32().unwrap());
    jpeg_quality_spinbutton.set_value(selection_export_prefs.jpeg_quality as f64);

    // Update prefs
    export_file_button.connect_clicked(
        clone!(@weak dialog, @weak filechooser, @weak appwindow => move |_| {
            dialog.hide();
            filechooser.show();
        }),
    );

    filechooser.connect_response(
        clone!(@weak export_file_label, @weak dialog, @weak appwindow => move |filechooser, responsetype| {
            match responsetype {
                ResponseType::Accept => {
                    if let Some(p) = filechooser.file().and_then(|f| f.path()) {
                        let path_string = p.to_string_lossy().to_string();
                        export_file_label.set_label(&path_string);
                    } else {
                        export_file_label.set_label(&gettext("- no file selected -"));
                    }
                }
                _ => {}
            }

            filechooser.hide();
            dialog.show();
        }),
    );

    with_background_switch.connect_active_notify(clone!(@weak appwindow => move |with_background_switch| {
        appwindow.canvas().engine().borrow_mut().export_prefs.selection_export_prefs.with_background = with_background_switch.is_active();
    }));

    export_format_row.connect_selected_notify(clone!(@weak filechooser, @weak appwindow => move |row| {
        let selected = row.selected();
        let export_format = SelectionExportFormat::try_from(selected).unwrap();
        appwindow.canvas().engine().borrow_mut().export_prefs.selection_export_prefs.export_format = export_format;

        // update the filechooser dependent on the selected export format
        update_export_selection_filechooser_with_prefs(&filechooser, appwindow.canvas().output_file(),&appwindow.canvas().engine().borrow().export_prefs.selection_export_prefs);
    }));

    dialog.connect_response(
        clone!(@weak with_background_switch, @strong filechooser, @weak appwindow => move |dialog, responsetype| {
            match responsetype {
                ResponseType::Apply => {
                    if let Some(file) = filechooser.file() {
                        glib::MainContext::default().spawn_local(clone!(@strong appwindow => async move {
                            appwindow.start_pulsing_canvas_progressbar();

                            if let Err(e) = appwindow.export_selection(&file, None).await {
                                log::error!("exporting selection failed with error `{}`", e);
                                adw::prelude::ActionGroupExt::activate_action(&appwindow, "error-toast", Some(&gettext("Export selection failed.").to_variant()));
                            } else {
                                adw::prelude::ActionGroupExt::activate_action(&appwindow, "text-toast", Some(&gettext("Exported selection successfully.").to_variant()));
                            }

                            appwindow.finish_canvas_progressbar();
                        }));
                    }
                }
                _ => {}
            }

            dialog.close();
        }));

    dialog.show();
    // keeping the filechooser around because otherwise GTK won't keep it alive
    *appwindow.filechoosernative().borrow_mut() = Some(filechooser);
}

fn create_filechooser_export_selection(appwindow: &RnoteAppWindow) -> FileChooserNative {
    let filechooser: FileChooserNative = FileChooserNative::builder()
        .title(&gettext("Export Selection"))
        .modal(true)
        .transient_for(appwindow)
        .accept_label(&gettext("Select"))
        .cancel_label(&gettext("Cancel"))
        .action(FileChooserAction::Save)
        .select_multiple(false)
        .build();

    if let Some(current_workspace_dir) = appwindow.workspacebrowser().selected_workspace_dir() {
        if let Err(e) =
            filechooser.set_current_folder(Some(&gio::File::for_path(current_workspace_dir)))
        {
            log::error!(
                "set_current_folder() for dialog_export_selection_as_svg failed with Err `{e}`"
            );
        }
    }

    update_export_selection_filechooser_with_prefs(
        &filechooser,
        appwindow.canvas().output_file(),
        &appwindow
            .canvas()
            .engine()
            .borrow()
            .export_prefs
            .selection_export_prefs,
    );

    filechooser
}

fn update_export_selection_filechooser_with_prefs(
    filechooser: &FileChooserNative,
    output_file: Option<gio::File>,
    selection_export_prefs: &SelectionExportPrefs,
) {
    let filter = FileFilter::new();

    match selection_export_prefs.export_format {
        SelectionExportFormat::Svg => {
            filter.add_mime_type("image/svg+xml");
            filter.add_pattern("*.svg");
            filter.set_name(Some(&gettext("Svg")));
        }
        SelectionExportFormat::Png => {
            filter.add_mime_type("image/png");
            filter.add_pattern("*.png");
            filter.set_name(Some(&gettext("Png")));
        }
        SelectionExportFormat::Jpeg => {
            filter.add_mime_type("image/jpeg");
            filter.add_pattern("*.jpg");
            filter.add_pattern("*.jpeg");
            filter.set_name(Some(&gettext("Jpeg")));
        }
    }

    filechooser.add_filter(&filter);

    let file_ext = selection_export_prefs.export_format.file_ext();

    let file_title = rnote_engine::utils::default_file_title_for_export(
        output_file,
        Some(&appwindow::OUTPUT_FILE_NEW_TITLE),
        Some(" - Selection"),
    );

    filechooser.set_current_name(&(file_title + "." + &file_ext));
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
    let change_workspace_dir_label: Label = builder.object("change_workspace_dir_label").unwrap();
    let change_workspace_dir_button: Button =
        builder.object("change_workspace_dir_button").unwrap();
    let change_workspace_icon_menubutton: MenuButton =
        builder.object("change_workspace_icon_menubutton").unwrap();
    let change_workspace_icon_picker: IconPicker =
        builder.object("change_workspace_icon_picker").unwrap();

    edit_workspace_preview_row.init(appwindow);
    dialog_edit_workspace.set_transient_for(Some(appwindow));

    // Sets the icons
    change_workspace_icon_picker.set_list(StringList::new(globals::WORKSPACELISTENTRY_ICONS_LIST));

    let filechooser_change_workspace_dir: FileChooserNative = FileChooserNative::builder()
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
            filechooser_change_workspace_dir.set_file(&gio::File::for_path(&row.entry().dir()))
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

    filechooser_change_workspace_dir.connect_response(
        clone!(@weak edit_workspace_preview_row, @weak change_workspace_dir_label, @weak dialog_edit_workspace, @weak appwindow => move |filechooser, responsetype| {
            match responsetype {
                ResponseType::Accept => {
                    if let Some(p) = filechooser.file().and_then(|f| f.path()) {
                        let path_string = p.to_string_lossy().to_string();
                        change_workspace_dir_label.set_label(&path_string);
                        edit_workspace_preview_row.entry().set_dir(path_string);
                    } else {
                        change_workspace_dir_label.set_label(&gettext("- no directory selected -"));
                    }
                }
                _ => {}
            }

            filechooser.hide();
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

                        // refreshing the files list
                        appwindow.workspacebrowser().refresh();
                        // And save the state
                        appwindow.workspacebrowser().save_to_settings(&appwindow.app_settings());
                    }
                }
                _ => {}
            }

            dialog_modify_workspace.close();
        }));

    change_workspace_dir_button.connect_clicked(
        clone!(@weak dialog_edit_workspace, @weak filechooser_change_workspace_dir, @weak appwindow => move |_| {
            dialog_edit_workspace.hide();
            filechooser_change_workspace_dir.show();
        }),
    );

    dialog_edit_workspace.show();
    *appwindow.filechoosernative().borrow_mut() = Some(filechooser_change_workspace_dir);
}

// FileChooser Dialogs

pub fn filechooser_open_doc(appwindow: &RnoteAppWindow) {
    let filter = FileFilter::new();
    filter.add_mime_type("application/rnote");
    filter.add_mime_type("application/x-xopp");
    filter.add_pattern("*.rnote");
    filter.set_name(Some(&gettext(".rnote / .xopp File")));

    let filechooser: FileChooserNative = FileChooserNative::builder()
        .title(&gettext("Open file"))
        .modal(true)
        .transient_for(appwindow)
        .accept_label(&gettext("Open"))
        .cancel_label(&gettext("Cancel"))
        .action(FileChooserAction::Open)
        .select_multiple(false)
        .build();

    filechooser.add_filter(&filter);

    if let Some(current_workspace_dir) = appwindow.workspacebrowser().selected_workspace_dir() {
        if let Err(e) =
            filechooser.set_current_folder(Some(&gio::File::for_path(current_workspace_dir)))
        {
            log::error!("set_current_folder() for dialog_open_doc failed with Err `{e}`");
        }
    }

    filechooser.connect_response(clone!(@weak appwindow => move |filechooser, responsetype| {
            match responsetype {
                ResponseType::Accept => {
                    if let Some(file) = filechooser.file() {
                        appwindow.app().set_input_file(Some(file));

                        if !appwindow.unsaved_changes() {
                            if let Some(input_file) = appwindow.app().input_file().as_ref() {
                                if let Err(e) = appwindow.load_in_file(input_file, None) {
                                    log::error!("failed to load in input file, {}", e);
                                    adw::prelude::ActionGroupExt::activate_action(&appwindow, "error-toast", Some(&gettext("Opening file failed.").to_variant()));
                                }
                            }
                        } else {
                            // Open a dialog to ask for overwriting the current doc
                            dialog_open_overwrite(&appwindow);
                        }
                    }
                },
                _ => {
                }
            }

        }));

    filechooser.show();

    // keeping the filechooser around because otherwise GTK won't keep it alive
    *appwindow.filechoosernative().borrow_mut() = Some(filechooser);
}

pub fn filechooser_save_doc_as(appwindow: &RnoteAppWindow) {
    let filter = FileFilter::new();
    filter.add_mime_type("application/rnote");
    filter.add_pattern("*.rnote");
    filter.set_name(Some(&gettext(".rnote file")));

    let filechooser: FileChooserNative = FileChooserNative::builder()
        .title(&gettext("Save document as"))
        .modal(true)
        .transient_for(appwindow)
        .accept_label(&gettext("Save"))
        .cancel_label(&gettext("Cancel"))
        .action(FileChooserAction::Save)
        .select_multiple(false)
        .build();

    filechooser.add_filter(&filter);

    // Set the output file as default, else at least the current workspace directory
    if let Some(output_file) = appwindow.canvas().output_file() {
        if let Err(e) = filechooser.set_file(&output_file) {
            log::error!("set_file() for dialog_save_doc_as failed with Err `{e}`");
        }
    } else {
        if let Some(current_workspace_dir) = appwindow.workspacebrowser().selected_workspace_dir() {
            if let Err(e) =
                filechooser.set_current_folder(Some(&gio::File::for_path(current_workspace_dir)))
            {
                log::error!("set_current_folder() for dialog_save_doc_as failed with Err `{e}`");
            }
        }

        let file_title = rnote_engine::utils::default_file_title_for_export(
            appwindow.canvas().output_file(),
            Some(&appwindow::OUTPUT_FILE_NEW_TITLE),
            None,
        );

        filechooser.set_current_name(&(file_title + "." + "rnote"));
    }

    filechooser.connect_response(
        clone!(@weak appwindow => move |filechooser, responsetype| {
            match responsetype {
                ResponseType::Accept => {
                    if let Some(file) = filechooser.file() {
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

    filechooser.show();
    // keeping the filechooser around because otherwise GTK won't keep it alive
    *appwindow.filechoosernative().borrow_mut() = Some(filechooser);
}

pub fn filechooser_import_file(appwindow: &RnoteAppWindow) {
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

    let filechooser: FileChooserNative = FileChooserNative::builder()
        .title(&gettext("Import file"))
        .modal(true)
        .transient_for(appwindow)
        .accept_label(&gettext("Import"))
        .cancel_label(&gettext("Cancel"))
        .action(FileChooserAction::Open)
        .select_multiple(false)
        .build();

    filechooser.add_filter(&filter);

    if let Some(current_workspace_dir) = appwindow.workspacebrowser().selected_workspace_dir() {
        if let Err(e) =
            filechooser.set_current_folder(Some(&gio::File::for_path(current_workspace_dir)))
        {
            log::error!("set_current_folder() for dialog_import_file failed with Err `{e}`");
        }
    }

    filechooser.connect_response(clone!(@weak appwindow => move |filechooser, responsetype| {
        match responsetype {
            ResponseType::Accept => {
                if let Some(file) = filechooser.file() {
                    appwindow.open_file_w_dialogs(&file, None);
                }
            }
            _ => {
            }
        }
    }));

    filechooser.show();
    // keeping the filechooser around because otherwise GTK won't keep it alive
    *appwindow.filechoosernative().borrow_mut() = Some(filechooser);
}

pub fn filechooser_export_doc(appwindow: &RnoteAppWindow) {
    let filter = FileFilter::new();
    filter.add_mime_type("image/svg+xml");
    filter.add_pattern("*.svg");
    filter.add_mime_type("application/pdf");
    filter.add_pattern("*.pdf");
    filter.add_mime_type("application/x-xopp");
    filter.add_pattern("*.xopp");
    filter.set_name(Some(&gettext("SVG / PDF / .xopp")));

    let filechooser: FileChooserNative = FileChooserNative::builder()
        .title(&gettext("Export document"))
        .modal(true)
        .transient_for(appwindow)
        .accept_label(&gettext("Export"))
        .cancel_label(&gettext("Cancel"))
        .action(FileChooserAction::Save)
        .select_multiple(false)
        .build();
    filechooser.add_filter(&filter);

    if let Some(current_workspace_dir) = appwindow.workspacebrowser().selected_workspace_dir() {
        if let Err(e) =
            filechooser.set_current_folder(Some(&gio::File::for_path(current_workspace_dir)))
        {
            log::error!("set_current_folder() for dialog_export_as_svg failed with Err `{e}`");
        }
    }

    let file_ext = appwindow
        .canvas()
        .engine()
        .borrow()
        .export_prefs
        .doc_export_prefs
        .export_format
        .file_ext();

    let file_title = rnote_engine::utils::default_file_title_for_export(
        appwindow.canvas().output_file(),
        Some(&appwindow::OUTPUT_FILE_NEW_TITLE),
        None,
    );

    filechooser.set_current_name(&(file_title.clone() + "." + &file_ext));

    filechooser.connect_response(
        clone!(@weak appwindow => move |dialog_export_doc, responsetype| {
            let file_title = file_title.clone();

            match responsetype {
                ResponseType::Accept => {
                    if let Some(file) = dialog_export_doc.file() {
                        glib::MainContext::default().spawn_local(clone!(@strong appwindow => async move {
                            appwindow.start_pulsing_canvas_progressbar();

                            if let Err(e) = appwindow.export_doc(&file, file_title, None).await {
                                log::error!("exporting document failed with error `{}`", e);
                                adw::prelude::ActionGroupExt::activate_action(&appwindow, "error-toast", Some(&gettext("Export document failed.").to_variant()));
                            } else {
                                adw::prelude::ActionGroupExt::activate_action(&appwindow, "text-toast", Some(&gettext("Exported document successfully.").to_variant()));
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

    filechooser.show();
    // keeping the filechooser around because otherwise GTK won't keep it alive
    *appwindow.filechoosernative().borrow_mut() = Some(filechooser);
}

pub fn filechooser_export_engine_state(appwindow: &RnoteAppWindow) {
    let filter = FileFilter::new();
    filter.add_mime_type("application/json");
    filter.add_pattern("*.json");
    filter.set_name(Some(&gettext("JSON file")));

    let filechooser: FileChooserNative = FileChooserNative::builder()
        .title(&gettext("Export engine state"))
        .modal(true)
        .transient_for(appwindow)
        .accept_label(&gettext("Export"))
        .cancel_label(&gettext("Cancel"))
        .action(FileChooserAction::Save)
        .select_multiple(false)
        .build();
    filechooser.add_filter(&filter);

    if let Some(current_workspace_dir) = appwindow.workspacebrowser().selected_workspace_dir() {
        if let Err(e) =
            filechooser.set_current_folder(Some(&gio::File::for_path(current_workspace_dir)))
        {
            log::error!(
                "set_current_folder() for dialog_export_engine_state failed with Err `{e}`"
            );
        }
    }

    let file_title = rnote_engine::utils::default_file_title_for_export(
        appwindow.canvas().output_file(),
        Some(&appwindow::OUTPUT_FILE_NEW_TITLE),
        Some(" - engine state"),
    );

    filechooser.set_current_name(&(file_title + "." + "json"));

    filechooser.connect_response(
        clone!(@weak appwindow => move |filechooser, responsetype| {
            match responsetype {
                ResponseType::Accept => {
                    if let Some(file) = filechooser.file() {
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

    filechooser.show();
    // keeping the filechooser around because otherwise GTK won't keep it alive
    *appwindow.filechoosernative().borrow_mut() = Some(filechooser);
}

pub fn filechooser_export_engine_config(appwindow: &RnoteAppWindow) {
    let filter = FileFilter::new();
    filter.add_mime_type("application/json");
    filter.add_pattern("*.json");
    filter.set_name(Some(&gettext("JSON file")));

    let filechooser: FileChooserNative = FileChooserNative::builder()
        .title(&gettext("Export engine config"))
        .modal(true)
        .transient_for(appwindow)
        .accept_label(&gettext("Export"))
        .cancel_label(&gettext("Cancel"))
        .action(FileChooserAction::Save)
        .select_multiple(false)
        .build();
    filechooser.add_filter(&filter);

    if let Some(current_workspace_dir) = appwindow.workspacebrowser().selected_workspace_dir() {
        if let Err(e) =
            filechooser.set_current_folder(Some(&gio::File::for_path(current_workspace_dir)))
        {
            log::error!(
                "set_current_folder() for dialog_export_engine_config failed with Err `{e}`"
            );
        }
    }

    let file_title = rnote_engine::utils::default_file_title_for_export(
        appwindow.canvas().output_file(),
        Some(&appwindow::OUTPUT_FILE_NEW_TITLE),
        Some(" - engine config"),
    );

    filechooser.set_current_name(&(file_title + "." + "json"));

    filechooser.connect_response(
        clone!(@weak appwindow => move |filechooser, responsetype| {
            match responsetype {
                ResponseType::Accept => {
                    if let Some(file) = filechooser.file() {
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

    filechooser.show();
    // keeping the filechooser around because otherwise GTK won't keep it alive
    *appwindow.filechoosernative().borrow_mut() = Some(filechooser);
}
