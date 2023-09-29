// Imports
use super::RnCanvas;
use futures::channel::oneshot;
use gtk4::{gio, prelude::*};
use rnote_compose::ext::Vector2Ext;
use rnote_engine::engine::export::{DocExportPrefs, DocPagesExportPrefs, SelectionExportPrefs};
use rnote_engine::engine::{EngineSnapshot, StrokeContent};
use rnote_engine::strokes::Stroke;
use rnote_engine::WidgetFlags;
use std::ops::Range;
use std::path::Path;

impl RnCanvas {
    /// Load the bytes of a `.rnote` file and imports it into the engine.
    ///
    /// `file_path` is optional but needs to be supplied when the origin file should be tracked.
    ///
    /// The function returns `WidgetFlags` instead of emitting the `handle_signal_flags` signal, because a signal
    /// handler might not yet be connected when this function is called.
    pub(crate) async fn load_in_rnote_bytes<P>(
        &self,
        bytes: Vec<u8>,
        file_path: Option<P>,
    ) -> anyhow::Result<WidgetFlags>
    where
        P: AsRef<Path>,
    {
        let engine_snapshot = EngineSnapshot::load_from_rnote_bytes(bytes).await?;
        let widget_flags = self.engine_mut().load_snapshot(engine_snapshot);

        if file_path.is_some() {
            self.dismiss_output_file_modified_toast();
        }
        self.set_output_file(file_path.map(gio::File::for_path));
        self.set_unsaved_changes(false);
        self.set_empty(false);
        Ok(widget_flags)
    }

    /// Reload the engine from the file that is set as origin file.
    ///
    /// If the origin file is set to None, this does nothing and returns an error.
    pub(crate) async fn reload_from_disk(&self) -> anyhow::Result<()> {
        let Some(output_file) = self.output_file() else {
            return Err(anyhow::anyhow!(
                "Failed to reload file from disk, no file path saved."
            ));
        };
        let (bytes, _) = output_file.load_bytes_future().await?;
        let widget_flags = self
            .load_in_rnote_bytes(bytes.to_vec(), output_file.path())
            .await?;
        self.emit_handle_widget_flags(widget_flags);
        Ok(())
    }

    pub(crate) async fn load_in_xopp_bytes(&self, bytes: Vec<u8>) -> anyhow::Result<()> {
        let xopp_import_prefs = self.engine_ref().import_prefs.xopp_import_prefs;
        let engine_snapshot =
            EngineSnapshot::load_from_xopp_bytes(bytes, xopp_import_prefs).await?;
        let widget_flags = self.engine_mut().load_snapshot(engine_snapshot);
        self.emit_handle_widget_flags(widget_flags);

        self.set_output_file(None);
        self.set_unsaved_changes(true);
        self.set_empty(false);
        Ok(())
    }

    /// Loads in bytes from a vector image and imports it.
    ///
    /// `target_pos` is in coordinate space of the doc.
    pub(crate) async fn load_in_vectorimage_bytes(
        &self,
        bytes: Vec<u8>,
        target_pos: Option<na::Vector2<f64>>,
    ) -> anyhow::Result<()> {
        let pos = target_pos.unwrap_or_else(|| {
            self.engine_ref()
                .camera
                .transform()
                .inverse()
                .transform_point(&na::Point2::from(Stroke::IMPORT_OFFSET_DEFAULT))
                .coords
                .maxs(&na::vector![
                    self.engine_ref().document.x,
                    self.engine_ref().document.y
                ])
        });

        // we need the split the import operation between generate_vectorimage_from_bytes()
        // which returns a receiver and import_generated_content(),
        // to avoid borrowing the entire engine refcell while awaiting the generated stroke
        let vectorimage_receiver = self
            .engine_mut()
            .generate_vectorimage_from_bytes(pos, bytes);
        let vectorimage = vectorimage_receiver.await??;
        let widget_flags = self
            .engine_mut()
            .import_generated_content(vec![(Stroke::VectorImage(vectorimage), None)]);

        self.emit_handle_widget_flags(widget_flags);
        Ok(())
    }

    /// Loads in bytes from a bitmap image and imports it.
    ///
    /// `target_pos` is in coordinate space of the doc.
    pub(crate) async fn load_in_bitmapimage_bytes(
        &self,
        bytes: Vec<u8>,
        target_pos: Option<na::Vector2<f64>>,
    ) -> anyhow::Result<()> {
        let pos = target_pos.unwrap_or_else(|| {
            self.engine_ref()
                .camera
                .transform()
                .inverse()
                .transform_point(&na::Point2::from(Stroke::IMPORT_OFFSET_DEFAULT))
                .coords
                .maxs(&na::vector![
                    self.engine_ref().document.x,
                    self.engine_ref().document.y
                ])
        });

        let bitmapimage_receiver = self
            .engine_mut()
            .generate_bitmapimage_from_bytes(pos, bytes);
        let bitmapimage = bitmapimage_receiver.await??;

        let widget_flags = self
            .engine_mut()
            .import_generated_content(vec![(Stroke::BitmapImage(bitmapimage), None)]);

        self.emit_handle_widget_flags(widget_flags);
        Ok(())
    }

    /// Loads in bytes from a pdf and imports it.
    ///
    /// `target_pos` is in coordinate space of the doc.
    pub(crate) async fn load_in_pdf_bytes(
        &self,
        bytes: Vec<u8>,
        target_pos: Option<na::Vector2<f64>>,
        page_range: Option<Range<u32>>,
    ) -> anyhow::Result<()> {
        let pos = target_pos.unwrap_or_else(|| {
            self.engine_ref()
                .camera
                .transform()
                .inverse()
                .transform_point(&na::Point2::from(Stroke::IMPORT_OFFSET_DEFAULT))
                .coords
                .maxs(&na::vector![
                    self.engine_ref().document.x,
                    self.engine_ref().document.y
                ])
        });

        let strokes_receiver = self
            .engine_mut()
            .generate_pdf_pages_from_bytes(bytes, pos, page_range);
        let strokes = strokes_receiver.await??;

        let widget_flags = self.engine_mut().import_generated_content(strokes);

        self.emit_handle_widget_flags(widget_flags);
        Ok(())
    }

    /// Imports a text.
    ///
    /// `target_pos` is in coordinate space of the doc.
    pub(crate) fn load_in_text(
        &self,
        text: String,
        target_pos: Option<na::Vector2<f64>>,
    ) -> anyhow::Result<()> {
        let pos = target_pos.unwrap_or_else(|| {
            self.engine_ref()
                .camera
                .transform()
                .inverse()
                .transform_point(&na::Point2::from(Stroke::IMPORT_OFFSET_DEFAULT))
                .coords
                .maxs(&na::vector![
                    self.engine_ref().document.x,
                    self.engine_ref().document.y
                ])
        });
        let widget_flags = self.engine_mut().insert_text(text, Some(pos));
        self.emit_handle_widget_flags(widget_flags);
        Ok(())
    }

    /// Deserializes the stroke content and inserts it into the engine.
    ///
    /// The data is usually coming from the clipboard, drop source, etc.
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
            .engine_ref()
            .camera
            .transform()
            .inverse()
            .transform_point(&na::Point2::from(Stroke::IMPORT_OFFSET_DEFAULT))
            .coords
            .maxs(&na::vector![
                self.engine_ref().document.x,
                self.engine_ref().document.y
            ]);

        let widget_flags = self.engine_mut().insert_stroke_content(content, pos);

        self.emit_handle_widget_flags(widget_flags);
        Ok(())
    }

    /// Saves the document to the given file.
    ///
    /// Returns Ok(true) if saved successfully, Ok(false) when a save is already in progress and no file operatiosn were
    /// executed, Err(e) when saving failed in any way.
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

        let rnote_bytes_receiver = self
            .engine_ref()
            .save_as_rnote_bytes(basename.to_string_lossy().to_string());

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
        let export_bytes = self.engine_ref().export_doc(title, export_prefs_override);

        crate::utils::create_replace_file_future(export_bytes.await??, file).await?;

        Ok(())
    }

    /// Exports document pages
    /// `file_stem_name`: the stem name of the created files. This is extended by an enumeration of the page number and
    /// file extension overwrites existing files with the same name!
    pub(crate) async fn export_doc_pages(
        &self,
        dir: &gio::File,
        file_stem_name: String,
        export_prefs_override: Option<DocPagesExportPrefs>,
    ) -> anyhow::Result<()> {
        let export_prefs =
            export_prefs_override.unwrap_or(self.engine_ref().export_prefs.doc_pages_export_prefs);

        let file_ext = export_prefs.export_format.file_ext();

        let export_bytes = self.engine_ref().export_doc_pages(export_prefs_override);

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
        let export_bytes = self.engine_ref().export_selection(export_prefs_override);

        if let Some(export_bytes) = export_bytes.await?? {
            crate::utils::create_replace_file_future(export_bytes, file).await?;
        }

        Ok(())
    }

    /// exports and writes the engine state as json into the file.
    /// Only for debugging!
    pub(crate) async fn export_engine_state(&self, file: &gio::File) -> anyhow::Result<()> {
        let exported_engine_state = self.engine_ref().export_state_as_json()?;

        crate::utils::create_replace_file_future(exported_engine_state.into_bytes(), file).await?;

        Ok(())
    }

    /// exports and writes the engine config as json into the file.
    /// Only for debugging!
    pub(crate) async fn export_engine_config(&self, file: &gio::File) -> anyhow::Result<()> {
        let exported_engine_config = self.engine_ref().export_engine_config_as_json()?;

        crate::utils::create_replace_file_future(exported_engine_config.into_bytes(), file).await?;

        Ok(())
    }
}
