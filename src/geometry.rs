use gtk4::graphene;

/// Match offset to the aspect ratio of the AABB
pub fn restrict_offset_to_aabb_aspect_ratio(
    aabb: p2d::bounding_volume::AABB,
    offset: na::Vector2<f64>,
) -> na::Vector2<f64> {
    let aspect_ratio = aabb.extents()[1] / aabb.extents()[0];
    let offsetted_aspect_ratio = (aabb.extents()[1] + offset[1]) / (aabb.extents()[0] + offset[0]);

    if offsetted_aspect_ratio > aspect_ratio {
        let scalefactor = aabb.extents()[1] / (aabb.extents()[1] - offset[1]);
        na::vector![aabb.extents()[0] - aabb.extents()[0] * scalefactor, offset[1]]
    } else {
        let scalefactor = aabb.extents()[0] / (aabb.extents()[0] - offset[0]);
        na::vector![ offset[0], aabb.extents()[1] - aabb.extents()[1] * scalefactor]
    }
    //na::vector![0.0, offset[1]]
}

/// AABB to graphene Rect
pub fn aabb_to_graphene_rect(aabb: p2d::bounding_volume::AABB) -> graphene::Rect {
    graphene::Rect::new(
        aabb.mins[0] as f32,
        aabb.mins[1] as f32,
        (aabb.maxs[0] - aabb.mins[0]) as f32,
        (aabb.maxs[1] - aabb.mins[1]) as f32,
    )
}

/// splits a aabb into multiple which have the given size. Their union contains the given aabb.
/// The boxes on the edges might extend the given aabb, so clipping these AABB probably is needed.
/// Used when generating the background
pub fn split_aabb_extended(
    aabb: p2d::bounding_volume::AABB,
    mut splitted_size: na::Vector2<f64>,
) -> Vec<p2d::bounding_volume::AABB> {
    let mut splitted_aabbs = Vec::new();

    let mut offset_x = aabb.mins[0];
    let mut offset_y = aabb.mins[1];
    let width = aabb.maxs[0] - aabb.mins[0];
    let height = aabb.maxs[1] - aabb.mins[1];

    if width <= splitted_size[0] {
        splitted_size[0] = width;
    }
    if height <= splitted_size[1] {
        splitted_size[1] = height;
    }

    while offset_y < height {
        while offset_x < width {
            splitted_aabbs.push(p2d::bounding_volume::AABB::new(
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

/// splits a aabb into multiple which have a maximum of the given size. Their union is the given aabb. The boxes on the edges are clipped to fit into the given aabb
pub fn split_aabb(
    aabb: p2d::bounding_volume::AABB,
    mut splitted_size: na::Vector2<f64>,
) -> Vec<p2d::bounding_volume::AABB> {
    let mut splitted_aabbs = Vec::new();

    let mut offset_x = aabb.mins[0];
    let mut offset_y = aabb.mins[1];
    let width = aabb.maxs[0] - aabb.mins[0];
    let height = aabb.maxs[1] - aabb.mins[1];

    if width <= splitted_size[0] {
        splitted_size[0] = width;
    }
    if height <= splitted_size[1] {
        splitted_size[1] = height;
    }

    while offset_y < height - splitted_size[0] {
        while offset_x < width - splitted_size[1] {
            splitted_aabbs.push(p2d::bounding_volume::AABB::new(
                na::point![offset_x, offset_y],
                na::point![offset_x + splitted_size[0], offset_y + splitted_size[1]],
            ));

            offset_x += splitted_size[0];
        }
        // get the last and clipped rectangle for the current row
        if offset_x < width {
            splitted_aabbs.push(p2d::bounding_volume::AABB::new(
                na::point![offset_x, offset_y],
                na::point![aabb.maxs[0], offset_y + splitted_size[1]],
            ));
        }

        offset_x = aabb.mins[0];
        offset_y += splitted_size[1];
    }
    // get the last and clipped rectangles for the last column
    if offset_y < height {
        while offset_x < width - splitted_size[1] {
            splitted_aabbs.push(p2d::bounding_volume::AABB::new(
                na::point![offset_x, offset_y],
                na::point![offset_x + splitted_size[0], aabb.maxs[1]],
            ));

            offset_x += splitted_size[0];
        }
        // get the last and clipped rectangle for the current row
        if offset_x < width {
            splitted_aabbs.push(p2d::bounding_volume::AABB::new(
                na::point![offset_x, offset_y],
                na::point![aabb.maxs[0], aabb.maxs[1]],
            ));
        }
    }

    splitted_aabbs
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

pub fn aabb_new_positive(
    start: na::Vector2<f64>,
    end: na::Vector2<f64>,
) -> p2d::bounding_volume::AABB {
    if start[0] <= end[0] && start[1] <= end[1] {
        p2d::bounding_volume::AABB::new(na::point![start[0], start[1]], na::point![end[0], end[1]])
    } else if start[0] > end[0] && start[1] <= end[1] {
        p2d::bounding_volume::AABB::new(na::point![end[0], start[1]], na::point![start[0], end[1]])
    } else if start[0] <= end[0] && start[1] > end[1] {
        p2d::bounding_volume::AABB::new(na::point![start[0], end[1]], na::point![end[0], start[1]])
    } else {
        p2d::bounding_volume::AABB::new(na::point![end[0], end[1]], na::point![start[0], start[1]])
    }
}

/// clamp a aabb to min size, max size
pub fn aabb_clamp(
    aabb: p2d::bounding_volume::AABB,
    min: Option<p2d::bounding_volume::AABB>,
    max: Option<p2d::bounding_volume::AABB>,
) -> p2d::bounding_volume::AABB {
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

    p2d::bounding_volume::AABB::new(
        na::point![aabb_mins_x, aabb_mins_y],
        na::point![aabb_maxs_x, aabb_maxs_y],
    )
}

/// Scale a aabb by the zoom
pub fn aabb_scale(aabb: p2d::bounding_volume::AABB, zoom: f64) -> p2d::bounding_volume::AABB {
    p2d::bounding_volume::AABB::new(
        na::Point2::<f64>::from(na::vector![aabb.mins[0], aabb.mins[1]].scale(zoom)),
        na::Point2::<f64>::from(na::vector![aabb.maxs[0], aabb.maxs[1]].scale(zoom)),
    )
}

pub fn aabb_translate(
    aabb: p2d::bounding_volume::AABB,
    offset: na::Vector2<f64>,
) -> p2d::bounding_volume::AABB {
    p2d::bounding_volume::AABB::new(
        na::point![aabb.mins[0] + offset[0], aabb.mins[1] + offset[1]],
        na::point![aabb.maxs[0] + offset[0], aabb.maxs[1] + offset[1]],
    )
}

/// Shrinks the aabb to the nearest integer of its vertices
pub fn aabb_floor(aabb: p2d::bounding_volume::AABB) -> p2d::bounding_volume::AABB {
    p2d::bounding_volume::AABB::new(
        na::point![aabb.mins[0].ceil(), aabb.mins[1].ceil()],
        na::point![aabb.maxs[0].floor(), aabb.maxs[1].floor()],
    )
}

/// Extends the aabb to the nearest integer of its vertices
pub fn aabb_ceil(aabb: p2d::bounding_volume::AABB) -> p2d::bounding_volume::AABB {
    p2d::bounding_volume::AABB::new(
        na::point![aabb.mins[0].floor(), aabb.mins[1].floor()],
        na::point![aabb.maxs[0].ceil(), aabb.maxs[1].ceil()],
    )
}
