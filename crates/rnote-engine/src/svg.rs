// Imports
use crate::image::{Image, ImageMemoryFormat, convert_image_bgra_to_rgba};
use anyhow::Context;
use once_cell::sync::Lazy;
use p2d::bounding_volume::{Aabb, BoundingVolume};
use piet::RenderContext;
use rnote_compose::ext::AabbExt;
use rnote_compose::shapes::Rectangle;
use std::sync::Arc;
use svg::Node;

/// Usvg font database
pub static USVG_FONTDB: Lazy<Arc<usvg::fontdb::Database>> = Lazy::new(|| {
    let mut db = usvg::fontdb::Database::new();
    db.load_system_fonts();
    Arc::new(db)
});

/// A Svg image.
#[derive(Debug, Clone)]
pub struct Svg {
    /// Svg data String.
    pub svg_data: String,
    /// Bounds of the Svg.
    pub bounds: Aabb,
}

impl Svg {
    pub const MIME_TYPE: &'static str = "image/svg+xml";

    pub fn merge<T>(&mut self, other: T)
    where
        T: IntoIterator<Item = Self>,
    {
        for svg in other {
            self.svg_data += format!("\n{}", svg.svg_data).as_str();
            self.bounds.merge(&svg.bounds);
        }
    }

    pub fn wrap_svg_root(
        &mut self,
        bounds: Option<Aabb>,
        viewbox: Option<Aabb>,
        preserve_aspectratio: bool,
    ) {
        self.svg_data = rnote_compose::utils::wrap_svg_root(
            self.svg_data.as_str(),
            bounds,
            viewbox,
            preserve_aspectratio,
        );
        if let Some(bounds) = bounds {
            self.bounds = bounds
        }
    }

    pub fn add_xml_header(&mut self) {
        self.svg_data = rnote_compose::utils::add_xml_header(&self.svg_data);
    }

    pub fn remove_xml_header(&mut self) {
        self.svg_data = rnote_compose::utils::remove_xml_header(&self.svg_data);
    }

    /// Simplify the Svg by passing it through [usvg].
    ///
    /// Also moves the bounds to mins: [0., 0.], maxs: extents
    pub fn simplify(&mut self) -> anyhow::Result<()> {
        const COORDINATES_PREC: u8 = 3;
        const TRANSFORMS_PREC: u8 = 8;

        let xml_options = usvg::WriteOptions {
            id_prefix: Some(rnote_compose::utils::svg_random_id_prefix()),
            preserve_text: true,
            coordinates_precision: COORDINATES_PREC,
            transforms_precision: TRANSFORMS_PREC,
            use_single_quote: false,
            indent: xmlwriter::Indent::None,
            attributes_indent: xmlwriter::Indent::None,
        };
        let bounds_simplified = Aabb::new(na::point![0.0, 0.0], self.bounds.extents().into());
        let svg_data_wrapped = rnote_compose::utils::wrap_svg_root(
            &rnote_compose::utils::remove_xml_header(&self.svg_data),
            Some(bounds_simplified),
            Some(self.bounds),
            false,
        );

        let usvg_tree = usvg::Tree::from_str(
            &svg_data_wrapped,
            &usvg::Options {
                fontdb: Arc::clone(&USVG_FONTDB),
                ..Default::default()
            },
        )?;

        self.svg_data = usvg_tree.to_string(&xml_options);
        self.bounds = bounds_simplified;

        Ok(())
    }

    /// Generate an Svg through cairo's SvgSurface.
    pub fn gen_with_cairo<F>(draw_func: F, mut bounds: Aabb) -> anyhow::Result<Self>
    where
        F: FnOnce(&cairo::Context) -> anyhow::Result<()>,
    {
        bounds.ensure_positive();
        bounds.assert_valid()?;

        let width = bounds.extents()[0];
        let height = bounds.extents()[1];
        let mut svg_surface =
            cairo::SvgSurface::for_stream(width, height, Vec::new()).map_err(|e| {
                anyhow::anyhow!(
                    "Creating svg surface with dimensions ({width}, {height}) failed, Err: {e:?}"
                )
            })?;
        svg_surface.set_document_unit(cairo::SvgUnit::Px);

        {
            let cairo_cx = cairo::Context::new(&svg_surface)?;
            // cairo only draws elements with positive coordinates, so we need to translate the content here
            cairo_cx.translate(-bounds.mins[0], -bounds.mins[1]);
            // apply the draw function
            draw_func(&cairo_cx)?;
        }

        let content = String::from_utf8(
            *svg_surface
                .finish_output_stream()
                .map_err(|e| {
                    anyhow::anyhow!("Finishing Svg surface output stream failed, Err: {e:?}")
                })?
                .downcast::<Vec<u8>>()
                .map_err(|e| {
                    anyhow::anyhow!("Downcasting Svg surface content failed, Err: {e:?}")
                })?,
        )?;
        let svg_data = rnote_compose::utils::remove_xml_header(&content);
        let mut group = svg::node::element::Group::new().add(svg::node::Blob::new(svg_data));
        // translate the content back to it's original position
        group.assign(
            "transform",
            format!("translate({} {})", bounds.mins[0], bounds.mins[1]),
        );

        Ok(Self {
            svg_data: rnote_compose::utils::svg_node_to_string(&group)?,
            bounds,
        })
    }

    /// Generate an Svg with piet, using the `piet_cairo` backend and cairo's SvgSurface.
    ///
    /// This might be preferable to the `piet_svg` backend, because especially text alignment and sizes can be different
    /// with it.
    pub fn gen_with_piet_cairo_backend<F>(draw_func: F, bounds: Aabb) -> anyhow::Result<Self>
    where
        F: FnOnce(&mut piet_cairo::CairoRenderContext) -> anyhow::Result<()>,
    {
        let cairo_draw_fn = |cairo_cx: &cairo::Context| {
            let mut piet_cx = piet_cairo::CairoRenderContext::new(cairo_cx);
            // Apply the draw function
            draw_func(&mut piet_cx)?;
            piet_cx
                .finish()
                .map_err(|e| anyhow::anyhow!("finishing piet context failed, Err: {e:?}"))
        };

        Self::gen_with_cairo(cairo_draw_fn, bounds)
    }

    pub fn draw_to_cairo(&self, cx: &cairo::Context) -> anyhow::Result<()> {
        let svg_data = rnote_compose::utils::wrap_svg_root(
            self.svg_data.as_str(),
            Some(self.bounds),
            Some(self.bounds),
            false,
        );
        let stream = gio::MemoryInputStream::from_bytes(&glib::Bytes::from(svg_data.as_bytes()));
        let handle = rsvg::Loader::new()
            .with_unlimited_size(true)
            .read_stream(&stream, None::<&gio::File>, None::<&gio::Cancellable>)
            .context("reading stream to rsvg loader failed.")?;
        let renderer = rsvg::CairoRenderer::new(&handle);
        renderer
            .render_document(
                cx,
                &cairo::Rectangle::new(
                    self.bounds.mins[0],
                    self.bounds.mins[1],
                    self.bounds.extents()[0],
                    self.bounds.extents()[1],
                ),
            )
            .map_err(|e| anyhow::anyhow!("rendering rsvg document failed, Err: {e:?}"))?;
        Ok(())
    }

    /// Generate an image from an Svg.
    ///
    /// Using rsvg for rendering.
    pub fn gen_image(&self, image_scale: f64) -> Result<Image, anyhow::Error> {
        let mut bounds = self.bounds;
        bounds.ensure_positive();
        bounds.assert_valid()?;

        let svg_data = rnote_compose::utils::wrap_svg_root(
            self.svg_data.as_str(),
            Some(bounds),
            Some(bounds),
            false,
        );
        let width_scaled = ((bounds.extents()[0]) * image_scale).round() as u32;
        let height_scaled = ((bounds.extents()[1]) * image_scale).round() as u32;

        let mut surface = cairo::ImageSurface::create(
                cairo::Format::ARgb32,
                width_scaled as i32,
                height_scaled as i32,
            )
            .map_err(|e| {
                anyhow::anyhow!(
                    "creating ImageSurface with dimensions ({width_scaled}, {height_scaled}) failed, Err: {e:?}"
                )
            })?;

        // Context in new scope, else accessing the surface data fails with a borrow error
        {
            let cx =
                cairo::Context::new(&surface).context("creating new cairo::Context failed.")?;
            cx.scale(image_scale, image_scale);
            cx.translate(-bounds.mins[0], -bounds.mins[1]);

            let stream =
                gio::MemoryInputStream::from_bytes(&glib::Bytes::from(svg_data.as_bytes()));

            let handle = rsvg::Loader::new()
                .with_unlimited_size(true)
                .read_stream::<gio::MemoryInputStream, gio::File, gio::Cancellable>(
                    &stream, None, None,
                )
                .context("read stream to rsvg loader failed.")?;

            let renderer = rsvg::CairoRenderer::new(&handle);
            renderer
                .render_document(
                    &cx,
                    &cairo::Rectangle::new(
                        bounds.mins[0],
                        bounds.mins[1],
                        bounds.extents()[0],
                        bounds.extents()[1],
                    ),
                )
                .map_err(|e| anyhow::anyhow!("rendering rsvg document failed, Err: {e:?}"))?;
        }

        let data = surface
            .data()
            .map_err(|e| anyhow::anyhow!("accessing imagesurface data failed, Err: {e:?}"))?
            .to_vec();

        Ok(Image {
            data: glib::Bytes::from_owned(convert_image_bgra_to_rgba(
                width_scaled,
                height_scaled,
                data,
            )),
            rect: Rectangle::from_p2d_aabb(bounds),
            pixel_width: width_scaled,
            pixel_height: height_scaled,
            // cairo renders to bgra8-premultiplied, but we convert it to rgba8-premultiplied
            memory_format: ImageMemoryFormat::R8g8b8a8Premultiplied,
        })
    }
}
