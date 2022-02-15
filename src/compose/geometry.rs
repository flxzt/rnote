use geo::line_string;
use gtk4::graphene;
use p2d::bounding_volume::AABB;

pub fn vector2_unit_tang(vec: na::Vector2<f64>) -> na::Vector2<f64> {
    if vec.magnitude() > 0.0 {
        vec.normalize()
    } else {
        na::Vector2::from_element(0.0)
    }
}

pub fn vector2_unit_norm(vec: na::Vector2<f64>) -> na::Vector2<f64> {
    let rot_90deg = na::Rotation2::new(std::f64::consts::PI / 2.0);

    let normalized = if vec.magnitude() > 0.0 {
        vec.normalize()
    } else {
        return na::Vector2::from_element(0.0);
    };

    rot_90deg * normalized
}

pub fn vector2_mins(vec: na::Vector2<f64>, other: na::Vector2<f64>) -> na::Vector2<f64> {
    na::vector![vec[0].min(other[0]), vec[1].min(other[1])]
}

pub fn vector2_maxs(vec: na::Vector2<f64>, other: na::Vector2<f64>) -> na::Vector2<f64> {
    na::vector![vec[0].max(other[0]), vec[1].max(other[1])]
}

/// Return mins, maxs
pub fn vec2_mins_maxs(
    first: na::Vector2<f64>,
    second: na::Vector2<f64>,
) -> (na::Vector2<f64>, na::Vector2<f64>) {
    if first[0] < second[0] && first[1] < second[1] {
        (first, second)
    } else if first[0] > second[0] && first[1] < second[1] {
        (
            na::vector![second[0], first[1]],
            na::vector![first[0], second[1]],
        )
    } else if first[0] < second[0] && first[1] > second[1] {
        (
            na::vector![first[0], second[1]],
            na::vector![second[0], first[1]],
        )
    } else {
        (second, first)
    }
}

/// AABB to graphene Rect
pub fn aabb_to_graphene_rect(aabb: AABB) -> graphene::Rect {
    graphene::Rect::new(
        aabb.mins[0] as f32,
        aabb.mins[1] as f32,
        (aabb.extents()[0]) as f32,
        (aabb.extents()[1]) as f32,
    )
}

/// splits a aabb into multiple of the given size. Their union contains the given aabb.
/// The boxes on the edges most likely extend beyond the given aabb.
pub fn split_aabb_extended(aabb: AABB, mut splitted_size: na::Vector2<f64>) -> Vec<AABB> {
    let mut splitted_aabbs = Vec::new();

    let mut offset_x = aabb.mins[0];
    let mut offset_y = aabb.mins[1];
    let width = aabb.extents()[0];
    let height = aabb.extents()[1];

    if width <= splitted_size[0] {
        splitted_size[0] = width;
    }
    if height <= splitted_size[1] {
        splitted_size[1] = height;
    }

    while offset_y < height {
        while offset_x < width {
            splitted_aabbs.push(AABB::new(
                na::point![offset_x, offset_y],
                na::point![offset_x + splitted_size[0], offset_y + splitted_size[1]],
            ));

            offset_x += splitted_size[0];
        }

        offset_x = aabb.mins[0];
        offset_y += splitted_size[1];
    }

    splitted_aabbs
}

/// splits a aabb into multiple of the given size. Their union contains the given aabb.
/// It is also guaranteed that bounding boxes are aligned to the origin, meaning (0.0,0.0) is the corner of four boxes.
/// The boxes on the edges most likely extend beyond the given aabb.
pub fn split_aabb_extended_origin_aligned(
    aabb: AABB,
    mut splitted_size: na::Vector2<f64>,
) -> Vec<AABB> {
    let mut splitted_aabbs = Vec::new();

    let width = aabb.extents()[0];
    let height = aabb.extents()[1];

    if width <= splitted_size[0] {
        splitted_size[0] = width;
    }
    if height <= splitted_size[1] {
        splitted_size[1] = height;
    }

    let n_columns = (aabb.extents()[0] / splitted_size[0]).ceil() as u32;
    let n_rows = (aabb.extents()[1] / splitted_size[1]).ceil() as u32;

    let offset = na::vector![
        (aabb.mins[0] / splitted_size[0]).floor() * splitted_size[0],
        (aabb.mins[1] / splitted_size[1]).floor() * splitted_size[1]
    ];

    for current_row in 0..=n_rows {
        for current_column in 0..=n_columns {
            let mins = na::point![
                offset[0] + f64::from(current_column) * splitted_size[0],
                offset[1] + f64::from(current_row) * splitted_size[1]
            ];
            let maxs = na::Point2::from(mins.coords + splitted_size);

            splitted_aabbs.push(AABB::new(mins, maxs));
        }
    }

    splitted_aabbs
}

/// splits a aabb into multiple which have a maximum of the given size. Their union is the given aabb.
/// the splitted bounds are exactly fitted to not overlap, or extend the given bounds
pub fn split_aabb(aabb: AABB, splitted_size: na::Vector2<f64>) -> Vec<AABB> {
    let mut splitted_aabbs = vec![aabb];

    // Split them horizontally
    while splitted_size[0] < splitted_aabbs[0].extents()[0] {
        let old_splitted = splitted_aabbs.clone();
        splitted_aabbs.clear();

        for old in old_splitted.iter() {
            splitted_aabbs.append(&mut aabb_hsplit(old).to_vec());
        }
    }

    // Split them vertically
    while splitted_size[1] < splitted_aabbs[0].extents()[1] {
        let old_splitted = splitted_aabbs.clone();
        splitted_aabbs.clear();

        for old in old_splitted.iter() {
            splitted_aabbs.append(&mut aabb_vsplit(old).to_vec());
        }
    }

    splitted_aabbs
}

// Splits the aab horizontally in the center
pub fn aabb_hsplit(aabb: &AABB) -> [AABB; 2] {
    [
        AABB::new(aabb.mins, na::point![aabb.center()[0], aabb.maxs[1]]),
        AABB::new(na::point![aabb.center()[0], aabb.mins[1]], aabb.maxs),
    ]
}

// Splits the aab vertically in the center
pub fn aabb_vsplit(aabb: &AABB) -> [AABB; 2] {
    [
        AABB::new(aabb.mins, na::point![aabb.maxs[0], aabb.center()[1]]),
        AABB::new(na::point![aabb.mins[0], aabb.center()[1]], aabb.maxs),
    ]
}

pub fn aabb_new_zero() -> AABB {
    AABB::new(na::point![0.0, 0.0], na::point![0.0, 0.0])
}

pub fn aabb_new_positive(start: na::Point2<f64>, end: na::Point2<f64>) -> AABB {
    if start[0] <= end[0] && start[1] <= end[1] {
        AABB::new(na::point![start[0], start[1]], na::point![end[0], end[1]])
    } else if start[0] > end[0] && start[1] <= end[1] {
        AABB::new(na::point![end[0], start[1]], na::point![start[0], end[1]])
    } else if start[0] <= end[0] && start[1] > end[1] {
        AABB::new(na::point![start[0], end[1]], na::point![end[0], start[1]])
    } else {
        AABB::new(na::point![end[0], end[1]], na::point![start[0], start[1]])
    }
}

pub fn aabb_ensure_valid(aabb: &mut AABB) {
    if aabb.mins[0] > aabb.maxs[0] {
        std::mem::swap(&mut aabb.mins[0], &mut aabb.maxs[0]);
    }
    if aabb.mins[1] > aabb.maxs[1] {
        std::mem::swap(&mut aabb.mins[1], &mut aabb.maxs[1]);
    }
}

/// clamp a aabb to min size, max size
pub fn aabb_clamp(aabb: AABB, min: Option<AABB>, max: Option<AABB>) -> AABB {
    let mut aabb_mins_x = aabb.mins[0];
    let mut aabb_mins_y = aabb.mins[1];
    let mut aabb_maxs_x = aabb.maxs[0];
    let mut aabb_maxs_y = aabb.maxs[1];

    if let Some(min) = min {
        aabb_mins_x = aabb.mins[0].min(min.mins[0]);
        aabb_mins_y = aabb.mins[1].min(min.mins[1]);
        aabb_maxs_x = aabb.maxs[0].max(min.maxs[0]);
        aabb_maxs_y = aabb.maxs[1].max(min.maxs[1]);
    }
    if let Some(max) = max {
        aabb_mins_x = aabb.mins[0].max(max.mins[0]);
        aabb_mins_y = aabb.mins[1].max(max.mins[1]);
        aabb_maxs_x = aabb.maxs[0].min(max.maxs[0]);
        aabb_maxs_y = aabb.maxs[1].min(max.maxs[1]);
    }

    AABB::new(
        na::point![aabb_mins_x, aabb_mins_y],
        na::point![aabb_maxs_x, aabb_maxs_y],
    )
}

/// Scale a aabb by the zoom
pub fn aabb_scale(aabb: AABB, zoom: f64) -> AABB {
    AABB::new(
        na::Point2::from(na::vector![aabb.mins[0], aabb.mins[1]].scale(zoom)),
        na::Point2::from(na::vector![aabb.maxs[0], aabb.maxs[1]].scale(zoom)),
    )
}

pub fn aabb_translate(aabb: AABB, offset: na::Vector2<f64>) -> AABB {
    aabb.transform_by(&na::convert(na::Translation2::from(offset)))
}

/// Shrinks the aabb to the nearest integer of its vertices
pub fn aabb_floor(aabb: AABB) -> AABB {
    AABB::new(
        na::point![aabb.mins[0].ceil(), aabb.mins[1].ceil()],
        na::point![aabb.maxs[0].floor(), aabb.maxs[1].floor()],
    )
}

/// Extends the aabb to the nearest integer of its vertices
pub fn aabb_ceil(aabb: AABB) -> AABB {
    AABB::new(
        na::point![aabb.mins[0].floor(), aabb.mins[1].floor()],
        na::point![aabb.maxs[0].ceil(), aabb.maxs[1].ceil()],
    )
}

pub fn aabb_expand(aabb: AABB, expand_by: na::Vector2<f64>) -> AABB {
    AABB::new(
        na::Point2::from(aabb.mins.coords - expand_by),
        na::Point2::from(aabb.maxs.coords + expand_by),
    )
}

/// Scale the source size with a specified max size, while keeping its aspect ratio
pub fn scale_with_locked_aspectratio(
    src_size: na::Vector2<f64>,
    max_size: na::Vector2<f64>,
) -> na::Vector2<f64> {
    let ratio = (max_size[0] / src_size[0]).min(max_size[1] / src_size[1]);

    src_size * ratio
}

pub fn p2d_aabb_to_geo_polygon(aabb: AABB) -> geo::Polygon<f64> {
    let line_string = line_string![
        (x: aabb.mins[0], y: aabb.mins[1]),
        (x: aabb.maxs[0], y: aabb.mins[1]),
        (x: aabb.maxs[0], y: aabb.maxs[1]),
        (x: aabb.mins[0], y: aabb.maxs[1]),
        (x: aabb.mins[0], y: aabb.mins[1]),
    ];
    geo::Polygon::new(line_string, vec![])
}

pub fn scale_inner_bounds_to_new_outer_bounds(
    old_inner_bounds: AABB,
    old_outer_bounds: AABB,
    new_outer_bounds: AABB,
) -> AABB {
    let offset = na::vector![
        new_outer_bounds.mins[0] - old_outer_bounds.mins[0],
        new_outer_bounds.mins[1] - old_outer_bounds.mins[1]
    ];

    let scalevector = na::vector![
        (new_outer_bounds.extents()[0]) / (old_outer_bounds.extents()[0]),
        (new_outer_bounds.extents()[1]) / (old_outer_bounds.extents()[1])
    ];

    AABB::new(
        na::point![
            (old_inner_bounds.mins[0] - old_outer_bounds.mins[0]) * scalevector[0]
                + old_outer_bounds.mins[0]
                + offset[0],
            (old_inner_bounds.mins[1] - old_outer_bounds.mins[1]) * scalevector[1]
                + old_outer_bounds.mins[1]
                + offset[1]
        ],
        na::point![
            (old_inner_bounds.mins[0] - old_outer_bounds.mins[0]) * scalevector[0]
                + old_outer_bounds.mins[0]
                + offset[0]
                + (old_inner_bounds.extents()[0]) * scalevector[0],
            (old_inner_bounds.mins[1] - old_outer_bounds.mins[1]) * scalevector[1]
                + old_outer_bounds.mins[1]
                + offset[1]
                + (old_inner_bounds.extents()[1]) * scalevector[1]
        ],
    )
}
