//adw::ToolbarView is a replacement for adw::Dialog but not suitable for an async flow

// Imports
use crate::canvas::RnCanvas;
use crate::{RnAppWindow, config};
use adw::prelude::*;
use futures::StreamExt;
use gettextrs::gettext;
use gtk4::{
    Builder, Button, FileDialog, FileFilter, Label, ToggleButton, gio, glib, glib::clone, graphene,
    gsk,
};
use num_traits::ToPrimitive;
use rnote_engine::engine::import::{PdfImportPageSpacing, PdfImportPagesType};
use tracing::{debug, error};

/// Opens a new rnote save file in a new tab
pub(crate) async fn filedialog_open_doc(appwindow: &RnAppWindow) {
    let filter = FileFilter::new();
    // note : mimetypes are not supported with the native file picker on windows
    // See the limitations on FileChooserNative
    // https://gtk-rs.org/gtk3-rs/stable/latest/docs/gtk/struct.FileChooserNative.html#win32-details--gtkfilechooserdialognative-win32
    if cfg!(target_os = "windows") {
        filter.add_pattern("*.rnote");
    } else {
        filter.add_mime_type("application/rnote");
    }
    filter.add_suffix("rnote");
    filter.set_name(Some(&gettext(".rnote")));

    let filter_list = gio::ListStore::new::<FileFilter>();
    filter_list.append(&filter);

    let filedialog = FileDialog::builder()
        .title(gettext("Open File"))
        .modal(true)
        .accept_label(gettext("Open"))
        .filters(&filter_list)
        .default_filter(&filter)
        .build();

    if let Some(current_workspace_dir) = appwindow.sidebar().workspacebrowser().dir_list_dir() {
        filedialog.set_initial_folder(Some(&gio::File::for_path(current_workspace_dir)));
    }

    match filedialog.open_future(Some(appwindow)).await {
        Ok(selected_file) => {
            appwindow
                .open_file_w_dialogs(selected_file, None, true)
                .await;
        }
        Err(e) => {
            debug!("Did not open document (Error or dialog dismissed by user), Err: {e:?}");
        }
    }
}

pub(crate) async fn filedialog_import_file(appwindow: &RnAppWindow) {
    let filter = FileFilter::new();
    // note : mimetypes are not supported with the native file picker on windows
    // See the limitations on FileChooserNative
    // https://gtk-rs.org/gtk3-rs/stable/latest/docs/gtk/struct.FileChooserNative.html#win32-details--gtkfilechooserdialognative-win32
    if cfg!(target_os = "windows") {
        filter.add_pattern("*.xopp");
        filter.add_pattern("*.pdf");
        filter.add_pattern("*.svg");
        filter.add_pattern("*.png");
        filter.add_pattern("*.jpeg");
        filter.add_pattern("*.txt");
    } else {
        filter.add_mime_type("application/x-xopp");
        filter.add_mime_type("application/pdf");
        filter.add_mime_type("image/svg+xml");
        filter.add_mime_type("image/png");
        filter.add_mime_type("image/jpeg");
        filter.add_mime_type("text/plain");
    }
    filter.add_suffix("xopp");
    filter.add_suffix("pdf");
    filter.add_suffix("svg");
    filter.add_suffix("png");
    filter.add_suffix("jpg");
    filter.add_suffix("jpeg");
    filter.add_suffix("txt");
    filter.set_name(Some(&gettext("Jpg, Pdf, Png, Svg, Xopp, Txt")));

    let filter_list = gio::ListStore::new::<FileFilter>();
    filter_list.append(&filter);

    let dialog = FileDialog::builder()
        .title(gettext("Import File"))
        .modal(true)
        .accept_label(gettext("Import"))
        .filters(&filter_list)
        .default_filter(&filter)
        .build();

    if let Some(current_workspace_dir) = appwindow.sidebar().workspacebrowser().dir_list_dir() {
        dialog.set_initial_folder(Some(&gio::File::for_path(current_workspace_dir)));
    }

    match dialog.open_future(Some(appwindow)).await {
        Ok(selected_file) => {
            appwindow
                .open_file_w_dialogs(selected_file, None, true)
                .await;
        }
        Err(e) => {
            debug!("Did not import file (Error or dialog dismissed by user), Err: {e:?}");
        }
    }
}

/// Check for a pdf encryption and request a password if needed from the user
///
/// Returns a password Option and a boolean weather the user canceled the file import or not
pub(crate) async fn pdf_encryption_check_and_dialog(
    appwindow: &RnAppWindow,
    input_file: &gio::File,
) -> (Option<String>, bool) {
    let builder = Builder::from_resource(
        (String::from(config::APP_IDPATH) + "ui/dialogs/import.ui").as_str(),
    );

    let dialog_import_pdf_password: adw::AlertDialog =
        builder.object("dialog_import_pdf_password").unwrap();
    let pdf_password_entry: adw::PasswordEntryRow = builder.object("pdf_password_entry").unwrap();
    let pdf_password_entry_box: gtk4::ListBox = builder.object("pdf_password_entry_box").unwrap();

    let target = adw::CallbackAnimationTarget::new(clone!(
        #[weak]
        pdf_password_entry_box,
        move |value| {
            let x = adw::lerp(0., 40.0, value);
            let p = graphene::Point::new(x as f32, 0.);
            let transform = gsk::Transform::new().translate(&p);
            pdf_password_entry_box.allocate(
                pdf_password_entry_box.width(),
                pdf_password_entry_box.height(),
                -1,
                Some(transform),
            );
        }
    ));

    let params = adw::SpringParams::new(0.2, 0.5, 500.0);

    let animation = adw::SpringAnimation::builder()
        .widget(&pdf_password_entry_box)
        .value_from(0.0)
        .value_to(0.0)
        .spring_params(&params)
        .target(&target)
        .initial_velocity(10.0)
        .epsilon(0.001) // If amplitude of oscillation < epsilon, animation stops
        .clamp(false)
        .build();

    let (tx, mut rx) = futures::channel::mpsc::unbounded::<(Option<String>, bool)>();
    let tx_cancel = tx.clone();
    let tx_unlock = tx.clone();

    dialog_import_pdf_password.connect_response(
        Some("unlock"),
        clone!(
            #[weak]
            pdf_password_entry,
            move |_, _| {
                tx_unlock
                    .unbounded_send((Some(pdf_password_entry.text().to_string()), false))
                    .unwrap();
            }
        ),
    );

    dialog_import_pdf_password.connect_response(Some("cancel"), move |_, _| {
        tx_cancel.unbounded_send((None, true)).unwrap();
    });

    let file_name = input_file.basename().map_or_else(
        || gettext("- no file name -"),
        |s| s.to_string_lossy().to_string(),
    );
    let dialog_body = dialog_import_pdf_password.body();
    let dialog_body = file_name.clone() + " " + &dialog_body;
    dialog_import_pdf_password.set_body(&dialog_body);

    let mut password: Option<String> = None;

    loop {
        match poppler::Document::from_gfile(
            input_file,
            password.as_deref(),
            None::<&gio::Cancellable>,
        ) {
            Ok(_) => return (password, false),
            Err(e) => {
                if e.matches(poppler::Error::Encrypted) {
                    dialog_import_pdf_password.present(appwindow.root().as_ref());
                    pdf_password_entry.grab_focus();

                    match rx.next().await {
                        Some((new_password, cancel)) => {
                            password = new_password;
                            if cancel {
                                return (None, true);
                            }
                        }
                        None => {
                            return (None, true);
                        }
                    }
                    animation.play();
                    pdf_password_entry.set_text("");
                } else {
                    return (None, true);
                }
            }
        };
    }
}

/// Imports the file as Pdf with an import dialog.
///
/// Returns true when the file was imported, else false.
pub(crate) async fn dialog_import_pdf_w_prefs(
    appwindow: &RnAppWindow,
    canvas: &RnCanvas,
    input_file: gio::File,
    target_pos: Option<na::Vector2<f64>>,
) -> anyhow::Result<bool> {
    let (password, cancel) = pdf_encryption_check_and_dialog(appwindow, &input_file).await;
    if cancel {
        return Ok(false);
    }

    let builder = Builder::from_resource(
        (String::from(config::APP_IDPATH) + "ui/dialogs/import.ui").as_str(),
    );
    let dialog: adw::Dialog = builder.object("dialog_import_pdf_w_prefs").unwrap();
    let pdf_page_start_row: adw::SpinRow = builder.object("pdf_page_start_row").unwrap();
    let pdf_page_end_row: adw::SpinRow = builder.object("pdf_page_end_row").unwrap();
    let pdf_info_label: Label = builder.object("pdf_info_label").unwrap();
    let pdf_import_width_row: adw::SpinRow = builder.object("pdf_import_width_row").unwrap();
    let pdf_import_page_spacing_row: adw::ComboRow =
        builder.object("pdf_import_page_spacing_row").unwrap();
    let pdf_import_as_bitmap_toggle: ToggleButton =
        builder.object("pdf_import_as_bitmap_toggle").unwrap();
    let pdf_import_as_vector_toggle: ToggleButton =
        builder.object("pdf_import_as_vector_toggle").unwrap();
    let pdf_import_bitmap_scalefactor_row: adw::SpinRow =
        builder.object("pdf_import_bitmap_scalefactor_row").unwrap();
    let pdf_import_page_borders_row: adw::SwitchRow =
        builder.object("pdf_import_page_borders_row").unwrap();
    let pdf_import_adjust_document_row: adw::SwitchRow =
        builder.object("pdf_import_adjust_document_row").unwrap();
    let import_pdf_button_cancel: Button = builder.object("import_pdf_button_cancel").unwrap();
    let import_pdf_button_confirm: Button = builder.object("import_pdf_button_confirm").unwrap();

    pdf_import_adjust_document_row
        .bind_property("active", &pdf_import_width_row, "sensitive")
        .invert_boolean()
        .sync_create()
        .build();
    pdf_import_adjust_document_row
        .bind_property("active", &pdf_import_page_spacing_row, "sensitive")
        .invert_boolean()
        .sync_create()
        .build();

    let pdf_import_prefs = appwindow
        .engine_config()
        .read()
        .import_prefs
        .pdf_import_prefs;

    // Set the widget state from the pdf import prefs
    pdf_import_width_row.set_value(pdf_import_prefs.page_width_perc);
    match pdf_import_prefs.pages_type {
        PdfImportPagesType::Bitmap => {
            pdf_import_as_bitmap_toggle.set_active(true);
            pdf_import_bitmap_scalefactor_row.set_sensitive(true);
        }
        PdfImportPagesType::Vector => {
            pdf_import_as_vector_toggle.set_active(true);
            pdf_import_bitmap_scalefactor_row.set_sensitive(false);
        }
    }
    pdf_import_page_spacing_row.set_selected(pdf_import_prefs.page_spacing.to_u32().unwrap());
    pdf_import_bitmap_scalefactor_row.set_value(pdf_import_prefs.bitmap_scalefactor);
    pdf_import_page_borders_row.set_active(pdf_import_prefs.page_borders);
    pdf_import_adjust_document_row.set_active(pdf_import_prefs.adjust_document);

    pdf_page_start_row
        .bind_property("value", &pdf_page_end_row.adjustment(), "lower")
        .sync_create()
        .build();
    pdf_page_end_row
        .bind_property("value", &pdf_page_start_row.adjustment(), "upper")
        .sync_create()
        .build();

    // Update preferences
    pdf_import_as_vector_toggle.connect_toggled(clone!(
        #[weak]
        pdf_import_bitmap_scalefactor_row,
        #[weak]
        appwindow,
        move |toggle| {
            if !toggle.is_active() {
                return;
            }
            appwindow
                .engine_config()
                .write()
                .import_prefs
                .pdf_import_prefs
                .pages_type = PdfImportPagesType::Vector;
            pdf_import_bitmap_scalefactor_row.set_sensitive(false);
        }
    ));

    pdf_import_as_bitmap_toggle.connect_toggled(clone!(
        #[weak]
        pdf_import_bitmap_scalefactor_row,
        #[weak]
        appwindow,
        move |toggle| {
            if !toggle.is_active() {
                return;
            }
            appwindow
                .engine_config()
                .write()
                .import_prefs
                .pdf_import_prefs
                .pages_type = PdfImportPagesType::Bitmap;
            pdf_import_bitmap_scalefactor_row.set_sensitive(true);
        }
    ));

    pdf_import_bitmap_scalefactor_row.connect_changed(clone!(
        #[weak]
        appwindow,
        move |row| {
            appwindow
                .engine_config()
                .write()
                .import_prefs
                .pdf_import_prefs
                .bitmap_scalefactor = row.value();
        }
    ));

    pdf_import_page_spacing_row.connect_selected_notify(clone!(
        #[weak]
        appwindow,
        move |row| {
            let page_spacing = PdfImportPageSpacing::try_from(row.selected()).unwrap();
            appwindow
                .engine_config()
                .write()
                .import_prefs
                .pdf_import_prefs
                .page_spacing = page_spacing;
        }
    ));

    pdf_import_width_row.connect_changed(clone!(
        #[weak]
        appwindow,
        move |row| {
            appwindow
                .engine_config()
                .write()
                .import_prefs
                .pdf_import_prefs
                .page_width_perc = row.value();
        }
    ));

    pdf_import_page_borders_row.connect_active_notify(clone!(
        #[weak]
        appwindow,
        move |row| {
            appwindow
                .engine_config()
                .write()
                .import_prefs
                .pdf_import_prefs
                .page_borders = row.is_active();
        }
    ));

    pdf_import_adjust_document_row.connect_active_notify(clone!(
        #[weak]
        appwindow,
        move |row| {
            appwindow
                .engine_config()
                .write()
                .import_prefs
                .pdf_import_prefs
                .adjust_document = row.is_active();
        }
    ));

    if let Ok(poppler_doc) =
        poppler::Document::from_gfile(&input_file, password.as_deref(), None::<&gio::Cancellable>)
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
            &format!("<b>{}  </b>{file_name}\n<b>{}  </b>{title}\n<b>{}  </b>{author}\n<b>{}  </b>{mod_date}\n<b>{}  </b>{n_pages}\n",
                &gettext("File name:"),
                &gettext("Title:"),
                &gettext("Author:"),
                &gettext("Modification date:"),
                &gettext("Pages:"))
        );

        // Configure pages spinners
        pdf_page_start_row.set_range(1.into(), n_pages.into());
        pdf_page_start_row.set_value(1.into());

        pdf_page_end_row.set_range(1.into(), n_pages.into());
        pdf_page_end_row.set_value(n_pages.into());
    }

    // Listen to responses

    let (tx, mut rx_confirm) = futures::channel::mpsc::unbounded::<(bool, bool)>();
    let tx_cancel = tx.clone();
    let tx_confirm = tx.clone();
    let tx_close = tx.clone();

    import_pdf_button_cancel.connect_clicked(clone!(move |_| {
        if let Err(e) = tx_cancel.unbounded_send((false, false)) {
            error!(
                "PDF import dialog closed, but failed to send signal through channel. Err: {e:?}"
            );
        }
    }));

    import_pdf_button_confirm.connect_clicked(clone!(move |_| {
        if let Err(e) = tx_confirm.unbounded_send((true, false)) {
            error!("PDF file imported, but failed to send signal through channel. Err: {e:?}");
        }
    }));

    // Send a cancel response when the dialog is closed
    dialog.connect_closed(clone!(move |_| {
        if let Err(e) = tx_close.unbounded_send((false, true)) {
            error!(
                "PDF import dialog closed, but failed to send signal through channel. Err: {e:?}"
            );
        }
    }));

    // Present than wait for a response from the dialog
    dialog.present(appwindow.root().as_ref());

    match rx_confirm.next().await {
        Some((confirm, dialog_closed)) => {
            if !dialog_closed {
                dialog.close();
            }
            if confirm {
                let (tx_import, mut rx_import) =
                    futures::channel::mpsc::unbounded::<anyhow::Result<bool>>();

                glib::spawn_future_local(clone!(
                    #[weak]
                    pdf_page_start_row,
                    #[weak]
                    pdf_page_end_row,
                    #[weak]
                    input_file,
                    #[strong]
                    password,
                    #[weak]
                    appwindow,
                    #[weak]
                    canvas,
                    async move {
                        let page_range = (pdf_page_start_row.value() as u32 - 1)
                            ..pdf_page_end_row.value() as u32;
                        let (bytes, _) = match input_file.load_bytes_future().await {
                            Ok(res) => res,
                            Err(err) => {
                                if let Err(e) = tx_import.unbounded_send(Err(err.into())) {
                                    error!(
                                        "Failed to load file, but failed to send signal through channel. Err: {e:?}"
                                    );
                                }
                                return;
                            }
                        };

                        if let Err(e) = canvas
                            .load_in_pdf_bytes(
                                &appwindow,
                                bytes.to_vec(),
                                target_pos,
                                Some(page_range),
                                password,
                            )
                            .await
                            && let Err(e) = tx_import.unbounded_send(Err(e))
                        {
                            error!(
                                "Failed to load PDF, but failed to send signal through channel. Err: {e:?}"
                            );
                            return;
                        };

                        if let Err(e) = tx_import.unbounded_send(Ok(true)) {
                            error!(
                                "PDF file imported, but failed to send signal through channel. Err: {e:?}"
                            );
                        }
                    }
                ));

                match rx_import.next().await {
                    Some(res) => res,
                    None => Err(anyhow::anyhow!(
                        "Channel closed before receiving a response from loader thread."
                    )),
                }
            } else {
                Ok(false)
            }
        }
        None => Err(anyhow::anyhow!(
            "Channel closed before receiving a response from dialog."
        )),
    }
}

/// Imports the file as Xopp with an import dialog.
///
/// Returns true when the file was imported, else false.
pub(crate) async fn dialog_import_xopp_w_prefs(
    appwindow: &RnAppWindow,
    canvas: &RnCanvas,
    input_file: gio::File,
) -> anyhow::Result<bool> {
    let builder = Builder::from_resource(
        (String::from(config::APP_IDPATH) + "ui/dialogs/import.ui").as_str(),
    );
    let dialog: adw::Dialog = builder.object("dialog_import_xopp_w_prefs").unwrap();
    let dpi_row: adw::SpinRow = builder.object("xopp_import_dpi_row").unwrap();
    let xopp_import_prefs = appwindow
        .engine_config()
        .read()
        .import_prefs
        .xopp_import_prefs;
    let import_xopp_button_cancel: Button = builder.object("import_xopp_button_cancel").unwrap();
    let import_xopp_button_confirm: Button = builder.object("import_xopp_button_confirm").unwrap();

    // Set initial widget state for preference
    dpi_row.set_value(xopp_import_prefs.dpi);

    // Update preferences
    dpi_row.connect_changed(clone!(
        #[weak]
        appwindow,
        move |row| {
            appwindow
                .engine_config()
                .write()
                .import_prefs
                .xopp_import_prefs
                .dpi = row.value();
        }
    ));

    // Listen to responses

    let (tx, mut rx_confirm) = futures::channel::mpsc::unbounded::<(bool, bool)>();
    let tx_cancel = tx.clone();
    let tx_confirm = tx.clone();
    let tx_close = tx.clone();

    import_xopp_button_cancel.connect_clicked(clone!(move |_| {
        if let Err(e) = tx_cancel.unbounded_send((false, false)) {
            error!(
                "XOPP import dialog cancelled, but failed to send signal through channel. Err: {e:?}"
            );
        }
    }));

    import_xopp_button_confirm.connect_clicked(clone!(move |_| {
        if let Err(e) = tx_confirm.unbounded_send((true,false)) {
            error!(
                "Xopp import dialog confirmed, but failed to send signal through channel. Err: {e:?}"
            );
        }
    }));

    // Send a cancel response when the dialog is closed
    dialog.connect_closed(clone!(move |_| {
        if let Err(e) = tx_close.unbounded_send((false, true)) {
            error!(
                "XOPP import dialog closed, but failed to send signal through channel. Err: {e:?}"
            );
        }
    }));

    // Present than wait for a response from the dialog
    dialog.present(appwindow.root().as_ref());

    match rx_confirm.next().await {
        Some((confirm, dialog_closed)) => {
            if !dialog_closed {
                dialog.close();
            }
            if confirm {
                let (tx_import, mut rx_import) =
                    futures::channel::mpsc::unbounded::<anyhow::Result<bool>>();

                glib::spawn_future_local(clone!(
                    #[weak]
                    input_file,
                    #[weak]
                    appwindow,
                    #[weak]
                    canvas,
                    async move {
                        let (bytes, _) = match input_file.load_bytes_future().await {
                            Ok(res) => res,
                            Err(err) => {
                                if let Err(e) = tx_import.unbounded_send(Err(err.into())) {
                                    error!(
                                        "Failed to load file, but failed to send signal through channel. Err: {e:?}"
                                    );
                                }
                                return;
                            }
                        };
                        if let Err(e) = canvas.load_in_xopp_bytes(&appwindow, bytes.to_vec()).await
                        {
                            if let Err(e) = tx_import.unbounded_send(Err(e)) {
                                error!(
                                    "Failed to load XOPP, but failed to send signal through channel. Err: {e:?}"
                                );
                            }
                            return;
                        };

                        if let Err(e) = tx_import.unbounded_send(Ok(true)) {
                            error!(
                                "XOPP file imported, but failed to send signal through channel. Err: {e:?}"
                            );
                        }
                    }
                ));

                match rx_import.next().await {
                    Some(res) => res,
                    None => Err(anyhow::anyhow!(
                        "Channel closed before receiving a response from loader thread."
                    )),
                }
            } else {
                Ok(false)
            }
        }
        None => Err(anyhow::anyhow!(
            "Channel closed before receiving a response from dialog."
        )),
    }
}
