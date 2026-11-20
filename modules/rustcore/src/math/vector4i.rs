//! Port of `core/math/vector4i.h` (integer 4D vector).
//!
//! Pending: `String` conversion.

use crate::math::funcs;
use crate::math::vector4::Vector4;
use crate::templates::hashfuncs;

/// `Math::snapped` for an integer pair: the engine passes the ints through the
/// `double` overload, so this does exactly the same `floor(v / step + 0.5)`.
#[inline]
pub(crate) fn snapped_scalar(value: i32, step: i32) -> i32 {
    funcs::snapped(value as f64, step as f64) as i32
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Vector4i {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub w: i32,
}

impl Vector4i {
    pub const ZERO: Vector4i = Vector4i { x: 0, y: 0, z: 0, w: 0 };
    pub const ONE: Vector4i = Vector4i { x: 1, y: 1, z: 1, w: 1 };

    #[inline]
    pub const fn new(x: i32, y: i32, z: i32, w: i32) -> Self {
        Self { x, y, z, w }
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
    pub fn max(self, other: Self) -> Self {
        Self::new(
            self.x.max(other.x),
            self.y.max(other.y),
            self.z.max(other.z),
            self.w.max(other.w),
        )
    }

    #[inline]
    pub fn abs(self) -> Self {
        Self::new(self.x.abs(), self.y.abs(), self.z.abs(), self.w.abs())
    }

    #[inline]
    pub fn sign(self) -> Self {
        fn s(v: i32) -> i32 {
            if v < 0 {
                -1
            } else if v > 0 {
                1
            } else {
                0
            }
        }
        Self::new(s(self.x), s(self.y), s(self.z), s(self.w))
    }

    #[inline]
    pub fn length_squared(self) -> i64 {
        self.x as i64 * self.x as i64
            + self.y as i64 * self.y as i64
            + self.z as i64 * self.z as i64
            + self.w as i64 * self.w as i64
    }

    #[inline]
    pub fn length(self) -> f64 {
        (self.length_squared() as f64).sqrt()
    }

    #[inline]
    pub fn dot(self, other: Self) -> i64 {
        self.x as i64 * other.x as i64
            + self.y as i64 * other.y as i64
            + self.z as i64 * other.z as i64
            + self.w as i64 * other.w as i64
    }

    #[inline]
    pub fn zero(&mut self) {
        *self = Self::ZERO;
    }

    #[inline]
    pub fn clamp(self, min: Self, max: Self) -> Self {
        Self::new(
            self.x.clamp(min.x, max.x),
            self.y.clamp(min.y, max.y),
            self.z.clamp(min.z, max.z),
            self.w.clamp(min.w, max.w),
        )
    }

    #[inline]
    pub fn clampi(self, min: i32, max: i32) -> Self {
        Self::new(
            self.x.clamp(min, max),
            self.y.clamp(min, max),
            self.z.clamp(min, max),
            self.w.clamp(min, max),
        )
    }

    #[inline]
    pub fn snapped(self, step: Self) -> Self {
        Self::new(
            snapped_scalar(self.x, step.x),
            snapped_scalar(self.y, step.y),
            snapped_scalar(self.z, step.z),
            snapped_scalar(self.w, step.w),
        )
    }

    #[inline]
    pub fn snappedi(self, step: i32) -> Self {
        Self::new(
            snapped_scalar(self.x, step),
            snapped_scalar(self.y, step),
            snapped_scalar(self.z, step),
            snapped_scalar(self.w, step),
        )
    }

    #[inline]
    pub fn min_axis_index(self) -> usize {
        let mut axis = 0;
        if self.y < self.x {
            axis = 1;
        }
        if self.z < [self.x, self.y][axis] {
            axis = 2;
        }
        if self.w < [self.x, self.y, self.z][axis] {
            axis = 3;
        }
        axis
    }

    #[inline]
    pub fn max_axis_index(self) -> usize {
        let mut axis = 0;
        if self.y > self.x {
            axis = 1;
        }
        if self.z > [self.x, self.y][axis] {
            axis = 2;
        }
        if self.w > [self.x, self.y, self.z][axis] {
            axis = 3;
        }
        axis
    }

    #[inline]
    pub fn to_vector4(self) -> Vector4 {
        Vector4::new(self.x as f32, self.y as f32, self.z as f32, self.w as f32)
    }

    #[inline]
    pub fn hash(self) -> u32 {
        let h = hashfuncs::hash_murmur3_one_32(self.x as u32, hashfuncs::HASH_MURMUR3_SEED);
        let h = hashfuncs::hash_murmur3_one_32(self.y as u32, h);
        let h = hashfuncs::hash_murmur3_one_32(self.z as u32, h);
        let h = hashfuncs::hash_murmur3_one_32(self.w as u32, h);
        hashfuncs::hash_fmix32(h)
    }
}

impl From<Vector4> for Vector4i {
    #[inline]
    fn from(v: Vector4) -> Self {
        Self::new(v.x as i32, v.y as i32, v.z as i32, v.w as i32)
    }
}

macro_rules! vec4i_binop {
    ($trait:ident, $method:ident, $op:tt) => {
        impl std::ops::$trait for Vector4i {
            type Output = Vector4i;
            #[inline]
            fn $method(self, o: Vector4i) -> Vector4i {
                Vector4i::new(
                    self.x $op o.x,
                    self.y $op o.y,
                    self.z $op o.z,
                    self.w $op o.w,
                )
            }
        }
        impl std::ops::$trait<i32> for Vector4i {
            type Output = Vector4i;
            #[inline]
            fn $method(self, s: i32) -> Vector4i {
                Vector4i::new(self.x $op s, self.y $op s, self.z $op s, self.w $op s)
            }
        }
    };
}

vec4i_binop!(Add, add, +);
vec4i_binop!(Sub, sub, -);
vec4i_binop!(Mul, mul, *);
vec4i_binop!(Div, div, /);

impl std::ops::Neg for Vector4i {
    type Output = Vector4i;
    #[inline]
    fn neg(self) -> Vector4i {
        Vector4i::new(-self.x, -self.y, -self.z, -self.w)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arithmetic_and_lengths() {
        let a = Vector4i::new(1, -2, 3, -4);
        let b = Vector4i::new(5, 6, -7, 8);
        assert_eq!(a + b, Vector4i::new(6, 4, -4, 4));
        assert_eq!(a.abs(), Vector4i::new(1, 2, 3, 4));
        assert_eq!(a.sign(), Vector4i::new(1, -1, 1, -1));
        assert_eq!(a.length_squared(), 30);
        assert_eq!(a.dot(b), 5 - 12 - 21 - 32);
        assert_eq!(a.min(b), Vector4i::new(1, -2, -7, -4));
        assert_eq!(a.max(b), Vector4i::new(5, 6, 3, 8));
    }

    #[test]
    fn clamped_and_snapped() {
        let a = Vector4i::new(-5, 3, 17, 0);
        assert_eq!(a.clampi(0, 10), Vector4i::new(0, 3, 10, 0));
        assert_eq!(
            a.clamp(Vector4i::new(-5, -5, -5, -5), Vector4i::new(0, 0, 0, 0)),
            Vector4i::new(-5, 0, 0, 0)
        );

        // Multiples of 4, with the engine's half-up rounding:
        // 3/4 = 0.75 -> 1; -5/4 = -1.25 -> -1; 17/4 = 4.25 -> 4.
        assert_eq!(a.snappedi(4), Vector4i::new(-4, 4, 16, 0));
        // Step 0 leaves the value untouched.
        assert_eq!(a.snappedi(0), a);
        // Per-axis steps.
        assert_eq!(
            Vector4i::new(1, 2, 3, 4).snapped(Vector4i::new(2, 3, 4, 5)),
            Vector4i::new(2, 3, 4, 5)
        );
        // The asymmetry that a Rust `round()` would get wrong: -1 snaps to 0,
        // not to -2.
        assert_eq!(Vector4i::new(-1, -1, -1, -1).snappedi(2), Vector4i::ZERO);
    }

    #[test]
    fn axis_indices_pick_first_extreme() {
        let v = Vector4i::new(3, 1, 4, 1);
        assert_eq!(v.min_axis_index(), 1);
        assert_eq!(v.max_axis_index(), 2);
        // Ties resolve to the lower axis, like the loop in `vector4i.cpp`.
        assert_eq!(Vector4i::new(2, 2, 5, 5).min_axis_index(), 0);
        assert_eq!(Vector4i::new(2, 2, 5, 5).max_axis_index(), 2);
    }

    #[test]
    fn conversion_truncates() {
        assert_eq!(
            Vector4i::from(Vector4::new(1.9, -1.9, 0.5, -0.5)),
            Vector4i::new(1, -1, 0, 0)
        );
    }
}
