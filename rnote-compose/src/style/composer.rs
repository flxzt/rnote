use p2d::bounding_volume::AABB;

/// Specifies that the implementating type can be composed and drawn with style options
pub trait Composer<O>
where
    O: std::fmt::Debug + Clone,
{
    /// the bounds of the composed shape. Styles might need to increase the original bounds to avoid clipping
    fn composed_bounds(&self, options: &O) -> AABB;

    /// composes and draws the shape onto the context, applying the style options to it.
    fn draw_composed(&self, cx: &mut impl piet::RenderContext, options: &O);
}
