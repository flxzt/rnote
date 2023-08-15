// Imports
use crate::transform::Transformable;
use p2d::bounding_volume::Aabb;

/// Types that behave as a shape.
pub trait Shapeable: Transformable {
    /// The bounds of the shape.
    fn bounds(&self) -> Aabb;
    /// The hitboxes of the shape.
    fn hitboxes(&self) -> Vec<Aabb>;
    /// The absolute position of the shapes upper-left corner.
    fn pos(&self) -> na::Vector2<f64> {
        self.bounds().mins.coords
    }
    /// Set the absolute position of the shapes upper-left corner.
    fn set_pos(&mut self, pos: na::Vector2<f64>) {
        self.translate(-self.pos());
        self.translate(pos)
    }
    /// generate the path of its outline, or if applicable itself as a [kurbo::BezPath].
    fn outline_path(&self) -> kurbo::BezPath;
}
