//adw::ToolbarView is a replacement for adw::Dialog but not suitable for an async flow

// Imports
use crate::canvas::RnCanvas;
use crate::{RnAppWindow, config};
use adw::prelude::*;
use anyhow::anyhow;
use futures::StreamExt;
use gettextrs::gettext;
use gtk4::{Builder, Button, FileDialog, FileFilter, Label, ToggleButton, gio, glib, glib::clone};
use gtk4::{graphene, gsk};
use hayro::hayro_syntax;
use num_traits::ToPrimitive;
use rnote_engine::engine::import::{PdfImportPageSpacing, PdfImportPagesType};
use std::sync::Arc;
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

    let animation = adw::SpringAnimation::builder()
        .widget(&pdf_password_entry_box)
        .value_from(0.0)
        .value_to(0.0)
        .spring_params(&adw::SpringParams::new(0.2, 0.5, 500.0))
        .target(&target)
        .initial_velocity(10.0)
        .epsilon(0.001) // If amplitude of oscillation < epsilon, animation stops
        .clamp(false)
        .build();

    let (tx, mut rx) = futures::channel::mpsc::unbounded::<(Option<String>, bool)>();

    dialog_import_pdf_password.connect_response(
        Some("unlock"),
        clone!(
            #[weak]
            pdf_password_entry,
            #[strong]
            tx,
            move |_, _| {
                tx.unbounded_send((Some(pdf_password_entry.text().to_string()), false))
                    .unwrap();
            }
        ),
    );

    dialog_import_pdf_password.connect_response(
        Some("cancel"),
        clone!(
            #[strong]
            tx,
            move |_, _| {
                tx.unbounded_send((None, true)).unwrap();
            }
        ),
    );

    let file_name = input_file.basename().map_or_else(
        || gettext("- no file name -"),
        |s| s.to_string_lossy().to_string(),
    );
    let dialog_body = dialog_import_pdf_password.body();
    let dialog_body = file_name.clone() + " " + &dialog_body;
    dialog_import_pdf_password.set_body(&dialog_body);

    let mut password: Option<String> = None;
    let pdf_data = match input_file.load_bytes_future().await {
        Ok(data) => Arc::new(data.0.to_vec()),
        Err(err) => {
            error!("Loading bytes from file failed, Err: {err:?}");
            return (None, true);
        }
    };

    loop {
        let pdf_res = if let Some(password) = password.as_ref() {
            hayro_syntax::Pdf::new_with_password(pdf_data.clone(), password)
        } else {
            hayro_syntax::Pdf::new(pdf_data.clone())
        };
        match pdf_res {
            Ok(_) => return (password, false),
            Err(hayro_syntax::LoadPdfError::Decryption(
                hayro_syntax::DecryptionError::PasswordProtected,
            )) => {
                dialog_import_pdf_password.present(appwindow.root().as_ref());
                pdf_password_entry.grab_focus();

                match rx.next().await {
                    Some((new_password, cancel)) => {
                        if cancel {
                            return (None, true);
                        }
                        password = new_password;
                    }
                    None => {
                        return (None, true);
                    }
                }
                animation.play();
                pdf_password_entry.set_text("");
            }
            Err(err) => {
                error!("Creating Pdf instance failed, Err: {err:?}");
                return (None, true);
            }
        }
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

    let pdf_data = Arc::new(input_file.load_bytes_future().await?.0.to_vec());
    let pdf = if let Some(password) = password.as_ref() {
        hayro_syntax::Pdf::new_with_password(pdf_data, password)
            .map_err(|err| anyhow!("Creating Pdf instance failed, Err: {err:?}"))?
    } else {
        hayro_syntax::Pdf::new(pdf_data)
            .map_err(|err| anyhow!("Creating Pdf instance failed, Err: {err:?}"))?
    };
    let pdf_metadata = pdf.metadata();

    let file_name = input_file.basename().map_or_else(
        || gettext("- no file name -"),
        |s| s.to_string_lossy().to_string(),
    );
    let title = pdf_metadata
        .title
        .to_owned()
        .and_then(|s| String::from_utf8(s).ok())
        .unwrap_or_else(|| gettext("- no title -"));
    let author = pdf_metadata
        .author
        .to_owned()
        .and_then(|s| String::from_utf8(s).ok())
        .unwrap_or_else(|| gettext("- no author -"));
    let mod_date = pdf_metadata
        .modification_date
        .and_then(|dt| {
            let dt = rnote_engine::utils::chrono_dt_from_hayro(dt)?;
            Some(dt.to_rfc3339())
        })
        .unwrap_or_else(|| gettext("- no modification date -"));
    let n_pages = pdf.pages().len();

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
    pdf_page_start_row.set_range(1.into(), n_pages as f64);
    pdf_page_start_row.set_value(1.into());

    pdf_page_end_row.set_range(1.into(), n_pages as f64);
    pdf_page_end_row.set_value(n_pages as f64);

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
                    #[weak]
                    appwindow,
                    #[weak]
                    canvas,
                    async move {
                        let page_range = (pdf_page_start_row.value() as usize).saturating_sub(1)
                            ..pdf_page_end_row.value() as usize;
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
