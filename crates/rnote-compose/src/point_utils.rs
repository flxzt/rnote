/// horizontally mirrors point around line 'x = centerline_x'
pub fn mirror_point_x(point: &mut na::Vector2<f64>, centerline_x: f64) {
    point.x -= centerline_x;
    point.x *= -1.0;
    point.x += centerline_x;
}

/// vertically mirrors point around line 'y = centerline_y'
pub fn mirror_point_y(point: &mut na::Vector2<f64>, centerline_y: f64) {
    point.y -= centerline_y;
    point.y *= -1.0;
    point.y += centerline_y;
}
