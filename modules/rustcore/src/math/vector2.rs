//! Port of `core/math/vector2.h` + `vector2.cpp`.
//!
//! `real_t` is `f32` in a default Godot build (REAL_T_IS_DOUBLE is off), so the
//! port uses `f32` everywhere.

use crate::math::funcs;
use crate::templates::hashfuncs;

pub const UNIT_EPSILON: f32 = 0.001;

/// 2D vector. `#[repr(C)]` and identical layout to the C++ `Vector2`
/// (two `real_t`), so it can cross the FFI boundary directly.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vector2 {
    pub x: f32,
    pub y: f32,
}

impl Vector2 {
    pub const ZERO: Vector2 = Vector2 { x: 0.0, y: 0.0 };
    pub const ONE: Vector2 = Vector2 { x: 1.0, y: 1.0 };
    pub const INF: Vector2 = Vector2 { x: f32::INFINITY, y: f32::INFINITY };
    pub const LEFT: Vector2 = Vector2 { x: -1.0, y: 0.0 };
    pub const RIGHT: Vector2 = Vector2 { x: 1.0, y: 0.0 };
    pub const UP: Vector2 = Vector2 { x: 0.0, y: -1.0 };
    pub const DOWN: Vector2 = Vector2 { x: 0.0, y: 1.0 };

    #[inline]
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    #[inline]
    pub const fn splat(v: f32) -> Self {
        Self { x: v, y: v }
    }

    /// Axis index of the smallest component (0 = x, 1 = y), matching
    /// `min_axis_index()`: ties select Y.
    #[inline]
    pub fn min_axis_index(self) -> usize {
        if self.x < self.y {
            0
        } else {
            1
        }
    }

    /// Ties select X, exactly like `max_axis_index()`.
    #[inline]
    pub fn max_axis_index(self) -> usize {
        if self.x < self.y {
            1
        } else {
            0
        }
    }

    #[inline]
    pub fn zero(&mut self) {
        self.x = 0.0;
        self.y = 0.0;
    }

    #[inline]
    pub fn length(self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    #[inline]
    pub fn length_squared(self) -> f32 {
        self.x * self.x + self.y * self.y
    }

    pub fn normalize(&mut self) {
        if !self.is_finite() {
            *self = Self::ZERO;
            return;
        }
        let l = self.length_squared();
        if l == 0.0 {
            *self = Self::ZERO;
        } else {
            let l = l.sqrt();
            self.x /= l;
            self.y /= l;
        }
    }

    pub fn normalized(self) -> Self {
        let mut v = self;
        v.normalize();
        v
    }

    #[inline]
    pub fn is_normalized(self) -> bool {
        // Squared length avoids a sqrt and is more stringent.
        funcs::is_equal_approx_tol(self.length_squared(), 1.0, UNIT_EPSILON)
    }

    #[inline]
    pub fn distance_to(self, other: Self) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }

    #[inline]
    pub fn distance_squared_to(self, other: Self) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        dx * dx + dy * dy
    }

    #[inline]
    pub fn angle_to(self, other: Self) -> f32 {
        self.cross(other).atan2(self.dot(other))
    }

    #[inline]
    pub fn angle_to_point(self, other: Self) -> f32 {
        (other - self).angle()
    }

    #[inline]
    pub fn direction_to(self, to: Self) -> Self {
        (to - self).normalized()
    }

    #[inline]
    pub fn dot(self, other: Self) -> f32 {
        self.x * other.x + self.y * other.y
    }

    #[inline]
    pub fn cross(self, other: Self) -> f32 {
        self.x * other.y - self.y * other.x
    }

    #[inline]
    pub fn posmod(self, m: f32) -> Self {
        Self::new(funcs::fposmod(self.x, m), funcs::fposmod(self.y, m))
    }

    #[inline]
    pub fn posmodv(self, m: Self) -> Self {
        Self::new(funcs::fposmod(self.x, m.x), funcs::fposmod(self.y, m.y))
    }

    #[inline]
    pub fn project(self, to: Self) -> Self {
        to * (self.dot(to) / to.length_squared())
    }

    #[inline]
    pub fn plane_project(self, d: f32, vec: Self) -> Self {
        vec - self * (self.dot(vec) - d)
    }

    #[inline]
    pub fn lerp(self, to: Self, weight: f32) -> Self {
        Self::new(
            funcs::lerp(self.x, to.x, weight),
            funcs::lerp(self.y, to.y, weight),
        )
    }

    pub fn slerp(self, to: Self, weight: f32) -> Self {
        let start_length_sq = self.length_squared();
        let end_length_sq = to.length_squared();
        if start_length_sq == 0.0 || end_length_sq == 0.0 {
            // Zero length vectors have no angle; lerp is the best fallback.
            return self.lerp(to, weight);
        }
        let start_length = start_length_sq.sqrt();
        let result_length = funcs::lerp(start_length, end_length_sq.sqrt(), weight);
        let angle = self.angle_to(to);
        self.rotated(angle * weight) * (result_length / start_length)
    }

    #[inline]
    pub fn cubic_interpolate(self, b: Self, pre_a: Self, post_b: Self, weight: f32) -> Self {
        Self::new(
            funcs::cubic_interpolate(self.x, b.x, pre_a.x, post_b.x, weight),
            funcs::cubic_interpolate(self.y, b.y, pre_a.y, post_b.y, weight),
        )
    }

    #[inline]
    pub fn cubic_interpolate_in_time(
        self,
        b: Self,
        pre_a: Self,
        post_b: Self,
        weight: f32,
        b_t: f32,
        pre_a_t: f32,
        post_b_t: f32,
    ) -> Self {
        Self::new(
            funcs::cubic_interpolate_in_time(self.x, b.x, pre_a.x, post_b.x, weight, b_t, pre_a_t, post_b_t),
            funcs::cubic_interpolate_in_time(self.y, b.y, pre_a.y, post_b.y, weight, b_t, pre_a_t, post_b_t),
        )
    }

    #[inline]
    pub fn bezier_interpolate(self, control_1: Self, control_2: Self, end: Self, t: f32) -> Self {
        Self::new(
            funcs::bezier_interpolate(self.x, control_1.x, control_2.x, end.x, t),
            funcs::bezier_interpolate(self.y, control_1.y, control_2.y, end.y, t),
        )
    }

    #[inline]
    pub fn bezier_derivative(self, control_1: Self, control_2: Self, end: Self, t: f32) -> Self {
        Self::new(
            funcs::bezier_derivative(self.x, control_1.x, control_2.x, end.x, t),
            funcs::bezier_derivative(self.y, control_1.y, control_2.y, end.y, t),
        )
    }

    #[inline]
    pub fn move_toward(self, to: Self, delta: f32) -> Self {
        let vd = to - self;
        let len = vd.length();
        if len <= delta || len < funcs::CMP_EPSILON as f32 {
            to
        } else {
            self + vd / len * delta
        }
    }

    /// Component of this vector along the plane with normal `p_normal`.
    #[inline]
    pub fn slide(self, normal: Self) -> Self {
        self - normal * self.dot(normal)
    }

    #[inline]
    pub fn bounce(self, normal: Self) -> Self {
        -self.reflect(normal)
    }

    #[inline]
    pub fn reflect(self, normal: Self) -> Self {
        2.0 * normal * self.dot(normal) - self
    }

    #[inline]
    pub fn min(self, other: Self) -> Self {
        Self::new(self.x.min(other.x), self.y.min(other.y))
    }

    #[inline]
    pub fn minf(self, scalar: f32) -> Self {
        Self::new(self.x.min(scalar), self.y.min(scalar))
    }

    #[inline]
    pub fn max(self, other: Self) -> Self {
        Self::new(self.x.max(other.x), self.y.max(other.y))
    }

    #[inline]
    pub fn maxf(self, scalar: f32) -> Self {
        Self::new(self.x.max(scalar), self.y.max(scalar))
    }

    #[inline]
    pub fn limit_length(self, len: f32) -> Self {
        let l = self.length();
        if l > 0.0 && len < l {
            self / l * len
        } else {
            self
        }
    }

    #[inline]
    pub fn is_equal_approx(self, other: Self) -> bool {
        funcs::is_equal_approx(self.x, other.x) && funcs::is_equal_approx(self.y, other.y)
    }

    #[inline]
    pub fn is_same(self, other: Self) -> bool {
        funcs::is_same(self.x, other.x) && funcs::is_same(self.y, other.y)
    }

    #[inline]
    pub fn is_zero_approx(self) -> bool {
        funcs::is_zero_approx(self.x) && funcs::is_zero_approx(self.y)
    }

    #[inline]
    pub fn is_finite(self) -> bool {
        self.x.is_finite() && self.y.is_finite()
    }

    #[inline]
    pub fn angle(self) -> f32 {
        self.y.atan2(self.x)
    }

    #[inline]
    pub fn from_angle(angle: f32) -> Self {
        Self::new(angle.cos(), angle.sin())
    }

    #[inline]
    pub fn abs(self) -> Self {
        Self::new(self.x.abs(), self.y.abs())
    }

    #[inline]
    pub fn rotated(self, by: f32) -> Self {
        let sine = by.sin();
        let cosi = by.cos();
        Self::new(self.x * cosi - self.y * sine, self.x * sine + self.y * cosi)
    }

    #[inline]
    pub fn orthogonal(self) -> Self {
        Self::new(self.y, -self.x)
    }

    #[inline]
    pub fn sign(self) -> Self {
        Self::new(funcs::sign_f32(self.x), funcs::sign_f32(self.y))
    }

    #[inline]
    pub fn floor(self) -> Self {
        Self::new(self.x.floor(), self.y.floor())
    }

    #[inline]
    pub fn ceil(self) -> Self {
        Self::new(self.x.ceil(), self.y.ceil())
    }

    #[inline]
    pub fn round(self) -> Self {
        Self::new(self.x.round(), self.y.round())
    }

    #[inline]
    pub fn snapped(self, step: Self) -> Self {
        Self::new(
            funcs::snapped(self.x as f64, step.x as f64) as f32,
            funcs::snapped(self.y as f64, step.y as f64) as f32,
        )
    }

    #[inline]
    pub fn snappedf(self, step: f32) -> Self {
        Self::new(
            funcs::snapped(self.x as f64, step as f64) as f32,
            funcs::snapped(self.y as f64, step as f64) as f32,
        )
    }

    #[inline]
    pub fn clamp(self, min: Self, max: Self) -> Self {
        Self::new(
            funcs::clamp_f32(self.x, min.x, max.x),
            funcs::clamp_f32(self.y, min.y, max.y),
        )
    }

    #[inline]
    pub fn clampf(self, min: f32, max: f32) -> Self {
        Self::new(
            funcs::clamp_f32(self.x, min, max),
            funcs::clamp_f32(self.y, min, max),
        )
    }

    #[inline]
    pub fn aspect(self) -> f32 {
        self.x / self.y
    }

    #[inline]
    pub fn hash(self) -> u32 {
        let h = hashfuncs::hash_murmur3_one_float(self.x, hashfuncs::HASH_MURMUR3_SEED);
        let h = hashfuncs::hash_murmur3_one_float(self.y, h);
        hashfuncs::hash_fmix32(h)
    }

    #[inline]
    pub fn to_array(self) -> [f32; 2] {
        [self.x, self.y]
    }

    #[inline]
    pub const fn from_array(a: [f32; 2]) -> Self {
        Self::new(a[0], a[1])
    }
}

impl std::ops::Index<usize> for Vector2 {
    type Output = f32;
    #[inline]
    fn index(&self, i: usize) -> &f32 {
        debug_assert!(i < 2);
        match i {
            0 => &self.x,
            _ => &self.y,
        }
    }
}

impl std::ops::IndexMut<usize> for Vector2 {
    #[inline]
    fn index_mut(&mut self, i: usize) -> &mut f32 {
        debug_assert!(i < 2);
        match i {
            0 => &mut self.x,
            _ => &mut self.y,
        }
    }
}

impl std::ops::Add for Vector2 {
    type Output = Vector2;
    #[inline]
    fn add(self, o: Vector2) -> Vector2 {
        Vector2::new(self.x + o.x, self.y + o.y)
    }
}

impl std::ops::AddAssign for Vector2 {
    #[inline]
    fn add_assign(&mut self, o: Vector2) {
        self.x += o.x;
        self.y += o.y;
    }
}

impl std::ops::Sub for Vector2 {
    type Output = Vector2;
    #[inline]
    fn sub(self, o: Vector2) -> Vector2 {
        Vector2::new(self.x - o.x, self.y - o.y)
    }
}

impl std::ops::SubAssign for Vector2 {
    #[inline]
    fn sub_assign(&mut self, o: Vector2) {
        self.x -= o.x;
        self.y -= o.y;
    }
}

impl std::ops::Mul<Vector2> for Vector2 {
    type Output = Vector2;
    #[inline]
    fn mul(self, o: Vector2) -> Vector2 {
        Vector2::new(self.x * o.x, self.y * o.y)
    }
}

impl std::ops::Mul<f32> for Vector2 {
    type Output = Vector2;
    #[inline]
    fn mul(self, s: f32) -> Vector2 {
        Vector2::new(self.x * s, self.y * s)
    }
}

impl std::ops::Mul<Vector2> for f32 {
    type Output = Vector2;
    #[inline]
    fn mul(self, v: Vector2) -> Vector2 {
        v * self
    }
}

impl std::ops::MulAssign<f32> for Vector2 {
    #[inline]
    fn mul_assign(&mut self, s: f32) {
        self.x *= s;
        self.y *= s;
    }
}

impl std::ops::Div<Vector2> for Vector2 {
    type Output = Vector2;
    #[inline]
    fn div(self, o: Vector2) -> Vector2 {
        Vector2::new(self.x / o.x, self.y / o.y)
    }
}

impl std::ops::Div<f32> for Vector2 {
    type Output = Vector2;
    #[inline]
    fn div(self, s: f32) -> Vector2 {
        Vector2::new(self.x / s, self.y / s)
    }
}

impl std::ops::MulAssign<Vector2> for Vector2 {
    #[inline]
    fn mul_assign(&mut self, o: Vector2) {
        self.x *= o.x;
        self.y *= o.y;
    }
}

impl std::ops::DivAssign<Vector2> for Vector2 {
    #[inline]
    fn div_assign(&mut self, o: Vector2) {
        self.x /= o.x;
        self.y /= o.y;
    }
}

impl std::ops::DivAssign<f32> for Vector2 {
    #[inline]
    fn div_assign(&mut self, s: f32) {
        self.x /= s;
        self.y /= s;
    }
}

impl std::ops::Neg for Vector2 {
    type Output = Vector2;
    #[inline]
    fn neg(self) -> Vector2 {
        Vector2::new(-self.x, -self.y)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_ops() {
        let a = Vector2::new(1.0, 2.0);
        let b = Vector2::new(3.0, 4.0);
        assert_eq!(a + b, Vector2::new(4.0, 6.0));
        assert_eq!(b - a, Vector2::new(2.0, 2.0));
        assert_eq!(a * 2.0, Vector2::new(2.0, 4.0));
        assert_eq!(-a, Vector2::new(-1.0, -2.0));
        assert_eq!(a.dot(b), 11.0);
        assert_eq!(a.cross(b), -2.0);
        assert_eq!(a.length_squared(), 5.0);
        assert!((a.length() - 5.0f32.sqrt()).abs() < 1e-6);
    }

    #[test]
    fn normalize_edge_cases() {
        let mut v = Vector2::ZERO;
        v.normalize();
        assert_eq!(v, Vector2::ZERO);

        let mut n = Vector2::new(f32::NAN, 1.0);
        n.normalize();
        assert_eq!(n, Vector2::ZERO);

        let u = Vector2::new(3.0, 4.0).normalized();
        assert!(u.is_normalized());
        assert!((u.length() - 1.0).abs() < 1e-6);
    }

    #[test]
    fn angle_and_rotation() {
        let v = Vector2::new(1.0, 0.0);
        let r = v.rotated(funcs::PI as f32 / 2.0);
        assert!(r.is_equal_approx(Vector2::new(0.0, 1.0)));
        assert!((r.angle() - funcs::PI as f32 / 2.0).abs() < 1e-6);
        let back = Vector2::from_angle(funcs::PI as f32 / 2.0);
        assert!(back.is_equal_approx(Vector2::new(0.0, 1.0)));
    }

    #[test]
    fn reflect_and_slide() {
        let v = Vector2::new(1.0, -1.0);
        let n = Vector2::new(0.0, 1.0);
        assert!(v.slide(n).is_equal_approx(Vector2::new(1.0, 0.0)));
        assert!(v.reflect(n).is_equal_approx(Vector2::new(-1.0, -1.0)));
        assert!(v.bounce(n).is_equal_approx(Vector2::new(1.0, 1.0)));
    }

    #[test]
    fn axis_index_ties() {
        assert_eq!(Vector2::new(1.0, 1.0).min_axis_index(), 1);
        assert_eq!(Vector2::new(1.0, 1.0).max_axis_index(), 0);
        assert_eq!(Vector2::new(1.0, 2.0).min_axis_index(), 0);
        assert_eq!(Vector2::new(1.0, 2.0).max_axis_index(), 1);
    }
}
