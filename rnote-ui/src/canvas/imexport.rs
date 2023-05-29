// Imports
use super::RnCanvas;
use futures::channel::oneshot;
use gtk4::{gio, prelude::*};
use rnote_compose::helpers::Vector2Helpers;
use rnote_engine::engine::export::{DocExportPrefs, DocPagesExportPrefs, SelectionExportPrefs};
use rnote_engine::engine::{EngineSnapshot, StrokeContent};
use rnote_engine::strokes::Stroke;
use std::ops::Range;
use std::path::Path;

impl RnCanvas {
    pub(crate) async fn load_in_rnote_bytes<P>(
        &self,
        bytes: Vec<u8>,
        file_path: Option<P>,
    ) -> anyhow::Result<()>
    where
        P: AsRef<Path>,
    {
        let engine_snapshot = EngineSnapshot::load_from_rnote_bytes(bytes).await?;

        let mut widget_flags = self.engine().borrow_mut().load_snapshot(engine_snapshot);

        if let Some(file_path) = file_path {
            let file = gio::File::for_path(file_path);
            self.dismiss_output_file_modified_toast();
            self.set_output_file(Some(file));
        }

        self.set_unsaved_changes(false);
        self.set_empty(false);
        self.return_to_origin_page();
        self.background_regenerate_pattern();
        widget_flags.merge(self.engine().borrow_mut().doc_resize_autoexpand());
        self.update_rendering_current_viewport();

        widget_flags.refresh_ui = true;

        self.emit_handle_widget_flags(widget_flags);
        Ok(())
    }

    pub(crate) async fn reload_from_disk(&self) -> anyhow::Result<()> {
        if let Some(output_file) = self.output_file() {
            let (bytes, _) = output_file.load_bytes_future().await?;

            self.load_in_rnote_bytes(bytes.to_vec(), output_file.path())
                .await?;
        }

        Ok(())
    }

    pub(crate) async fn load_in_vectorimage_bytes(
        &self,
        bytes: Vec<u8>,
        // In coordinate space of the doc
        target_pos: Option<na::Vector2<f64>>,
    ) -> anyhow::Result<()> {
        let pos = target_pos.unwrap_or_else(|| {
            self.engine()
                .borrow()
                .camera
                .transform()
                .inverse()
                .transform_point(&na::Point2::from(Stroke::IMPORT_OFFSET_DEFAULT))
                .coords
                .maxs(&na::vector![
                    self.engine().borrow().document.x,
                    self.engine().borrow().document.y
                ])
        });

        // we need the split the import operation between generate_vectorimage_from_bytes() which returns a receiver and import_generated_strokes(),
        // to avoid borrowing the entire engine refcell while awaiting the stroke
        let vectorimage_receiver = self
            .engine()
            .borrow_mut()
            .generate_vectorimage_from_bytes(pos, bytes);
        let vectorimage = vectorimage_receiver.await??;

        let widget_flags = self
            .engine()
            .borrow_mut()
            .import_generated_strokes(vec![(Stroke::VectorImage(vectorimage), None)]);

        self.emit_handle_widget_flags(widget_flags);
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
            self.engine()
                .borrow()
                .camera
                .transform()
                .inverse()
                .transform_point(&na::Point2::from(Stroke::IMPORT_OFFSET_DEFAULT))
                .coords
                .maxs(&na::vector![
                    self.engine().borrow().document.x,
                    self.engine().borrow().document.y
                ])
        });

        let bitmapimage_receiver = self
            .engine()
            .borrow_mut()
            .generate_bitmapimage_from_bytes(pos, bytes);
        let bitmapimage = bitmapimage_receiver.await??;

        let widget_flags = self
            .engine()
            .borrow_mut()
            .import_generated_strokes(vec![(Stroke::BitmapImage(bitmapimage), None)]);

        self.emit_handle_widget_flags(widget_flags);
        Ok(())
    }

    pub(crate) async fn load_in_xopp_bytes(&self, bytes: Vec<u8>) -> anyhow::Result<()> {
        let xopp_import_prefs = self.engine().borrow_mut().import_prefs.xopp_import_prefs;

        let engine_snapshot =
            EngineSnapshot::load_from_xopp_bytes(bytes, xopp_import_prefs).await?;

        let mut widget_flags = self.engine().borrow_mut().load_snapshot(engine_snapshot);

        self.set_output_file(None);
        self.set_unsaved_changes(true);
        self.set_empty(false);
        self.return_to_origin_page();
        self.background_regenerate_pattern();
        widget_flags.merge(self.engine().borrow_mut().doc_resize_autoexpand());
        self.update_rendering_current_viewport();

        widget_flags.refresh_ui = true;

        self.emit_handle_widget_flags(widget_flags);
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
            self.engine()
                .borrow()
                .camera
                .transform()
                .inverse()
                .transform_point(&na::Point2::from(Stroke::IMPORT_OFFSET_DEFAULT))
                .coords
                .maxs(&na::vector![
                    self.engine().borrow().document.x,
                    self.engine().borrow().document.y
                ])
        });

        let strokes_receiver = self
            .engine()
            .borrow_mut()
            .generate_pdf_pages_from_bytes(bytes, pos, page_range);
        let strokes = strokes_receiver.await??;

        let widget_flags = self.engine().borrow_mut().import_generated_strokes(strokes);

        self.emit_handle_widget_flags(widget_flags);
        Ok(())
    }

    /// Target position is in the coordinate space of the doc
    pub(crate) fn load_in_text(
        &self,
        text: String,
        target_pos: Option<na::Vector2<f64>>,
    ) -> anyhow::Result<()> {
        let pos = target_pos.unwrap_or_else(|| {
            self.engine()
                .borrow()
                .camera
                .transform()
                .inverse()
                .transform_point(&na::Point2::from(Stroke::IMPORT_OFFSET_DEFAULT))
                .coords
                .maxs(&na::vector![
                    self.engine().borrow().document.x,
                    self.engine().borrow().document.y
                ])
        });

        let widget_flags = self.engine().borrow_mut().insert_text(text, pos)?;

        self.emit_handle_widget_flags(widget_flags);
        Ok(())
    }

    /// Deserializes the stroke content and inserts it into the engine. The data is usually coming from the clipboard, drop source, etc.
    pub(crate) async fn insert_stroke_content(&self, json_string: String) -> anyhow::Result<()> {
        let (oneshot_sender, oneshot_receiver) =
            oneshot::channel::<anyhow::Result<StrokeContent>>();

        rayon::spawn(move || {
            let result = || -> Result<StrokeContent, anyhow::Error> {
                Ok(serde_json::from_str(&json_string)?)
            };
            if let Err(_data) = oneshot_sender.send(result()) {
                log::error!("sending result to receiver in insert_stroke_content() failed. Receiver already dropped");
            }
        });
        let content = oneshot_receiver.await??;
        let pos = self
            .engine()
            .borrow()
            .camera
            .transform()
            .inverse()
            .transform_point(&na::Point2::from(Stroke::IMPORT_OFFSET_DEFAULT))
            .coords
            .maxs(&na::vector![
                self.engine().borrow().document.x,
                self.engine().borrow().document.y
            ]);

        let widget_flags = self
            .engine()
            .borrow_mut()
            .insert_stroke_content(content, pos);

        self.emit_handle_widget_flags(widget_flags);
        Ok(())
    }

    /// Saves the document to the given file.
    ///
    /// Returns Ok(true) if saved successfully, Ok(false) when a save is already in progress and no file operatiosn were executed,
    /// Err(e) when saving failed in any way.
    pub(crate) async fn save_document_to_file(&self, file: &gio::File) -> anyhow::Result<bool> {
        // skip saving when it is already in progress
        if self.save_in_progress() {
            log::debug!("saving file already in progress");
            return Ok(false);
        }

        let file_path = file.path().ok_or_else(|| {
            anyhow::anyhow!(
                "save_document_to_file() failed, could not get a path for file: {file:?}"
            )
        })?;

        let basename = file.basename().ok_or_else(|| {
            anyhow::anyhow!(
                "save_document_to_file() failed, could not retrieve basename for file: {file:?}"
            )
        })?;

        self.set_save_in_progress(true);

        let rnote_bytes_receiver = match self
            .engine()
            .borrow()
            .save_as_rnote_bytes(basename.to_string_lossy().to_string())
        {
            Ok(r) => r,
            Err(e) => {
                self.set_save_in_progress(false);
                return Err(e);
            }
        };

        let mut skip_set_output_file = false;
        if let Some(current_file_path) = self.output_file().and_then(|f| f.path()) {
            if same_file::is_same_file(current_file_path, file_path).unwrap_or(false) {
                skip_set_output_file = true;
            }
        }

        // this **must** come before actually saving the file to disk,
        // else the event might not be caught by the monitor for new or changed files
        if !skip_set_output_file {
            self.set_output_file(Some(file.to_owned()));
        }

        self.dismiss_output_file_modified_toast();
        self.set_output_file_expect_write(true);

        let res = async move {
            crate::utils::create_replace_file_future(rnote_bytes_receiver.await??, file).await
        }
        .await;

        if let Err(e) = res {
            self.set_save_in_progress(false);

            // If the file operations failed in any way, we make sure to clear the expect_write flag
            // because we can't know for sure if the output_file monitor will be able to.
            self.set_output_file_expect_write(false);
            return Err(e);
        }

        self.set_unsaved_changes(false);
        self.set_save_in_progress(false);

        Ok(true)
    }

    pub(crate) async fn export_doc(
        &self,
        file: &gio::File,
        title: String,
        export_prefs_override: Option<DocExportPrefs>,
    ) -> anyhow::Result<()> {
        let export_bytes = self
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
        let export_prefs = export_prefs_override
            .unwrap_or(self.engine().borrow().export_prefs.doc_pages_export_prefs);

        let file_ext = export_prefs.export_format.file_ext();

        let export_bytes = self
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
        let exported_engine_state = self.engine().borrow().export_state_as_json()?;

        crate::utils::create_replace_file_future(exported_engine_state.into_bytes(), file).await?;

        Ok(())
    }

    /// exports and writes the engine config as json into the file.
    /// Only for debugging!
    pub(crate) async fn export_engine_config(&self, file: &gio::File) -> anyhow::Result<()> {
        let exported_engine_config = self.engine().borrow().export_engine_config_as_json()?;

        crate::utils::create_replace_file_future(exported_engine_config.into_bytes(), file).await?;

        Ok(())
    }
}
