// Imports
use vello_cpu::RenderContext;

/// Trait for types that can draw themselves on a [RenderContext].
pub trait Drawable {
    /// Draw itself.
    ///
    /// The implementers are expected to save/restore the drawing context state themselves.
    /// `image_scale` is the scale-factor of generated images within the type.
    /// The content should not be zoomed by it!
    fn draw(&self, cx: &mut RenderContext, image_scale: f64) -> anyhow::Result<()>;

    /// Draw itself to a [cairo::Context].
    fn draw_to_cairo(
        &self,
        cx: &cairo::Context,
        width: u16,
        height: u16,
        image_scale: f64,
    ) -> anyhow::Result<()> {
        let mut vello_target = vello_cpu::Pixmap::new(width, height);
        let mut vello_resources = vello_cpu::Resources::new();
        let mut vello_cx = RenderContext::new(width, height);
        self.draw(&mut vello_cx, image_scale)?;
        vello_cx.flush();
        vello_cx.render(&mut vello_target, &mut vello_resources);

        let mut cairo_surface = cairo::ImageSurface::create_for_data(
            vello_target.data_as_u8_slice().to_vec(),
            cairo::Format::ARgb32,
            width as i32,
            height as i32,
            4 * width as i32,
        )?;
        cx.set_source_surface(&mut cairo_surface, 0., 0.)?;
        cx.paint()?;
        Ok(())
    }
}
