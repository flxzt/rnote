use p2d::bounding_volume::Aabb;

/// Helpers that extend the Vector2 type
pub trait Vector2Helpers
where
    Self: Sized,
{
    /// The orthogonal vector, normalized to length 1
    fn orth_unit(&self) -> Self;
    /// a new vector by taking the mins of each x and y values
    fn mins(&self, other: &Self) -> Self;
    /// a new vector by taking the maxs of each x and y values
    fn maxs(&self, other: &Self) -> Self;
    /// new vectors by taking the mins and maxs of each x and y values
    fn mins_maxs(&self, other: &Self) -> (Self, Self);
    /// calculates the angle self is "ahead" of other (counter clockwise)
    fn angle_ahead(&self, other: &Self) -> f64;
    /// Ceil to the next integer
    fn ceil(&self) -> Self;
    /// Floor to the next integer
    fn floor(&self) -> Self;
    /// Converts to kurbo::Point
    fn to_kurbo_point(&self) -> kurbo::Point;
    /// Converts to kurbo::Vec2
    fn to_kurbo_vec(&self) -> kurbo::Vec2;
    /// Converts from kurbo::Point
    fn from_kurbo_point(kurbo_point: kurbo::Point) -> Self;
    /// Converts from kurbo::Vec2
    fn from_kurbo_vec(kurbo_vec: kurbo::Vec2) -> Self;
}

impl Vector2Helpers for na::Vector2<f64> {
    fn orth_unit(&self) -> Self {
        let rot_90deg = na::Rotation2::new(std::f64::consts::PI * 0.5);

        let normalized = if self.magnitude() > 0.0 {
            self.normalize()
        } else {
            return na::Vector2::from_element(0.0);
        };

        rot_90deg * normalized
    }

    fn mins(&self, other: &Self) -> Self {
        na::vector![self[0].min(other[0]), self[1].min(other[1])]
    }

    fn maxs(&self, other: &Self) -> Self {
        na::vector![self[0].max(other[0]), self[1].max(other[1])]
    }

    fn mins_maxs(&self, other: &Self) -> (Self, Self) {
        if self[0] < other[0] && self[1] < other[1] {
            (*self, *other)
        } else if self[0] > other[0] && self[1] < other[1] {
            (
                na::vector![other[0], self[1]],
                na::vector![self[0], other[1]],
            )
        } else if self[0] < other[0] && self[1] > other[1] {
            (
                na::vector![self[0], other[1]],
                na::vector![other[0], self[1]],
            )
        } else {
            (*other, *self)
        }
    }

    fn angle_ahead(&self, other: &Self) -> f64 {
        other[1].atan2(other[0]) - self[1].atan2(self[0])
    }

    fn ceil(&self) -> Self {
        na::vector![self[0].ceil(), self[1].ceil()]
    }

    fn floor(&self) -> Self {
        na::vector![self[0].floor(), self[1].floor()]
    }

    fn to_kurbo_point(&self) -> kurbo::Point {
        kurbo::Point {
            x: self[0],
            y: self[1],
        }
    }

    fn to_kurbo_vec(&self) -> kurbo::Vec2 {
        kurbo::Vec2 {
            x: self[0],
            y: self[1],
        }
    }

    fn from_kurbo_point(kurbo_point: kurbo::Point) -> Self {
        na::vector![kurbo_point.x, kurbo_point.y]
    }

    fn from_kurbo_vec(kurbo_vec: kurbo::Vec2) -> Self {
        na::vector![kurbo_vec.x, kurbo_vec.y]
    }
}

/// Helpers that extend the Aabb type
pub trait AabbHelpers
where
    Self: Sized,
{
    /// New Aabb at position zero, with size zero
    fn new_zero() -> Self;
    /// New Aabb, ensuring its mins, maxs are valid (maxs >= mins)
    fn new_positive(start: na::Point2<f64>, end: na::Point2<f64>) -> Self;
    /// Asserts the Aabb is valid
    fn assert_valid(&self) -> anyhow::Result<()>;
    /// Translates the Aabb by a offset
    fn translate(&self, offset: na::Vector2<f64>) -> Self;
    /// Shrinks the aabb to the nearest integer of its vertices
    fn floor(&self) -> Self;
    /// Extends the aabb to the nearest integer of its vertices
    fn ceil(&self) -> Self;
    /// Clamps to the min and max bounds
    fn clamp(&self, min: Option<Self>, max: Option<Self>) -> Self;
    /// extends on every side by the given size
    fn extend_by(&self, extend_by: na::Vector2<f64>) -> Self;
    /// extends on left side by the given size
    fn extend_left_by(&self, extend: f64) -> Self;
    /// extends on right side by the given size
    fn extend_right_by(&self, extend: f64) -> Self;
    /// extends on top side by the given size
    fn extend_top_by(&self, extend: f64) -> Self;
    /// extends on bottom side by the given size
    fn extend_bottom_by(&self, extend: f64) -> Self;
    /// Scales the Aabb by the scalefactor
    fn scale(&self, scale: f64) -> Self;
    /// Scales the Aabb by the scale vector
    fn scale_non_uniform(&self, scale: na::Vector2<f64>) -> Self;
    /// Ensures the Aabb is positive (maxs >= mins)
    fn ensure_positive(&mut self);
    /// Splits the Aabb horizontally in the center
    fn hsplit(&self) -> [Self; 2];
    /// Splits the Aabb vertically in the center
    fn vsplit(&self) -> [Self; 2];
    /// splits a aabb into multiple which have a maximum of the given size. Their union is the given aabb.
    /// the splitted bounds are exactly fitted to not overlap, or extend the given bounds
    fn split(self, splitted_size: na::Vector2<f64>) -> Vec<Self>;
    /// splits a aabb into multiple of the given size. Their union contains the given aabb.
    /// The boxes on the edges most likely extend beyond the given aabb.
    fn split_extended(self, splitted_size: na::Vector2<f64>) -> Vec<Self>;
    /// splits a aabb into multiple of the given size. Their union contains the given aabb.
    /// It is also guaranteed that bounding boxes are aligned to the origin, meaning (0.0,0.0) is the corner of four boxes.
    /// The boxes on the edges most likely extend beyond the given aabb.
    fn split_extended_origin_aligned(self, splitted_size: na::Vector2<f64>) -> Vec<Self>;
    /// Converts a Aabb to a kurbo Rectangle
    fn to_kurbo_rect(&self) -> kurbo::Rect;
    /// Converts a kurbo Rectangle to Aabb
    fn from_kurbo_rect(rect: kurbo::Rect) -> Self;
}

impl AabbHelpers for Aabb {
    fn new_zero() -> Self {
        Aabb::new(na::point![0.0, 0.0], na::point![0.0, 0.0])
    }

    fn new_positive(start: na::Point2<f64>, end: na::Point2<f64>) -> Self {
        if start[0] <= end[0] && start[1] <= end[1] {
            Aabb::new(na::point![start[0], start[1]], na::point![end[0], end[1]])
        } else if start[0] > end[0] && start[1] <= end[1] {
            Aabb::new(na::point![end[0], start[1]], na::point![start[0], end[1]])
        } else if start[0] <= end[0] && start[1] > end[1] {
            Aabb::new(na::point![start[0], end[1]], na::point![end[0], start[1]])
        } else {
            Aabb::new(na::point![end[0], end[1]], na::point![start[0], start[1]])
        }
    }

    fn assert_valid(&self) -> anyhow::Result<()> {
        if self.extents()[0] < 0.0
            || self.extents()[1] < 0.0
            || self.maxs[0] < self.mins[0]
            || self.maxs[1] < self.mins[1]
        {
            Err(anyhow::anyhow!(
                "bounds assert_valid() failed, invalid bounds `{:?}`",
                self,
            ))
        } else {
            Ok(())
        }
    }

    fn translate(&self, offset: na::Vector2<f64>) -> Aabb {
        self.transform_by(&na::convert(na::Translation2::from(offset)))
    }

    fn floor(&self) -> Aabb {
        Aabb::new(
            na::point![self.mins[0].ceil(), self.mins[1].ceil()],
            na::point![self.maxs[0].floor(), self.maxs[1].floor()],
        )
    }

    fn ceil(&self) -> Aabb {
        Aabb::new(
            na::point![self.mins[0].floor(), self.mins[1].floor()],
            na::point![self.maxs[0].ceil(), self.maxs[1].ceil()],
        )
    }

    fn clamp(&self, min: Option<Self>, max: Option<Self>) -> Self {
        let mut aabb_mins_x = self.mins[0];
        let mut aabb_mins_y = self.mins[1];
        let mut aabb_maxs_x = self.maxs[0];
        let mut aabb_maxs_y = self.maxs[1];

        if let Some(min) = min {
            aabb_mins_x = self.mins[0].min(min.mins[0]);
            aabb_mins_y = self.mins[1].min(min.mins[1]);
            aabb_maxs_x = self.maxs[0].max(min.maxs[0]);
            aabb_maxs_y = self.maxs[1].max(min.maxs[1]);
        }
        if let Some(max) = max {
            aabb_mins_x = self.mins[0].max(max.mins[0]);
            aabb_mins_y = self.mins[1].max(max.mins[1]);
            aabb_maxs_x = self.maxs[0].min(max.maxs[0]);
            aabb_maxs_y = self.maxs[1].min(max.maxs[1]);
        }

        Aabb::new(
            na::point![aabb_mins_x, aabb_mins_y],
            na::point![aabb_maxs_x, aabb_maxs_y],
        )
    }

    fn extend_by(&self, extend_by: nalgebra::Vector2<f64>) -> Aabb {
        Aabb::new(
            na::Point2::from(self.mins.coords - extend_by),
            na::Point2::from(self.maxs.coords + extend_by),
        )
    }

    fn extend_left_by(&self, extend: f64) -> Aabb {
        Aabb::new(
            na::point![self.mins.coords[0] - extend, self.mins.coords[1]],
            na::Point2::from(self.maxs.coords),
        )
    }

    fn extend_right_by(&self, extend: f64) -> Aabb {
        Aabb::new(
            na::Point2::from(self.mins.coords),
            na::point![self.maxs.coords[0] + extend, self.maxs.coords[1]],
        )
    }

    fn extend_top_by(&self, extend: f64) -> Aabb {
        Aabb::new(
            na::point![self.mins.coords[0], self.mins.coords[1] - extend],
            na::Point2::from(self.maxs.coords),
        )
    }

    fn extend_bottom_by(&self, extend: f64) -> Aabb {
        Aabb::new(
            na::Point2::from(self.mins.coords),
            na::point![self.maxs.coords[0], self.maxs.coords[1] + extend],
        )
    }

    fn scale(&self, scale: f64) -> Aabb {
        Aabb::new(
            na::Point2::from(self.mins.coords.scale(scale)),
            na::Point2::from(self.maxs.coords.scale(scale)),
        )
    }

    fn scale_non_uniform(&self, scale: nalgebra::Vector2<f64>) -> Aabb {
        Aabb::new(
            na::Point2::from(self.mins.coords.component_mul(&scale)),
            na::Point2::from(self.maxs.coords.component_mul(&scale)),
        )
    }

    fn ensure_positive(&mut self) {
        if self.mins[0] > self.maxs[0] {
            std::mem::swap(&mut self.mins[0], &mut self.maxs[0]);
        }
        if self.mins[1] > self.maxs[1] {
            std::mem::swap(&mut self.mins[1], &mut self.maxs[1]);
        }
    }

    fn hsplit(&self) -> [Self; 2] {
        [
            Aabb::new(self.mins, na::point![self.center()[0], self.maxs[1]]),
            Aabb::new(na::point![self.center()[0], self.mins[1]], self.maxs),
        ]
    }

    fn vsplit(&self) -> [Self; 2] {
        [
            Aabb::new(self.mins, na::point![self.maxs[0], self.center()[1]]),
            Aabb::new(na::point![self.mins[0], self.center()[1]], self.maxs),
        ]
    }

    fn split(self, splitted_size: nalgebra::Vector2<f64>) -> Vec<Self> {
        let mut splitted_aabbs = vec![self];

        // Split them horizontally
        while splitted_size[0] < splitted_aabbs[0].extents()[0] {
            let old_splitted = splitted_aabbs.clone();
            splitted_aabbs.clear();

            for old in old_splitted.iter() {
                splitted_aabbs.append(&mut old.hsplit().to_vec());
            }
        }

        // Split them vertically
        while splitted_size[1] < splitted_aabbs[0].extents()[1] {
            let old_splitted = splitted_aabbs.clone();
            splitted_aabbs.clear();

            for old in old_splitted.iter() {
                splitted_aabbs.append(&mut old.vsplit().to_vec());
            }
        }

        splitted_aabbs
    }

    fn split_extended(self, mut splitted_size: na::Vector2<f64>) -> Vec<Self> {
        let mut splitted_aabbs = Vec::new();

        let mut offset_x = self.mins[0];
        let mut offset_y = self.mins[1];
        let width = self.extents()[0];
        let height = self.extents()[1];

        if width <= splitted_size[0] {
            splitted_size[0] = width;
        }
        if height <= splitted_size[1] {
            splitted_size[1] = height;
        }

        while offset_y < height {
            while offset_x < width {
                splitted_aabbs.push(Aabb::new(
                    na::point![offset_x, offset_y],
                    na::point![offset_x + splitted_size[0], offset_y + splitted_size[1]],
                ));

                offset_x += splitted_size[0];
            }

            offset_x = self.mins[0];
            offset_y += splitted_size[1];
        }

        splitted_aabbs
    }

    fn split_extended_origin_aligned(self, splitted_size: na::Vector2<f64>) -> Vec<Self> {
        let mut splitted_aabbs = Vec::new();

        if splitted_size[0] <= 0.0 || splitted_size[1] <= 0.0 {
            return vec![];
        }

        let mut offset_y = (self.mins[1] / splitted_size[1]).floor() * splitted_size[1];

        while offset_y < self.maxs[1] {
            let mut offset_x = (self.mins[0] / splitted_size[0]).floor() * splitted_size[0];

            while offset_x < self.maxs[0] {
                let mins = na::point![offset_x, offset_y];
                splitted_aabbs.push(Aabb::new(mins, mins + splitted_size));

                offset_x += splitted_size[0];
            }

            offset_y += splitted_size[1];
        }

        splitted_aabbs
    }

    fn to_kurbo_rect(&self) -> kurbo::Rect {
        kurbo::Rect::from_points(
            self.mins.coords.to_kurbo_point(),
            self.maxs.coords.to_kurbo_point(),
        )
    }

    fn from_kurbo_rect(rect: kurbo::Rect) -> Self {
        Aabb::new(na::point![rect.x0, rect.y0], na::point![rect.x1, rect.y1])
    }
}

/// Helpers that extend the Affine2 type
pub trait Affine2Helpers
where
    Self: Sized,
{
    /// converting to kurbo affine
    fn to_kurbo(self) -> kurbo::Affine;
    /// converting from kurbo affine
    fn from_kurbo(affine: kurbo::Affine) -> Self;
}

impl Affine2Helpers for na::Affine2<f64> {
    fn to_kurbo(self) -> kurbo::Affine {
        let matrix = self.to_homogeneous();

        kurbo::Affine::new([
            matrix[(0, 0)],
            matrix[(1, 0)],
            matrix[(0, 1)],
            matrix[(1, 1)],
            matrix[(0, 2)],
            matrix[(1, 2)],
        ])
    }

    fn from_kurbo(affine: kurbo::Affine) -> Self {
        let matrix = affine.as_coeffs();
        na::try_convert(na::Matrix3::new(
            matrix[0], matrix[2], matrix[4], matrix[1], matrix[3], matrix[5], 0.0, 0.0, 1.0,
        ))
        .unwrap()
    }
}

/// Scale the source size with a specified max size, while keeping its aspect ratio
pub fn scale_w_locked_aspectratio(
    src_size: nalgebra::Vector2<f64>,
    max_size: nalgebra::Vector2<f64>,
) -> na::Vector2<f64> {
    let ratio = (max_size[0] / src_size[0]).min(max_size[1] / src_size[1]);

    src_size * ratio
}

/// Scales inner bounds in context to new outer bounds
pub fn scale_inner_bounds_in_context_new_outer_bounds(
    old_inner_bounds: Aabb,
    old_outer_bounds: Aabb,
    new_outer_bounds: Aabb,
) -> Aabb {
    let offset = na::vector![
        new_outer_bounds.mins[0] - old_outer_bounds.mins[0],
        new_outer_bounds.mins[1] - old_outer_bounds.mins[1]
    ];

    let scalevector = na::vector![
        (new_outer_bounds.extents()[0]) / (old_outer_bounds.extents()[0]),
        (new_outer_bounds.extents()[1]) / (old_outer_bounds.extents()[1])
    ];

    Aabb::new(
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

/// Helpers that extend types in kurbo that implement kurbo::Shape
pub trait KurboHelpers
where
    Self: Sized + kurbo::Shape,
{
    /// Converting the bounds to parry2d aabb bounds
    fn bounds_as_p2d_aabb(&self) -> Aabb {
        let rect = self.bounding_box();
        Aabb::new(na::point![rect.x0, rect.y0], na::point![rect.x1, rect.y1])
    }
}

impl KurboHelpers for kurbo::PathSeg {}
impl KurboHelpers for kurbo::Arc {}
impl KurboHelpers for kurbo::BezPath {}
impl KurboHelpers for kurbo::Circle {}
impl KurboHelpers for kurbo::CircleSegment {}
impl KurboHelpers for kurbo::CubicBez {}
impl KurboHelpers for kurbo::Ellipse {}
impl KurboHelpers for kurbo::Line {}
impl KurboHelpers for kurbo::QuadBez {}
impl KurboHelpers for kurbo::Rect {}
impl KurboHelpers for kurbo::RoundedRect {}
