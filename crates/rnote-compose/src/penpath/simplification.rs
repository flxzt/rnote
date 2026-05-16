use crate::penpath::Element;

/// Projection parameter of point `p` onto segment `a..b`.
fn segment_projection(p: na::Vector2<f64>, a: na::Vector2<f64>, b: na::Vector2<f64>) -> f64 {
    let ab = b - a;
    let ab_len_sq = ab.norm_squared();

    if ab_len_sq == 0.0 {
        // if a == b, interpret as midpoint (for pressure interpolation)
        return 0.5;
    }

    ((p - a).dot(&ab) / ab_len_sq).clamp(0.0, 1.0)
}

/// Modified Ramer-Douglas-Peucker simplification that additionally considers pressure.
/// Returns the optimized polyline elements that approximate the original path.
///
/// https://en.wikipedia.org/wiki/Ramer%E2%80%93Douglas%E2%80%93Peucker_algorithm
pub(crate) fn ramer_douglas_peucker(
    points: &[Element],
    geometry_epsilon: f64,
    pressure_epsilon: f64,
) -> Vec<Element> {
    if points.len() < 3 {
        return points.to_vec();
    }

    let geometry_epsilon_sq = geometry_epsilon * geometry_epsilon;

    let mut keep_mask = vec![false; points.len()];
    keep_mask[0] = true;
    keep_mask[points.len() - 1] = true;

    let mut ranges_stack = vec![(0, points.len() - 1)];

    while let Some((start_index, end_index)) = ranges_stack.pop() {
        if end_index <= start_index + 1 {
            continue;
        }

        let start_point = &points[start_index];
        let end_point = &points[end_index];

        let mut max_score = 0.0;
        let mut max_score_index = start_index;

        for point_index in (start_index + 1)..end_index {
            let current_point = &points[point_index];

            let t = segment_projection(current_point.pos, start_point.pos, end_point.pos);
            let projected_point = start_point.pos + t * (end_point.pos - start_point.pos);
            let projected_point_pressure =
                start_point.pressure + t * (end_point.pressure - start_point.pressure);

            // squared distance to the closest point on the segment
            let geometry_score =
                (current_point.pos - projected_point).norm_squared() / geometry_epsilon_sq;

            // pressure deviation from the interpolated pressure of the closest point on the segment
            let pressure_score =
                (current_point.pressure - projected_point_pressure).abs() / pressure_epsilon;

            let combined_score = geometry_score.max(pressure_score);

            if combined_score > max_score {
                max_score = combined_score;
                max_score_index = point_index;
            }
        }

        if max_score > 1.0 {
            keep_mask[max_score_index] = true;
            ranges_stack.push((start_index, max_score_index));
            ranges_stack.push((max_score_index, end_index));
        }
    }

    keep_mask
        .iter()
        .zip(points.iter())
        .filter_map(|(&keep, point)| keep.then_some(*point))
        .collect()
}
