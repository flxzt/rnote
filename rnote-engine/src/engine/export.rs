use anyhow::Context;
use futures::channel::oneshot;
use p2d::bounding_volume::AABB;
use piet::RenderContext;
use std::sync::Arc;

use rnote_compose::helpers::Vector2Helpers;
use rnote_compose::transform::TransformBehaviour;
use rnote_fileformats::rnoteformat::RnotefileMaj0Min5;
use rnote_fileformats::{xoppformat, FileFormatSaver};

use crate::store::StrokeKey;
use crate::{render, DrawBehaviour};

use super::RnoteEngine;

impl RnoteEngine {
    /// Saves the current state as a .rnote file.
    pub fn save_as_rnote_bytes(
        &self,
        file_name: String,
    ) -> anyhow::Result<oneshot::Receiver<anyhow::Result<Vec<u8>>>> {
        let (oneshot_sender, oneshot_receiver) = oneshot::channel::<anyhow::Result<Vec<u8>>>();

        let mut store_snapshot = self.store.take_store_snapshot();
        Arc::make_mut(&mut store_snapshot).process_before_saving();

        // the doc is currently not thread safe, so we have to serialize it in the same thread that holds the engine
        let doc = serde_json::to_value(&self.document)?;

        rayon::spawn(move || {
            let result = || -> anyhow::Result<Vec<u8>> {
                let rnote_file = RnotefileMaj0Min5 {
                    document: doc,
                    store_snapshot: serde_json::to_value(&*store_snapshot)?,
                };

                rnote_file.save_as_bytes(&file_name)
            };

            if let Err(_data) = oneshot_sender.send(result()) {
                log::error!("sending result to receiver in save_as_rnote_bytes() failed. Receiver already dropped.");
            }
        });

        Ok(oneshot_receiver)
    }

    /// Exports the entire engine state as JSON string
    /// Only use for debugging
    pub fn export_state_as_json(&self) -> anyhow::Result<String> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    /// generates the doc svg.
    /// The coordinates are translated so that the svg has origin 0.0, 0.0
    /// without root or xml header.
    pub fn gen_doc_svg(&self, with_background: bool) -> Result<render::Svg, anyhow::Error> {
        let doc_bounds = self.document.bounds();

        let strokes = self.store.stroke_keys_as_rendered();

        let mut doc_svg = if with_background {
            let mut background_svg = self.document.background.gen_svg(doc_bounds)?;

            background_svg.wrap_svg_root(
                Some(AABB::new(
                    na::point![0.0, 0.0],
                    na::Point2::from(doc_bounds.extents()),
                )),
                Some(doc_bounds),
                true,
            );

            background_svg
        } else {
            // we can have invalid bounds here, because we know we merge them with the strokes svg
            render::Svg {
                svg_data: String::new(),
                bounds: AABB::new(na::point![0.0, 0.0], na::Point2::from(doc_bounds.extents())),
            }
        };

        doc_svg.merge([render::Svg::gen_with_piet_cairo_backend(
            |piet_cx| {
                piet_cx.transform(kurbo::Affine::translate(
                    doc_bounds.mins.coords.to_kurbo_vec(),
                ));

                self.store.draw_stroke_keys_to_piet(
                    &strokes,
                    piet_cx,
                    RnoteEngine::EXPORT_IMAGE_SCALE,
                )
            },
            AABB::new(na::point![0.0, 0.0], na::Point2::from(doc_bounds.extents())),
        )?]);

        Ok(doc_svg)
    }

    /// generates the doc svg for the given viewport.
    /// The coordinates are translated so that the svg has origin 0.0, 0.0
    /// without root or xml header.
    pub fn gen_doc_svg_with_viewport(
        &self,
        viewport: AABB,
        with_background: bool,
    ) -> Result<render::Svg, anyhow::Error> {
        // Background bounds are still doc bounds, for correct alignment of the background pattern
        let mut doc_svg = if with_background {
            let mut background_svg = self.document.background.gen_svg(viewport)?;

            background_svg.wrap_svg_root(
                Some(AABB::new(
                    na::point![0.0, 0.0],
                    na::Point2::from(viewport.extents()),
                )),
                Some(viewport),
                true,
            );

            background_svg
        } else {
            // we can have invalid bounds here, because we know we merge them with the other svg
            render::Svg {
                svg_data: String::new(),
                bounds: AABB::new(na::point![0.0, 0.0], na::Point2::from(viewport.extents())),
            }
        };

        let strokes_in_viewport = self
            .store
            .stroke_keys_as_rendered_intersecting_bounds(viewport);

        doc_svg.merge([render::Svg::gen_with_piet_cairo_backend(
            |piet_cx| {
                piet_cx.transform(kurbo::Affine::translate(
                    -viewport.mins.coords.to_kurbo_vec(),
                ));

                self.store.draw_stroke_keys_to_piet(
                    &strokes_in_viewport,
                    piet_cx,
                    RnoteEngine::EXPORT_IMAGE_SCALE,
                )
            },
            AABB::new(na::point![0.0, 0.0], na::Point2::from(viewport.extents())),
        )?]);

        Ok(doc_svg)
    }

    /// generates the selection svg.
    /// The coordinates are translated so that the svg has origin 0.0, 0.0
    /// without root or xml header.
    pub fn gen_selection_svg(
        &self,
        with_background: bool,
    ) -> Result<Option<render::Svg>, anyhow::Error> {
        let selection_keys = self.store.selection_keys_as_rendered();

        if selection_keys.is_empty() {
            return Ok(None);
        }

        let selection_bounds =
            if let Some(selection_bounds) = self.store.bounds_for_strokes(&selection_keys) {
                selection_bounds
            } else {
                return Ok(None);
            };

        let mut selection_svg = if with_background {
            let mut background_svg = self.document.background.gen_svg(selection_bounds)?;

            background_svg.wrap_svg_root(
                Some(AABB::new(
                    na::point![0.0, 0.0],
                    na::Point2::from(selection_bounds.extents()),
                )),
                Some(selection_bounds),
                true,
            );

            background_svg
        } else {
            render::Svg {
                svg_data: String::new(),
                bounds: AABB::new(
                    na::point![0.0, 0.0],
                    na::Point2::from(selection_bounds.extents()),
                ),
            }
        };

        selection_svg.merge([render::Svg::gen_with_piet_cairo_backend(
            |piet_cx| {
                piet_cx.transform(kurbo::Affine::translate(
                    -selection_bounds.mins.coords.to_kurbo_vec(),
                ));

                self.store.draw_stroke_keys_to_piet(
                    &selection_keys,
                    piet_cx,
                    RnoteEngine::EXPORT_IMAGE_SCALE,
                )
            },
            AABB::new(
                na::point![0.0, 0.0],
                na::Point2::from(selection_bounds.extents()),
            ),
        )?]);

        Ok(Some(selection_svg))
    }

    /// Exports the doc with the strokes as a SVG string.
    pub fn export_doc_as_svg_string(&self, with_background: bool) -> Result<String, anyhow::Error> {
        let doc_svg = self.gen_doc_svg(with_background)?;

        Ok(rnote_compose::utils::add_xml_header(
            rnote_compose::utils::wrap_svg_root(
                doc_svg.svg_data.as_str(),
                Some(doc_svg.bounds),
                Some(doc_svg.bounds),
                true,
            )
            .as_str(),
        ))
    }

    /// Exports the current selection as a SVG string
    pub fn export_selection_as_svg_string(
        &self,
        with_background: bool,
    ) -> anyhow::Result<Option<String>> {
        let selection_svg = match self.gen_selection_svg(with_background)? {
            Some(selection_svg) => selection_svg,
            None => return Ok(None),
        };

        Ok(Some(rnote_compose::utils::add_xml_header(
            rnote_compose::utils::wrap_svg_root(
                selection_svg.svg_data.as_str(),
                Some(selection_svg.bounds),
                Some(selection_svg.bounds),
                true,
            )
            .as_str(),
        )))
    }

    /// Exporting doc as encoded image bytes (Png / Jpg, etc.)
    pub fn export_doc_as_bitmapimage_bytes(
        &self,
        format: image::ImageOutputFormat,
        with_background: bool,
    ) -> Result<Vec<u8>, anyhow::Error> {
        let image_scale = 1.0;

        let doc_svg = self.gen_doc_svg(with_background)?;
        let doc_svg_bounds = doc_svg.bounds;

        render::Image::gen_image_from_svg(doc_svg, doc_svg_bounds, image_scale)?
            .into_encoded_bytes(format)
    }

    /// Exporting selection as encoded image bytes (Png / Jpg, etc.)
    pub fn export_selection_as_bitmapimage_bytes(
        &self,
        format: image::ImageOutputFormat,
        with_background: bool,
    ) -> Result<Option<Vec<u8>>, anyhow::Error> {
        let image_scale = 1.0;

        let selection_svg = match self.gen_selection_svg(with_background)? {
            Some(selection_svg) => selection_svg,
            None => return Ok(None),
        };
        let selection_svg_bounds = selection_svg.bounds;

        Ok(Some(
            render::Image::gen_image_from_svg(selection_svg, selection_svg_bounds, image_scale)?
                .into_encoded_bytes(format)?,
        ))
    }

    /// Exports the doc with the strokes as a Xournal++ .xopp file. Excluding the current selection.
    pub fn export_doc_as_xopp_bytes(&self, filename: &str) -> Result<Vec<u8>, anyhow::Error> {
        let current_dpi = self.document.format.dpi;

        // Only one background for all pages
        let background = xoppformat::XoppBackground {
            name: None,
            bg_type: xoppformat::XoppBackgroundType::Solid {
                color: self.document.background.color.into(),
                style: xoppformat::XoppBackgroundSolidStyle::Plain,
            },
        };

        // xopp spec needs at least one page in vec, but its fine because pages_bounds_w_content() always produces at least one
        let pages = self
            .pages_bounds_w_content()
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

    /// Exports the doc with the strokes as a PDF file.
    pub fn export_doc_as_pdf_bytes(
        &self,
        title: String,
        with_background: bool,
    ) -> oneshot::Receiver<anyhow::Result<Vec<u8>>> {
        let (oneshot_sender, oneshot_receiver) = oneshot::channel::<anyhow::Result<Vec<u8>>>();
        let doc_bounds = self.document.bounds();
        let format_size = na::vector![self.document.format.width, self.document.format.height];
        let store_snapshot = self.store.take_store_snapshot();

        let background_svg = if with_background {
            self.document
                .background
                .gen_svg(doc_bounds)
                .map_err(|e| {
                    log::error!(
                        "background.gen_svg() failed in export_doc_as_pdf_bytes() with Err {}",
                        e
                    )
                })
                .ok()
        } else {
            None
        };

        let pages_strokes = self
            .pages_bounds_w_content()
            .into_iter()
            .map(|page_bounds| {
                let strokes_in_viewport = self
                    .store
                    .stroke_keys_as_rendered_intersecting_bounds(page_bounds);

                (page_bounds, strokes_in_viewport)
            })
            .collect::<Vec<(AABB, Vec<StrokeKey>)>>();

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

                    for (i, (page_bounds, page_strokes)) in pages_strokes.into_iter().enumerate() {
                        // We can't render the background svg with piet, so we have to do it with cairo.
                        cairo_cx.save()?;
                        cairo_cx.translate(-page_bounds.mins[0], -page_bounds.mins[1]);

                        if let Some(background_svg) = background_svg.clone() {
                            render::Svg::draw_svgs_to_cairo_context(&[background_svg], &cairo_cx)?;
                        }
                        cairo_cx.restore()?;

                        // Draw the strokes with piet
                        let mut piet_cx = piet_cairo::CairoRenderContext::new(&cairo_cx);
                        piet_cx.save().map_err(|e| anyhow::anyhow!("{}", e))?;
                        piet_cx.transform(kurbo::Affine::translate(
                            -page_bounds.mins.coords.to_kurbo_vec(),
                        ));

                        for stroke in page_strokes.into_iter() {
                            if let Some(stroke) = store_snapshot.stroke_components.get(stroke) {
                                stroke.draw(&mut piet_cx, RnoteEngine::EXPORT_IMAGE_SCALE)?;
                            }
                        }

                        cairo_cx.show_page().map_err(|e| {
                            anyhow::anyhow!(
                                "show_page() failed when exporting page {} as pdf, Err {}",
                                i,
                                e
                            )
                        })?;

                        piet_cx.restore().map_err(|e| anyhow::anyhow!("{}", e))?;
                    }
                }
                let data = *surface
                    .finish_output_stream()
                    .map_err(|e| {
                        anyhow::anyhow!(
                            "finish_outputstream() failed in export_doc_as_pdf_bytes with Err {:?}",
                            e
                        )
                    })?
                    .downcast::<Vec<u8>>()
                    .map_err(|e| {
                        anyhow::anyhow!(
                            "downcast() finished output stream failed in export_doc_as_pdf_bytes with Err {:?}",
                            e
                        )
                    })?;

                Ok(data)
            };

            if let Err(_data) = oneshot_sender.send(result()) {
                log::error!("sending result to receiver in export_doc_as_pdf_bytes() failed. Receiver already dropped.");
            }
        });

        oneshot_receiver
    }
}
