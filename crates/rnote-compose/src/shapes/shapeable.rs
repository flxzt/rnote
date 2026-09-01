// Imports
use crate::Transformable;
use p2d::bounding_volume::Aabb;
use p2d::math::Vector2;

/// Types that behave as a shape.
pub trait Shapeable: Transformable {
    /// The bounds of the shape.
    fn bounds(&self) -> Aabb;
    /// The hitboxes of the shape.
    fn hitboxes(&self) -> Vec<Aabb>;
    /// The absolute position of the shapes upper-left corner.
    fn pos(&self) -> Vector2 {
        self.bounds().mins
    }
    /// Set the absolute position of the shapes upper-left corner.
    fn set_pos(&mut self, pos: Vector2) {
        self.translate(-self.pos());
        self.translate(pos)
    }
    /// generate the path of its outline, or if applicable itself as a [kurbo::BezPath].
    fn outline_path(&self) -> kurbo::BezPath;
}
