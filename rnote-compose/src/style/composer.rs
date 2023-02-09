use p2d::bounding_volume::Aabb;

/// Specifies that the implementing type can be composed and drawn with a style
pub trait Composer<O>
where
    O: std::fmt::Debug + Clone,
{
    /// the bounds of the composed shape.
    fn composed_bounds(&self, options: &O) -> Aabb;

    /// composes and draws the shape onto the context, applying the style options to it.
    fn draw_composed(&self, cx: &mut impl piet::RenderContext, options: &O);
}
