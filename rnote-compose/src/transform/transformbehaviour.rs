/// Specifies that a type can be (geometrically) transformed
pub trait TransformBehaviour {
    /// translates (as in moves) the stroke with offset
    fn translate(&mut self, offset: na::Vector2<f64>);
    /// rotates in angle (rad)
    fn rotate(&mut self, angle: f64, center: na::Point2<f64>);
    /// scales by the desired scale
    fn scale(&mut self, scale: na::Vector2<f64>);
}
