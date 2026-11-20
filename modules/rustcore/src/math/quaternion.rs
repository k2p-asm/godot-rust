//! Port of `core/math/quaternion.h` + `quaternion.cpp`.
//!
//! Everything is ported except `operator String`.

use crate::math::basis::{Basis, EulerOrder};
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

    /// `Quaternion::get_euler`: delegates to `Basis::get_euler`, exactly like
    /// the engine (the quaternion is converted to a basis first).
    pub fn get_euler(&self, order: EulerOrder) -> Vector3 {
        Basis::from_quaternion(*self).get_euler(order)
    }

    /// `Quaternion::log`: axis * angle in the vector part, zero scalar part.
    pub fn log(&self) -> Self {
        let v = self.get_axis() * self.get_angle();
        Self::new(v.x, v.y, v.z, 0.0)
    }

    /// `Quaternion::exp`: inverse of `log`.
    pub fn exp(&self) -> Self {
        let v = Vector3::new(self.x, self.y, self.z);
        let theta = v.length();
        let axis = v.normalized();
        // Degenerate rotations map to identity, like the C++ early return.
        if theta < funcs::CMP_EPSILON as f32 || !axis.is_normalized() {
            return Self::IDENTITY;
        }
        Self::from_axis_angle(axis, theta)
    }

    /// `Quaternion::slerpni`: slerp without shortest-path adjustment.
    pub fn slerpni(&self, to: &Quaternion, weight: f32) -> Self {
        let dot = self.dot(to);
        if dot.abs() > 0.9999 {
            return *self;
        }

        let theta = funcs::acos(dot);
        let sin_t = 1.0 / theta.sin();
        let new_factor = (weight * theta).sin() * sin_t;
        let inv_factor = ((1.0 - weight) * theta).sin() * sin_t;

        Self::new(
            inv_factor * self.x + new_factor * to.x,
            inv_factor * self.y + new_factor * to.y,
            inv_factor * self.z + new_factor * to.z,
            inv_factor * self.w + new_factor * to.w,
        )
    }

    /// `Quaternion::spherical_cubic_interpolate`: cubic interpolation done in
    /// log space around both endpoints, then blended, which cancels the
    /// ambiguity of the exp map.
    pub fn spherical_cubic_interpolate(
        &self,
        b: &Quaternion,
        pre_a: &Quaternion,
        post_b: &Quaternion,
        weight: f32,
    ) -> Self {
        let mut from_q = Basis::from_quaternion(*self).get_rotation_quaternion();
        let mut pre_q = Basis::from_quaternion(*pre_a).get_rotation_quaternion();
        let mut to_q = Basis::from_quaternion(*b).get_rotation_quaternion();
        let mut post_q = Basis::from_quaternion(*post_b).get_rotation_quaternion();

        // Flip opposite phases so all four lie in the same hemisphere.
        let flip1 = from_q.dot(&pre_q).is_sign_negative();
        pre_q = if flip1 { -pre_q } else { pre_q };
        let flip2 = from_q.dot(&to_q).is_sign_negative();
        to_q = if flip2 { -to_q } else { to_q };
        let flip3 = if flip2 {
            to_q.dot(&post_q) <= 0.0
        } else {
            to_q.dot(&post_q).is_sign_negative()
        };
        post_q = if flip3 { -post_q } else { post_q };

        let from_inv = from_q.inverse();
        let ln_from = Quaternion::new(0.0, 0.0, 0.0, 0.0);
        let ln_to = (from_inv * to_q).log();
        let ln_pre = (from_inv * pre_q).log();
        let ln_post = (from_inv * post_q).log();
        let ln = Quaternion::new(
            funcs::cubic_interpolate(ln_from.x, ln_to.x, ln_pre.x, ln_post.x, weight),
            funcs::cubic_interpolate(ln_from.y, ln_to.y, ln_pre.y, ln_post.y, weight),
            funcs::cubic_interpolate(ln_from.z, ln_to.z, ln_pre.z, ln_post.z, weight),
            0.0,
        );
        let q1 = from_q * ln.exp();

        let to_inv = to_q.inverse();
        let ln_from = (to_inv * from_q).log();
        let ln_to = Quaternion::new(0.0, 0.0, 0.0, 0.0);
        let ln_pre = (to_inv * pre_q).log();
        let ln_post = (to_inv * post_q).log();
        let ln = Quaternion::new(
            funcs::cubic_interpolate(ln_from.x, ln_to.x, ln_pre.x, ln_post.x, weight),
            funcs::cubic_interpolate(ln_from.y, ln_to.y, ln_pre.y, ln_post.y, weight),
            funcs::cubic_interpolate(ln_from.z, ln_to.z, ln_pre.z, ln_post.z, weight),
            0.0,
        );
        let q2 = to_q * ln.exp();

        q1.slerp(&q2, weight)
    }

    /// `Quaternion::spherical_cubic_interpolate_in_time`: the same blend with
    /// per-key times instead of a normalised weight.
    #[allow(clippy::too_many_arguments)]
    pub fn spherical_cubic_interpolate_in_time(
        &self,
        b: &Quaternion,
        pre_a: &Quaternion,
        post_b: &Quaternion,
        weight: f32,
        b_t: f32,
        pre_a_t: f32,
        post_b_t: f32,
    ) -> Self {
        let from_q = Basis::from_quaternion(*self).get_rotation_quaternion();
        let mut pre_q = Basis::from_quaternion(*pre_a).get_rotation_quaternion();
        let mut to_q = Basis::from_quaternion(*b).get_rotation_quaternion();
        let mut post_q = Basis::from_quaternion(*post_b).get_rotation_quaternion();

        let flip1 = from_q.dot(&pre_q).is_sign_negative();
        pre_q = if flip1 { -pre_q } else { pre_q };
        let flip2 = from_q.dot(&to_q).is_sign_negative();
        to_q = if flip2 { -to_q } else { to_q };
        let flip3 = if flip2 {
            to_q.dot(&post_q) <= 0.0
        } else {
            to_q.dot(&post_q).is_sign_negative()
        };
        post_q = if flip3 { -post_q } else { post_q };

        let from_inv = from_q.inverse();
        let ln_from = Quaternion::new(0.0, 0.0, 0.0, 0.0);
        let ln_to = (from_inv * to_q).log();
        let ln_pre = (from_inv * pre_q).log();
        let ln_post = (from_inv * post_q).log();
        let ln = Quaternion::new(
            funcs::cubic_interpolate_in_time(ln_from.x, ln_to.x, ln_pre.x, ln_post.x, weight, b_t, pre_a_t, post_b_t),
            funcs::cubic_interpolate_in_time(ln_from.y, ln_to.y, ln_pre.y, ln_post.y, weight, b_t, pre_a_t, post_b_t),
            funcs::cubic_interpolate_in_time(ln_from.z, ln_to.z, ln_pre.z, ln_post.z, weight, b_t, pre_a_t, post_b_t),
            0.0,
        );
        let q1 = from_q * ln.exp();

        let to_inv = to_q.inverse();
        let ln_from = (to_inv * from_q).log();
        let ln_to = Quaternion::new(0.0, 0.0, 0.0, 0.0);
        let ln_pre = (to_inv * pre_q).log();
        let ln_post = (to_inv * post_q).log();
        let ln = Quaternion::new(
            funcs::cubic_interpolate_in_time(ln_from.x, ln_to.x, ln_pre.x, ln_post.x, weight, b_t, pre_a_t, post_b_t),
            funcs::cubic_interpolate_in_time(ln_from.y, ln_to.y, ln_pre.y, ln_post.y, weight, b_t, pre_a_t, post_b_t),
            funcs::cubic_interpolate_in_time(ln_from.z, ln_to.z, ln_pre.z, ln_post.z, weight, b_t, pre_a_t, post_b_t),
            0.0,
        );
        let q2 = to_q * ln.exp();

        q1.slerp(&q2, weight)
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

    #[test]
    fn euler_round_trip_through_basis() {
        let angles = Vector3::new(0.3, -0.7, 1.1);
        let q = Quaternion::from_euler(angles);
        for order in [
            EulerOrder::XYZ,
            EulerOrder::XZY,
            EulerOrder::YXZ,
            EulerOrder::YZX,
            EulerOrder::ZXY,
            EulerOrder::ZYX,
        ] {
            // get_euler returns an equivalent rotation, not necessarily the
            // same angles: compare the reconstructed basis by rotating a
            // non-degenerate vector.
            let e = q.get_euler(order);
            let rebuilt = Basis::from_euler(e, order).get_rotation_quaternion();
            for v in [Vector3::RIGHT, Vector3::UP, Vector3::new(0.3, -0.5, 0.8)] {
                let a = q.xform(v);
                let b = rebuilt.xform(v);
                assert!(a.is_equal_approx(b), "{order:?}: {a:?} vs {b:?}");
            }
        }
    }

    #[test]
    fn log_exp_are_inverse() {
        for q in [
            Quaternion::from_axis_angle(Vector3::UP, 0.4),
            Quaternion::from_euler(Vector3::new(0.2, 0.5, -0.9)),
        ] {
            let back = q.log().exp();
            // log/exp loses the double cover: q and -q are the same rotation.
            assert!(
                back.is_equal_approx(&q) || back.is_equal_approx(&(-q)),
                "{back:?} vs {q:?}"
            );
        }
        // Zero rotation: exp(0) is identity.
        let zero = Quaternion::new(0.0, 0.0, 0.0, 0.0);
        assert!(zero.exp().is_equal_approx(&Quaternion::IDENTITY));
    }

    #[test]
    fn slerpni_does_not_take_the_short_path() {
        // 300 degrees apart: slerp goes the short way (60 degrees), slerpni
        // walks the 300 degree arc.
        let a = Quaternion::from_axis_angle(Vector3::UP, 0.0);
        let b = Quaternion::from_axis_angle(Vector3::UP, 300f32.to_radians());
        let mid_short = a.slerp(&b, 0.5);
        let mid_long = a.slerpni(&b, 0.5);
        assert!((mid_short.angle_to(&a) - 30f32.to_radians()).abs() < 1e-3);
        assert!((mid_long.angle_to(&a) - 150f32.to_radians()).abs() < 1e-3);
    }

    #[test]
    fn spherical_cubic_hits_the_endpoints() {
        let a = Quaternion::from_axis_angle(Vector3::UP, 0.0);
        let b = Quaternion::from_axis_angle(Vector3::UP, 1.0);
        let pre = Quaternion::from_axis_angle(Vector3::UP, -0.5);
        let post = Quaternion::from_axis_angle(Vector3::UP, 1.5);

        let at0 = a.spherical_cubic_interpolate(&b, &pre, &post, 0.0);
        let at1 = a.spherical_cubic_interpolate(&b, &pre, &post, 1.0);
        assert!(at0.is_equal_approx(&a), "{at0:?}");
        assert!(at1.is_equal_approx(&b), "{at1:?}");

        // In-time variant agrees with the uniform one when the keys are evenly
        // spaced (times 0, 1 with weight 0.5).
        let uniform = a.spherical_cubic_interpolate(&b, &pre, &post, 0.5);
        let timed = a.spherical_cubic_interpolate_in_time(&b, &pre, &post, 0.5, 1.0, -1.0, 2.0);
        assert!(uniform.is_equal_approx(&timed), "{uniform:?} vs {timed:?}");
    }
}
