//! Port of `core/math/vector3.h` + `vector3.cpp`.
//!
//! `rotated()` builds the same axis-angle rotation matrix that
//! `Basis::set_axis_angle` builds in `core/math/basis.cpp` and applies it row by
//! row, so results are bit-identical to the C++ engine (no Rodrigues shortcut).

use crate::math::funcs;
use crate::math::vector2::Vector2;
use crate::templates::hashfuncs;

pub const UNIT_EPSILON: f32 = 0.001;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vector3 {
    pub const ZERO: Vector3 = Vector3 { x: 0.0, y: 0.0, z: 0.0 };
    pub const ONE: Vector3 = Vector3 { x: 1.0, y: 1.0, z: 1.0 };
    pub const INF: Vector3 = Vector3 { x: f32::INFINITY, y: f32::INFINITY, z: f32::INFINITY };
    pub const LEFT: Vector3 = Vector3 { x: -1.0, y: 0.0, z: 0.0 };
    pub const RIGHT: Vector3 = Vector3 { x: 1.0, y: 0.0, z: 0.0 };
    pub const UP: Vector3 = Vector3 { x: 0.0, y: 1.0, z: 0.0 };
    pub const DOWN: Vector3 = Vector3 { x: 0.0, y: -1.0, z: 0.0 };
    pub const FORWARD: Vector3 = Vector3 { x: 0.0, y: 0.0, z: -1.0 };
    pub const BACK: Vector3 = Vector3 { x: 0.0, y: 0.0, z: 1.0 };
    pub const MODEL_LEFT: Vector3 = Vector3 { x: 1.0, y: 0.0, z: 0.0 };
    pub const MODEL_RIGHT: Vector3 = Vector3 { x: -1.0, y: 0.0, z: 0.0 };
    pub const MODEL_TOP: Vector3 = Vector3 { x: 0.0, y: 1.0, z: 0.0 };
    pub const MODEL_BOTTOM: Vector3 = Vector3 { x: 0.0, y: -1.0, z: 0.0 };
    pub const MODEL_FRONT: Vector3 = Vector3 { x: 0.0, y: 0.0, z: 1.0 };
    pub const MODEL_REAR: Vector3 = Vector3 { x: 0.0, y: 0.0, z: -1.0 };

    #[inline]
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    #[inline]
    pub const fn splat(v: f32) -> Self {
        Self { x: v, y: v, z: v }
    }

    #[inline]
    pub fn min_axis_index(self) -> usize {
        if self.x < self.y {
            if self.x < self.z {
                0
            } else {
                2
            }
        } else if self.y < self.z {
            1
        } else {
            2
        }
    }

    #[inline]
    pub fn max_axis_index(self) -> usize {
        if self.x < self.y {
            if self.y < self.z {
                2
            } else {
                1
            }
        } else if self.x < self.z {
            2
        } else {
            0
        }
    }

    #[inline]
    pub fn length(self) -> f32 {
        let x2 = self.x * self.x;
        let y2 = self.y * self.y;
        let z2 = self.z * self.z;
        (x2 + y2 + z2).sqrt()
    }

    #[inline]
    pub fn length_squared(self) -> f32 {
        let x2 = self.x * self.x;
        let y2 = self.y * self.y;
        let z2 = self.z * self.z;
        x2 + y2 + z2
    }

    pub fn normalize(&mut self) {
        if !self.is_finite() {
            *self = Self::ZERO;
            return;
        }
        let mut l = self.length_squared();
        if l == 0.0 {
            *self = Self::ZERO;
        } else {
            l = l.sqrt();
            self.x /= l;
            self.y /= l;
            self.z /= l;
        }
    }

    #[inline]
    pub fn normalized(self) -> Self {
        let mut v = self;
        v.normalize();
        v
    }

    #[inline]
    pub fn is_normalized(self) -> bool {
        funcs::is_equal_approx_tol(self.length_squared(), 1.0, UNIT_EPSILON)
    }

    #[inline]
    pub fn inverse(self) -> Self {
        Self::new(1.0 / self.x, 1.0 / self.y, 1.0 / self.z)
    }

    #[inline]
    pub fn zero(&mut self) {
        self.x = 0.0;
        self.y = 0.0;
        self.z = 0.0;
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
    pub fn snap(&mut self, step: Self) {
        self.x = funcs::snapped(self.x as f64, step.x as f64) as f32;
        self.y = funcs::snapped(self.y as f64, step.y as f64) as f32;
        self.z = funcs::snapped(self.z as f64, step.z as f64) as f32;
    }

    #[inline]
    pub fn snapped(self, step: Self) -> Self {
        let mut v = self;
        v.snap(step);
        v
    }

    #[inline]
    pub fn snapf(&mut self, step: f32) {
        self.x = funcs::snapped(self.x as f64, step as f64) as f32;
        self.y = funcs::snapped(self.y as f64, step as f64) as f32;
        self.z = funcs::snapped(self.z as f64, step as f64) as f32;
    }

    #[inline]
    pub fn snappedf(self, step: f32) -> Self {
        let mut v = self;
        v.snapf(step);
        v
    }

    /// Rotation matrix from axis and angle, identical to `Basis::set_axis_angle`.
    /// The axis is assumed to be normalized (as in the engine, which only
    /// validates it under `MATH_CHECKS`).
    #[inline]
    fn axis_angle_matrix(axis: Self, angle: f32) -> [[f32; 3]; 3] {
        let axis_sq = Self::new(axis.x * axis.x, axis.y * axis.y, axis.z * axis.z);
        let cosine = angle.cos();
        let sine = angle.sin();
        let t = 1.0 - cosine;

        let mut rows = [[0.0f32; 3]; 3];
        rows[0][0] = axis_sq.x + cosine * (1.0 - axis_sq.x);
        rows[1][1] = axis_sq.y + cosine * (1.0 - axis_sq.y);
        rows[2][2] = axis_sq.z + cosine * (1.0 - axis_sq.z);

        let mut xyzt = axis.x * axis.y * t;
        let mut zyxs = axis.z * sine;
        rows[0][1] = xyzt - zyxs;
        rows[1][0] = xyzt + zyxs;

        xyzt = axis.x * axis.z * t;
        zyxs = axis.y * sine;
        rows[0][2] = xyzt + zyxs;
        rows[2][0] = xyzt - zyxs;

        xyzt = axis.y * axis.z * t;
        zyxs = axis.x * sine;
        rows[1][2] = xyzt - zyxs;
        rows[2][1] = xyzt + zyxs;

        rows
    }

    #[inline]
    pub fn rotate(&mut self, axis: Self, angle: f32) {
        let m = Self::axis_angle_matrix(axis, angle);
        let v = *self;
        *self = Self::new(
            m[0][0] * v.x + m[0][1] * v.y + m[0][2] * v.z,
            m[1][0] * v.x + m[1][1] * v.y + m[1][2] * v.z,
            m[2][0] * v.x + m[2][1] * v.y + m[2][2] * v.z,
        );
    }

    #[inline]
    pub fn rotated(self, axis: Self, angle: f32) -> Self {
        let mut v = self;
        v.rotate(axis, angle);
        v
    }

    #[inline]
    pub fn lerp(self, to: Self, weight: f32) -> Self {
        Self::new(
            funcs::lerp(self.x, to.x, weight),
            funcs::lerp(self.y, to.y, weight),
            funcs::lerp(self.z, to.z, weight),
        )
    }

    pub fn slerp(self, to: Self, weight: f32) -> Self {
        let start_length_sq = self.length_squared();
        let end_length_sq = to.length_squared();
        if start_length_sq == 0.0 || end_length_sq == 0.0 {
            return self.lerp(to, weight);
        }
        let mut axis = self.cross(to);
        let axis_length_sq = axis.length_squared();
        if axis_length_sq == 0.0 {
            // Colinear vectors: no rotation axis, lerp is the fallback.
            return self.lerp(to, weight);
        }
        axis /= axis_length_sq.sqrt();
        let start_length = start_length_sq.sqrt();
        let result_length = funcs::lerp(start_length, end_length_sq.sqrt(), weight);
        let angle = self.angle_to(to);
        self.rotated(axis, angle * weight) * (result_length / start_length)
    }

    #[inline]
    pub fn cubic_interpolate(self, b: Self, pre_a: Self, post_b: Self, weight: f32) -> Self {
        Self::new(
            funcs::cubic_interpolate(self.x, b.x, pre_a.x, post_b.x, weight),
            funcs::cubic_interpolate(self.y, b.y, pre_a.y, post_b.y, weight),
            funcs::cubic_interpolate(self.z, b.z, pre_a.z, post_b.z, weight),
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
            funcs::cubic_interpolate_in_time(self.z, b.z, pre_a.z, post_b.z, weight, b_t, pre_a_t, post_b_t),
        )
    }

    #[inline]
    pub fn bezier_interpolate(self, control_1: Self, control_2: Self, end: Self, t: f32) -> Self {
        Self::new(
            funcs::bezier_interpolate(self.x, control_1.x, control_2.x, end.x, t),
            funcs::bezier_interpolate(self.y, control_1.y, control_2.y, end.y, t),
            funcs::bezier_interpolate(self.z, control_1.z, control_2.z, end.z, t),
        )
    }

    #[inline]
    pub fn bezier_derivative(self, control_1: Self, control_2: Self, end: Self, t: f32) -> Self {
        Self::new(
            funcs::bezier_derivative(self.x, control_1.x, control_2.x, end.x, t),
            funcs::bezier_derivative(self.y, control_1.y, control_2.y, end.y, t),
            funcs::bezier_derivative(self.z, control_1.z, control_2.z, end.z, t),
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

    /// Octahedral encoding of a unit vector into the unit square.
    pub fn octahedron_encode(self) -> Vector2 {
        let mut n = self;
        n /= n.x.abs() + n.y.abs() + n.z.abs();
        let mut o = Vector2::ZERO;
        if n.z >= 0.0 {
            o.x = n.x;
            o.y = n.y;
        } else {
            o.x = (1.0 - n.y.abs()) * if n.x >= 0.0 { 1.0 } else { -1.0 };
            o.y = (1.0 - n.x.abs()) * if n.y >= 0.0 { 1.0 } else { -1.0 };
        }
        o.x = o.x * 0.5 + 0.5;
        o.y = o.y * 0.5 + 0.5;
        o
    }

    pub fn octahedron_decode(oct: Vector2) -> Self {
        let f = Vector2::new(oct.x * 2.0 - 1.0, oct.y * 2.0 - 1.0);
        let mut n = Self::new(f.x, f.y, 1.0 - f.x.abs() - f.y.abs());
        let t = funcs::clamp_f32(-n.z, 0.0, 1.0);
        n.x += if n.x >= 0.0 { -t } else { t };
        n.y += if n.y >= 0.0 { -t } else { t };
        n.normalized()
    }

    pub fn octahedron_tangent_encode(self, sign: f32) -> Vector2 {
        let bias = 1.0f32 / 32767.0f32;
        let mut res = self.octahedron_encode();
        res.y = res.y.max(bias);
        res.y = res.y * 0.5 + 0.5;
        res.y = if sign >= 0.0 { res.y } else { 1.0 - res.y };
        res
    }

    pub fn octahedron_tangent_decode(oct: Vector2) -> (Self, f32) {
        let mut compressed = oct;
        compressed.y = compressed.y * 2.0 - 1.0;
        let sign = if compressed.y >= 0.0 { 1.0 } else { -1.0 };
        compressed.y = compressed.y.abs();
        (Self::octahedron_decode(compressed), sign)
    }

    /// Outer product, returned as a row-major 3x3 matrix (becomes `Basis` once
    /// that type is ported).
    #[inline]
    pub fn outer(self, with: Self) -> [[f32; 3]; 3] {
        [
            [self.x * with.x, self.x * with.y, self.x * with.z],
            [self.y * with.x, self.y * with.y, self.y * with.z],
            [self.z * with.x, self.z * with.y, self.z * with.z],
        ]
    }

    #[inline]
    pub fn cross(self, with: Self) -> Self {
        Self::new(
            (self.y * with.z) - (self.z * with.y),
            (self.z * with.x) - (self.x * with.z),
            (self.x * with.y) - (self.y * with.x),
        )
    }

    #[inline]
    pub fn dot(self, with: Self) -> f32 {
        self.x * with.x + self.y * with.y + self.z * with.z
    }

    pub fn get_any_perpendicular(self) -> Self {
        if self.is_zero_approx() {
            return Self::ZERO;
        }
        let base = if self.x.abs() <= self.y.abs() && self.x.abs() <= self.z.abs() {
            Self::RIGHT
        } else {
            Self::UP
        };
        self.cross(base).normalized()
    }

    #[inline]
    pub fn abs(self) -> Self {
        Self::new(self.x.abs(), self.y.abs(), self.z.abs())
    }

    #[inline]
    pub fn sign(self) -> Self {
        Self::new(
            funcs::sign_f32(self.x),
            funcs::sign_f32(self.y),
            funcs::sign_f32(self.z),
        )
    }

    #[inline]
    pub fn floor(self) -> Self {
        Self::new(self.x.floor(), self.y.floor(), self.z.floor())
    }

    #[inline]
    pub fn ceil(self) -> Self {
        Self::new(self.x.ceil(), self.y.ceil(), self.z.ceil())
    }

    #[inline]
    pub fn round(self) -> Self {
        Self::new(self.x.round(), self.y.round(), self.z.round())
    }

    #[inline]
    pub fn distance_to(self, to: Self) -> f32 {
        (to - self).length()
    }

    #[inline]
    pub fn distance_squared_to(self, to: Self) -> f32 {
        (to - self).length_squared()
    }

    #[inline]
    pub fn posmod(self, m: f32) -> Self {
        Self::new(
            funcs::fposmod(self.x, m),
            funcs::fposmod(self.y, m),
            funcs::fposmod(self.z, m),
        )
    }

    #[inline]
    pub fn posmodv(self, m: Self) -> Self {
        Self::new(
            funcs::fposmod(self.x, m.x),
            funcs::fposmod(self.y, m.y),
            funcs::fposmod(self.z, m.z),
        )
    }

    #[inline]
    pub fn project(self, to: Self) -> Self {
        to * (self.dot(to) / to.length_squared())
    }

    #[inline]
    pub fn angle_to(self, to: Self) -> f32 {
        self.cross(to).length().atan2(self.dot(to))
    }

    #[inline]
    pub fn signed_angle_to(self, to: Self, axis: Self) -> f32 {
        let cross_to = self.cross(to);
        let unsigned_angle = cross_to.length().atan2(self.dot(to));
        let sign = cross_to.dot(axis);
        if sign < 0.0 {
            -unsigned_angle
        } else {
            unsigned_angle
        }
    }

    #[inline]
    pub fn direction_to(self, to: Self) -> Self {
        (to - self).normalized()
    }

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
        Self::new(self.x.min(other.x), self.y.min(other.y), self.z.min(other.z))
    }

    #[inline]
    pub fn minf(self, scalar: f32) -> Self {
        Self::new(self.x.min(scalar), self.y.min(scalar), self.z.min(scalar))
    }

    #[inline]
    pub fn max(self, other: Self) -> Self {
        Self::new(self.x.max(other.x), self.y.max(other.y), self.z.max(other.z))
    }

    #[inline]
    pub fn maxf(self, scalar: f32) -> Self {
        Self::new(self.x.max(scalar), self.y.max(scalar), self.z.max(scalar))
    }

    #[inline]
    pub fn clamp(self, min: Self, max: Self) -> Self {
        Self::new(
            funcs::clamp_f32(self.x, min.x, max.x),
            funcs::clamp_f32(self.y, min.y, max.y),
            funcs::clamp_f32(self.z, min.z, max.z),
        )
    }

    #[inline]
    pub fn clampf(self, min: f32, max: f32) -> Self {
        Self::new(
            funcs::clamp_f32(self.x, min, max),
            funcs::clamp_f32(self.y, min, max),
            funcs::clamp_f32(self.z, min, max),
        )
    }

    #[inline]
    pub fn is_equal_approx(self, other: Self) -> bool {
        funcs::is_equal_approx(self.x, other.x)
            && funcs::is_equal_approx(self.y, other.y)
            && funcs::is_equal_approx(self.z, other.z)
    }

    #[inline]
    pub fn is_same(self, other: Self) -> bool {
        funcs::is_same(self.x, other.x)
            && funcs::is_same(self.y, other.y)
            && funcs::is_same(self.z, other.z)
    }

    #[inline]
    pub fn is_zero_approx(self) -> bool {
        funcs::is_zero_approx(self.x)
            && funcs::is_zero_approx(self.y)
            && funcs::is_zero_approx(self.z)
    }

    #[inline]
    pub fn is_finite(self) -> bool {
        self.x.is_finite() && self.y.is_finite() && self.z.is_finite()
    }

    #[inline]
    pub fn hash(self) -> u32 {
        let h = hashfuncs::hash_murmur3_one_float(self.x, hashfuncs::HASH_MURMUR3_SEED);
        let h = hashfuncs::hash_murmur3_one_float(self.y, h);
        let h = hashfuncs::hash_murmur3_one_float(self.z, h);
        hashfuncs::hash_fmix32(h)
    }

    #[inline]
    pub fn to_array(self) -> [f32; 3] {
        [self.x, self.y, self.z]
    }

    #[inline]
    pub const fn from_array(a: [f32; 3]) -> Self {
        Self::new(a[0], a[1], a[2])
    }
}

impl std::ops::Index<usize> for Vector3 {
    type Output = f32;
    #[inline]
    fn index(&self, i: usize) -> &f32 {
        debug_assert!(i < 3);
        match i {
            0 => &self.x,
            1 => &self.y,
            _ => &self.z,
        }
    }
}

impl std::ops::IndexMut<usize> for Vector3 {
    #[inline]
    fn index_mut(&mut self, i: usize) -> &mut f32 {
        debug_assert!(i < 3);
        match i {
            0 => &mut self.x,
            1 => &mut self.y,
            _ => &mut self.z,
        }
    }
}

macro_rules! vec3_binop {
    ($trait:ident, $method:ident, $op:tt) => {
        impl std::ops::$trait for Vector3 {
            type Output = Vector3;
            #[inline]
            fn $method(self, o: Vector3) -> Vector3 {
                Vector3::new(self.x $op o.x, self.y $op o.y, self.z $op o.z)
            }
        }
        impl std::ops::$trait<f32> for Vector3 {
            type Output = Vector3;
            #[inline]
            fn $method(self, s: f32) -> Vector3 {
                Vector3::new(self.x $op s, self.y $op s, self.z $op s)
            }
        }
    };
}

vec3_binop!(Add, add, +);
vec3_binop!(Sub, sub, -);
vec3_binop!(Mul, mul, *);
vec3_binop!(Div, div, /);

impl std::ops::Mul<Vector3> for f32 {
    type Output = Vector3;
    #[inline]
    fn mul(self, v: Vector3) -> Vector3 {
        v * self
    }
}

impl std::ops::AddAssign for Vector3 {
    #[inline]
    fn add_assign(&mut self, o: Vector3) {
        self.x += o.x;
        self.y += o.y;
        self.z += o.z;
    }
}

impl std::ops::SubAssign for Vector3 {
    #[inline]
    fn sub_assign(&mut self, o: Vector3) {
        self.x -= o.x;
        self.y -= o.y;
        self.z -= o.z;
    }
}

impl std::ops::MulAssign<f32> for Vector3 {
    #[inline]
    fn mul_assign(&mut self, s: f32) {
        self.x *= s;
        self.y *= s;
        self.z *= s;
    }
}

impl std::ops::DivAssign<f32> for Vector3 {
    #[inline]
    fn div_assign(&mut self, s: f32) {
        self.x /= s;
        self.y /= s;
        self.z /= s;
    }
}

impl std::ops::MulAssign<Vector3> for Vector3 {
    #[inline]
    fn mul_assign(&mut self, o: Vector3) {
        self.x *= o.x;
        self.y *= o.y;
        self.z *= o.z;
    }
}

impl std::ops::DivAssign<Vector3> for Vector3 {
    #[inline]
    fn div_assign(&mut self, o: Vector3) {
        self.x /= o.x;
        self.y /= o.y;
        self.z /= o.z;
    }
}

impl std::ops::Neg for Vector3 {
    type Output = Vector3;
    #[inline]
    fn neg(self) -> Vector3 {
        Vector3::new(-self.x, -self.y, -self.z)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cross_and_dot() {
        let a = Vector3::new(1.0, 0.0, 0.0);
        let b = Vector3::new(0.0, 1.0, 0.0);
        assert_eq!(a.cross(b), Vector3::new(0.0, 0.0, 1.0));
        assert_eq!(a.dot(b), 0.0);
        assert_eq!(a.cross(a), Vector3::ZERO);
    }

    #[test]
    fn rotation_matches_cpp_semantics() {
        // Rotation about Y by +90 degrees maps +X onto -Z (right-handed, as in Godot).
        let v = Vector3::new(1.0, 0.0, 0.0).rotated(Vector3::UP, funcs::PI as f32 / 2.0);
        assert!(v.is_equal_approx(Vector3::new(0.0, 0.0, -1.0)), "{v:?}");

        // Rotation about a normalized diagonal axis preserves length.
        let axis = Vector3::new(1.0, 1.0, 0.0).normalized();
        let r = Vector3::new(0.3, -0.7, 0.2).rotated(axis, 1.234);
        assert!((r.length() - Vector3::new(0.3, -0.7, 0.2).length()).abs() < 1e-5);
    }

    #[test]
    fn octahedron_roundtrip() {
        for v in [
            Vector3::new(0.0, 0.0, 1.0),
            Vector3::new(1.0, 0.0, 0.0),
            Vector3::new(0.0, -1.0, 0.0),
            Vector3::new(0.3, 0.5, -0.8).normalized(),
        ] {
            let enc = v.octahedron_encode();
            let dec = Vector3::octahedron_decode(enc);
            assert!(dec.is_equal_approx(v), "{v:?} -> {enc:?} -> {dec:?}");
        }
    }

    #[test]
    fn slide_reflect() {
        let v = Vector3::new(1.0, -1.0, 2.0);
        let n = Vector3::UP;
        assert!(v.slide(n).is_equal_approx(Vector3::new(1.0, 0.0, 2.0)));
        assert!(v.reflect(n).is_equal_approx(Vector3::new(-1.0, -1.0, -2.0)));
    }

    #[test]
    fn perpendicular_is_orthogonal() {
        let v = Vector3::new(1.0, 2.0, 3.0);
        let p = v.get_any_perpendicular();
        assert!(p.is_normalized());
        assert!(v.dot(p).abs() < 1e-5);
    }
}
