// Imports
use p2d::bounding_volume::Aabb;

/// Trait for types can be composed and drawn with a style.
pub trait Composer<O>
where
    O: std::fmt::Debug + Clone,
{
    /// Bounds of the composed shape.
    fn composed_bounds(&self, options: &O) -> Aabb;

    /// Draw with vello_cpu.
    fn draw_composed_vello(&self, cx: &mut vello_cpu::RenderContext, options: &O);
}
