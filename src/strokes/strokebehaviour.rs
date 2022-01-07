/// Specifies that a type behaves as a stroke
pub trait StrokeBehaviour {
    /// translates (as in moves) the type for offset
    fn translate(&mut self, offset: na::Vector2<f64>);
    /// resizes the type to the desired new_bounds
    fn resize(&mut self, new_bounds: p2d::bounding_volume::AABB);
}
