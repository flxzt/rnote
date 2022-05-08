use p2d::bounding_volume::AABB;

use crate::transform::TransformBehaviour;

/// types that behave as a shape
pub trait ShapeBehaviour: TransformBehaviour {
    /// The bounds of the shape
    fn bounds(&self) -> AABB;

    /// The hitboxes of the shape
    fn hitboxes(&self) -> Vec<AABB>;
}
