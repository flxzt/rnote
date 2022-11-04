use std::ops::Range;
use std::path::Path;

use gettextrs::gettext;
use gtk4::{gio, glib, glib::clone, prelude::*};
use rnote_engine::engine::export::{DocExportPrefs, SelectionExportPrefs};
use rnote_engine::strokes::{BitmapImage, Stroke, VectorImage};

use crate::dialogs;

use super::RnoteAppWindow;

impl RnoteAppWindow {
    pub fn open_file_w_dialogs(&self, file: &gio::File, target_pos: Option<na::Vector2<f64>>) {
        let app = self.app();

        match crate::utils::FileType::lookup_file_type(file) {
            crate::utils::FileType::RnoteFile | crate::utils::FileType::XoppFile => {
                // Set as input file to hand it to the dialog
                app.set_input_file(Some(file.clone()));

                if self.unsaved_changes() {
                    dialogs::import::dialog_open_overwrite(self);
                } else if let Err(e) = self.load_in_file(file, target_pos) {
                    log::error!(
                        "failed to load in file with FileType::RnoteFile | FileType::XoppFile, {}",
                        e
                    );
                }
            }
            crate::utils::FileType::VectorImageFile | crate::utils::FileType::BitmapImageFile => {
                if let Err(e) = self.load_in_file(file, target_pos) {
                    log::error!("failed to load in file with FileType::VectorImageFile / FileType::BitmapImageFile / FileType::Pdf, {}", e);
                }
            }
            crate::utils::FileType::PdfFile => {
                // Set as input file to hand it to the dialog
                app.set_input_file(Some(file.clone()));

                dialogs::import::dialog_import_pdf_w_prefs(self, target_pos);
            }
            crate::utils::FileType::Folder => {
                if let Some(dir) = file.path() {
                    self.workspacebrowser().set_current_workspace_dir(dir);
                }
            }
            crate::utils::FileType::Unsupported => {
                log::error!("tried to open unsupported file type.");
            }
        }
    }

    /// Loads in a file of any supported type into the engine.
    pub fn load_in_file(
        &self,
        file: &gio::File,
        target_pos: Option<na::Vector2<f64>>,
    ) -> anyhow::Result<()> {
        let main_cx = glib::MainContext::default();
        let app = self.app();
        let file = file.clone();

        match crate::utils::FileType::lookup_file_type(&file) {
            crate::utils::FileType::RnoteFile => {
                main_cx.spawn_local(clone!(@strong self as appwindow => async move {
                    appwindow.start_pulsing_canvas_progressbar();

                    let result = file.load_bytes_future().await;

                    if let Ok((file_bytes, _)) = result {
                        if let Err(e) = appwindow.load_in_rnote_bytes(file_bytes.to_vec(), file.path()).await {
                            adw::prelude::ActionGroupExt::activate_action(&appwindow, "error-toast", Some(&gettext("Opening .rnote file failed.").to_variant()));
                            log::error!(
                                "load_in_rnote_bytes() failed in load_in_file() with Err {}",
                                e
                            );
                        }
                    }

                    appwindow.finish_canvas_progressbar();
                }));
            }
            crate::utils::FileType::XoppFile => {
                main_cx.spawn_local(clone!(@strong self as appwindow => async move {
                    appwindow.start_pulsing_canvas_progressbar();

                    let result = file.load_bytes_future().await;

                    if let Ok((file_bytes, _)) = result {
                        if let Err(e) = appwindow.load_in_xopp_bytes(file_bytes.to_vec(), file.path()) {
                            adw::prelude::ActionGroupExt::activate_action(&appwindow, "error-toast", Some(&gettext("Opening .xopp file failed.").to_variant()));
                            log::error!(
                                "load_in_xopp_bytes() failed in load_in_file() with Err {}",
                                e
                            );
                        }
                    }

                    appwindow.finish_canvas_progressbar();
                }));
            }
            crate::utils::FileType::VectorImageFile => {
                main_cx.spawn_local(clone!(@strong self as appwindow => async move {
                    appwindow.start_pulsing_canvas_progressbar();

                    let result = file.load_bytes_future().await;

                    if let Ok((file_bytes, _)) = result {
                        if let Err(e) = appwindow.load_in_vectorimage_bytes(file_bytes.to_vec(), target_pos).await {
                            adw::prelude::ActionGroupExt::activate_action(&appwindow, "error-toast", Some(&gettext("Opening vector image file failed.").to_variant()));
                            log::error!(
                                "load_in_rnote_bytes() failed in load_in_file() with Err {}",
                                e
                            );
                        }
                    }

                    appwindow.finish_canvas_progressbar();
                }));
            }
            crate::utils::FileType::BitmapImageFile => {
                main_cx.spawn_local(clone!(@strong self as appwindow => async move {
                    appwindow.start_pulsing_canvas_progressbar();

                    let result = file.load_bytes_future().await;

                    if let Ok((file_bytes, _)) = result {
                        if let Err(e) = appwindow.load_in_bitmapimage_bytes(file_bytes.to_vec(), target_pos).await {
                            adw::prelude::ActionGroupExt::activate_action(&appwindow, "error-toast", Some(&gettext("Opening bitmap image file failed.").to_variant()));
                            log::error!(
                                "load_in_rnote_bytes() failed in load_in_file() with Err {}",
                                e
                            );
                        }
                    }

                    appwindow.finish_canvas_progressbar();
                }));
            }
            crate::utils::FileType::PdfFile => {
                main_cx.spawn_local(clone!(@strong self as appwindow => async move {
                    appwindow.start_pulsing_canvas_progressbar();

                    let result = file.load_bytes_future().await;

                    if let Ok((file_bytes, _)) = result {
                        if let Err(e) = appwindow.load_in_pdf_bytes(file_bytes.to_vec(), target_pos, None).await {
                            adw::prelude::ActionGroupExt::activate_action(&appwindow, "error-toast", Some(&gettext("Opening PDF file failed.").to_variant()));
                            log::error!(
                                "load_in_rnote_bytes() failed in load_in_file() with Err {}",
                                e
                            );
                        }
                    }

                    appwindow.finish_canvas_progressbar();
                }));
            }
            crate::utils::FileType::Folder => {
                app.set_input_file(None);
                log::error!("tried to open a folder as a file.");
                adw::prelude::ActionGroupExt::activate_action(
                    self,
                    "error-toast",
                    Some(&gettext("Error: Tried opening folder as file").to_variant()),
                );
            }
            crate::utils::FileType::Unsupported => {
                app.set_input_file(None);
                log::error!("tried to open a unsupported file type.");
                adw::prelude::ActionGroupExt::activate_action(
                    self,
                    "error-toast",
                    Some(&gettext("Failed to open file: Unsupported file type.").to_variant()),
                );
            }
        }

        Ok(())
    }

    pub async fn load_in_rnote_bytes<P>(
        &self,
        bytes: Vec<u8>,
        path: Option<P>,
    ) -> anyhow::Result<()>
    where
        P: AsRef<Path>,
    {
        let app = self.app();

        let store_snapshot_receiver = self
            .canvas()
            .engine()
            .borrow_mut()
            .open_from_rnote_bytes_p1(bytes)?;

        let store_snapshot = store_snapshot_receiver.await??;

        self.canvas()
            .engine()
            .borrow_mut()
            .open_from_store_snapshot_p2(&store_snapshot)?;

        self.canvas().set_unsaved_changes(false);
        app.set_input_file(None);
        if let Some(path) = path {
            let file = gio::File::for_path(path);
            self.canvas().set_output_file(Some(file));
        }

        self.canvas().set_unsaved_changes(false);
        self.canvas().set_empty(false);
        self.canvas().return_to_origin_page();

        self.canvas().regenerate_background_pattern();
        self.canvas().engine().borrow_mut().resize_autoexpand();
        self.canvas().update_engine_rendering();

        adw::prelude::ActionGroupExt::activate_action(self, "refresh-ui-for-engine", None);

        Ok(())
    }

    pub fn load_in_xopp_bytes<P>(&self, bytes: Vec<u8>, _path: Option<P>) -> anyhow::Result<()>
    where
        P: AsRef<Path>,
    {
        self.canvas()
            .engine()
            .borrow_mut()
            .open_from_xopp_bytes(bytes)?;

        self.app().set_input_file(None);
        self.canvas().set_output_file(None);

        self.canvas().set_unsaved_changes(true);
        self.canvas().set_empty(false);
        self.canvas().return_to_origin_page();

        self.canvas().regenerate_background_pattern();
        self.canvas().engine().borrow_mut().resize_autoexpand();
        self.canvas().update_engine_rendering();

        adw::prelude::ActionGroupExt::activate_action(self, "refresh-ui-for-engine", None);

        Ok(())
    }

    pub async fn load_in_vectorimage_bytes(
        &self,
        bytes: Vec<u8>,
        // In coordinate space of the doc
        target_pos: Option<na::Vector2<f64>>,
    ) -> anyhow::Result<()> {
        let app = self.app();

        let pos = target_pos.unwrap_or_else(|| {
            (self.canvas().engine().borrow().camera.transform().inverse()
                * na::Point2::from(VectorImage::IMPORT_OFFSET_DEFAULT))
            .coords
        });

        // we need the split the import operation between generate_vectorimage_from_bytes() which returns a receiver and import_generated_strokes(),
        // to avoid borrowing the entire engine refcell while awaiting the stroke
        let vectorimage_receiver = self
            .canvas()
            .engine()
            .borrow_mut()
            .generate_vectorimage_from_bytes(pos, bytes);
        let vectorimage = vectorimage_receiver.await??;

        let widget_flags = self
            .canvas()
            .engine()
            .borrow_mut()
            .import_generated_strokes(vec![(Stroke::VectorImage(vectorimage), None)]);
        self.handle_widget_flags(widget_flags);

        app.set_input_file(None);

        Ok(())
    }

    /// Target position is in the coordinate space of the doc
    pub async fn load_in_bitmapimage_bytes(
        &self,
        bytes: Vec<u8>,
        // In the coordinate space of the doc
        target_pos: Option<na::Vector2<f64>>,
    ) -> anyhow::Result<()> {
        let app = self.app();

        let pos = target_pos.unwrap_or_else(|| {
            (self.canvas().engine().borrow().camera.transform().inverse()
                * na::Point2::from(BitmapImage::IMPORT_OFFSET_DEFAULT))
            .coords
        });

        let bitmapimage_receiver = self
            .canvas()
            .engine()
            .borrow_mut()
            .generate_bitmapimage_from_bytes(pos, bytes);
        let bitmapimage = bitmapimage_receiver.await??;

        let widget_flags = self
            .canvas()
            .engine()
            .borrow_mut()
            .import_generated_strokes(vec![(Stroke::BitmapImage(bitmapimage), None)]);
        self.handle_widget_flags(widget_flags);

        app.set_input_file(None);

        Ok(())
    }

    /// Target position is in the coordinate space of the doc
    pub async fn load_in_pdf_bytes(
        &self,
        bytes: Vec<u8>,
        // In the coordinate space of the doc
        target_pos: Option<na::Vector2<f64>>,
        page_range: Option<Range<u32>>,
    ) -> anyhow::Result<()> {
        let app = self.app();

        let pos = target_pos.unwrap_or_else(|| {
            (self.canvas().engine().borrow().camera.transform().inverse()
                * na::Point2::from(VectorImage::IMPORT_OFFSET_DEFAULT))
            .coords
        });

        let strokes_receiver = self
            .canvas()
            .engine()
            .borrow_mut()
            .generate_pdf_pages_from_bytes(bytes, pos, page_range);
        let strokes = strokes_receiver.await??;

        let widget_flags = self
            .canvas()
            .engine()
            .borrow_mut()
            .import_generated_strokes(strokes);
        self.handle_widget_flags(widget_flags);

        app.set_input_file(None);

        Ok(())
    }

    pub async fn save_document_to_file(&self, file: &gio::File) -> anyhow::Result<()> {
        if let Some(basename) = file.basename() {
            let rnote_bytes_receiver = self
                .canvas()
                .engine()
                .borrow()
                .save_as_rnote_bytes(basename.to_string_lossy().to_string())?;

            crate::utils::replace_file_future(rnote_bytes_receiver.await??, file).await?;

            self.canvas().set_output_file(Some(file.to_owned()));
            self.canvas().set_unsaved_changes(false);
        }
        Ok(())
    }

    pub async fn export_doc(
        &self,
        file: &gio::File,
        title: String,
        export_prefs_override: Option<DocExportPrefs>,
    ) -> anyhow::Result<()> {
        let export_bytes = self
            .canvas()
            .engine()
            .borrow()
            .export_doc(title, export_prefs_override);

        crate::utils::replace_file_future(export_bytes.await??, file).await?;

        Ok(())
    }

    pub async fn export_selection(
        &self,
        file: &gio::File,
        export_prefs_override: Option<SelectionExportPrefs>,
    ) -> anyhow::Result<()> {
        let export_bytes = self
            .canvas()
            .engine()
            .borrow()
            .export_selection(export_prefs_override);

        if let Some(export_bytes) = export_bytes.await?? {
            crate::utils::replace_file_future(export_bytes, file).await?;
        }

        Ok(())
    }

    /// exports and writes the engine state as json into the file.
    /// Only for debugging!
    pub async fn export_engine_state(&self, file: &gio::File) -> anyhow::Result<()> {
        let exported_engine_state = self.canvas().engine().borrow().export_state_as_json()?;

        crate::utils::replace_file_future(exported_engine_state.into_bytes(), file).await?;

        Ok(())
    }

    /// exports and writes the engine config as json into the file.
    /// Only for debugging!
    pub async fn export_engine_config(&self, file: &gio::File) -> anyhow::Result<()> {
        let exported_engine_config = self.canvas().engine().borrow().save_engine_config()?;

        crate::utils::replace_file_future(exported_engine_config.into_bytes(), file).await?;

        Ok(())
    }
}
