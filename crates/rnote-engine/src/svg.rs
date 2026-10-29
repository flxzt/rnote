// Imports
use crate::{Drawable, Image};
use anyhow::Context;
use once_cell::sync::Lazy;
use p2d::bounding_volume::{Aabb, BoundingVolume};
use p2d::math::Vector2;
use rnote_compose::ext::AabbExt;
use std::sync::Arc;
use svg::Node;
use vello_cpu::{RenderContext, peniko};

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

impl Drawable for Svg {
    fn pos(&self) -> Vector2 {
        self.bounds.mins
    }

    /// Draws itself on a [RenderContext].
    ///
    /// Using rsvg for rendering.
    fn draw(&self, cx: &mut RenderContext, image_scale: f64) -> anyhow::Result<()> {
        let mut bounds = self.bounds;
        bounds.ensure_positive();
        bounds.assert_valid()?;

        let svg_data = rnote_compose::utils::wrap_svg_root(
            self.svg_data.as_str(),
            Some(bounds),
            Some(bounds),
            false,
        );
        let surface_width = ((bounds.extents()[0]) * image_scale).round() as u32;
        let surface_height = ((bounds.extents()[1]) * image_scale).round() as u32;
        let surface = cairo::ImageSurface::create(
                cairo::Format::ARgb32,
                surface_width as i32,
                surface_height as i32,
            )
            .map_err(|e| {
                anyhow::anyhow!(
                    "creating ImageSurface with dimensions ({surface_width}, {surface_height}) failed, Err: {e:?}"
                )
            })?;

        // Context in new scope, else accessing the surface data fails with a borrow error
        {
            let cx =
                cairo::Context::new(&surface).context("creating new cairo::Context failed.")?;
            let stream =
                gio::MemoryInputStream::from_bytes(&glib::Bytes::from(svg_data.as_bytes()));
            let handle = rsvg::Loader::new()
                .with_unlimited_size(true)
                .read_stream::<gio::MemoryInputStream, gio::File, gio::Cancellable>(
                    &stream, None, None,
                )
                .context("read stream to rsvg loader failed.")?;
            let renderer = rsvg::CairoRenderer::new(&handle);

            cx.scale(image_scale, image_scale);
            cx.translate(-bounds.mins[0], -bounds.mins[1]);
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
                .context("rendering rsvg document failed")?;
        }
        let pixmap = Image::try_from_cairo_surface_argb(surface, bounds)?.to_vello_pixmap();

        cx.set_paint(vello_cpu::Image {
            image: vello_cpu::ImageSource::Pixmap(Arc::new(pixmap)),
            sampler: peniko::ImageSampler::default(),
        });
        cx.fill_rect(&bounds.to_kurbo_rect());

        Ok(())
    }

    fn draw_to_cairo(
        &self,
        cx: &cairo::Context,
        _width: u16,
        _height: u16,
        _image_scale: f64,
    ) -> anyhow::Result<()> {
        let svg_data = self.to_document();
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
            .context("rendering rsvg document failed")?;
        Ok(())
    }
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

    /// Returns [Svg::svg_data] as a complete document, including xml headers and a svg
    /// root element that uses the current bounds for both the viewport and the viewbox.
    pub fn to_document(&self) -> String {
        rnote_compose::utils::add_xml_header(&rnote_compose::utils::wrap_svg_root(
            self.svg_data.as_str(),
            Some(self.bounds),
            Some(self.bounds),
            false,
        ))
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
        let bounds_simplified = Aabb::new(Vector2::ZERO, self.bounds.extents());
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

        // usvg writes a complete document, but Svg.svg_data is a fragment
        self.svg_data = rnote_compose::utils::remove_svg_root(&usvg_tree.to_string(&xml_options));
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
        let mut svg_surface = cairo::SvgSurface::for_stream(width, height, Vec::new())
            .context("Creating svg surface with dimensions ({width}, {height}) failed")?;
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

    /// Export as bitmap image.
    pub fn export_as_image(&self, image_scale: f64) -> Result<Image, anyhow::Error> {
        Image::gen_with_vello(|cx| self.draw(cx, 1.0), self.bounds, image_scale)
    }
}
