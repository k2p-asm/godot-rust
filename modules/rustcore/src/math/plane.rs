//! Port of `core/math/plane.h` + `plane.cpp`.

use crate::math::funcs;
use crate::math::vector3::Vector3;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Plane {
    pub normal: Vector3,
    pub d: f32,
}

impl Plane {
    pub const PLANE_YZ: Plane = Plane { normal: Vector3::RIGHT, d: 0.0 };
    pub const PLANE_XZ: Plane = Plane { normal: Vector3::UP, d: 0.0 };
    pub const PLANE_XY: Plane = Plane { normal: Vector3::BACK, d: 0.0 };

    #[inline]
    pub const fn new(a: f32, b: f32, c: f32, d: f32) -> Self {
        Self { normal: Vector3::new(a, b, c), d }
    }

    /// From a normal and a point on the plane.
    #[inline]
    pub fn from_normal_point(normal: Vector3, point: Vector3) -> Self {
        Self { normal, d: normal.dot(point) }
    }

    /// From three points; the winding matches the C++ `CLOCKWISE` default.
    pub fn from_points(p1: Vector3, p2: Vector3, p3: Vector3) -> Self {
        let mut normal = (p1 - p3).cross(p1 - p2);
        normal.normalize();
        Self { normal, d: normal.dot(p1) }
    }

    #[inline]
    pub fn set_normal(&mut self, normal: Vector3) {
        self.normal = normal;
    }

    #[inline]
    pub fn get_normal(&self) -> Vector3 {
        self.normal
    }

    #[inline]
    pub fn zero(&mut self) {
        self.normal.zero();
        self.d = 0.0;
    }

    pub fn normalize(&mut self) {
        let l = self.normal.length();
        if l == 0.0 {
            self.zero();
            return;
        }
        self.normal /= l;
        self.d /= l;
    }

    pub fn normalized(self) -> Self {
        let mut p = self;
        p.normalize();
        p
    }

    /// Center of the plane, the point where the normal line passes through.
    #[inline]
    pub fn get_center(&self) -> Vector3 {
        self.normal * self.d
    }

    pub fn get_any_perpendicular_normal(&self) -> Vector3 {
        let p1 = Vector3::RIGHT;
        let p2 = Vector3::UP;
        let mut p = if self.normal.dot(p1).abs() > 0.99 {
            p2
        } else {
            p1
        };
        p -= self.normal * self.normal.dot(p);
        p.normalize();
        p
    }

    #[inline]
    pub fn is_point_over(&self, point: Vector3) -> bool {
        self.normal.dot(point) > self.d
    }

    #[inline]
    pub fn distance_to(&self, point: Vector3) -> f32 {
        self.normal.dot(point) - self.d
    }

    #[inline]
    pub fn has_point(&self, point: Vector3, tolerance: f32) -> bool {
        (self.normal.dot(point) - self.d).abs() <= tolerance
    }

    #[inline]
    pub fn project(&self, point: Vector3) -> Vector3 {
        point - self.normal * self.distance_to(point)
    }

    /// Intersection of three planes. Returns `None` when they are parallel.
    pub fn intersect_3(&self, p1: &Plane, p2: &Plane) -> Option<Vector3> {
        let n0 = self.normal;
        let n1 = p1.normal;
        let n2 = p2.normal;

        let denom = n0.cross(n1).dot(n2);
        if funcs::is_zero_approx(denom) {
            return None;
        }
        Some(
            ((n1.cross(n2) * self.d) + (n2.cross(n0) * p1.d) + (n0.cross(n1) * p2.d)) / denom,
        )
    }

    /// `true` when the ray hits the plane in front of `from`.
    pub fn intersects_ray(&self, from: Vector3, dir: Vector3) -> Option<Vector3> {
        let den = self.normal.dot(dir);
        if funcs::is_zero_approx(den) {
            return None;
        }
        let mut dist = (self.normal.dot(from) - self.d) / den;
        if dist > funcs::CMP_EPSILON as f32 {
            // A ray does not exist behind its origin.
            return None;
        }
        dist = -dist;
        Some(from + dir * dist)
    }

    pub fn intersects_segment(&self, begin: Vector3, end: Vector3) -> Option<Vector3> {
        let segment = begin - end;
        let den = self.normal.dot(segment);
        if funcs::is_zero_approx(den) {
            return None;
        }
        let mut dist = (self.normal.dot(begin) - self.d) / den;
        let eps = funcs::CMP_EPSILON as f32;
        if dist < -eps || dist > 1.0 + eps {
            return None;
        }
        dist = -dist;
        Some(begin + segment * dist)
    }

    #[inline]
    pub fn is_equal_approx(&self, other: &Plane) -> bool {
        self.normal.is_equal_approx(other.normal) && funcs::is_equal_approx(self.d, other.d)
    }

    /// Approximate equality ignoring the sign of the plane.
    #[inline]
    pub fn is_equal_approx_any_side(&self, other: &Plane) -> bool {
        (self.normal.is_equal_approx(other.normal) && funcs::is_equal_approx(self.d, other.d))
            || (self.normal.is_equal_approx(-other.normal)
                && funcs::is_equal_approx(self.d, -other.d))
    }

    #[inline]
    pub fn is_same(&self, other: &Plane) -> bool {
        self.normal.is_same(other.normal) && funcs::is_same(self.d, other.d)
    }

    #[inline]
    pub fn is_finite(&self) -> bool {
        self.normal.is_finite() && self.d.is_finite()
    }
}

impl std::ops::Neg for Plane {
    type Output = Plane;
    #[inline]
    fn neg(self) -> Plane {
        Plane { normal: -self.normal, d: -self.d }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn distance_and_projection() {
        let p = Plane::from_normal_point(Vector3::UP, Vector3::new(0.0, 5.0, 0.0));
        assert_eq!(p.d, 5.0);
        assert!((p.distance_to(Vector3::new(1.0, 7.0, 2.0)) - 2.0).abs() < 1e-6);
        assert!(p.project(Vector3::new(1.0, 7.0, 2.0)).is_equal_approx(Vector3::new(1.0, 5.0, 2.0)));
        assert!(p.has_point(Vector3::new(3.0, 5.0, -1.0), 1e-5));
    }

    #[test]
    fn three_planes_intersect() {
        let a = Plane::PLANE_YZ;
        let b = Plane::PLANE_XZ;
        let c = Plane::PLANE_XY;
        let hit = a.intersect_3(&b, &c).unwrap();
        assert!(hit.is_zero_approx(), "{hit:?}");
    }

    #[test]
    fn ray_and_segment() {
        let p = Plane::from_normal_point(Vector3::UP, Vector3::new(0.0, 1.0, 0.0));
        // Ray going down from above hits at (0,1,0).
        let hit = p.intersects_ray(Vector3::new(0.0, 3.0, 0.0), Vector3::new(0.0, -1.0, 0.0)).unwrap();
        assert!(hit.is_equal_approx(Vector3::new(0.0, 1.0, 0.0)), "{hit:?}");
        // Ray going up from below never hits (behind the origin).
        assert!(p.intersects_ray(Vector3::new(0.0, 0.0, 0.0), Vector3::new(0.0, -1.0, 0.0)).is_none());
        // Segment crossing the plane.
        let s = p
            .intersects_segment(Vector3::new(0.0, 2.0, 0.0), Vector3::new(0.0, 0.0, 0.0))
            .unwrap();
        assert!(s.is_equal_approx(Vector3::new(0.0, 1.0, 0.0)), "{s:?}");
    }
}
