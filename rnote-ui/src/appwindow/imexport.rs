use std::ops::Range;
use std::path::Path;

use gettextrs::gettext;
use gtk4::{gio, glib, glib::clone, prelude::*};
use rnote_engine::engine::export::{DocExportPrefs, DocPagesExportPrefs, SelectionExportPrefs};
use rnote_engine::engine::EngineSnapshot;
use rnote_engine::strokes::Stroke;

use crate::dialogs;

use super::RnoteAppWindow;

impl RnoteAppWindow {
    pub(crate) fn open_file_w_dialogs(
        &self,
        input_file: gio::File,
        target_pos: Option<na::Vector2<f64>>,
    ) {
        match crate::utils::FileType::lookup_file_type(&input_file) {
            crate::utils::FileType::RnoteFile => {
                if self.canvas().unsaved_changes() {
                    dialogs::import::dialog_open_overwrite(self, input_file);
                } else if let Err(e) = self.load_in_file(input_file, target_pos) {
                    log::error!(
                        "failed to load in file with FileType::RnoteFile | FileType::XoppFile, {e:?}"
                    );
                }
            }
            crate::utils::FileType::VectorImageFile | crate::utils::FileType::BitmapImageFile => {
                if let Err(e) = self.load_in_file(input_file, target_pos) {
                    log::error!("failed to load in file with FileType::VectorImageFile / FileType::BitmapImageFile / FileType::Pdf, {e:?}");
                }
            }
            crate::utils::FileType::XoppFile => {
                dialogs::import::dialog_import_xopp_w_prefs(self, input_file);
            }
            crate::utils::FileType::PdfFile => {
                dialogs::import::dialog_import_pdf_w_prefs(self, input_file, target_pos);
            }
            crate::utils::FileType::Folder => {
                if let Some(dir) = input_file.path() {
                    self.workspacebrowser()
                        .workspacesbar()
                        .set_selected_workspace_dir(dir);
                }
            }
            crate::utils::FileType::Unsupported => {
                log::error!("tried to open unsupported file type.");
            }
        }
    }

    /// Loads in a file of any supported type into the engine.
    pub(crate) fn load_in_file(
        &self,
        file: gio::File,
        target_pos: Option<na::Vector2<f64>>,
    ) -> anyhow::Result<()> {
        let main_cx = glib::MainContext::default();

        match crate::utils::FileType::lookup_file_type(&file) {
            crate::utils::FileType::RnoteFile => {
                main_cx.spawn_local(clone!(@strong self as appwindow => async move {
                    appwindow.canvas_wrapper().start_pulsing_progressbar();

                    let result = file.load_bytes_future().await;

                    if let Ok((file_bytes, _)) = result {
                        if let Err(e) = appwindow.load_in_rnote_bytes(file_bytes.to_vec(), file.path()).await {
                            appwindow.canvas_wrapper().dispatch_toast_error(&gettext("Opening .rnote file failed."));
                            log::error!(
                                "load_in_rnote_bytes() failed in load_in_file() with Err: {e:?}"
                            );
                        }
                    }

                    appwindow.canvas_wrapper().finish_progressbar();
                }));
            }
            crate::utils::FileType::VectorImageFile => {
                main_cx.spawn_local(clone!(@strong self as appwindow => async move {
                    appwindow.canvas_wrapper().start_pulsing_progressbar();

                    let result = file.load_bytes_future().await;

                    if let Ok((file_bytes, _)) = result {
                        if let Err(e) = appwindow.load_in_vectorimage_bytes(file_bytes.to_vec(), target_pos).await {
                            appwindow.canvas_wrapper().dispatch_toast_error(&gettext("Opening vector image file failed."));
                            log::error!(
                                "load_in_vectorimage_bytes() failed in load_in_file() with Err: {e:?}"
                            );
                        }
                    }

                    appwindow.canvas_wrapper().finish_progressbar();
                }));
            }
            crate::utils::FileType::BitmapImageFile => {
                main_cx.spawn_local(clone!(@strong self as appwindow => async move {
                    appwindow.canvas_wrapper().start_pulsing_progressbar();

                    let result = file.load_bytes_future().await;

                    if let Ok((file_bytes, _)) = result {
                        if let Err(e) = appwindow.load_in_bitmapimage_bytes(file_bytes.to_vec(), target_pos).await {
                            appwindow.canvas_wrapper().dispatch_toast_error(&gettext("Opening bitmap image file failed."));
                            log::error!(
                                "load_in_bitmapimage_bytes() failed in load_in_file() with Err: {e:?}"
                            );
                        }
                    }

                    appwindow.canvas_wrapper().finish_progressbar();
                }));
            }
            crate::utils::FileType::XoppFile => {
                main_cx.spawn_local(clone!(@strong self as appwindow => async move {
                    appwindow.canvas_wrapper().start_pulsing_progressbar();

                    let result = file.load_bytes_future().await;

                    if let Ok((file_bytes, _)) = result {
                        if let Err(e) = appwindow.load_in_xopp_bytes(file_bytes.to_vec()).await {
                            appwindow.canvas_wrapper().dispatch_toast_error(&gettext("Opening Xournal++ file failed."));
                            log::error!(
                                "load_in_xopp_bytes() failed in load_in_file() with Err: {e:?}"
                            );
                        }
                    }

                    appwindow.canvas_wrapper().finish_progressbar();
                }));
            }
            crate::utils::FileType::PdfFile => {
                main_cx.spawn_local(clone!(@strong self as appwindow => async move {
                    appwindow.canvas_wrapper().start_pulsing_progressbar();

                    let result = file.load_bytes_future().await;

                    if let Ok((file_bytes, _)) = result {
                        if let Err(e) = appwindow.load_in_pdf_bytes(file_bytes.to_vec(), target_pos, None).await {
                            appwindow.canvas_wrapper().dispatch_toast_error(&gettext("Opening PDF file failed."));
                            log::error!(
                                "load_in_pdf_bytes() failed in load_in_file() with Err: {e:?}"
                            );
                        }
                    }

                    appwindow.canvas_wrapper().finish_progressbar();
                }));
            }
            crate::utils::FileType::Folder => {
                log::error!("tried to open a folder as a file.");
                self.canvas_wrapper()
                    .dispatch_toast_error(&gettext("Error: Tried opening folder as file"));
            }
            crate::utils::FileType::Unsupported => {
                log::error!("tried to open a unsupported file type.");
                self.canvas_wrapper()
                    .dispatch_toast_error(&gettext("Failed to open file: Unsupported file type."));
            }
        }

        Ok(())
    }

    pub(crate) async fn load_in_rnote_bytes<P>(
        &self,
        bytes: Vec<u8>,
        path: Option<P>,
    ) -> anyhow::Result<()>
    where
        P: AsRef<Path>,
    {
        let engine_snapshot = EngineSnapshot::load_from_rnote_bytes(bytes).await?;

        let widget_flags = self
            .canvas()
            .engine()
            .borrow_mut()
            .load_snapshot(engine_snapshot);
        self.handle_widget_flags(widget_flags);

        if let Some(path) = path {
            let file = gio::File::for_path(path);
            self.canvas().dismiss_output_file_modified_toast();
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

    pub(crate) async fn load_in_vectorimage_bytes(
        &self,
        bytes: Vec<u8>,
        // In coordinate space of the doc
        target_pos: Option<na::Vector2<f64>>,
    ) -> anyhow::Result<()> {
        let pos = target_pos.unwrap_or_else(|| {
            (self.canvas().engine().borrow().camera.transform().inverse()
                * na::Point2::from(Stroke::IMPORT_OFFSET_DEFAULT))
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

        Ok(())
    }

    /// Target position is in the coordinate space of the doc
    pub(crate) async fn load_in_bitmapimage_bytes(
        &self,
        bytes: Vec<u8>,
        // In the coordinate space of the doc
        target_pos: Option<na::Vector2<f64>>,
    ) -> anyhow::Result<()> {
        let pos = target_pos.unwrap_or_else(|| {
            (self.canvas().engine().borrow().camera.transform().inverse()
                * na::Point2::from(Stroke::IMPORT_OFFSET_DEFAULT))
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

        Ok(())
    }

    pub(crate) async fn load_in_xopp_bytes(&self, bytes: Vec<u8>) -> anyhow::Result<()> {
        let xopp_import_prefs = self
            .canvas()
            .engine()
            .borrow_mut()
            .import_prefs
            .xopp_import_prefs;

        let engine_snapshot =
            EngineSnapshot::load_from_xopp_bytes(bytes, xopp_import_prefs).await?;

        let widget_flags = self
            .canvas()
            .engine()
            .borrow_mut()
            .load_snapshot(engine_snapshot);
        self.handle_widget_flags(widget_flags);

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

    /// Target position is in the coordinate space of the doc
    pub(crate) async fn load_in_pdf_bytes(
        &self,
        bytes: Vec<u8>,
        target_pos: Option<na::Vector2<f64>>,
        page_range: Option<Range<u32>>,
    ) -> anyhow::Result<()> {
        let pos = target_pos.unwrap_or_else(|| {
            (self.canvas().engine().borrow().camera.transform().inverse()
                * na::Point2::from(Stroke::IMPORT_OFFSET_DEFAULT))
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

        Ok(())
    }

    /// Target position is in the coordinate space of the doc
    pub(crate) fn load_in_text(
        &self,
        text: String,
        target_pos: Option<na::Vector2<f64>>,
    ) -> anyhow::Result<()> {
        let pos = target_pos.unwrap_or_else(|| {
            (self.canvas().engine().borrow().camera.transform().inverse()
                * na::Point2::from(Stroke::IMPORT_OFFSET_DEFAULT))
            .coords
        });

        let widget_flags = self.canvas().engine().borrow_mut().insert_text(text, pos)?;
        self.handle_widget_flags(widget_flags);

        Ok(())
    }

    pub(crate) async fn save_document_to_file(&self, file: &gio::File) -> anyhow::Result<()> {
        if let Some(basename) = file.basename() {
            let rnote_bytes_receiver = self
                .canvas()
                .engine()
                .borrow()
                .save_as_rnote_bytes(basename.to_string_lossy().to_string())?;

            self.canvas().set_output_file_expect_write(true);
            self.canvas().dismiss_output_file_modified_toast();

            crate::utils::create_replace_file_future(rnote_bytes_receiver.await??, file).await?;

            self.canvas().set_output_file(Some(file.to_owned()));
            self.canvas().set_unsaved_changes(false);
        }
        Ok(())
    }

    pub(crate) async fn export_doc(
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

        crate::utils::create_replace_file_future(export_bytes.await??, file).await?;

        Ok(())
    }

    /// Exports document pages
    /// file_stem_name: the stem name of the created files. This is extended by an enumeration of the page number and file extension
    /// overwrites existing files with the same name!
    pub(crate) async fn export_doc_pages(
        &self,
        dir: &gio::File,
        file_stem_name: String,
        export_prefs_override: Option<DocPagesExportPrefs>,
    ) -> anyhow::Result<()> {
        let export_prefs = export_prefs_override.unwrap_or(
            self.canvas()
                .engine()
                .borrow()
                .export_prefs
                .doc_pages_export_prefs,
        );
        let file_ext = export_prefs.export_format.file_ext();

        let export_bytes = self
            .canvas()
            .engine()
            .borrow()
            .export_doc_pages(export_prefs_override);

        if dir.query_file_type(gio::FileQueryInfoFlags::NONE, gio::Cancellable::NONE)
            != gio::FileType::Directory
        {
            return Err(anyhow::anyhow!(
                "export_doc_pages() failed, target is not a directory."
            ));
        }

        let pages_bytes = export_bytes.await??;

        for (i, page_bytes) in pages_bytes.into_iter().enumerate() {
            crate::utils::create_replace_file_future(
                page_bytes,
                &dir.child(
                    &(rnote_engine::utils::doc_pages_files_names(file_stem_name.clone(), i + 1)
                        + "."
                        + &file_ext),
                ),
            )
            .await?;
        }

        Ok(())
    }

    pub(crate) async fn export_selection(
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
            crate::utils::create_replace_file_future(export_bytes, file).await?;
        }

        Ok(())
    }

    /// exports and writes the engine state as json into the file.
    /// Only for debugging!
    pub(crate) async fn export_engine_state(&self, file: &gio::File) -> anyhow::Result<()> {
        let exported_engine_state = self.canvas().engine().borrow().export_state_as_json()?;

        crate::utils::create_replace_file_future(exported_engine_state.into_bytes(), file).await?;

        Ok(())
    }

    /// exports and writes the engine config as json into the file.
    /// Only for debugging!
    pub(crate) async fn export_engine_config(&self, file: &gio::File) -> anyhow::Result<()> {
        let exported_engine_config = self.canvas().engine().borrow().save_engine_config()?;

        crate::utils::create_replace_file_future(exported_engine_config.into_bytes(), file).await?;

        Ok(())
    }
}
