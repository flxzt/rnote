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
/// Returns a mask for which points to keep.
///
/// https://en.wikipedia.org/wiki/Ramer%E2%80%93Douglas%E2%80%93Peucker_algorithm
pub(crate) fn ramer_douglas_peucker(
    points: &[Element],
    geometry_epsilon: f64,
    pressure_epsilon: f64,
) -> Vec<bool> {
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
}

/// Redistribute pressure from removed points to their neighboring kept points.
pub(crate) fn apply_mask_redistribute_pressure(
    points: &[Element],
    keep_mask: &[bool],
) -> Vec<Element> {
    // build ordered kept indices
    let kept_indices: Vec<usize> = keep_mask
        .iter()
        .enumerate()
        .filter_map(|(idx, &keep)| keep.then_some(idx))
        .collect();

    let mut pressure_sums = vec![0.0; points.len()];
    let mut pressure_weights = vec![0.0; points.len()];

    let mut span_index = 0;
    for point_index in 0..points.len() {
        if keep_mask[point_index] {
            continue;
        }

        while span_index + 1 < kept_indices.len() && point_index > kept_indices[span_index + 1] {
            span_index += 1;
        }

        let left_index = kept_indices[span_index];
        let right_index = kept_indices[span_index + 1];

        let span_length = (right_index - left_index) as f64;
        let t = (point_index - left_index) as f64 / span_length;
        let left_weight = 1.0 - t;
        let right_weight = t;

        let pressure_value = points[point_index].pressure;
        pressure_sums[left_index] += pressure_value * left_weight;
        pressure_weights[left_index] += left_weight;
        pressure_sums[right_index] += pressure_value * right_weight;
        pressure_weights[right_index] += right_weight;
    }

    kept_indices
        .into_iter()
        .map(|index| {
            let mut kept_point = points[index];
            let weight = pressure_weights[index];
            if weight > 0.0 {
                kept_point.pressure = (pressure_sums[index] / weight).clamp(0.0, 1.0);
            }
            kept_point
        })
        .collect()
}
