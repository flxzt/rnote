// Imports
use super::RnCanvas;
use anyhow::Context;
use futures::channel::oneshot;
use futures::AsyncWriteExt;
use gtk4::{gio, prelude::*};
use rnote_compose::ext::Vector2Ext;
use rnote_engine::engine::export::{DocExportPrefs, DocPagesExportPrefs, SelectionExportPrefs};
use rnote_engine::engine::{EngineSnapshot, StrokeContent};
use rnote_engine::strokes::resize::ImageSizeOption;
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
        let mut widget_flags = self.engine_mut().load_snapshot(engine_snapshot);
        widget_flags |= self
            .engine_mut()
            .set_scale_factor(self.scale_factor() as f64);

        self.set_output_file(file_path.map(gio::File::for_path));
        self.dismiss_output_file_modified_toast();
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
        respect_borders: bool,
    ) -> anyhow::Result<()> {
        let pos = self.determine_stroke_import_pos(target_pos);

        // Splitting the import operation into two parts: a receiver that gets awaited with the content, and
        // the blocking import avoids borrowing the entire engine RefCell while awaiting the content, avoiding panics.
        let vectorimage_receiver =
            self.engine_mut()
                .generate_vectorimage_from_bytes(pos, bytes, respect_borders);
        let vectorimage = vectorimage_receiver.await??;
        let widget_flags = self
            .engine_mut()
            .import_generated_content(vec![(Stroke::VectorImage(vectorimage), None)], false);

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
        respect_borders: bool,
    ) -> anyhow::Result<()> {
        let pos = self.determine_stroke_import_pos(target_pos);

        let bitmapimage_receiver =
            self.engine_mut()
                .generate_bitmapimage_from_bytes(pos, bytes, respect_borders);
        let bitmapimage = bitmapimage_receiver.await??;
        let widget_flags = self
            .engine_mut()
            .import_generated_content(vec![(Stroke::BitmapImage(bitmapimage), None)], false);

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
        let pos = self.determine_stroke_import_pos(target_pos);
        let adjust_document = self
            .engine_ref()
            .import_prefs
            .pdf_import_prefs
            .adjust_document;

        let strokes_receiver = self
            .engine_mut()
            .generate_pdf_pages_from_bytes(bytes, pos, page_range);
        let strokes = strokes_receiver.await??;
        let widget_flags = self
            .engine_mut()
            .import_generated_content(strokes, adjust_document);

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
        let pos = self.determine_stroke_import_pos(target_pos);

        let widget_flags = self.engine_mut().insert_text(text, Some(pos));

        self.emit_handle_widget_flags(widget_flags);
        Ok(())
    }

    /// Deserializes the stroke content and inserts it into the engine.
    ///
    /// The data is usually coming from the clipboard, drop source, etc.
    pub(crate) async fn insert_stroke_content(
        &self,
        json_string: String,
        resize_option: ImageSizeOption,
        target_pos: Option<na::Vector2<f64>>,
    ) -> anyhow::Result<()> {
        let (oneshot_sender, oneshot_receiver) =
            oneshot::channel::<anyhow::Result<StrokeContent>>();
        let pos = self.determine_stroke_import_pos(target_pos);

        rayon::spawn(move || {
            let result = || -> Result<StrokeContent, anyhow::Error> {
                Ok(serde_json::from_str(&json_string)?)
            };
            if oneshot_sender.send(result()).is_err() {
                tracing::error!(
                    "Sending result to receiver while inserting stroke content failed. Receiver already dropped."
                );
            }
        });
        let content = oneshot_receiver.await??;
        let widget_flags = self
            .engine_mut()
            .insert_stroke_content(content, pos, resize_option);

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
            tracing::debug!("Saving file already in progress.");
            return Ok(false);
        }
        self.set_save_in_progress(true);

        let file_path = file
            .path()
            .ok_or_else(|| anyhow::anyhow!("Could not get a path for file: `{file:?}`."))?;
        let basename = file
            .basename()
            .ok_or_else(|| anyhow::anyhow!("Could not retrieve basename for file: `{file:?}`."))?;
        let rnote_bytes_receiver = self
            .engine_ref()
            .save_as_rnote_bytes(basename.to_string_lossy().to_string());
        let mut skip_set_output_file = false;
        if let Some(current_file_path) = self.output_file().and_then(|f| f.path()) {
            if crate::utils::paths_abs_eq(current_file_path, &file_path).unwrap_or(false) {
                skip_set_output_file = true;
            }
        }

        self.dismiss_output_file_modified_toast();

        let file_write_operation = async move {
            let bytes = rnote_bytes_receiver.await??;
            self.set_output_file_expect_write(true);
            let mut write_file = async_fs::OpenOptions::new()
                .create(true)
                .truncate(true)
                .write(true)
                .open(&file_path)
                .await
                .context(format!(
                    "Failed to create/open/truncate file for path '{}'",
                    file_path.display()
                ))?;
            if !skip_set_output_file {
                // this installs the file watcher.
                self.set_output_file(Some(file.to_owned()));
            }
            write_file.write_all(&bytes).await.context(format!(
                "Failed to write bytes to file with path '{}'",
                file_path.display()
            ))?;
            write_file.sync_all().await.context(format!(
                "Failed to sync file after writing with path '{}'",
                file_path.display()
            ))?;
            Ok(())
        };

        if let Err(e) = file_write_operation.await {
            self.set_save_in_progress(false);
            // If the file operations failed in any way, we make sure to clear the expect_write flag
            // because we can't know for sure if the output-file watcher will be able to.
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

        self.set_last_export_dir(file.parent());

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
        if dir.query_file_type(gio::FileQueryInfoFlags::NONE, gio::Cancellable::NONE)
            != gio::FileType::Directory
        {
            return Err(anyhow::anyhow!(
                "Supplied target file `{dir:?}` is not a directory."
            ));
        }
        let export_prefs =
            export_prefs_override.unwrap_or(self.engine_ref().export_prefs.doc_pages_export_prefs);
        let file_ext = export_prefs.export_format.file_ext();

        let export_bytes_recv = self.engine_ref().export_doc_pages(export_prefs_override);
        let export_bytes = export_bytes_recv.await??;

        for (i, page_bytes) in export_bytes.into_iter().enumerate() {
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

        self.set_last_export_dir(Some(dir.clone()));

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

        self.set_last_export_dir(file.parent());

        Ok(())
    }

    /// exports and writes the engine state as json into the file.
    /// Only for debugging!
    pub(crate) async fn export_engine_state(&self, file: &gio::File) -> anyhow::Result<()> {
        let exported_engine_state = self.engine_ref().export_state_as_json()?;

        crate::utils::create_replace_file_future(exported_engine_state.into_bytes(), file).await?;

        self.set_last_export_dir(file.parent());

        Ok(())
    }

    /// exports and writes the engine config as json into the file.
    /// Only for debugging!
    pub(crate) async fn export_engine_config(&self, file: &gio::File) -> anyhow::Result<()> {
        let exported_engine_config = self.engine_ref().export_engine_config_as_json()?;

        crate::utils::create_replace_file_future(exported_engine_config.into_bytes(), file).await?;

        self.set_last_export_dir(file.parent());

        Ok(())
    }

    fn determine_stroke_import_pos(
        &self,
        target_pos: Option<na::Vector2<f64>>,
    ) -> na::Vector2<f64> {
        target_pos.unwrap_or_else(|| {
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
        })
    }
}
