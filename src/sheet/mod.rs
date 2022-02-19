pub mod background;
pub mod format;

use std::sync::{Arc, RwLock};

use crate::compose::color::Color;
use crate::compose::geometry::AABBHelpers;
use crate::compose::shapes;
use crate::compose::smooth::SmoothOptions;
use crate::compose::transformable::{Transform, Transformable};
use crate::pens::brush::{Brush, BrushStyle};
use crate::render::Renderer;
use crate::strokes::bitmapimage::{self, BitmapImage};
use crate::strokes::brushstroke::BrushStroke;
use crate::strokes::strokestyle::{Element, InputData, StrokeStyle};
use crate::strokesstate::StrokesState;
use crate::{config, render, utils};
use notetakingfileformats::xoppformat;
use notetakingfileformats::FileFormatLoader;
use notetakingfileformats::FileFormatSaver;

use self::{background::Background, format::Format};

use gtk4::{glib, Snapshot};
use p2d::bounding_volume::{BoundingVolume, AABB};
use serde::{Deserialize, Serialize};

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
            version: String::from(config::APP_VERSION),
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
        let keys = self.strokes_state.keys_sorted_chrono();
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

        let keys = self.strokes_state.keys_as_rendered();
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
        let sheet: Sheet = serde_json::from_str(&String::from_utf8(decompressed_bytes)?)?;

        self.import_sheet(sheet);

        Ok(())
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
        let x_offset = 0.0;
        let mut y_offset = 0.0;

        for (_page_i, page) in xopp_file.xopp_root.pages.into_iter().enumerate() {
            for layers in page.layers.into_iter() {
                // import strokes
                for stroke in layers.strokes.into_iter() {
                    let mut width_iter = stroke.width.iter();

                    let mut smooth_options = SmoothOptions::default();
                    smooth_options.stroke_color = Some(Color::from(stroke.color));

                    // The first element is the absolute width, every following is the relative width (between 0.0 and 1.0)
                    if let Some(&width) = width_iter.next() {
                        smooth_options.width = width;
                    }

                    let brush = Brush {
                        style: BrushStyle::Solid,
                        smooth_options,
                        ..Brush::default()
                    };

                    let elements = stroke.coords.into_iter().map(|mut coords| {
                        coords[0] += x_offset;
                        coords[1] += y_offset;
                        // Defaulting to PRESSURE_DEFAULT if width iterator is shorter than the coords vec
                        let pressure = width_iter
                            .next()
                            .map(|&width| width / smooth_options.width)
                            .unwrap_or(InputData::PRESSURE_DEFAULT);

                        Element::new(InputData::new(coords, pressure))
                    });

                    if let Some(new_stroke) = BrushStroke::new_w_elements(elements, &brush) {
                        sheet
                            .strokes_state
                            .insert_stroke(StrokeStyle::BrushStroke(new_stroke));
                    }
                }

                // import images
                for image in layers.images.into_iter() {
                    let bounds = AABB::new(
                        na::point![image.left, image.top],
                        na::point![image.right, image.bottom],
                    )
                    .translate(na::vector![x_offset, y_offset]);

                    let intrinsic_size =
                        bitmapimage::extract_dimensions(&base64::decode(&image.data)?)?;

                    let rectangle = shapes::Rectangle {
                        cuboid: p2d::shape::Cuboid::new(bounds.half_extents()),
                        transform: Transform::new_w_isometry(na::Isometry2::new(
                            bounds.center().coords,
                            0.0,
                        )),
                    };

                    let mut bitmapimage = BitmapImage {
                        data_base64: image.data,
                        // Xopp images are always Png
                        format: bitmapimage::BitmapImageFormat::Png,
                        intrinsic_size,
                        rectangle,
                        ..BitmapImage::default()
                    };
                    bitmapimage.update_geometry();

                    sheet
                        .strokes_state
                        .insert_stroke(StrokeStyle::BitmapImage(bitmapimage));
                }
            }

            // Only add to y offset, results in vertical pages
            y_offset += page.height;
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

                        stroke.to_xopp(current_dpi, Arc::clone(&renderer))
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

    /// Generates all containing svgs for the sheet without root or xml header.
    pub fn gen_svgs(&self) -> Result<Vec<render::Svg>, anyhow::Error> {
        let sheet_bounds = self.bounds();
        let mut svgs = vec![];

        svgs.push(self.background.gen_svg(sheet_bounds.loosened(1.0))?);

        svgs.append(&mut self.strokes_state.gen_svgs_all_strokes());

        Ok(svgs)
    }

    /// Generates all containing svgs for the sheet without root or xml header.
    pub fn gen_svgs_for_viewport(&self, viewport: AABB) -> Result<Vec<render::Svg>, anyhow::Error> {
        let sheet_bounds = self.bounds();
        let mut svgs = vec![];

        // Background bounds are still sheet bounds, for alignment
        svgs.push(self.background.gen_svg(sheet_bounds.loosened(1.0))?);

        svgs.append(&mut self.strokes_state.gen_svgs_for_bounds(viewport));

        Ok(svgs)
    }
}
