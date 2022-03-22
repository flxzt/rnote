use p2d::bounding_volume::AABB;

use crate::transform::TransformBehaviour;

pub trait ShapeBehaviour: TransformBehaviour {
    fn bounds(&self) -> AABB;
}
