use p2d::bounding_volume::Aabb;

use crate::transform::TransformBehaviour;

/// types that behave as a shape
pub trait ShapeBehaviour: TransformBehaviour {
    /// The bounds of the shape
    fn bounds(&self) -> Aabb;

    /// The hitboxes of the shape
    fn hitboxes(&self) -> Vec<Aabb>;
}
