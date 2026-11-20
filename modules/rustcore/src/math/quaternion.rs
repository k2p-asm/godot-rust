//! Port of `core/math/quaternion.h` + `quaternion.cpp` (rotation math only).
//!
//! Pending: `get_euler` (depends on `Basis::get_euler`), `log`/`exp` and the
//! spherical cubic interpolation helpers, which depend on Basis rotation
//! decomposition.

use crate::math::funcs;
use crate::math::vector3::Vector3;

pub const UNIT_EPSILON: f32 = 0.001;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Quaternion {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Default for Quaternion {
    fn default() -> Self {
        Self { x: 0.0, y: 0.0, z: 0.0, w: 1.0 }
    }
}

impl Quaternion {
    pub const IDENTITY: Quaternion = Quaternion { x: 0.0, y: 0.0, z: 0.0, w: 1.0 };

    #[inline]
    pub const fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { x, y, z, w }
    }

    /// Axis-angle constructor, identical to `Quaternion(const Vector3 &, real_t)`.
    pub fn from_axis_angle(axis: Vector3, angle: f32) -> Self {
        let d = axis.length();
        if d == 0.0 {
            Self::new(0.0, 0.0, 0.0, 0.0)
        } else {
            let sin_angle = (angle * 0.5).sin();
            let cos_angle = (angle * 0.5).cos();
            let s = sin_angle / d;
            Self::new(axis.x * s, axis.y * s, axis.z * s, cos_angle)
        }
    }

    /// `R = Y(a1).X(a2).Z(a3)` convention, as used by the engine's Euler angles.
    pub fn from_euler(euler: Vector3) -> Self {
        let half_a1 = euler.y * 0.5;
        let half_a2 = euler.x * 0.5;
        let half_a3 = euler.z * 0.5;

        let cos_a1 = half_a1.cos();
        let sin_a1 = half_a1.sin();
        let cos_a2 = half_a2.cos();
        let sin_a2 = half_a2.sin();
        let cos_a3 = half_a3.cos();
        let sin_a3 = half_a3.sin();

        Self::new(
            sin_a1 * cos_a2 * sin_a3 + cos_a1 * sin_a2 * cos_a3,
            sin_a1 * cos_a2 * cos_a3 - cos_a1 * sin_a2 * sin_a3,
            -sin_a1 * sin_a2 * cos_a3 + cos_a1 * cos_a2 * sin_a3,
            sin_a1 * sin_a2 * sin_a3 + cos_a1 * cos_a2 * cos_a3,
        )
    }

    #[inline]
    pub fn dot(&self, other: &Quaternion) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z + self.w * other.w
    }

    #[inline]
    pub fn length_squared(&self) -> f32 {
        self.dot(self)
    }

    #[inline]
    pub fn length(&self) -> f32 {
        self.length_squared().sqrt()
    }

    #[inline]
    pub fn normalize(&mut self) {
        let l = self.length();
        self.x /= l;
        self.y /= l;
        self.z /= l;
        self.w /= l;
    }

    #[inline]
    pub fn normalized(&self) -> Self {
        let mut q = *self;
        q.normalize();
        q
    }

    #[inline]
    pub fn is_normalized(&self) -> bool {
        funcs::is_equal_approx_tol(self.length_squared(), 1.0, UNIT_EPSILON)
    }

    #[inline]
    pub fn inverse(&self) -> Self {
        Self::new(-self.x, -self.y, -self.z, self.w)
    }

    /// Angle between two rotations, in radians.
    #[inline]
    pub fn angle_to(&self, to: &Quaternion) -> f32 {
        let d = self.dot(to);
        funcs::acos(d * d * 2.0 - 1.0)
    }

    /// Shortest-path spherical interpolation (`slerp`, not `slerpni`).
    pub fn slerp(&self, to: &Quaternion, weight: f32) -> Self {
        let mut cosom = self.dot(to);
        let to1 = if cosom < 0.0 {
            cosom = -cosom;
            -*to
        } else {
            *to
        };

        let (scale0, scale1);
        if (1.0 - cosom) > funcs::CMP_EPSILON as f32 {
            let omega = cosom.acos();
            let sinom = omega.sin();
            scale0 = ((1.0 - weight) * omega).sin() / sinom;
            scale1 = (weight * omega).sin() / sinom;
        } else {
            scale0 = 1.0 - weight;
            scale1 = weight;
        }

        Self::new(
            scale0 * self.x + scale1 * to1.x,
            scale0 * self.y + scale1 * to1.y,
            scale0 * self.z + scale1 * to1.z,
            scale0 * self.w + scale1 * to1.w,
        )
    }

    #[inline]
    pub fn get_axis(&self) -> Vector3 {
        if self.w.abs() > 1.0 - funcs::CMP_EPSILON as f32 {
            return Vector3::new(self.x, self.y, self.z);
        }
        let r = 1.0 / (1.0 - self.w * self.w).sqrt();
        Vector3::new(self.x * r, self.y * r, self.z * r)
    }

    #[inline]
    pub fn get_angle(&self) -> f32 {
        2.0 * funcs::acos(self.w)
    }

    #[inline]
    pub fn xform(&self, v: Vector3) -> Vector3 {
        let u = Vector3::new(self.x, self.y, self.z);
        let uv = u.cross(v);
        v + ((uv * self.w) + u.cross(uv)) * 2.0
    }

    #[inline]
    pub fn is_equal_approx(&self, other: &Quaternion) -> bool {
        funcs::is_equal_approx(self.x, other.x)
            && funcs::is_equal_approx(self.y, other.y)
            && funcs::is_equal_approx(self.z, other.z)
            && funcs::is_equal_approx(self.w, other.w)
    }

    #[inline]
    pub fn is_same(&self, other: &Quaternion) -> bool {
        funcs::is_same(self.x, other.x)
            && funcs::is_same(self.y, other.y)
            && funcs::is_same(self.z, other.z)
            && funcs::is_same(self.w, other.w)
    }

    #[inline]
    pub fn is_finite(&self) -> bool {
        self.x.is_finite() && self.y.is_finite() && self.z.is_finite() && self.w.is_finite()
    }
}

impl std::ops::Mul for Quaternion {
    type Output = Quaternion;
    #[inline]
    fn mul(self, q: Quaternion) -> Quaternion {
        Quaternion::new(
            self.w * q.x + self.x * q.w + self.y * q.z - self.z * q.y,
            self.w * q.y + self.y * q.w + self.z * q.x - self.x * q.z,
            self.w * q.z + self.z * q.w + self.x * q.y - self.y * q.x,
            self.w * q.w - self.x * q.x - self.y * q.y - self.z * q.z,
        )
    }
}

impl std::ops::Mul<f32> for Quaternion {
    type Output = Quaternion;
    #[inline]
    fn mul(self, s: f32) -> Quaternion {
        Quaternion::new(self.x * s, self.y * s, self.z * s, self.w * s)
    }
}

impl std::ops::Div<f32> for Quaternion {
    type Output = Quaternion;
    #[inline]
    fn div(self, s: f32) -> Quaternion {
        Quaternion::new(self.x / s, self.y / s, self.z / s, self.w / s)
    }
}

impl std::ops::Add for Quaternion {
    type Output = Quaternion;
    #[inline]
    fn add(self, q: Quaternion) -> Quaternion {
        Quaternion::new(self.x + q.x, self.y + q.y, self.z + q.z, self.w + q.w)
    }
}

impl std::ops::Sub for Quaternion {
    type Output = Quaternion;
    #[inline]
    fn sub(self, q: Quaternion) -> Quaternion {
        Quaternion::new(self.x - q.x, self.y - q.y, self.z - q.z, self.w - q.w)
    }
}

impl std::ops::Neg for Quaternion {
    type Output = Quaternion;
    #[inline]
    fn neg(self) -> Quaternion {
        Quaternion::new(-self.x, -self.y, -self.z, -self.w)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn axis_angle_rotation() {
        let q = Quaternion::from_axis_angle(Vector3::UP, funcs::PI as f32 / 2.0);
        let v = q.xform(Vector3::RIGHT);
        assert!(v.is_equal_approx(Vector3::new(0.0, 0.0, -1.0)), "{v:?}");
        assert!(q.is_normalized());
    }

    #[test]
    fn euler_matches_axis_order() {
        // Pure Y rotation must equal the axis-angle form.
        let e = Vector3::new(0.0, 1.2, 0.0);
        let q1 = Quaternion::from_euler(e);
        let q2 = Quaternion::from_axis_angle(Vector3::UP, 1.2);
        assert!(q1.is_equal_approx(&q2), "{q1:?} vs {q2:?}");
    }

    #[test]
    fn slerp_endpoints() {
        let a = Quaternion::IDENTITY;
        let b = Quaternion::from_axis_angle(Vector3::UP, 1.0);
        assert!(a.slerp(&b, 0.0).is_equal_approx(&a));
        assert!(a.slerp(&b, 1.0).is_equal_approx(&b));
        assert!((a.angle_to(&b) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn inverse_undoes_rotation() {
        let q = Quaternion::from_axis_angle(Vector3::new(1.0, 2.0, 3.0).normalized(), 0.9);
        let v = Vector3::new(0.4, -1.2, 2.5);
        let back = q.inverse().xform(q.xform(v));
        assert!(back.is_equal_approx(v), "{back:?}");
    }
}
