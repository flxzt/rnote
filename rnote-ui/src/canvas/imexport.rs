use std::ops::Range;
use std::path::Path;

use gtk4::{gio, prelude::*};
use rnote_engine::engine::export::{DocExportPrefs, DocPagesExportPrefs, SelectionExportPrefs};
use rnote_engine::engine::EngineSnapshot;
use rnote_engine::strokes::Stroke;
use rnote_engine::WidgetFlags;

use super::RnoteCanvas;

impl RnoteCanvas {
    pub(crate) async fn load_in_rnote_bytes<P>(
        &self,
        bytes: Vec<u8>,
        path: Option<P>,
    ) -> anyhow::Result<WidgetFlags>
    where
        P: AsRef<Path>,
    {
        let engine_snapshot = EngineSnapshot::load_from_rnote_bytes(bytes).await?;

        let mut widget_flags = self.engine().borrow_mut().load_snapshot(engine_snapshot);

        if let Some(path) = path {
            let file = gio::File::for_path(path);
            self.dismiss_output_file_modified_toast();
            self.set_output_file(Some(file));
        }

        self.set_unsaved_changes(false);
        self.set_empty(false);
        self.return_to_origin_page();

        self.regenerate_background_pattern();
        self.engine().borrow_mut().resize_autoexpand();
        self.update_engine_rendering();

        widget_flags.refresh_ui = true;

        Ok(widget_flags)
    }

    pub(crate) async fn load_in_vectorimage_bytes(
        &self,
        bytes: Vec<u8>,
        // In coordinate space of the doc
        target_pos: Option<na::Vector2<f64>>,
    ) -> anyhow::Result<WidgetFlags> {
        let pos = target_pos.unwrap_or_else(|| {
            (self.engine().borrow().camera.transform().inverse()
                * na::Point2::from(Stroke::IMPORT_OFFSET_DEFAULT))
            .coords
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

        Ok(widget_flags)
    }

    /// Target position is in the coordinate space of the doc
    pub(crate) async fn load_in_bitmapimage_bytes(
        &self,
        bytes: Vec<u8>,
        // In the coordinate space of the doc
        target_pos: Option<na::Vector2<f64>>,
    ) -> anyhow::Result<WidgetFlags> {
        let pos = target_pos.unwrap_or_else(|| {
            (self.engine().borrow().camera.transform().inverse()
                * na::Point2::from(Stroke::IMPORT_OFFSET_DEFAULT))
            .coords
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

        Ok(widget_flags)
    }

    pub(crate) async fn load_in_xopp_bytes(&self, bytes: Vec<u8>) -> anyhow::Result<WidgetFlags> {
        let xopp_import_prefs = self.engine().borrow_mut().import_prefs.xopp_import_prefs;

        let engine_snapshot =
            EngineSnapshot::load_from_xopp_bytes(bytes, xopp_import_prefs).await?;

        let mut widget_flags = self.engine().borrow_mut().load_snapshot(engine_snapshot);

        self.set_output_file(None);
        self.set_unsaved_changes(true);
        self.set_empty(false);
        self.return_to_origin_page();
        self.regenerate_background_pattern();
        self.engine().borrow_mut().resize_autoexpand();
        self.update_engine_rendering();

        widget_flags.refresh_ui = true;

        Ok(widget_flags)
    }

    /// Target position is in the coordinate space of the doc
    pub(crate) async fn load_in_pdf_bytes(
        &self,
        bytes: Vec<u8>,
        target_pos: Option<na::Vector2<f64>>,
        page_range: Option<Range<u32>>,
    ) -> anyhow::Result<WidgetFlags> {
        let pos = target_pos.unwrap_or_else(|| {
            (self.engine().borrow().camera.transform().inverse()
                * na::Point2::from(Stroke::IMPORT_OFFSET_DEFAULT))
            .coords
        });

        let strokes_receiver = self
            .engine()
            .borrow_mut()
            .generate_pdf_pages_from_bytes(bytes, pos, page_range);
        let strokes = strokes_receiver.await??;

        let widget_flags = self.engine().borrow_mut().import_generated_strokes(strokes);

        Ok(widget_flags)
    }

    /// Target position is in the coordinate space of the doc
    pub(crate) fn load_in_text(
        &self,
        text: String,
        target_pos: Option<na::Vector2<f64>>,
    ) -> anyhow::Result<WidgetFlags> {
        let pos = target_pos.unwrap_or_else(|| {
            (self.engine().borrow().camera.transform().inverse()
                * na::Point2::from(Stroke::IMPORT_OFFSET_DEFAULT))
            .coords
        });

        let widget_flags = self.engine().borrow_mut().insert_text(text, pos)?;

        Ok(widget_flags)
    }

    pub(crate) async fn save_document_to_file(&self, file: &gio::File) -> anyhow::Result<()> {
        if let Some(basename) = file.basename() {
            let rnote_bytes_receiver = self
                .engine()
                .borrow()
                .save_as_rnote_bytes(basename.to_string_lossy().to_string())?;

            self.set_output_file_expect_write(true);
            self.dismiss_output_file_modified_toast();

            crate::utils::create_replace_file_future(rnote_bytes_receiver.await??, file).await?;

            self.set_output_file(Some(file.to_owned()));
            self.set_unsaved_changes(false);
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
        let exported_engine_config = self.engine().borrow().save_engine_config()?;

        crate::utils::create_replace_file_future(exported_engine_config.into_bytes(), file).await?;

        Ok(())
    }
}
