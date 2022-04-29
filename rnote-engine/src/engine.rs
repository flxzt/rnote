use crate::pens::penholder::PenHolderEvent;
use crate::sheet::{background, Background, Format};
use crate::store::StoreTask;
use crate::strokes::Stroke;
use crate::{render, DrawOnSheetBehaviour, SurfaceFlags};
use crate::{Camera, PenHolder, Sheet, StrokeStore};
use gtk4::Snapshot;
use itertools::Itertools;
use num_derive::{FromPrimitive, ToPrimitive};
use rnote_compose::helpers::AABBHelpers;
use rnote_compose::transform::TransformBehaviour;
use rnote_fileformats::rnoteformat::RnotefileMaj0Min5;
use rnote_fileformats::xoppformat;
use rnote_fileformats::FileFormatLoader;
use rnote_fileformats::FileFormatSaver;

use anyhow::Context;
use futures::channel::oneshot;
use p2d::bounding_volume::{BoundingVolume, AABB};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, FromPrimitive, ToPrimitive)]
#[serde(rename = "expand_mode")]
pub enum ExpandMode {
    #[serde(rename = "fixed_size")]
    FixedSize,
    #[serde(rename = "endless_vertical")]
    EndlessVertical,
    #[serde(rename = "infinite")]
    Infinite,
}

impl Default for ExpandMode {
    fn default() -> Self {
        Self::Infinite
    }
}

#[allow(missing_debug_implementations)]
#[derive(Serialize, Deserialize)]
#[serde(default, rename = "engine_config")]
struct EngineConfig {
    #[serde(rename = "sheet")]
    sheet: serde_json::Value,
    #[serde(rename = "penholder")]
    penholder: serde_json::Value,
    #[serde(rename = "expand_mode")]
    expand_mode: serde_json::Value,
}

impl Default for EngineConfig {
    fn default() -> Self {
        let engine = RnoteEngine::default();
        Self {
            sheet: serde_json::to_value(&engine.sheet).unwrap(),
            penholder: serde_json::to_value(&engine.penholder).unwrap(),
            expand_mode: serde_json::to_value(&engine.expand_mode).unwrap(),
        }
    }
}

#[allow(missing_debug_implementations)]
#[derive(Serialize, Deserialize)]
#[serde(default, rename = "engine")]
pub struct RnoteEngine {
    #[serde(rename = "sheet")]
    pub sheet: Sheet,
    #[serde(rename = "penholder")]
    pub penholder: PenHolder,
    #[serde(rename = "store")]
    pub store: StrokeStore,

    #[serde(rename = "expand_mode")]
    expand_mode: ExpandMode,
    #[serde(rename = "camera")]
    pub camera: Camera,
    #[serde(skip)]
    pub visual_debug: bool,
}

impl Default for RnoteEngine {
    fn default() -> Self {
        Self {
            sheet: Sheet::default(),
            penholder: PenHolder::default(),
            store: StrokeStore::default(),

            expand_mode: ExpandMode::default(),
            camera: Camera::default(),
            visual_debug: false,
        }
    }
}

impl RnoteEngine {
    pub fn expand_mode(&self) -> ExpandMode {
        self.expand_mode
    }

    pub fn set_expand_mode(&mut self, expand_mode: ExpandMode) {
        self.expand_mode = expand_mode;

        self.resize_to_fit_strokes();
        self.store.regenerate_rendering_in_viewport_threaded(
            false,
            self.camera.viewport(),
            self.camera.image_scale(),
        );
    }

    /// processes the received task from tasks_rx.
    /// Returns surface flags to indicate what needs to be updated in the UI.
    /// An example how to use it:
    /// ```rust, ignore
    /// let main_cx = glib::MainContext::default();

    /// main_cx.spawn_local(clone!(@strong canvas, @strong appwindow => async move {
    ///            let mut task_rx = canvas.engine().borrow_mut().store.tasks_rx.take().unwrap();

    ///           loop {
    ///              if let Some(task) = task_rx.next().await {
    ///                    let surface_flags = canvas.engine().borrow_mut().process_received_task(task);
    ///                    appwindow.handle_surface_flags(surface_flags);
    ///                }
    ///            }
    ///        }));
    /// ```
    pub fn process_received_task(&mut self, task: StoreTask) -> SurfaceFlags {
        self.store.process_received_task(task, &self.camera)
    }

    /// Public method to call to handle pen events and to change pen holder state
    pub fn handle_penholder_event(&mut self, event: PenHolderEvent) -> SurfaceFlags {
        self.penholder.handle_penholder_event(
            event,
            &mut self.sheet,
            &mut self.store,
            &mut self.camera,
        )
    }

    // Generates bounds for each page which is containing content, extended to fit the sheet format
    pub fn pages_bounds_containing_content(&self) -> Vec<AABB> {
        let sheet_bounds = self.sheet.bounds();
        let keys = self.store.stroke_keys_as_rendered();
        let strokes_bounds = self.store.bounds_for_strokes(&keys);

        if self.sheet.format.height > 0.0 && self.sheet.format.width > 0.0 {
            sheet_bounds
                .split_extended_origin_aligned(na::vector![
                    self.sheet.format.width,
                    self.sheet.format.height
                ])
                .into_iter()
                .filter(|current_page_bounds| {
                    strokes_bounds
                        .iter()
                        .any(|stroke_bounds| stroke_bounds.intersects(&current_page_bounds))
                })
                .collect::<Vec<AABB>>()
        } else {
            vec![]
        }
    }

    /// Generates bounds which contain all pages with content, and are extended to align with the format.
    pub fn bounds_w_content_extended(&self) -> Option<AABB> {
        let bounds = self.pages_bounds_containing_content();
        if bounds.is_empty() {
            return None;
        }

        let sheet_bounds = self.sheet.bounds();

        Some(
            bounds
                .into_iter()
                // Filter out the page bounds that are not intersecting with the sheet bounds.
                .filter(|bounds| sheet_bounds.intersects(&bounds.tightened(2.0)))
                .fold(AABB::new_invalid(), |prev, next| prev.merged(&next)),
        )
    }

    /// Generates all containing svgs for the sheet without root or xml header for the entire size
    pub fn gen_svgs(&self) -> Result<Vec<render::Svg>, anyhow::Error> {
        let sheet_bounds = self.sheet.bounds();
        let mut svgs = vec![];

        svgs.push(self.sheet.background.gen_svg(sheet_bounds.loosened(1.0))?);

        let strokes = self.store.stroke_keys_as_rendered();
        svgs.append(&mut self.store.gen_svgs_for_strokes(&strokes));

        Ok(svgs)
    }

    /// Generates all svgs intersecting the specified bounds, including the background, without root or xml header
    pub fn gen_svgs_intersecting_bounds(
        &self,
        viewport: AABB,
    ) -> Result<Vec<render::Svg>, anyhow::Error> {
        let sheet_bounds = self.sheet.bounds();
        let mut svgs = vec![];

        // Background bounds are still sheet bounds, for alignment
        svgs.push(self.sheet.background.gen_svg(sheet_bounds.loosened(1.0))?);

        let keys = self
            .store
            .stroke_keys_as_rendered_intersecting_bounds(viewport);

        svgs.append(&mut self.store.gen_svgs_for_strokes(&keys));

        Ok(svgs)
    }

    /// Called when sheet should resize to the format and to fit all strokes
    pub fn resize_to_fit_strokes(&mut self) {
        match self.expand_mode {
            ExpandMode::FixedSize => {
                self.sheet.resize_sheet_mode_fixed_size(&self.store);
            }
            ExpandMode::EndlessVertical => {
                self.sheet.resize_sheet_mode_endless_vertical(&self.store);
            }
            ExpandMode::Infinite => {
                self.sheet
                    .resize_sheet_mode_infinite_to_fit_strokes(&self.store);
                self.sheet
                    .expand_sheet_mode_infinite(self.camera.viewport());
            }
        }

        if let Err(e) = self
            .sheet
            .background
            .update_rendernodes(self.sheet.bounds())
        {
            log::error!("failed to update rendernodes for background in resize_to_fit_strokes() with Err {}", e);
        }
    }

    /// resize the sheet when in autoexpanding expand modes. called e.g. when finishing a new stroke
    pub fn resize_autoexpand(&mut self) {
        match self.expand_mode {
            ExpandMode::FixedSize => {
                // Does not resize in fixed size mode, use resize_sheet_to_fit_strokes() for it.
            }
            ExpandMode::EndlessVertical => {
                self.sheet.resize_sheet_mode_endless_vertical(&self.store);
            }
            ExpandMode::Infinite => {
                self.sheet
                    .resize_sheet_mode_infinite_to_fit_strokes(&self.store);
                self.sheet
                    .expand_sheet_mode_infinite(self.camera.viewport());
            }
        }

        if let Err(e) = self
            .sheet
            .background
            .update_rendernodes(self.sheet.bounds())
        {
            log::error!("failed to update rendernodes for background in resize_autoexpand() with Err {}", e);
        }
    }

    pub fn resize_new_offset(&mut self) {
        match self.expand_mode {
            ExpandMode::FixedSize => {
                // Does not resize in fixed size mode, use resize_sheet_to_fit_strokes() for it.
            }
            ExpandMode::EndlessVertical => {
                self.sheet.resize_sheet_mode_endless_vertical(&self.store);
            }
            ExpandMode::Infinite => {
                self.sheet
                    .expand_sheet_mode_infinite(self.camera.viewport());
            }
        }

        if let Err(e) = self
            .sheet
            .background
            .update_rendernodes(self.sheet.bounds())
        {
            log::error!("failed to update rendernodes for background in resize_new_offset() with Err {}", e);
        }
    }

    pub fn update_selector(&mut self) {
        self.penholder
            .selector
            .update_from_store(&self.store);
    }

    /// Import and replace the engine config. NOT for opening files
    pub fn load_engine_config(&mut self, serialized_config: &str) -> anyhow::Result<()> {
        let engine_config = serde_json::from_str::<EngineConfig>(serialized_config)?;

        self.sheet = serde_json::from_value(engine_config.sheet)?;
        self.penholder
            .import(serde_json::from_value(engine_config.penholder)?);
        self.expand_mode = serde_json::from_value(engine_config.expand_mode)?;

        Ok(())
    }

    pub fn save_engine_config(&self) -> anyhow::Result<String> {
        let engine_config = EngineConfig {
            sheet: serde_json::to_value(&self.sheet)?,
            penholder: serde_json::to_value(&self.penholder)?,
            expand_mode: serde_json::to_value(&self.expand_mode)?,
        };

        Ok(serde_json::to_string(&engine_config)?)
    }

    pub fn open_from_rnote_bytes(&mut self, bytes: &[u8]) -> anyhow::Result<()> {
        let rnote_file = RnotefileMaj0Min5::load_from_bytes(bytes)?;

        self.sheet = serde_json::from_value(rnote_file.sheet)?;
        self.expand_mode = serde_json::from_value(rnote_file.expand_mode)?;
        self.store
            .import_store(serde_json::from_value(rnote_file.store)?);

        self.update_selector();

        Ok(())
    }

    pub fn save_as_rnote_bytes(&self, file_name: &str) -> anyhow::Result<Vec<u8>> {
        let rnote_file = RnotefileMaj0Min5 {
            sheet: serde_json::to_value(&self.sheet)?,
            expand_mode: serde_json::to_value(&self.expand_mode)?,
            store: serde_json::to_value(&self.store)?,
        };

        Ok(rnote_file.save_as_bytes(file_name)?)
    }

    /// for debugging the current engine state
    pub fn export_state_as_json(&self) -> anyhow::Result<String> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    pub fn open_from_xopp_bytes(&mut self, bytes: &[u8]) -> anyhow::Result<()> {
        let xopp_file = xoppformat::XoppFile::load_from_bytes(bytes)?;

        // Extract the largest width of all sheets, add together all heights
        let (sheet_width, sheet_height) = xopp_file
            .xopp_root
            .pages
            .iter()
            .map(|page| (page.width, page.height))
            .fold((0_f64, 0_f64), |prev, next| {
                // Max of width, sum heights
                (prev.0.max(next.0), prev.1 + next.1)
            });
        let no_pages = xopp_file.xopp_root.pages.len() as u32;

        let mut sheet = Sheet::default();
        let mut format = Format::default();
        let mut background = Background::default();
        let mut store = StrokeStore::default();
        // We set the sheet dpi to the hardcoded xournal++ dpi, so no need to convert values or coordinates anywhere
        sheet.format.dpi = xoppformat::XoppFile::DPI;

        sheet.x = 0.0;
        sheet.y = 0.0;
        sheet.width = sheet_width;
        sheet.height = sheet_height;

        format.width = sheet_width;
        format.height = sheet_height / f64::from(no_pages);

        if let Some(first_page) = xopp_file.xopp_root.pages.get(0) {
            if let xoppformat::XoppBackgroundType::Solid {
                color: _color,
                style: _style,
            } = &first_page.background.bg_type
            {
                // Background styles would not align with Rnotes background patterns, so everything is plain
                background.pattern = background::PatternStyle::None;
            }
        }

        // Offsetting as rnote has one global coordinate space
        let mut offset = na::Vector2::<f64>::zeros();

        for (_page_i, page) in xopp_file.xopp_root.pages.into_iter().enumerate() {
            for layers in page.layers.into_iter() {
                // import strokes
                for new_xoppstroke in layers.strokes.into_iter() {
                    match Stroke::from_xoppstroke(new_xoppstroke, offset) {
                        Ok(new_stroke) => {
                            store.insert_stroke(new_stroke);
                        }
                        Err(e) => {
                            log::error!(
                                "from_xoppstroke() failed in open_from_xopp_bytes() with Err {}",
                                e
                            );
                        }
                    }
                }

                // import images
                for new_xoppimage in layers.images.into_iter() {
                    match Stroke::from_xoppimage(new_xoppimage, offset) {
                        Ok(new_image) => {
                            store.insert_stroke(new_image);
                        }
                        Err(e) => {
                            log::error!(
                                "from_xoppimage() failed in open_from_xopp_bytes() with Err {}",
                                e
                            );
                        }
                    }
                }
            }

            // Only add to y offset, results in vertical pages
            offset[1] += page.height;
        }

        sheet.background = background;
        sheet.format = format;

        // Import into engine
        self.sheet = sheet;
        self.store.import_store(store);

        self.update_selector();

        Ok(())
    }

    pub fn export_sheet_as_svg_string(&self) -> Result<String, anyhow::Error> {
        let bounds = if let Some(bounds) = self.bounds_w_content_extended() {
            bounds
        } else {
            return Err(anyhow::anyhow!(
                "export_sheet_as_svg() failed, bounds_with_content() returned None"
            ));
        };

        let svgs = self.gen_svgs()?;

        let mut svg_data = svgs
            .iter()
            .map(|svg| svg.svg_data.as_str())
            .collect::<Vec<&str>>()
            .join("\n");

        svg_data = rnote_compose::utils::wrap_svg_root(
            svg_data.as_str(),
            Some(bounds),
            Some(bounds),
            true,
        );

        Ok(svg_data)
    }

    pub fn export_selection_as_svg_string(&self) -> anyhow::Result<Option<String>> {
        let selection_keys = self.store.selection_keys_as_rendered();
        if let Some(selection_bounds) = self.store.gen_bounds_for_strokes(&selection_keys) {
            let mut svg_data = self
                .store
                .gen_svgs_for_strokes(&selection_keys)
                .into_iter()
                .map(|svg| svg.svg_data)
                .join("\n");

            svg_data = rnote_compose::utils::wrap_svg_root(
                svg_data.as_str(),
                Some(selection_bounds),
                Some(selection_bounds),
                true,
            );
            Ok(Some(svg_data))
        } else {
            Ok(None)
        }
    }

    pub fn export_sheet_as_xopp_bytes(&self, filename: &str) -> Result<Vec<u8>, anyhow::Error> {
        let current_dpi = self.sheet.format.dpi;

        // Only one background for all pages
        let background = xoppformat::XoppBackground {
            name: None,
            bg_type: xoppformat::XoppBackgroundType::Solid {
                color: self.sheet.background.color.into(),
                style: xoppformat::XoppBackgroundSolidStyle::Plain,
            },
        };

        // xopp spec needs at least one page in vec, but its fine since pages_bounds() always produces at least one
        let pages = self
            .pages_bounds_containing_content()
            .iter()
            .map(|&page_bounds| {
                let page_keys = self
                    .store
                    .stroke_keys_as_rendered_intersecting_bounds(page_bounds);

                let strokes = self.store.clone_strokes(&page_keys);

                // Translate strokes to to page mins and convert to XoppStrokStyle
                let xopp_strokestyles = strokes
                    .into_iter()
                    .filter_map(|mut stroke| {
                        stroke.translate(-page_bounds.mins.coords);

                        stroke.into_xopp(current_dpi)
                    })
                    .collect::<Vec<xoppformat::XoppStrokeType>>();

                // Extract the strokes
                let xopp_strokes = xopp_strokestyles
                    .iter()
                    .filter_map(|stroke| {
                        if let xoppformat::XoppStrokeType::XoppStroke(xoppstroke) = stroke {
                            Some(xoppstroke.clone())
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<xoppformat::XoppStroke>>();

                // Extract the texts
                let xopp_texts = xopp_strokestyles
                    .iter()
                    .filter_map(|stroke| {
                        if let xoppformat::XoppStrokeType::XoppText(xopptext) = stroke {
                            Some(xopptext.clone())
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<xoppformat::XoppText>>();

                // Extract the images
                let xopp_images = xopp_strokestyles
                    .iter()
                    .filter_map(|stroke| {
                        if let xoppformat::XoppStrokeType::XoppImage(xoppstroke) = stroke {
                            Some(xoppstroke.clone())
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<xoppformat::XoppImage>>();

                let layer = xoppformat::XoppLayer {
                    name: None,
                    strokes: xopp_strokes,
                    texts: xopp_texts,
                    images: xopp_images,
                };

                let page_dimensions = crate::utils::convert_coord_dpi(
                    page_bounds.extents(),
                    current_dpi,
                    xoppformat::XoppFile::DPI,
                );

                xoppformat::XoppPage {
                    width: page_dimensions[0],
                    height: page_dimensions[1],
                    background: background.clone(),
                    layers: vec![layer],
                }
            })
            .collect::<Vec<xoppformat::XoppPage>>();

        let title = String::from("Xournal++ document - see https://github.com/xournalpp/xournalpp (exported from Rnote - see https://github.com/flxzt/rnote)");

        let xopp_root = xoppformat::XoppRoot {
            title,
            fileversion: String::from("4"),
            preview: String::from(""),
            pages,
        };
        let xopp_file = xoppformat::XoppFile { xopp_root };

        let xoppfile_bytes = xopp_file.save_as_bytes(filename)?;

        Ok(xoppfile_bytes)
    }

    /// Returns the receiver to be awaited on for the bytes
    pub fn export_sheet_as_pdf_bytes(
        &self,
        title: String,
    ) -> oneshot::Receiver<anyhow::Result<Vec<u8>>> {
        let (oneshot_sender, oneshot_receiver) = oneshot::channel::<anyhow::Result<Vec<u8>>>();

        let pages = self
            .pages_bounds_containing_content()
            .into_iter()
            .filter_map(|page_bounds| {
                Some((
                    page_bounds,
                    self.gen_svgs_intersecting_bounds(page_bounds).ok()?,
                ))
            })
            .collect::<Vec<(AABB, Vec<render::Svg>)>>();

        let sheet_bounds = self.sheet.bounds();
        let format_size = na::vector![
            f64::from(self.sheet.format.width),
            f64::from(self.sheet.format.height)
        ];

        // Fill the pdf surface on a new thread to avoid blocking
        rayon::spawn(move || {
            let result = || -> anyhow::Result<Vec<u8>> {
                let surface =
                    cairo::PdfSurface::for_stream(format_size[0], format_size[1], Vec::<u8>::new())
                        .context("pdfsurface creation failed")?;

                surface
                    .set_metadata(cairo::PdfMetadata::Title, title.as_str())
                    .context("set pdf surface title metadata failed")?;
                surface
                    .set_metadata(
                        cairo::PdfMetadata::CreateDate,
                        crate::utils::now_formatted_string().as_str(),
                    )
                    .context("set pdf surface date metadata failed")?;

                // New scope to avoid errors when flushing
                {
                    let cairo_cx =
                        cairo::Context::new(&surface).context("cario cx new() failed")?;

                    for (page_bounds, page_svgs) in pages.into_iter() {
                        cairo_cx.translate(-page_bounds.mins[0], -page_bounds.mins[1]);
                        render::Svg::draw_svgs_to_cairo_context(
                            &page_svgs,
                            sheet_bounds,
                            &cairo_cx,
                        )?;
                        cairo_cx.show_page().context("show page failed")?;
                        cairo_cx.translate(page_bounds.mins[0], page_bounds.mins[1]);
                    }
                }
                let data = *surface
                    .finish_output_stream()
                    .map_err(|e| {
                        anyhow::anyhow!(
                            "finish_outputstream() failed in export_sheet_as_pdf_bytes with Err {:?}",
                            e
                        )
                    })?
                    .downcast::<Vec<u8>>()
                    .map_err(|e| {
                        anyhow::anyhow!(
                            "downcast() finished output stream failed in export_sheet_as_pdf_bytes with Err {:?}",
                            e
                        )
                    })?;

                Ok(data)
            };

            if let Err(_data) = oneshot_sender.send(result()) {
                log::error!("sending result from export_sheet_as_pdf_bytes to oneshot_receiver failed. Receiving end already dropped.");
            }
        });

        oneshot_receiver
    }

    pub fn draw(&self, snapshot: &Snapshot, _surface_bounds: AABB) -> anyhow::Result<()> {
        let sheet_bounds = self.sheet.bounds();
        let viewport = self.camera.viewport();

        snapshot.save();
        snapshot.transform(Some(&self.camera.transform_for_gtk_snapshot()));

        self.sheet.draw_shadow(snapshot);

        self.sheet
            .background
            .draw(snapshot, sheet_bounds, &self.camera)?;

        self.sheet
            .format
            .draw(snapshot, sheet_bounds, &self.camera)?;

        self.store
            .draw_strokes_snapshot(snapshot, sheet_bounds, viewport);
        self.store
            .draw_selection_snapshot(snapshot, sheet_bounds, viewport);

        snapshot.restore();

        self.penholder
            .draw_on_sheet_snapshot(snapshot, sheet_bounds, &self.camera)?;
        /*
               {
                   use crate::utils::GrapheneRectHelpers;
                   use gtk4::graphene;
                   use piet::RenderContext;
                   use rnote_compose::helpers::Affine2Helpers;

                   let zoom = self.camera.zoom();

                   let cairo_cx = snapshot.append_cairo(&graphene::Rect::from_p2d_aabb(surface_bounds));
                   let mut piet_cx = piet_cairo::CairoRenderContext::new(&cairo_cx);

                   // Transform to sheet coordinate space
                   piet_cx.transform(self.camera.transform().to_kurbo());

                   piet_cx.save().map_err(|e| anyhow::anyhow!("{}", e))?;
                   self.store
                       .draw_strokes_immediate_w_piet(&mut piet_cx, sheet_bounds, viewport, zoom)?;
                   piet_cx.restore().map_err(|e| anyhow::anyhow!("{}", e))?;

                   piet_cx.save().map_err(|e| anyhow::anyhow!("{}", e))?;

                   self.penholder
                       .draw_on_sheet(&mut piet_cx, sheet_bounds, &self.camera)?;
                   piet_cx.restore().map_err(|e| anyhow::anyhow!("{}", e))?;

                   piet_cx.finish().map_err(|e| anyhow::anyhow!("{}", e))?;
               }
        */
        snapshot.save();
        snapshot.transform(Some(&self.camera.transform_for_gtk_snapshot()));

        // visual debugging
        if self.visual_debug {
            visual_debug::draw_debug(self, snapshot, 1.0 / self.camera.total_zoom());
        }

        snapshot.restore();

        Ok(())
    }
}

/// module for visual debugging
pub mod visual_debug {
    use gtk4::{gdk, graphene, gsk, Snapshot};
    use p2d::bounding_volume::{BoundingVolume, AABB};

    use crate::pens::eraser::EraserState;
    use crate::pens::penholder::PenStyle;
    use crate::utils::{GdkRGBAHelpers, GrapheneRectHelpers};
    use crate::{DrawOnSheetBehaviour, RnoteEngine};
    use rnote_compose::Color;

    pub const COLOR_POS: Color = Color {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };
    pub const COLOR_POS_ALT: Color = Color {
        r: 1.0,
        g: 1.0,
        b: 0.0,
        a: 1.0,
    };
    pub const COLOR_STROKE_HITBOX: Color = Color {
        r: 0.0,
        g: 0.8,
        b: 0.2,
        a: 0.5,
    };
    pub const COLOR_STROKE_BOUNDS: Color = Color {
        r: 0.0,
        g: 0.8,
        b: 0.8,
        a: 1.0,
    };
    pub const COLOR_IMAGE_BOUNDS: Color = Color {
        r: 0.0,
        g: 0.5,
        b: 1.0,
        a: 1.0,
    };
    pub const COLOR_STROKE_REGENERATE_FLAG: Color = Color {
        r: 0.9,
        g: 0.0,
        b: 0.8,
        a: 0.15,
    };
    pub const COLOR_SELECTOR_BOUNDS: Color = Color {
        r: 1.0,
        g: 0.0,
        b: 0.8,
        a: 1.0,
    };
    pub const COLOR_SHEET_BOUNDS: Color = Color {
        r: 0.8,
        g: 0.0,
        b: 0.8,
        a: 1.0,
    };

    pub fn draw_bounds(bounds: AABB, color: Color, snapshot: &Snapshot, width: f64) {
        let bounds = graphene::Rect::new(
            bounds.mins[0] as f32,
            bounds.mins[1] as f32,
            (bounds.extents()[0]) as f32,
            (bounds.extents()[1]) as f32,
        );

        let rounded_rect = gsk::RoundedRect::new(
            bounds,
            graphene::Size::zero(),
            graphene::Size::zero(),
            graphene::Size::zero(),
            graphene::Size::zero(),
        );

        snapshot.append_border(
            &rounded_rect,
            &[width as f32, width as f32, width as f32, width as f32],
            &[
                gdk::RGBA::from_compose_color(color),
                gdk::RGBA::from_compose_color(color),
                gdk::RGBA::from_compose_color(color),
                gdk::RGBA::from_compose_color(color),
            ],
        )
    }

    pub fn draw_pos(pos: na::Vector2<f64>, color: Color, snapshot: &Snapshot, width: f64) {
        snapshot.append_color(
            &gdk::RGBA::from_compose_color(color),
            &graphene::Rect::new(
                (pos[0] - 0.5 * width) as f32,
                (pos[1] - 0.5 * width) as f32,
                width as f32,
                width as f32,
            ),
        );
    }

    pub fn draw_fill(rect: AABB, color: Color, snapshot: &Snapshot) {
        snapshot.append_color(
            &gdk::RGBA::from_compose_color(color),
            &graphene::Rect::from_p2d_aabb(rect),
        );
    }

    // Draw bounds, positions, .. for visual debugging purposes
    pub fn draw_debug(engine: &RnoteEngine, snapshot: &Snapshot, border_widths: f64) {
        let viewport = engine.camera.viewport();
        let sheet_bounds = engine.sheet.bounds();

        draw_bounds(sheet_bounds, COLOR_SHEET_BOUNDS, snapshot, border_widths);

        let tightened_viewport = viewport.tightened(3.0);
        draw_bounds(
            tightened_viewport,
            COLOR_STROKE_BOUNDS,
            snapshot,
            border_widths,
        );

        // Draw the strokes and selection
        engine.store.draw_debug(snapshot, border_widths);

        // Draw the pens
        let current_pen_style = engine.penholder.style_w_override();

        match current_pen_style {
            PenStyle::Eraser => {
                if let EraserState::Down(current_element) = engine.penholder.eraser.state {
                    draw_pos(
                        current_element.pos,
                        COLOR_POS_ALT,
                        snapshot,
                        border_widths * 4.0,
                    );
                }
            }
            PenStyle::Selector => {
                if let Some(bounds) = engine
                    .penholder
                    .selector
                    .bounds_on_sheet(sheet_bounds, &engine.camera)
                {
                    draw_bounds(bounds, COLOR_SELECTOR_BOUNDS, snapshot, border_widths);
                }
            }
            PenStyle::Brush | PenStyle::Shaper | PenStyle::Tools => {}
        }
    }
}
