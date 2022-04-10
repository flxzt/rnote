use crate::pens::penholder::PenHolderEvent;
use crate::sheet::{background, Background, Format};
use crate::strokes::Stroke;
use crate::utils;
use crate::{render, DrawOnSheetBehaviour, SurfaceFlags};
use crate::{Camera, PenHolder, Sheet, StrokesState};
use gtk4::Snapshot;
use num_derive::{FromPrimitive, ToPrimitive};
use rnote_compose::helpers::AABBHelpers;
use rnote_compose::transform::TransformBehaviour;
use rnote_fileformats::xoppformat;
use rnote_fileformats::FileFormatLoader;
use rnote_fileformats::FileFormatSaver;

use anyhow::Context;
use futures::channel::oneshot;
use p2d::bounding_volume::{BoundingVolume, AABB};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, FromPrimitive, ToPrimitive)]
pub enum ExpandMode {
    FixedSize,
    EndlessVertical,
    Infinite,
}

impl Default for ExpandMode {
    fn default() -> Self {
        Self::FixedSize
    }
}

#[allow(missing_debug_implementations)]
#[derive(Serialize, Deserialize)]
pub struct RnoteEngine {
    pub sheet: Sheet,
    pub penholder: PenHolder,
    #[serde(rename = "strokes_state")]
    pub strokes_state: StrokesState,

    #[serde(rename = "expand_mode")]
    expand_mode: ExpandMode,
    #[serde(skip)]
    pub camera: Camera,

    pub visual_debug: bool,
}

impl Default for RnoteEngine {
    fn default() -> Self {
        Self {
            sheet: Sheet::default(),
            penholder: PenHolder::default(),
            strokes_state: StrokesState::default(),

            expand_mode: ExpandMode::default(),
            camera: Camera::default(),
            visual_debug: false,
        }
    }
}

impl RnoteEngine {
    /// Import and replace the engine. NOT for opening files
    pub fn import_engine(&mut self, engine: Self) {
        self.sheet.import_sheet(engine.sheet);
        self.penholder = engine.penholder;
        self.strokes_state
            .import_strokes_state(engine.strokes_state);
        self.expand_mode = engine.expand_mode;
        self.camera = engine.camera;
        self.visual_debug = engine.visual_debug;
    }

    pub fn expand_mode(&self) -> ExpandMode {
        self.expand_mode
    }

    pub fn set_expand_mode(&mut self, expand_mode: ExpandMode) {
        self.expand_mode = expand_mode;

        let viewport = self.camera.viewport();
        let image_scale = self.camera.image_scale();
        self.resize_to_fit_strokes();
        self.strokes_state
            .regenerate_rendering_in_viewport_threaded(false, viewport, image_scale);
    }

    /// Public method to handle pen events coming from ui event handlers
    pub fn handle_event(&mut self, event: PenHolderEvent) -> SurfaceFlags {
        self.penholder.handle_event(
            event,
            &mut self.sheet,
            &mut self.strokes_state,
            &mut self.camera,
        )
    }

    // Generates bounds for each page which is containing content, extended to fit the sheet format
    pub fn pages_bounds_containing_content(&self) -> Vec<AABB> {
        let sheet_bounds = self.sheet.bounds();
        let keys = self.strokes_state.keys_as_rendered();
        let strokes_bounds = self.strokes_state.strokes_bounds(&keys);

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

        svgs.append(&mut self.strokes_state.gen_svgs_all_strokes());

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

        svgs.append(&mut self.strokes_state.gen_svgs_for_bounds(viewport));

        Ok(svgs)
    }

    /// Called when sheet should resize to the format and to fit all strokes
    pub fn resize_to_fit_strokes(&mut self) {
        match self.expand_mode {
            ExpandMode::FixedSize => {
                self.sheet.resize_sheet_mode_fixed_size(&self.strokes_state);
            }
            ExpandMode::EndlessVertical => {
                self.sheet
                    .resize_sheet_mode_endless_vertical(&self.strokes_state);
            }
            ExpandMode::Infinite => {
                self.sheet
                    .resize_sheet_mode_infinite_to_fit_strokes(&self.strokes_state);
                self.sheet
                    .expand_sheet_mode_infinite(self.camera.viewport());
            }
        }
    }

    /// resize the sheet when in autoexpanding expand modes. called e.g. when finishing a new stroke
    pub fn resize_autoexpand(&mut self) {
        match self.expand_mode {
            ExpandMode::FixedSize => {
                // Does not resize in fixed size mode, use resize_sheet_to_fit_strokes() for it.
            }
            ExpandMode::EndlessVertical => {
                self.sheet
                    .resize_sheet_mode_endless_vertical(&self.strokes_state);
            }
            ExpandMode::Infinite => {
                self.sheet
                    .resize_sheet_mode_infinite_to_fit_strokes(&self.strokes_state);
                self.sheet
                    .expand_sheet_mode_infinite(self.camera.viewport());
            }
        }
    }

    pub fn resize_new_offset(&mut self) {
        match self.expand_mode {
            ExpandMode::FixedSize => {
                // Does not resize in fixed size mode, use resize_sheet_to_fit_strokes() for it.
            }
            ExpandMode::EndlessVertical => {
                self.sheet
                    .resize_sheet_mode_endless_vertical(&self.strokes_state);
            }
            ExpandMode::Infinite => {
                self.sheet
                    .expand_sheet_mode_infinite(self.camera.viewport());
            }
        }
    }

    pub fn open_sheet_from_rnote_bytes(&mut self, bytes: &[u8]) -> Result<(), anyhow::Error> {
        let decompressed_bytes = utils::decompress_from_gzip(bytes)?;
        let engine: Self = serde_json::from_str(&String::from_utf8(decompressed_bytes)?)?;

        self.sheet.import_sheet(engine.sheet);
        self.strokes_state
            .import_strokes_state(engine.strokes_state);
        self.expand_mode = engine.expand_mode;

        Ok(())
    }

    pub fn open_from_xopp_bytes(&mut self, bytes: &[u8]) -> Result<(), anyhow::Error> {
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
        let mut strokes_state = StrokesState::default();
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
                            strokes_state.insert_stroke(new_stroke);
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
                            strokes_state.insert_stroke(new_image);
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

        self.sheet.import_sheet(sheet);
        self.strokes_state.import_strokes_state(strokes_state);

        Ok(())
    }

    pub fn save_sheet_as_rnote_bytes(&self, filename: &str) -> Result<Vec<u8>, anyhow::Error> {
        let json_output = serde_json::to_string(self)?;

        let compressed_bytes = utils::compress_to_gzip(json_output.as_bytes(), filename)?;

        Ok(compressed_bytes)
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
                let page_keys = self.strokes_state.keys_intersecting_bounds(page_bounds);

                let strokes = self.strokes_state.clone_strokes_for_keys(&page_keys);

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

                let page_dimensions = utils::convert_coord_dpi(
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
    pub fn export_sheet_as_pdf_bytes(&self, title: String) -> oneshot::Receiver<Vec<u8>> {
        let (oneshot_sender, oneshot_receiver) = oneshot::channel::<Vec<u8>>();

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
            if let Err(e) = || -> Result<(), anyhow::Error> {
                let surface =
                    cairo::PdfSurface::for_stream(format_size[0], format_size[1], Vec::<u8>::new())
                        .context("pdfsurface creation failed")?;

                surface
                    .set_metadata(cairo::PdfMetadata::Title, title.as_str())
                    .context("set pdf surface title metadata failed")?;
                surface
                    .set_metadata(
                        cairo::PdfMetadata::CreateDate,
                        utils::now_formatted_string().as_str(),
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

                oneshot_sender.send(data).map_err(|e| {
                    anyhow::anyhow!(
                        "oneshot_sender.send() failed in export_sheet_as_pdf_bytes with Err {:?}",
                        e
                    )
                })?;
                Ok(())
            }() {
                log::error!("export_sheet_as pdf_bytes() failed with Err, {}", e);
            }
        });

        oneshot_receiver
    }

    pub fn draw(&self, snapshot: &Snapshot, _surface_bounds: AABB) -> Result<(), anyhow::Error> {
        let sheet_bounds = self.sheet.bounds();
        let viewport = self.camera.viewport();

        snapshot.save();
        snapshot.transform(Some(&self.camera.transform_as_gsk()));

        self.sheet.draw_shadow(snapshot);
        self.sheet.background.draw(snapshot, sheet_bounds);
        self.sheet
            .format
            .draw(snapshot, sheet_bounds, Some(viewport))?;

        self.strokes_state
            .draw_strokes(snapshot, sheet_bounds, Some(viewport));
        self.strokes_state
            .draw_selection(snapshot, sheet_bounds, Some(viewport));

        snapshot.restore();

        self.penholder
            .draw_on_sheet_snapshot(snapshot, sheet_bounds, &self.camera)?;

        /*         {
            use piet::RenderContext;
            use rnote_compose::helpers::Affine2Helpers;
            let zoom = self.camera.zoom();

            let cairo_cx = snapshot.append_cairo(&surface_bounds.to_graphene_rect());
            let mut piet_cx = piet_cairo::CairoRenderContext::new(&cairo_cx);

            // Transform to sheet coordinate space
            piet_cx.transform(self.camera.transform().to_kurbo());

            piet_cx.save().map_err(|e| anyhow::anyhow!("{}", e))?;
            self.strokes_state
                .draw_strokes_immediate_w_piet(&mut piet_cx, sheet_bounds, Some(viewport), zoom)?;
            piet_cx.restore().map_err(|e| anyhow::anyhow!("{}", e))?;

            piet_cx.save().map_err(|e| anyhow::anyhow!("{}", e))?;
            self.penholder
                .draw_on_sheet(&mut piet_cx, sheet_bounds, viewport)?;
            piet_cx.restore().map_err(|e| anyhow::anyhow!("{}", e))?;

            piet_cx.finish().map_err(|e| anyhow::anyhow!("{}", e))?;
        } */
        snapshot.save();

        snapshot.transform(Some(&self.camera.transform_as_gsk()));

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
    use gtk4::{graphene, gsk, Snapshot, gdk};
    use p2d::bounding_volume::{BoundingVolume, AABB};

    use rnote_compose::shapes::ShapeBehaviour;
    use rnote_compose::Color;

    use crate::pens::penholder::PenStyle;
    use crate::strokes::Stroke;
    use crate::utils::{GdkRGBAHelpers, GrapheneRectHelpers};
    use crate::{DrawOnSheetBehaviour, RnoteEngine};

    const COLOR_POS: Color = Color {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };
    const COLOR_POS_ALT: Color = Color {
        r: 1.0,
        g: 1.0,
        b: 0.0,
        a: 1.0,
    };
    const COLOR_STROKE_HITBOX: Color = Color {
        r: 0.0,
        g: 0.8,
        b: 0.2,
        a: 0.5,
    };
    const COLOR_STROKE_BOUNDS: Color = Color {
        r: 0.0,
        g: 0.8,
        b: 0.8,
        a: 1.0,
    };
    const COLOR_STROKE_REGENERATE_FLAG: Color = Color {
        r: 0.9,
        g: 0.0,
        b: 0.8,
        a: 0.15,
    };
    const COLOR_SELECTOR_BOUNDS: Color = Color {
        r: 1.0,
        g: 0.0,
        b: 0.8,
        a: 1.0,
    };
    const COLOR_SHEET_BOUNDS: Color = Color {
        r: 0.8,
        g: 0.0,
        b: 0.8,
        a: 1.0,
    };

    fn draw_bounds(bounds: AABB, color: Color, snapshot: &Snapshot, width: f64) {
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
                gdk::RGBA::from_compose_color(color)
            ],
        )
    }

    fn draw_pos(pos: na::Vector2<f64>, color: Color, snapshot: &Snapshot, width: f64) {
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

    fn draw_fill(rect: AABB, color: Color, snapshot: &Snapshot) {
        snapshot.append_color(&gdk::RGBA::from_compose_color(color), &graphene::Rect::from_aabb(rect));
    }

    // Draw bounds, positions, .. for visual debugging purposes
    pub fn draw_debug(engine: &RnoteEngine, snapshot: &Snapshot, border_widths: f64) {
        let viewport = engine.camera.viewport();
        let sheet_bounds = engine.sheet.bounds();
        let pen_shown = engine.penholder.pen_shown();

        draw_bounds(sheet_bounds, COLOR_SHEET_BOUNDS, snapshot, border_widths);

        let tightened_viewport = viewport.tightened(3.0);
        draw_bounds(
            tightened_viewport,
            COLOR_STROKE_BOUNDS,
            snapshot,
            border_widths,
        );

        // Draw the strokes and selection
        engine
            .strokes_state
            .keys_as_rendered()
            .iter()
            .chain(engine.strokes_state.selection_keys_as_rendered().iter())
            .for_each(|&key| {
                if let Some(stroke) = engine.strokes_state.strokes.get(key) {
                    // Push blur and opacity for strokes which are normally hidden
                    if let Some(render_comp) = engine.strokes_state.render_components.get(key) {
                        if let Some(trash_comp) = engine.strokes_state.trash_components.get(key) {
                            if render_comp.render && trash_comp.trashed {
                                snapshot.push_blur(3.0);
                                snapshot.push_opacity(0.2);
                            }
                        }
                        if render_comp.regenerate_flag {
                            draw_fill(stroke.bounds(), COLOR_STROKE_REGENERATE_FLAG, snapshot);
                        }
                    }
                    match stroke {
                        Stroke::BrushStroke(brushstroke) => {
                            for element in brushstroke.path.clone().into_elements().iter() {
                                draw_pos(element.pos, COLOR_POS, snapshot, border_widths * 4.0)
                            }
                            for &hitbox_elem in brushstroke.hitboxes.iter() {
                                draw_bounds(
                                    hitbox_elem,
                                    COLOR_STROKE_HITBOX,
                                    snapshot,
                                    border_widths,
                                );
                            }
                            draw_bounds(
                                brushstroke.bounds(),
                                COLOR_STROKE_BOUNDS,
                                snapshot,
                                border_widths,
                            );
                        }
                        Stroke::ShapeStroke(shapestroke) => {
                            draw_bounds(
                                shapestroke.bounds(),
                                COLOR_STROKE_BOUNDS,
                                snapshot,
                                border_widths,
                            );
                        }
                        Stroke::VectorImage(vectorimage) => {
                            draw_bounds(
                                vectorimage.bounds(),
                                COLOR_STROKE_BOUNDS,
                                snapshot,
                                border_widths,
                            );
                        }
                        Stroke::BitmapImage(bitmapimage) => {
                            draw_bounds(
                                bitmapimage.bounds(),
                                COLOR_STROKE_BOUNDS,
                                snapshot,
                                border_widths,
                            );
                        }
                    }
                    // Pop Blur and opacity for hidden strokes
                    if let (Some(render_comp), Some(trash_comp)) = (
                        engine.strokes_state.render_components.get(key),
                        engine.strokes_state.trash_components.get(key),
                    ) {
                        if render_comp.render && trash_comp.trashed {
                            snapshot.pop();
                            snapshot.pop();
                        }
                    }
                }
            });

        // Draw the pens
        if pen_shown {
            let current_pen_style = engine.penholder.style_w_override();

            match current_pen_style {
                PenStyle::Eraser => {
                    if let Some(current_input) = engine.penholder.eraser.current_input {
                        draw_pos(
                            current_input.pos,
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
}
