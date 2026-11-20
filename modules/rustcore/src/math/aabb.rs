//! Port of `core/math/aabb.h` + `aabb.cpp`.
//!
//! This is the culling primitive: `intersects`, `encloses`, `smits_intersect_ray`
//! and `project_range_in_plane` are called per-object per-frame by the renderer.

use crate::math::funcs;
use crate::math::plane::Plane;
use crate::math::vector3::Vector3;
use crate::templates::hashfuncs;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Aabb {
    pub position: Vector3,
    pub size: Vector3,
}

impl Aabb {
    #[inline]
    pub const fn new(position: Vector3, size: Vector3) -> Self {
        Self { position, size }
    }

    #[inline]
    pub fn get_volume(&self) -> f32 {
        self.size.x * self.size.y * self.size.z
    }

    #[inline]
    pub fn has_volume(&self) -> bool {
        self.size.x > 0.0 && self.size.y > 0.0 && self.size.z > 0.0
    }

    #[inline]
    pub fn has_surface(&self) -> bool {
        self.size.x > 0.0 || self.size.y > 0.0 || self.size.z > 0.0
    }

    #[inline]
    pub fn get_position(&self) -> Vector3 {
        self.position
    }

    #[inline]
    pub fn set_position(&mut self, pos: Vector3) {
        self.position = pos;
    }

    #[inline]
    pub fn get_size(&self) -> Vector3 {
        self.size
    }

    #[inline]
    pub fn set_size(&mut self, size: Vector3) {
        self.size = size;
    }

    #[inline]
    pub fn get_end(&self) -> Vector3 {
        self.position + self.size
    }

    #[inline]
    pub fn set_end(&mut self, end: Vector3) {
        self.size = end - self.position;
    }

    #[inline]
    pub fn get_center(&self) -> Vector3 {
        self.position + (self.size * 0.5)
    }

    /// Same box with a non-negative size.
    #[inline]
    pub fn abs(&self) -> Self {
        Self::new(self.position + self.size.minf(0.0), self.size.abs())
    }

    #[inline]
    pub fn is_equal_approx(&self, other: &Aabb) -> bool {
        self.position.is_equal_approx(other.position) && self.size.is_equal_approx(other.size)
    }

    #[inline]
    pub fn is_same(&self, other: &Aabb) -> bool {
        self.position.is_same(other.position) && self.size.is_same(other.size)
    }

    #[inline]
    pub fn is_finite(&self) -> bool {
        self.position.is_finite() && self.size.is_finite()
    }

    /// Both boxes overlap (touching faces do not count).
    #[inline]
    pub fn intersects(&self, other: &Aabb) -> bool {
        if self.position.x >= other.position.x + other.size.x {
            return false;
        }
        if self.position.x + self.size.x <= other.position.x {
            return false;
        }
        if self.position.y >= other.position.y + other.size.y {
            return false;
        }
        if self.position.y + self.size.y <= other.position.y {
            return false;
        }
        if self.position.z >= other.position.z + other.size.z {
            return false;
        }
        if self.position.z + self.size.z <= other.position.z {
            return false;
        }
        true
    }

    /// Like `intersects` but touching faces count as overlap.
    #[inline]
    pub fn intersects_inclusive(&self, other: &Aabb) -> bool {
        if self.position.x > other.position.x + other.size.x {
            return false;
        }
        if self.position.x + self.size.x < other.position.x {
            return false;
        }
        if self.position.y > other.position.y + other.size.y {
            return false;
        }
        if self.position.y + self.size.y < other.position.y {
            return false;
        }
        if self.position.z > other.position.z + other.size.z {
            return false;
        }
        if self.position.z + self.size.z < other.position.z {
            return false;
        }
        true
    }

    /// `other` is completely inside this box.
    #[inline]
    pub fn encloses(&self, other: &Aabb) -> bool {
        let src_min = self.position;
        let src_max = self.position + self.size;
        let dst_min = other.position;
        let dst_max = other.position + other.size;

        src_min.x <= dst_min.x
            && src_max.x >= dst_max.x
            && src_min.y <= dst_min.y
            && src_max.y >= dst_max.y
            && src_min.z <= dst_min.z
            && src_max.z >= dst_max.z
    }

    pub fn merge(&self, other: &Aabb) -> Self {
        let mut aabb = *self;
        aabb.merge_with(other);
        aabb
    }

    pub fn merge_with(&mut self, other: &Aabb) {
        let beg_1 = self.position;
        let beg_2 = other.position;
        let end_1 = self.size + beg_1;
        let end_2 = other.size + beg_2;

        let min = Vector3::new(
            if beg_1.x < beg_2.x { beg_1.x } else { beg_2.x },
            if beg_1.y < beg_2.y { beg_1.y } else { beg_2.y },
            if beg_1.z < beg_2.z { beg_1.z } else { beg_2.z },
        );
        let max = Vector3::new(
            if end_1.x > end_2.x { end_1.x } else { end_2.x },
            if end_1.y > end_2.y { end_1.y } else { end_2.y },
            if end_1.z > end_2.z { end_1.z } else { end_2.z },
        );

        self.position = min;
        self.size = max - min;
    }

    /// Box where the two intersect; empty box when disjoint.
    pub fn intersection(&self, other: &Aabb) -> Self {
        let src_min = self.position;
        let src_max = self.position + self.size;
        let dst_min = other.position;
        let dst_max = other.position + other.size;

        if src_min.x > dst_max.x || src_max.x < dst_min.x {
            return Self::default();
        }
        let min_x = if src_min.x > dst_min.x { src_min.x } else { dst_min.x };
        let max_x = if src_max.x < dst_max.x { src_max.x } else { dst_max.x };

        if src_min.y > dst_max.y || src_max.y < dst_min.y {
            return Self::default();
        }
        let min_y = if src_min.y > dst_min.y { src_min.y } else { dst_min.y };
        let max_y = if src_max.y < dst_max.y { src_max.y } else { dst_max.y };

        if src_min.z > dst_max.z || src_max.z < dst_min.z {
            return Self::default();
        }
        let min_z = if src_min.z > dst_min.z { src_min.z } else { dst_min.z };
        let max_z = if src_max.z < dst_max.z { src_max.z } else { dst_max.z };

        let min = Vector3::new(min_x, min_y, min_z);
        let max = Vector3::new(max_x, max_y, max_z);
        Self::new(min, max - min)
    }

    /// Slab test. Returns the intersection point and the face normal.
    /// `inside` is true when the ray starts inside the box, in which case the
    /// returned point is the entry point *behind* the ray origin.
    pub fn find_intersects_ray(
        &self,
        from: Vector3,
        dir: Vector3,
    ) -> Option<RayHit> {
        let end = self.position + self.size;
        let mut tmin = -1e20f32;
        let mut tmax = 1e20f32;
        let mut axis = 0usize;

        for i in 0..3 {
            if dir[i] == 0.0 {
                if from[i] < self.position[i] || from[i] > end[i] {
                    return None;
                }
            } else {
                let mut t1 = (self.position[i] - from[i]) / dir[i];
                let mut t2 = (end[i] - from[i]) / dir[i];
                if t1 > t2 {
                    std::mem::swap(&mut t1, &mut t2);
                }
                if t1 >= tmin {
                    tmin = t1;
                    axis = i;
                }
                if t2 < tmax {
                    if t2 < 0.0 {
                        return None;
                    }
                    tmax = t2;
                }
                if tmin > tmax {
                    return None;
                }
            }
        }

        let inside = tmin < 0.0;
        let mut point = from + dir * tmin;
        // Keep the point exactly on the box border on the relevant axis.
        point[axis] = if dir[axis] >= 0.0 {
            self.position[axis]
        } else {
            end[axis]
        };

        let mut normal = Vector3::ZERO;
        normal[axis] = if dir[axis] >= 0.0 { -1.0 } else { 1.0 };

        Some(RayHit { inside, point, normal, t: tmin })
    }

    #[inline]
    pub fn intersects_ray(&self, from: Vector3, dir: Vector3) -> bool {
        self.find_intersects_ray(from, dir).is_some()
    }

    /// Segment/box intersection; returns `(t, point, normal)`.
    pub fn intersects_segment(&self, from: Vector3, to: Vector3) -> Option<SegmentHit> {
        let mut min = 0.0f32;
        let mut max = 1.0f32;
        let mut axis = 0usize;
        let mut sign = 0.0f32;

        for i in 0..3 {
            let seg_from = from[i];
            let seg_to = to[i];
            let box_begin = self.position[i];
            let box_end = box_begin + self.size[i];
            let (cmin, cmax, csign);

            if seg_from < seg_to {
                if seg_from > box_end || seg_to < box_begin {
                    return None;
                }
                let length = seg_to - seg_from;
                cmin = if seg_from < box_begin {
                    (box_begin - seg_from) / length
                } else {
                    0.0
                };
                cmax = if seg_to > box_end {
                    (box_end - seg_from) / length
                } else {
                    1.0
                };
                csign = -1.0;
            } else {
                if seg_to > box_end || seg_from < box_begin {
                    return None;
                }
                let length = seg_to - seg_from;
                cmin = if seg_from > box_end {
                    (box_end - seg_from) / length
                } else {
                    0.0
                };
                cmax = if seg_to < box_begin {
                    (box_begin - seg_from) / length
                } else {
                    1.0
                };
                csign = 1.0;
            }

            if cmin > min {
                min = cmin;
                axis = i;
                sign = csign;
            }
            if cmax < max {
                max = cmax;
            }
            if max < min {
                return None;
            }
        }

        let rel = to - from;
        let mut normal = Vector3::ZERO;
        normal[axis] = sign;
        Some(SegmentHit { t: min, point: from + rel * min, normal })
    }

    /// Smits' branchless ray/box test, used by the lightmapper and GI probes.
    #[allow(clippy::many_single_char_names)]
    pub fn smits_intersect_ray(&self, from: Vector3, dir: Vector3, t0: f32, t1: f32) -> bool {
        let divx = 1.0 / dir.x;
        let divy = 1.0 / dir.y;
        let divz = 1.0 / dir.z;

        let upbound = self.position + self.size;
        let (mut tmin, mut tmax, tymin, tymax, tzmin, tzmax);

        if dir.x >= 0.0 {
            tmin = (self.position.x - from.x) * divx;
            tmax = (upbound.x - from.x) * divx;
        } else {
            tmin = (upbound.x - from.x) * divx;
            tmax = (self.position.x - from.x) * divx;
        }
        if dir.y >= 0.0 {
            tymin = (self.position.y - from.y) * divy;
            tymax = (upbound.y - from.y) * divy;
        } else {
            tymin = (upbound.y - from.y) * divy;
            tymax = (self.position.y - from.y) * divy;
        }
        if tmin > tymax || tymin > tmax {
            return false;
        }
        if tymin > tmin {
            tmin = tymin;
        }
        if tymax < tmax {
            tmax = tymax;
        }
        if dir.z >= 0.0 {
            tzmin = (self.position.z - from.z) * divz;
            tzmax = (upbound.z - from.z) * divz;
        } else {
            tzmin = (upbound.z - from.z) * divz;
            tzmax = (self.position.z - from.z) * divz;
        }
        if tmin > tzmax || tzmin > tmax {
            return false;
        }
        if tzmin > tmin {
            tmin = tzmin;
        }
        if tzmax < tmax {
            tmax = tzmax;
        }
        tmin < t1 && tmax > t0
    }

    pub fn intersects_plane(&self, plane: &Plane) -> bool {
        let mut over = false;
        let mut under = false;
        for i in 0..8 {
            if plane.distance_to(self.get_endpoint(i)) > 0.0 {
                over = true;
            } else {
                under = true;
            }
        }
        under && over
    }

    #[inline]
    pub fn has_point(&self, point: Vector3) -> bool {
        if point.x < self.position.x
            || point.y < self.position.y
            || point.z < self.position.z
        {
            return false;
        }
        if point.x > self.position.x + self.size.x
            || point.y > self.position.y + self.size.y
            || point.z > self.position.z + self.size.z
        {
            return false;
        }
        true
    }

    #[inline]
    pub fn get_support(&self, dir: Vector3) -> Vector3 {
        let mut support = self.position;
        if dir.x > 0.0 {
            support.x += self.size.x;
        }
        if dir.y > 0.0 {
            support.y += self.size.y;
        }
        if dir.z > 0.0 {
            support.z += self.size.z;
        }
        support
    }

    #[inline]
    pub fn get_longest_axis_size(&self) -> f32 {
        let mut max_size = self.size.x;
        if self.size.y > max_size {
            max_size = self.size.y;
        }
        if self.size.z > max_size {
            max_size = self.size.z;
        }
        max_size
    }

    #[inline]
    pub fn get_shortest_axis_size(&self) -> f32 {
        let mut min_size = self.size.x;
        if self.size.y < min_size {
            min_size = self.size.y;
        }
        if self.size.z < min_size {
            min_size = self.size.z;
        }
        min_size
    }

    pub fn get_longest_axis(&self) -> Vector3 {
        let mut axis = Vector3::RIGHT;
        let mut max_size = self.size.x;
        if self.size.y > max_size {
            axis = Vector3::UP;
            max_size = self.size.y;
        }
        if self.size.z > max_size {
            axis = Vector3::BACK;
        }
        axis
    }

    pub fn get_longest_axis_index(&self) -> usize {
        let mut axis = 0;
        let mut max_size = self.size.x;
        if self.size.y > max_size {
            axis = 1;
            max_size = self.size.y;
        }
        if self.size.z > max_size {
            axis = 2;
        }
        axis
    }

    pub fn get_shortest_axis(&self) -> Vector3 {
        let mut axis = Vector3::RIGHT;
        let mut min_size = self.size.x;
        if self.size.y < min_size {
            axis = Vector3::UP;
            min_size = self.size.y;
        }
        if self.size.z < min_size {
            axis = Vector3::BACK;
        }
        axis
    }

    pub fn get_shortest_axis_index(&self) -> usize {
        let mut axis = 0;
        let mut min_size = self.size.x;
        if self.size.y < min_size {
            axis = 1;
            min_size = self.size.y;
        }
        if self.size.z < min_size {
            axis = 2;
        }
        axis
    }

    pub fn grow(&self, by: f32) -> Self {
        let mut aabb = *self;
        aabb.grow_by(by);
        aabb
    }

    #[inline]
    pub fn grow_by(&mut self, amount: f32) {
        self.position.x -= amount;
        self.position.y -= amount;
        self.position.z -= amount;
        self.size.x += 2.0 * amount;
        self.size.y += 2.0 * amount;
        self.size.z += 2.0 * amount;
    }

    #[inline]
    pub fn get_endpoint(&self, point: usize) -> Vector3 {
        match point {
            0 => Vector3::new(self.position.x, self.position.y, self.position.z),
            1 => Vector3::new(self.position.x, self.position.y, self.position.z + self.size.z),
            2 => Vector3::new(self.position.x, self.position.y + self.size.y, self.position.z),
            3 => Vector3::new(self.position.x, self.position.y + self.size.y, self.position.z + self.size.z),
            4 => Vector3::new(self.position.x + self.size.x, self.position.y, self.position.z),
            5 => Vector3::new(self.position.x + self.size.x, self.position.y, self.position.z + self.size.z),
            6 => Vector3::new(self.position.x + self.size.x, self.position.y + self.size.y, self.position.z),
            _ => Vector3::new(self.position.x + self.size.x, self.position.y + self.size.y, self.position.z + self.size.z),
        }
    }

    pub fn get_edge(&self, edge: usize) -> (Vector3, Vector3) {
        let p = self.position;
        let s = self.size;
        match edge {
            0 => (Vector3::new(p.x + s.x, p.y, p.z), Vector3::new(p.x, p.y, p.z)),
            1 => (Vector3::new(p.x + s.x, p.y, p.z + s.z), Vector3::new(p.x + s.x, p.y, p.z)),
            2 => (Vector3::new(p.x, p.y, p.z + s.z), Vector3::new(p.x + s.x, p.y, p.z + s.z)),
            3 => (Vector3::new(p.x, p.y, p.z), Vector3::new(p.x, p.y, p.z + s.z)),
            4 => (Vector3::new(p.x, p.y + s.y, p.z), Vector3::new(p.x + s.x, p.y + s.y, p.z)),
            5 => (Vector3::new(p.x + s.x, p.y + s.y, p.z), Vector3::new(p.x + s.x, p.y + s.y, p.z + s.z)),
            6 => (Vector3::new(p.x + s.x, p.y + s.y, p.z + s.z), Vector3::new(p.x, p.y + s.y, p.z + s.z)),
            7 => (Vector3::new(p.x, p.y + s.y, p.z + s.z), Vector3::new(p.x, p.y + s.y, p.z)),
            8 => (Vector3::new(p.x, p.y, p.z + s.z), Vector3::new(p.x, p.y + s.y, p.z + s.z)),
            9 => (Vector3::new(p.x, p.y, p.z), Vector3::new(p.x, p.y + s.y, p.z)),
            10 => (Vector3::new(p.x + s.x, p.y, p.z), Vector3::new(p.x + s.x, p.y + s.y, p.z)),
            _ => (Vector3::new(p.x + s.x, p.y, p.z + s.z), Vector3::new(p.x + s.x, p.y + s.y, p.z + s.z)),
        }
    }

    pub fn expand(&self, point: Vector3) -> Self {
        let mut aabb = *self;
        aabb.expand_to(point);
        aabb
    }

    #[inline]
    pub fn expand_to(&mut self, point: Vector3) {
        let mut begin = self.position;
        let mut end = self.position + self.size;

        if point.x < begin.x {
            begin.x = point.x;
        }
        if point.y < begin.y {
            begin.y = point.y;
        }
        if point.z < begin.z {
            begin.z = point.z;
        }

        if point.x > end.x {
            end.x = point.x;
        }
        if point.y > end.y {
            end.y = point.y;
        }
        if point.z > end.z {
            end.z = point.z;
        }

        self.position = begin;
        self.size = end - begin;
    }

    /// Projection range of the box onto a plane normal axis (frustum culling).
    #[inline]
    pub fn project_range_in_plane(&self, plane: &Plane) -> (f32, f32) {
        let half_extents = Vector3::new(self.size.x * 0.5, self.size.y * 0.5, self.size.z * 0.5);
        let center = Vector3::new(
            self.position.x + half_extents.x,
            self.position.y + half_extents.y,
            self.position.z + half_extents.z,
        );
        let length = plane.normal.abs().dot(half_extents);
        let distance = plane.distance_to(center);
        (distance - length, distance + length)
    }

    /// SAT-style test of the box against a convex shape defined by planes and points.
    pub fn intersects_convex_shape(&self, planes: &[Plane], points: &[Vector3]) -> bool {
        let half_extents = self.size * 0.5;
        let ofs = self.position + half_extents;

        for p in planes {
            let point = Vector3::new(
                if p.normal.x > 0.0 { -half_extents.x } else { half_extents.x },
                if p.normal.y > 0.0 { -half_extents.y } else { half_extents.y },
                if p.normal.z > 0.0 { -half_extents.z } else { half_extents.z },
            ) + ofs;
            if p.is_point_over(point) {
                return false;
            }
        }

        // Ensure the shape's points are not fully separated on any axis.
        for k in 0..3 {
            let mut positive = 0;
            let mut negative = 0;
            for point in points {
                if point[k] > ofs[k] + half_extents[k] {
                    positive += 1;
                }
                if point[k] < ofs[k] - half_extents[k] {
                    negative += 1;
                }
            }
            if negative == points.len() || positive == points.len() {
                return false;
            }
        }

        true
    }

    pub fn inside_convex_shape(&self, planes: &[Plane]) -> bool {
        let half_extents = self.size * 0.5;
        let ofs = self.position + half_extents;

        for p in planes {
            let point = Vector3::new(
                if p.normal.x > 0.0 { half_extents.x } else { -half_extents.x },
                if p.normal.y > 0.0 { half_extents.y } else { -half_extents.y },
                if p.normal.z > 0.0 { half_extents.z } else { -half_extents.z },
            ) + ofs;
            if p.is_point_over(point) {
                return false;
            }
        }

        true
    }

    #[inline]
    pub fn quantize(&mut self, unit: f32) {
        self.size += self.position;

        self.position.x -= funcs::fposmodp(self.position.x, unit);
        self.position.y -= funcs::fposmodp(self.position.y, unit);
        self.position.z -= funcs::fposmodp(self.position.z, unit);

        self.size.x -= funcs::fposmodp(self.size.x, unit);
        self.size.y -= funcs::fposmodp(self.size.y, unit);
        self.size.z -= funcs::fposmodp(self.size.z, unit);

        self.size.x += unit;
        self.size.y += unit;
        self.size.z += unit;

        self.size -= self.position;
    }

    #[inline]
    pub fn quantized(&self, unit: f32) -> Self {
        let mut ret = *self;
        ret.quantize(unit);
        ret
    }

    #[inline]
    pub fn hash(&self) -> u32 {
        let mut h = hashfuncs::hash_murmur3_one_float(self.position.x, hashfuncs::HASH_MURMUR3_SEED);
        h = hashfuncs::hash_murmur3_one_float(self.position.y, h);
        h = hashfuncs::hash_murmur3_one_float(self.position.z, h);
        h = hashfuncs::hash_murmur3_one_float(self.size.x, h);
        h = hashfuncs::hash_murmur3_one_float(self.size.y, h);
        h = hashfuncs::hash_murmur3_one_float(self.size.z, h);
        hashfuncs::hash_fmix32(h)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct RayHit {
    pub inside: bool,
    pub point: Vector3,
    pub normal: Vector3,
    pub t: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct SegmentHit {
    pub t: f32,
    pub point: Vector3,
    pub normal: Vector3,
}

impl std::ops::Neg for Aabb {
    type Output = Aabb;
    #[inline]
    fn neg(self) -> Aabb {
        Aabb::new(-self.position, -self.size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unit() -> Aabb {
        Aabb::new(Vector3::ZERO, Vector3::ONE)
    }

    #[test]
    fn overlap_rules() {
        let a = unit();
        let b = Aabb::new(Vector3::new(1.0, 0.0, 0.0), Vector3::ONE);
        // Touching face: not an intersection, but inclusive yes.
        assert!(!a.intersects(&b));
        assert!(a.intersects_inclusive(&b));
        assert!(a.intersects(&Aabb::new(Vector3::new(0.5, 0.5, 0.5), Vector3::ONE)));
        assert!(a.encloses(&Aabb::new(Vector3::new(0.1, 0.1, 0.1), Vector3::splat(0.5))));
        assert!(!a.encloses(&b));
    }

    #[test]
    fn merge_and_intersection() {
        let a = Aabb::new(Vector3::ZERO, Vector3::ONE);
        let b = Aabb::new(Vector3::new(2.0, 0.0, 0.0), Vector3::ONE);
        let m = a.merge(&b);
        assert_eq!(m.position, Vector3::ZERO);
        assert_eq!(m.size, Vector3::new(3.0, 1.0, 1.0));
        assert_eq!(a.intersection(&b), Aabb::default());

        let c = Aabb::new(Vector3::new(0.5, 0.5, 0.5), Vector3::ONE);
        let i = a.intersection(&c);
        assert_eq!(i.position, Vector3::new(0.5, 0.5, 0.5));
        assert_eq!(i.size, Vector3::new(0.5, 0.5, 0.5));
    }

    #[test]
    fn ray_hits_and_normals() {
        let a = unit();
        let hit = a
            .find_intersects_ray(Vector3::new(0.5, 0.5, -3.0), Vector3::new(0.0, 0.0, 1.0))
            .unwrap();
        assert!(!hit.inside);
        assert!((hit.t - 3.0).abs() < 1e-5);
        assert_eq!(hit.normal, Vector3::new(0.0, 0.0, -1.0));

        let inside = a
            .find_intersects_ray(Vector3::new(0.5, 0.5, 0.5), Vector3::new(0.0, 0.0, 1.0))
            .unwrap();
        assert!(inside.inside);

        assert!(a.find_intersects_ray(Vector3::new(-2.0, 0.5, 0.5), Vector3::new(0.0, 0.0, 1.0)).is_none());
    }

    #[test]
    fn segment_and_plane() {
        let a = unit();
        let hit = a
            .intersects_segment(Vector3::new(0.5, 0.5, -1.0), Vector3::new(0.5, 0.5, 1.0))
            .unwrap();
        assert!((hit.t - 0.5).abs() < 1e-5);
        assert_eq!(hit.normal, Vector3::new(0.0, 0.0, -1.0));

        let p = Plane::from_normal_point(Vector3::UP, Vector3::new(0.0, 0.5, 0.0));
        assert!(a.intersects_plane(&p));
        let outside = Plane::from_normal_point(Vector3::UP, Vector3::new(0.0, 5.0, 0.0));
        assert!(!a.intersects_plane(&outside));
    }

    #[test]
    fn axis_queries() {
        let a = Aabb::new(Vector3::ZERO, Vector3::new(1.0, 5.0, 3.0));
        assert_eq!(a.get_longest_axis_index(), 1);
        assert_eq!(a.get_shortest_axis_index(), 0);
        assert_eq!(a.get_longest_axis_size(), 5.0);
        assert_eq!(a.get_shortest_axis_size(), 1.0);
        assert_eq!(a.get_volume(), 15.0);
    }
}
