pub mod background;
pub mod format;

use std::sync::{Arc, RwLock};

use crate::compose;
use crate::compose::geometry::AABBHelpers;
use crate::compose::transformable::Transformable;
use crate::render::{self, Renderer};
use crate::strokes::strokestyle::StrokeStyle;
use crate::strokesstate::StrokesState;
use crate::utils;
use anyhow::Context;
use futures::channel::oneshot;
use rnote_fileformats::xoppformat;
use rnote_fileformats::FileFormatLoader;
use rnote_fileformats::FileFormatSaver;

use self::{background::Background, format::Format};

use gtk4::{glib, Snapshot};
use p2d::bounding_volume::{BoundingVolume, AABB};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Serialize, Deserialize)]
#[serde(default, rename = "sheet")]
pub struct Sheet {
    #[serde(rename = "version")]
    pub version: String,
    #[serde(rename = "x")]
    pub x: f64,
    #[serde(rename = "y")]
    pub y: f64,
    #[serde(rename = "width")]
    pub width: f64,
    #[serde(rename = "height")]
    pub height: f64,
    #[serde(rename = "strokes_state")]
    pub strokes_state: StrokesState,
    #[serde(rename = "format")]
    pub format: Format,
    #[serde(rename = "background")]
    pub background: Background,
}

impl Default for Sheet {
    fn default() -> Self {
        Self {
            version: String::from("0.1"),
            x: 0.0,
            y: 0.0,
            width: Format::default().width,
            height: Format::default().height,
            strokes_state: StrokesState::default(),
            format: Format::default(),
            background: Background::default(),
        }
    }
}

impl Sheet {
    pub fn bounds(&self) -> AABB {
        AABB::new(
            na::point![self.x, self.y],
            na::point![self.x + self.width, self.y + self.height],
        )
    }

    /// Generates bounds which contain all pages with content, and are extended to fit the format size.
    pub fn bounds_w_content_extended(&self) -> Option<AABB> {
        let bounds = self.pages_bounds_containing_content();
        if bounds.is_empty() {
            return None;
        }

        let sheet_bounds = self.bounds();

        Some(
            bounds
                .into_iter()
                // Filter out the page bounds that are not intersecting with the sheet bounds.
                .filter(|bounds| sheet_bounds.intersects(&bounds.tightened(2.0)))
                .fold(AABB::new_invalid(), |prev, next| prev.merged(&next)),
        )
    }

    // Generates bounds for each page for the sheet size, extended to fit the sheet format. May contain many empty pages (in infinite mode)
    pub fn pages_bounds(&self) -> Vec<AABB> {
        let sheet_bounds = self.bounds();

        if self.format.height > 0.0 && self.format.width > 0.0 {
            sheet_bounds
                .split_extended_origin_aligned(na::vector![self.format.width, self.format.height])
        } else {
            vec![]
        }
    }

    // Generates bounds for each page which is containing content, extended to fit the sheet format
    pub fn pages_bounds_containing_content(&self) -> Vec<AABB> {
        let sheet_bounds = self.bounds();
        let keys = self.strokes_state.keys_as_rendered();
        let strokes_bounds = &self.strokes_state.strokes_bounds(&keys);

        if self.format.height > 0.0 && self.format.width > 0.0 {
            sheet_bounds
                .split_extended_origin_aligned(na::vector![self.format.width, self.format.height])
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

    pub fn calc_n_pages(&self) -> u32 {
        // Avoid div by 0
        if self.format.height > 0.0 && self.format.width > 0.0 {
            (self.width / self.format.width).round() as u32
                * (self.height / self.format.height).round() as u32
        } else {
            0
        }
    }

    /// Generates all containing svgs for the sheet without root or xml header for the entire size
    pub fn gen_svgs(&self) -> Result<Vec<render::Svg>, anyhow::Error> {
        let sheet_bounds = self.bounds();
        let mut svgs = vec![];

        svgs.push(self.background.gen_svg(sheet_bounds.loosened(1.0))?);

        svgs.append(&mut self.strokes_state.gen_svgs_all_strokes());

        Ok(svgs)
    }

    /// Generates all containing svgs for the sheet without root or xml header for the given viewport
    pub fn gen_svgs_for_viewport(&self, viewport: AABB) -> Result<Vec<render::Svg>, anyhow::Error> {
        let sheet_bounds = self.bounds();
        let mut svgs = vec![];

        // Background bounds are still sheet bounds, for alignment
        svgs.push(self.background.gen_svg(sheet_bounds.loosened(1.0))?);

        svgs.append(&mut self.strokes_state.gen_svgs_for_bounds(viewport));

        Ok(svgs)
    }

    pub fn resize_sheet_mode_fixed_size(&mut self) {
        let format_height = self.format.height;

        let new_width = self.format.width;
        // +1.0 because then 'fraction'.ceil() is at least 1
        let new_height =
            (f64::from(self.strokes_state.calc_height() + 1.0) / f64::from(format_height)).ceil()
                * format_height;

        self.x = 0.0;
        self.y = 0.0;
        self.width = new_width;
        self.height = new_height;
    }

    pub fn resize_sheet_mode_endless_vertical(&mut self) {
        let padding_bottom = self.format.height;
        let new_height = self.strokes_state.calc_height() + padding_bottom;
        let new_width = self.format.width;

        self.x = 0.0;
        self.y = 0.0;
        self.width = new_width;
        self.height = new_height;
    }

    pub fn resize_sheet_mode_infinite_to_fit_strokes(&mut self) {
        let padding_horizontal = self.format.width * 2.0;
        let padding_vertical = self.format.height * 2.0;

        let mut keys = self.strokes_state.keys_as_rendered();
        keys.append(&mut self.strokes_state.selection_keys_as_rendered());

        let new_bounds = if let Some(new_bounds) = self.strokes_state.gen_bounds(&keys) {
            new_bounds.expand(na::vector![padding_horizontal, padding_vertical])
        } else {
            // If sheet is empty, resize to one page with the format size
            AABB::new(
                na::point![0.0, 0.0],
                na::point![self.format.width, self.format.height],
            )
            .expand(na::vector![padding_horizontal, padding_vertical])
        };
        self.x = new_bounds.mins[0];
        self.y = new_bounds.mins[1];
        self.width = new_bounds.extents()[0];
        self.height = new_bounds.extents()[1];
    }

    pub fn expand_sheet_mode_infinite_for_viewport(&mut self, viewport: AABB) {
        let padding_horizontal = self.format.width * 2.0;
        let padding_vertical = self.format.height * 2.0;

        let new_bounds = self
            .bounds()
            .merged(&viewport.expand(na::vector![padding_horizontal, padding_vertical]));

        self.x = new_bounds.mins[0];
        self.y = new_bounds.mins[1];
        self.width = new_bounds.extents()[0];
        self.height = new_bounds.extents()[1];
    }

    // a new sheet should always be imported with this method, as to not replace the threadpool, channel handlers, ..
    pub fn import_sheet(&mut self, sheet: Self) {
        self.x = sheet.x;
        self.y = sheet.y;
        self.width = sheet.width;
        self.height = sheet.height;
        self.strokes_state.import_strokes_state(sheet.strokes_state);
        self.format = sheet.format;
        self.background = sheet.background;
    }

    pub fn draw(&self, zoom: f64, snapshot: &Snapshot, with_borders: bool) {
        snapshot.push_clip(
            &self
                .bounds()
                .scale(na::Vector2::from_element(zoom))
                .to_graphene_rect(),
        );

        self.background.draw(snapshot);

        if with_borders {
            self.format.draw(self.bounds(), snapshot, zoom);
        }

        snapshot.pop();
    }

    pub fn open_sheet_from_rnote_bytes(&mut self, bytes: glib::Bytes) -> Result<(), anyhow::Error> {
        let decompressed_bytes = utils::decompress_from_gzip(&bytes)?;
        let mut sheet: serde_json::Value =
            serde_json::from_str(&String::from_utf8(decompressed_bytes)?)?;

        sheet = Self::update_sheet_syntax(sheet).unwrap();

        self.import_sheet(serde_json::from_value(sheet)?);

        Ok(())
    }

    fn update_sheet_syntax(mut sheet: serde_json::Value) -> Option<serde_json::Value> {
        if sheet["version"].as_str()?.starts_with("0.3.") {
            for stroke in sheet["strokes_state"]["strokes"].as_array_mut()? {
                let val = stroke.get_mut("value")?;
                if val.is_null() {
                    continue;
                }

                if let Some(bstroke) = val.as_object_mut()?.remove("markerstroke") {
                    val.as_object_mut()?.insert(String::from("brushstroke"), bstroke);

                    let brushstroke = val["brushstroke"].as_object_mut()?;
                    let options = brushstroke.remove("marker")?;
                    brushstroke.insert(
                        "style".to_string(),
                        json!({
                            "marker": {
                                "options": options
                            }
                        }),
                    );
                } else if let Some(style) = val.pointer_mut("/brushstroke/style") {
                    let style = style.as_object_mut()?;

                    if let Some(solid_settings) = style.remove("smooth") {
                        style.insert("solid".to_string(), solid_settings);
                    }
                }

                if val.get("shapestroke").is_some() {
                    let drawstyle = val["shapestroke"]["drawstyle"].as_object_mut()?;

                    if let Some(smooth) = drawstyle.remove("Smooth") {
                        drawstyle.insert(String::from("smooth"), smooth);
                    } else if let Some(smooth) = drawstyle.remove("Rough") {
                        drawstyle.insert(String::from("rough"), smooth);
                    }
                }
            }
        }

        Some(sheet)
    }

    pub fn open_from_xopp_bytes(&mut self, bytes: glib::Bytes) -> Result<(), anyhow::Error> {
        // We set the sheet dpi to the hardcoded xournal++ dpi, so no need to convert values or coordinates anywhere
        self.format.dpi = xoppformat::XoppFile::DPI;

        let xopp_file = xoppformat::XoppFile::load_from_bytes(&bytes)?;

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

        let mut sheet = Self::default();
        let mut format = Format::default();
        let mut background = Background::default();

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
                    match StrokeStyle::from_xoppstroke(new_xoppstroke, offset) {
                        Ok(new_stroke) => {
                            sheet.strokes_state.insert_stroke(new_stroke);
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
                    match StrokeStyle::from_xoppimage(new_xoppimage, offset) {
                        Ok(new_image) => {
                            sheet.strokes_state.insert_stroke(new_image);
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

        self.import_sheet(sheet);

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

        svg_data = compose::wrap_svg_root(svg_data.as_str(), Some(bounds), Some(bounds), true);

        Ok(svg_data)
    }

    pub fn export_sheet_as_xopp_bytes(
        &self,
        filename: &str,
        renderer: Arc<RwLock<Renderer>>,
    ) -> Result<Vec<u8>, anyhow::Error> {
        let current_dpi = self.format.dpi;

        // Only one background for all pages
        let background = xoppformat::XoppBackground {
            name: None,
            bg_type: xoppformat::XoppBackgroundType::Solid {
                color: self.background.color.into(),
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

                        stroke.into_xopp(current_dpi, Arc::clone(&renderer))
                    })
                    .collect::<Vec<xoppformat::XoppStrokeStyle>>();

                // Extract the strokes
                let xopp_strokes = xopp_strokestyles
                    .iter()
                    .filter_map(|stroke| {
                        if let xoppformat::XoppStrokeStyle::XoppStroke(xoppstroke) = stroke {
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
                        if let xoppformat::XoppStrokeStyle::XoppText(xopptext) = stroke {
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
                        if let xoppformat::XoppStrokeStyle::XoppImage(xoppstroke) = stroke {
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
                Some((page_bounds, self.gen_svgs_for_viewport(page_bounds).ok()?))
            })
            .collect::<Vec<(AABB, Vec<render::Svg>)>>();

        let sheet_bounds = self.bounds();
        let format_size = na::vector![f64::from(self.format.width), f64::from(self.format.height)];

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
                        render::draw_svgs_to_cairo_context(
                            1.0,
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
}
