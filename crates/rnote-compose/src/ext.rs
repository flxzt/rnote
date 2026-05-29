// Imports
use crate::SplitOrder;
use p2d::bounding_volume::Aabb;
use p2d::glamx::DAffine2;
use p2d::glamx::prelude::{DPose2, DRot2};
use p2d::math::Vector2;

/// Extension trait for [`p2d::math::Vector2`].
pub trait Vector2Ext
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
    /// The mean of x,y values of the vector
    fn mean(&self) -> f64;
    /// Round to the next integer
    fn round(&self) -> Self;
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
    /// Approximate equality
    fn approx_eq(&self, other: &Self) -> bool;
}

impl Vector2Ext for Vector2 {
    fn orth_unit(&self) -> Self {
        self.normalize_or_zero()
            .rotate(Vector2::from_angle(std::f64::consts::PI * 0.5))
    }

    fn mins(&self, other: &Self) -> Self {
        Vector2::new(self.x.min(other.x), self.y.min(other.y))
    }

    fn maxs(&self, other: &Self) -> Self {
        Vector2::new(self.x.max(other.x), self.y.max(other.y))
    }

    fn mins_maxs(&self, other: &Self) -> (Self, Self) {
        if self.x < other.x && self.y < other.y {
            (*self, *other)
        } else if self.x > other.x && self.y < other.y {
            (Vector2::new(other.x, self.y), Vector2::new(self.x, other.y))
        } else if self.x < other.x && self.y > other.y {
            (Vector2::new(self.x, other.y), Vector2::new(other.x, self.y))
        } else {
            (*other, *self)
        }
    }

    fn mean(&self) -> f64 {
        (self.x + self.y) / 2.
    }

    fn round(&self) -> Self {
        Vector2::new(self.x.round(), self.y.round())
    }

    fn ceil(&self) -> Self {
        Vector2::new(self.x.ceil(), self.y.ceil())
    }

    fn floor(&self) -> Self {
        Vector2::new(self.x.floor(), self.y.floor())
    }

    fn to_kurbo_point(&self) -> kurbo::Point {
        kurbo::Point {
            x: self.x,
            y: self.y,
        }
    }

    fn to_kurbo_vec(&self) -> kurbo::Vec2 {
        kurbo::Vec2 {
            x: self.x,
            y: self.y,
        }
    }

    fn from_kurbo_point(kurbo_point: kurbo::Point) -> Self {
        Vector2::new(kurbo_point.x, kurbo_point.y)
    }

    fn from_kurbo_vec(kurbo_vec: kurbo::Vec2) -> Self {
        Vector2::new(kurbo_vec.x, kurbo_vec.y)
    }

    fn approx_eq(&self, other: &Self) -> bool {
        approx::relative_eq!(self.x, other.x) && approx::relative_eq!(self.y, other.y)
    }
}

/// Extension trait for [p2d::bounding_volume::Aabb].
pub trait AabbExt
where
    Self: Sized,
{
    /// New Aabb at position zero, with size zero
    fn new_zero() -> Self;
    /// New Aabb, ensuring its mins, maxs are valid (maxs >= mins)
    fn new_positive(start: Vector2, end: Vector2) -> Self;
    /// Asserts the Aabb is valid
    fn assert_valid(&self) -> anyhow::Result<()>;
    /// Translates the Aabb by a translation
    fn translate(&self, offset: Vector2) -> Self;
    /// Shrinks the aabb to the nearest integer of its vertices
    fn floor(&self) -> Self;
    /// Extends the aabb to the nearest integer of its vertices
    fn ceil(&self) -> Self;
    /// Clamps to the min and max bounds
    fn clamp(&self, min: Option<Self>, max: Option<Self>) -> Self;
    /// extends on every side by the given size
    fn extend_by(&self, extend_by: Vector2) -> Self;
    /// extends on left side by the given size
    fn extend_left_by(&self, extend: f64) -> Self;
    /// extends on right side by the given size
    fn extend_right_by(&self, extend: f64) -> Self;
    /// extends on top side by the given size
    fn extend_top_by(&self, extend: f64) -> Self;
    /// extends on bottom side by the given size
    fn extend_bottom_by(&self, extend: f64) -> Self;
    /// extends on right and bottom side by the given size
    fn extend_right_and_bottom_by(&self, extend_by: Vector2) -> Self;
    /// Scales the Aabb by the scalefactor
    fn scale(&self, scale: f64) -> Self;
    /// Scales the Aabb by the scale vector
    fn scale_non_uniform(&self, scale: Vector2) -> Self;
    /// Ensures the Aabb is positive (maxs >= mins)
    fn ensure_positive(&mut self);
    /// Splits the Aabb horizontally in the center
    fn hsplit(&self) -> [Self; 2];
    /// Splits the Aabb vertically in the center
    fn vsplit(&self) -> [Self; 2];
    /// splits a aabb into multiple which have a maximum of the given size. Their union is the given aabb.
    /// the split bounds are exactly fitted to not overlap, or extend the given bounds
    fn split(self, split_size: Vector2) -> Vec<Self>;
    /// splits a aabb into multiple of the given size. Their union contains the given aabb.
    /// The boxes on the edges most likely extend beyond the given aabb.
    fn split_extended(self, split_size: Vector2) -> Vec<Self>;
    /// splits a aabb into multiple of the given size. Their union contains the given aabb.
    /// It is also guaranteed that bounding boxes are aligned to the origin, meaning (0.0,0.0) is the corner of four boxes.
    /// The boxes on the edges most likely extend beyond the given aabb.
    fn split_extended_origin_aligned(
        self,
        split_size: Vector2,
        split_order: SplitOrder,
    ) -> Vec<Self>;
    /// Converts a Aabb to a kurbo Rectangle
    fn to_kurbo_rect(&self) -> kurbo::Rect;
    /// Converts a kurbo Rectangle to Aabb
    fn from_kurbo_rect(rect: kurbo::Rect) -> Self;
    /// Check if the bounds intersect with a tolerance
    fn intersects_w_tolerance(&self, other: &Self, tolerance: f64) -> bool;
    /// Approximate equality
    fn approx_eq(&self, other: &Self) -> bool;
}

impl AabbExt for Aabb {
    fn new_zero() -> Self {
        Aabb::new(Vector2::ZERO, Vector2::ZERO)
    }

    fn new_positive(start: Vector2, end: Vector2) -> Self {
        if start.x <= end.x && start.y <= end.y {
            Aabb::new(Vector2::new(start.x, start.y), Vector2::new(end.x, end.y))
        } else if start.x > end.x && start.y <= end.y {
            Aabb::new(Vector2::new(end.x, start.y), Vector2::new(start.x, end.y))
        } else if start.x <= end.x && start.y > end.y {
            Aabb::new(Vector2::new(start.x, end.y), Vector2::new(end.x, start.y))
        } else {
            Aabb::new(Vector2::new(end.x, end.y), Vector2::new(start.x, start.y))
        }
    }

    fn assert_valid(&self) -> anyhow::Result<()> {
        if self.extents().x < 0.0
            || self.extents().y < 0.0
            || self.maxs.x < self.mins.x
            || self.maxs.y < self.mins.y
        {
            Err(anyhow::anyhow!(
                "Assert bounds valid failed, invalid bounds `{:?}`.",
                self,
            ))
        } else {
            Ok(())
        }
    }

    fn translate(&self, offset: Vector2) -> Aabb {
        self.transform_by(&DPose2::from_translation(offset))
    }

    fn floor(&self) -> Aabb {
        Aabb::new(
            Vector2::new(self.mins.x.ceil(), self.mins.y.ceil()),
            Vector2::new(self.maxs.x.floor(), self.maxs.y.floor()),
        )
    }

    fn ceil(&self) -> Aabb {
        Aabb::new(
            Vector2::new(self.mins.x.floor(), self.mins.y.floor()),
            Vector2::new(self.maxs.x.ceil(), self.maxs.y.ceil()),
        )
    }

    fn clamp(&self, min: Option<Self>, max: Option<Self>) -> Self {
        let mut aabb_mins_x = self.mins.x;
        let mut aabb_mins_y = self.mins.y;
        let mut aabb_maxs_x = self.maxs.x;
        let mut aabb_maxs_y = self.maxs.y;

        if let Some(min) = min {
            aabb_mins_x = self.mins.x.min(min.mins.x);
            aabb_mins_y = self.mins.y.min(min.mins.y);
            aabb_maxs_x = self.maxs.x.max(min.maxs.x);
            aabb_maxs_y = self.maxs.y.max(min.maxs.y);
        }
        if let Some(max) = max {
            aabb_mins_x = self.mins.x.max(max.mins.x);
            aabb_mins_y = self.mins.y.max(max.mins.y);
            aabb_maxs_x = self.maxs.x.min(max.maxs.x);
            aabb_maxs_y = self.maxs.y.min(max.maxs.y);
        }

        Aabb::new(
            Vector2::new(aabb_mins_x, aabb_mins_y),
            Vector2::new(aabb_maxs_x, aabb_maxs_y),
        )
    }

    fn extend_by(&self, extend_by: Vector2) -> Aabb {
        Aabb::new(self.mins - extend_by, self.maxs + extend_by)
    }

    fn extend_left_by(&self, extend: f64) -> Aabb {
        Aabb::new(Vector2::new(self.mins.x - extend, self.mins.y), self.maxs)
    }

    fn extend_right_by(&self, extend: f64) -> Aabb {
        Aabb::new(self.mins, Vector2::new(self.maxs.x + extend, self.maxs.y))
    }

    fn extend_top_by(&self, extend: f64) -> Aabb {
        Aabb::new(Vector2::new(self.mins.x, self.mins.y - extend), self.maxs)
    }

    fn extend_bottom_by(&self, extend: f64) -> Aabb {
        Aabb::new(self.mins, Vector2::new(self.maxs.x, self.maxs.y + extend))
    }

    fn extend_right_and_bottom_by(&self, extend_by: Vector2) -> Aabb {
        Aabb::new(self.mins, self.maxs + extend_by)
    }

    fn scale(&self, scale: f64) -> Aabb {
        Aabb::new(self.mins * scale, self.maxs * scale)
    }

    fn scale_non_uniform(&self, scale: Vector2) -> Aabb {
        Aabb::new(self.mins * scale, self.maxs * scale)
    }

    fn ensure_positive(&mut self) {
        if self.mins.x > self.maxs.x {
            std::mem::swap(&mut self.mins.x, &mut self.maxs.x);
        }
        if self.mins.y > self.maxs.y {
            std::mem::swap(&mut self.mins.y, &mut self.maxs.y);
        }
    }

    fn hsplit(&self) -> [Self; 2] {
        [
            Aabb::new(self.mins, Vector2::new(self.center().x, self.maxs.y)),
            Aabb::new(Vector2::new(self.center().x, self.mins.y), self.maxs),
        ]
    }

    fn vsplit(&self) -> [Self; 2] {
        [
            Aabb::new(self.mins, Vector2::new(self.maxs.x, self.center().y)),
            Aabb::new(Vector2::new(self.mins.x, self.center().y), self.maxs),
        ]
    }

    fn split(self, split_size: Vector2) -> Vec<Self> {
        let mut split_aabbs = vec![self];

        // Split them horizontally
        while split_size.x < split_aabbs[0].extents().x {
            let old_split = split_aabbs.clone();
            split_aabbs.clear();

            for old in old_split.iter() {
                split_aabbs.append(&mut old.hsplit().to_vec());
            }
        }

        // Split them vertically
        while split_size.y < split_aabbs[0].extents().y {
            let old_split = split_aabbs.clone();
            split_aabbs.clear();

            for old in old_split.iter() {
                split_aabbs.append(&mut old.vsplit().to_vec());
            }
        }

        split_aabbs
    }

    fn split_extended(self, mut split_size: Vector2) -> Vec<Self> {
        let mut split_aabbs = Vec::new();

        let mut offset_x = self.mins.x;
        let mut offset_y = self.mins.y;
        let width = self.extents().x;
        let height = self.extents().y;

        if width <= split_size.x {
            split_size.x = width;
        }
        if height <= split_size.y {
            split_size.y = height;
        }

        while offset_y < height {
            while offset_x < width {
                split_aabbs.push(Aabb::new(
                    Vector2::new(offset_x, offset_y),
                    Vector2::new(offset_x + split_size.x, offset_y + split_size.y),
                ));

                offset_x += split_size.x;
            }

            offset_x = self.mins.x;
            offset_y += split_size.y;
        }

        split_aabbs
    }

    fn split_extended_origin_aligned(
        self,
        split_size: Vector2,
        split_order: SplitOrder,
    ) -> Vec<Self> {
        let mut split_aabbs = Vec::new();

        if split_size.x <= 0.0 || split_size.y <= 0.0 {
            return vec![];
        }

        let (outer_idx, inner_idx) = match split_order {
            SplitOrder::RowMajor => (1, 0),
            SplitOrder::ColumnMajor => (0, 1),
        };

        let mut offset_outer =
            (self.mins[outer_idx] / split_size[outer_idx]).floor() * split_size[outer_idx];

        while offset_outer < self.maxs[outer_idx] {
            let mut offset_inner =
                (self.mins[inner_idx] / split_size[inner_idx]).floor() * split_size[inner_idx];

            while offset_inner < self.maxs[inner_idx] {
                let mins = match split_order {
                    SplitOrder::RowMajor => Vector2::new(offset_inner, offset_outer),
                    SplitOrder::ColumnMajor => Vector2::new(offset_outer, offset_inner),
                };

                split_aabbs.push(Aabb::new(mins, mins + split_size));

                offset_inner += split_size[inner_idx];
            }

            offset_outer += split_size[outer_idx];
        }

        split_aabbs
    }

    fn to_kurbo_rect(&self) -> kurbo::Rect {
        kurbo::Rect::from_points(self.mins.to_kurbo_point(), self.maxs.to_kurbo_point())
    }

    fn from_kurbo_rect(rect: kurbo::Rect) -> Self {
        Aabb::new(
            Vector2::new(rect.x0, rect.y0),
            Vector2::new(rect.x1, rect.y1),
        )
    }

    fn intersects_w_tolerance(&self, other: &Self, tolerance: f64) -> bool {
        let Some(intersection) = self.intersection(other) else {
            return false;
        };
        intersection.extents().x > tolerance && intersection.extents().y > tolerance
    }

    fn approx_eq(&self, other: &Self) -> bool {
        self.mins.approx_eq(&other.mins) && self.maxs.approx_eq(&other.maxs)
    }
}

/// Extension trait for [`DAffine2`].
pub trait DAffine2Ext
where
    Self: Sized,
{
    /// converting to kurbo affine
    fn to_kurbo(self) -> kurbo::Affine;
    /// converting from kurbo affine
    fn from_kurbo(affine: kurbo::Affine) -> Self;
    /// Transforms the Aabb vertices and calculates a new that contains them.
    fn transform_aabb(&self, aabb: Aabb) -> Aabb;
    /// Append a translation to the transform.
    fn append_translation_mut(&mut self, offset: Vector2);
    /// Append a rotation around a center to the transform.
    fn append_rotation_wrt_center_mut(&mut self, angle: f64, center: Vector2);
    /// Append a scale to the transform.
    fn append_scale_mut(&mut self, scale: Vector2);
    /// Convert the transform to a Svg attribute string, insertable into svg elements.
    fn to_svg_transform_attr_str(&self) -> String;
}

impl DAffine2Ext for DAffine2 {
    fn to_kurbo(self) -> kurbo::Affine {
        let array = self.to_cols_array_2d();
        kurbo::Affine::new([
            array[0][0],
            array[0][1],
            array[1][0],
            array[1][1],
            array[2][0],
            array[2][1],
        ])
    }

    fn from_kurbo(affine: kurbo::Affine) -> Self {
        let matrix = affine.as_coeffs();
        Self::from_cols_array(&[
            matrix[0], matrix[1], matrix[2], matrix[3], matrix[4], matrix[5],
        ])
    }

    fn transform_aabb(&self, aabb: Aabb) -> Aabb {
        let p0 = self.transform_point2(Vector2::new(aabb.mins.x, aabb.mins.y));
        let p1 = self.transform_point2(Vector2::new(aabb.mins.x, aabb.maxs.y));
        let p2 = self.transform_point2(Vector2::new(aabb.maxs.x, aabb.maxs.y));
        let p3 = self.transform_point2(Vector2::new(aabb.maxs.x, aabb.mins.y));
        let min_x = p0.x.min(p1.x).min(p2.x).min(p3.x);
        let min_y = p0.y.min(p1.y).min(p2.y).min(p3.y);
        let max_x = p0.x.max(p1.x).max(p2.x).max(p3.x);
        let max_y = p0.y.max(p1.y).max(p2.y).max(p3.y);
        Aabb::new_positive(Vector2::new(min_x, min_y), Vector2::new(max_x, max_y))
    }

    fn append_translation_mut(&mut self, offset: Vector2) {
        *self = DAffine2::from_translation(offset) * *self;
    }

    fn append_rotation_wrt_center_mut(&mut self, angle: f64, center: Vector2) {
        *self = DAffine2::from_translation(-center) * *self;
        *self = DAffine2::from_angle(angle) * *self;
        *self = DAffine2::from_translation(center) * *self;
    }

    fn append_scale_mut(&mut self, scale: Vector2) {
        *self = DAffine2::from_scale(scale) * *self;
    }

    /// Convert the transform to a Svg attribute string, insertable into svg elements.
    fn to_svg_transform_attr_str(&self) -> String {
        let array = self.to_cols_array_2d();
        format!(
            "matrix({:.3} {:.3} {:.3} {:.3} {:.3} {:.3})",
            array[0][0], array[0][1], array[1][0], array[1][1], array[2][0], array[2][1],
        )
    }
}

/// Extension trait for [`DPose2`].
pub trait DPose2Ext
where
    Self: Sized,
{
    /// Append rotation with regards to a supplied center
    fn append_rotation_wrt_center(self, rotation: f64, center: Vector2) -> Self;
}

impl DPose2Ext for DPose2 {
    fn append_rotation_wrt_center(self, angle: f64, center: Vector2) -> Self {
        DPose2::from_rotation(DRot2::from_angle(angle))
            .prepend_translation(-center)
            .append_translation(center)
            * self
    }
}

/// Extension trait for types that implement [kurbo::Shape].
pub trait KurboShapeExt
where
    Self: Sized + kurbo::Shape,
{
    /// Converting the bounds to parry2d aabb bounds
    fn bounds_to_p2d_aabb(&self) -> Aabb {
        let rect = self.bounding_box();
        Aabb::new(
            Vector2::new(rect.x0, rect.y0),
            Vector2::new(rect.x1, rect.y1),
        )
    }
}

impl KurboShapeExt for kurbo::PathSeg {}
impl KurboShapeExt for kurbo::Arc {}
impl KurboShapeExt for kurbo::BezPath {}
impl KurboShapeExt for kurbo::Circle {}
impl KurboShapeExt for kurbo::CircleSegment {}
impl KurboShapeExt for kurbo::CubicBez {}
impl KurboShapeExt for kurbo::Ellipse {}
impl KurboShapeExt for kurbo::Line {}
impl KurboShapeExt for kurbo::QuadBez {}
impl KurboShapeExt for kurbo::Rect {}
impl KurboShapeExt for kurbo::RoundedRect {}
