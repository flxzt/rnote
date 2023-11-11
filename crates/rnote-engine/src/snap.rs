// Imports
use p2d::bounding_volume::Aabb;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SnapCorner {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

impl SnapCorner {
    /// Determine the corner for the position to snap to depending on to which corner of the bounds it is closest to.
    pub fn determine_from_bounds(bounds: Aabb, pos: na::Vector2<f64>) -> Self {
        let dist_left = (pos[0] - bounds.mins[0]).abs();
        let dist_right = (pos[0] - bounds.maxs[0]).abs();
        let dist_top = (pos[1] - bounds.mins[1]).abs();
        let dist_bottom = (pos[1] - bounds.maxs[1]).abs();

        let snap_left = dist_left < dist_right;
        let snap_top = dist_top < dist_bottom;

        match (snap_left, snap_top) {
            (true, true) => Self::TopLeft,
            (true, false) => Self::BottomLeft,
            (false, true) => Self::TopRight,
            (false, false) => Self::BottomRight,
        }
    }
}
