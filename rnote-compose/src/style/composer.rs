use p2d::bounding_volume::AABB;
use piet::RenderContext;

// Specifies that the implementating type can be composed and drawn with a style
pub trait Composer<O>
where
    O: std::fmt::Debug + Clone,
{
    fn composed_bounds(&self, options: &O) -> AABB;

    /// composes and draws the shape onto the context, applying the style options to it.
    fn draw_composed(&self, cx: &mut impl piet::RenderContext, options: &O);
}

pub fn piet_svg_cx_to_svg(mut cx: piet_svg::RenderContext) -> Result<String, anyhow::Error> {
    cx.finish()
        .map_err(|e| anyhow::anyhow!("cx.finish() failed in svg_cx_to_svg() with Err {}", e))?;

    let mut data: Vec<u8> = vec![];
    cx.write(&mut data)?;

    let svg_data = crate::utils::strip_svg_root(String::from_utf8(data)?.as_str());

    Ok(svg_data)
}
