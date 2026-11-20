//! Port of `core/math/vector4.h` + `vector4.cpp`.
//!
//! Used by shader uniforms, GPU storage buffers and 4D interpolation.

use crate::math::funcs;
use crate::templates::hashfuncs;

pub const UNIT_EPSILON: f32 = 0.001;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vector4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Vector4 {
    pub const ZERO: Vector4 = Vector4 { x: 0.0, y: 0.0, z: 0.0, w: 0.0 };
    pub const ONE: Vector4 = Vector4 { x: 1.0, y: 1.0, z: 1.0, w: 1.0 };
    pub const INF: Vector4 =
        Vector4 { x: f32::INFINITY, y: f32::INFINITY, z: f32::INFINITY, w: f32::INFINITY };

    #[inline]
    pub const fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { x, y, z, w }
    }

    #[inline]
    pub const fn splat(v: f32) -> Self {
        Self { x: v, y: v, z: v, w: v }
    }

    #[inline]
    pub fn min_axis_index(self) -> usize {
        let mut idx = 0;
        let mut best = self.x;
        if self.y < best {
            idx = 1;
            best = self.y;
        }
        if self.z < best {
            idx = 2;
            best = self.z;
        }
        if self.w < best {
            idx = 3;
        }
        idx
    }

    #[inline]
    pub fn max_axis_index(self) -> usize {
        let mut idx = 0;
        let mut best = self.x;
        if self.y > best {
            idx = 1;
            best = self.y;
        }
        if self.z > best {
            idx = 2;
            best = self.z;
        }
        if self.w > best {
            idx = 3;
        }
        idx
    }

    #[inline]
    pub fn min(self, other: Self) -> Self {
        Self::new(
            self.x.min(other.x),
            self.y.min(other.y),
            self.z.min(other.z),
            self.w.min(other.w),
        )
    }

    #[inline]
    pub fn minf(self, scalar: f32) -> Self {
        Self::new(
            self.x.min(scalar),
            self.y.min(scalar),
            self.z.min(scalar),
            self.w.min(scalar),
        )
    }

    #[inline]
    pub fn max(self, other: Self) -> Self {
        Self::new(
            self.x.max(other.x),
            self.y.max(other.y),
            self.z.max(other.z),
            self.w.max(other.w),
        )
    }

    #[inline]
    pub fn maxf(self, scalar: f32) -> Self {
        Self::new(
            self.x.max(scalar),
            self.y.max(scalar),
            self.z.max(scalar),
            self.w.max(scalar),
        )
    }

    #[inline]
    pub fn length_squared(self) -> f32 {
        self.dot(self)
    }

    #[inline]
    pub fn length(self) -> f32 {
        self.length_squared().sqrt()
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
            self.z /= l;
            self.w /= l;
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
    pub fn zero(&mut self) {
        *self = Self::ZERO;
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
    pub fn direction_to(self, to: Self) -> Self {
        (to - self).normalized()
    }

    #[inline]
    pub fn abs(self) -> Self {
        Self::new(self.x.abs(), self.y.abs(), self.z.abs(), self.w.abs())
    }

    #[inline]
    pub fn sign(self) -> Self {
        Self::new(
            funcs::sign_f32(self.x),
            funcs::sign_f32(self.y),
            funcs::sign_f32(self.z),
            funcs::sign_f32(self.w),
        )
    }

    #[inline]
    pub fn floor(self) -> Self {
        Self::new(self.x.floor(), self.y.floor(), self.z.floor(), self.w.floor())
    }

    #[inline]
    pub fn ceil(self) -> Self {
        Self::new(self.x.ceil(), self.y.ceil(), self.z.ceil(), self.w.ceil())
    }

    #[inline]
    pub fn round(self) -> Self {
        Self::new(self.x.round(), self.y.round(), self.z.round(), self.w.round())
    }

    #[inline]
    pub fn lerp(self, to: Self, weight: f32) -> Self {
        Self::new(
            funcs::lerp(self.x, to.x, weight),
            funcs::lerp(self.y, to.y, weight),
            funcs::lerp(self.z, to.z, weight),
            funcs::lerp(self.w, to.w, weight),
        )
    }

    #[inline]
    pub fn cubic_interpolate(self, b: Self, pre_a: Self, post_b: Self, weight: f32) -> Self {
        Self::new(
            funcs::cubic_interpolate(self.x, b.x, pre_a.x, post_b.x, weight),
            funcs::cubic_interpolate(self.y, b.y, pre_a.y, post_b.y, weight),
            funcs::cubic_interpolate(self.z, b.z, pre_a.z, post_b.z, weight),
            funcs::cubic_interpolate(self.w, b.w, pre_a.w, post_b.w, weight),
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
            funcs::cubic_interpolate_in_time(self.w, b.w, pre_a.w, post_b.w, weight, b_t, pre_a_t, post_b_t),
        )
    }

    #[inline]
    pub fn posmod(self, m: f32) -> Self {
        Self::new(
            funcs::fposmod(self.x, m),
            funcs::fposmod(self.y, m),
            funcs::fposmod(self.z, m),
            funcs::fposmod(self.w, m),
        )
    }

    #[inline]
    pub fn posmodv(self, m: Self) -> Self {
        Self::new(
            funcs::fposmod(self.x, m.x),
            funcs::fposmod(self.y, m.y),
            funcs::fposmod(self.z, m.z),
            funcs::fposmod(self.w, m.w),
        )
    }

    #[inline]
    pub fn snap(&mut self, step: Self) {
        self.x = funcs::snapped(self.x as f64, step.x as f64) as f32;
        self.y = funcs::snapped(self.y as f64, step.y as f64) as f32;
        self.z = funcs::snapped(self.z as f64, step.z as f64) as f32;
        self.w = funcs::snapped(self.w as f64, step.w as f64) as f32;
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
        self.w = funcs::snapped(self.w as f64, step as f64) as f32;
    }

    #[inline]
    pub fn snappedf(self, step: f32) -> Self {
        let mut v = self;
        v.snapf(step);
        v
    }

    #[inline]
    pub fn clamp(self, min: Self, max: Self) -> Self {
        Self::new(
            funcs::clamp_f32(self.x, min.x, max.x),
            funcs::clamp_f32(self.y, min.y, max.y),
            funcs::clamp_f32(self.z, min.z, max.z),
            funcs::clamp_f32(self.w, min.w, max.w),
        )
    }

    #[inline]
    pub fn clampf(self, min: f32, max: f32) -> Self {
        Self::new(
            funcs::clamp_f32(self.x, min, max),
            funcs::clamp_f32(self.y, min, max),
            funcs::clamp_f32(self.z, min, max),
            funcs::clamp_f32(self.w, min, max),
        )
    }

    #[inline]
    pub fn inverse(self) -> Self {
        Self::new(1.0 / self.x, 1.0 / self.y, 1.0 / self.z, 1.0 / self.w)
    }

    #[inline]
    pub fn dot(self, other: Self) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z + self.w * other.w
    }

    #[inline]
    pub fn is_equal_approx(self, other: Self) -> bool {
        funcs::is_equal_approx(self.x, other.x)
            && funcs::is_equal_approx(self.y, other.y)
            && funcs::is_equal_approx(self.z, other.z)
            && funcs::is_equal_approx(self.w, other.w)
    }

    #[inline]
    pub fn is_zero_approx(self) -> bool {
        funcs::is_zero_approx(self.x)
            && funcs::is_zero_approx(self.y)
            && funcs::is_zero_approx(self.z)
            && funcs::is_zero_approx(self.w)
    }

    #[inline]
    pub fn is_same(self, other: Self) -> bool {
        funcs::is_same(self.x, other.x)
            && funcs::is_same(self.y, other.y)
            && funcs::is_same(self.z, other.z)
            && funcs::is_same(self.w, other.w)
    }

    #[inline]
    pub fn is_finite(self) -> bool {
        self.x.is_finite() && self.y.is_finite() && self.z.is_finite() && self.w.is_finite()
    }

    #[inline]
    pub fn hash(self) -> u32 {
        let h = hashfuncs::hash_murmur3_one_float(self.x, hashfuncs::HASH_MURMUR3_SEED);
        let h = hashfuncs::hash_murmur3_one_float(self.y, h);
        let h = hashfuncs::hash_murmur3_one_float(self.z, h);
        let h = hashfuncs::hash_murmur3_one_float(self.w, h);
        hashfuncs::hash_fmix32(h)
    }

    #[inline]
    pub fn to_array(self) -> [f32; 4] {
        [self.x, self.y, self.z, self.w]
    }

    #[inline]
    pub const fn from_array(a: [f32; 4]) -> Self {
        Self::new(a[0], a[1], a[2], a[3])
    }
}

impl std::ops::Index<usize> for Vector4 {
    type Output = f32;
    #[inline]
    fn index(&self, i: usize) -> &f32 {
        debug_assert!(i < 4);
        match i {
            0 => &self.x,
            1 => &self.y,
            2 => &self.z,
            _ => &self.w,
        }
    }
}

impl std::ops::IndexMut<usize> for Vector4 {
    #[inline]
    fn index_mut(&mut self, i: usize) -> &mut f32 {
        debug_assert!(i < 4);
        match i {
            0 => &mut self.x,
            1 => &mut self.y,
            2 => &mut self.z,
            _ => &mut self.w,
        }
    }
}

macro_rules! vec4_binop {
    ($trait:ident, $method:ident, $op:tt) => {
        impl std::ops::$trait for Vector4 {
            type Output = Vector4;
            #[inline]
            fn $method(self, o: Vector4) -> Vector4 {
                Vector4::new(self.x $op o.x, self.y $op o.y, self.z $op o.z, self.w $op o.w)
            }
        }
        impl std::ops::$trait<f32> for Vector4 {
            type Output = Vector4;
            #[inline]
            fn $method(self, s: f32) -> Vector4 {
                Vector4::new(self.x $op s, self.y $op s, self.z $op s, self.w $op s)
            }
        }
    };
}

vec4_binop!(Add, add, +);
vec4_binop!(Sub, sub, -);
vec4_binop!(Mul, mul, *);
vec4_binop!(Div, div, /);

impl std::ops::Neg for Vector4 {
    type Output = Vector4;
    #[inline]
    fn neg(self) -> Vector4 {
        Vector4::new(-self.x, -self.y, -self.z, -self.w)
    }
}

impl std::ops::MulAssign<f32> for Vector4 {
    #[inline]
    fn mul_assign(&mut self, s: f32) {
        self.x *= s;
        self.y *= s;
        self.z *= s;
        self.w *= s;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arithmetic() {
        let a = Vector4::new(1.0, 2.0, 3.0, 4.0);
        let b = Vector4::new(-1.0, 0.5, 0.0, 2.0);
        assert_eq!(a + b, Vector4::new(0.0, 2.5, 3.0, 6.0));
        assert_eq!(a.dot(b), -1.0 + 1.0 + 0.0 + 8.0);
        assert_eq!(a.length_squared(), 30.0);
        assert_eq!(a.min(b), Vector4::new(-1.0, 0.5, 0.0, 2.0));
        assert_eq!(a.max(b), Vector4::new(1.0, 2.0, 3.0, 4.0));
        assert_eq!(a.abs(), a);
        assert_eq!(a.sign(), Vector4::new(1.0, 1.0, 1.0, 1.0));
    }

    #[test]
    fn normalize_edge_cases() {
        let mut v = Vector4::ZERO;
        v.normalize();
        assert_eq!(v, Vector4::ZERO);
        let n = Vector4::splat(2.0).normalized();
        assert!(n.is_normalized());
        assert!((n.x - 0.5).abs() < 1e-6);
    }

    #[test]
    fn axis_order() {
        assert_eq!(Vector4::new(1.0, 2.0, 3.0, 4.0).max_axis_index(), 3);
        assert_eq!(Vector4::new(1.0, 2.0, 3.0, 4.0).min_axis_index(), 0);
        assert_eq!(Vector4::new(-1.0, -5.0, 3.0, 4.0).min_axis_index(), 1);
    }
}
