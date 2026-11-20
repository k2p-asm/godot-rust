//! Port of `core/math/vector2i.h` (integer vector).
//!
//! Pending: `snapped`, `posmod` and `String` conversion.

use crate::math::vector2::Vector2;
use crate::templates::hashfuncs;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Vector2i {
    pub x: i32,
    pub y: i32,
}

impl Vector2i {
    pub const ZERO: Vector2i = Vector2i { x: 0, y: 0 };
    pub const ONE: Vector2i = Vector2i { x: 1, y: 1 };
    pub const LEFT: Vector2i = Vector2i { x: -1, y: 0 };
    pub const RIGHT: Vector2i = Vector2i { x: 1, y: 0 };
    pub const UP: Vector2i = Vector2i { x: 0, y: -1 };
    pub const DOWN: Vector2i = Vector2i { x: 0, y: 1 };

    #[inline]
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    #[inline]
    pub const fn splat(v: i32) -> Self {
        Self { x: v, y: v }
    }

    #[inline]
    pub fn min_axis_index(self) -> usize {
        if self.x < self.y {
            0
        } else {
            1
        }
    }

    #[inline]
    pub fn max_axis_index(self) -> usize {
        if self.x < self.y {
            1
        } else {
            0
        }
    }

    #[inline]
    pub fn min(self, other: Self) -> Self {
        Self::new(self.x.min(other.x), self.y.min(other.y))
    }

    #[inline]
    pub fn mini(self, scalar: i32) -> Self {
        Self::new(self.x.min(scalar), self.y.min(scalar))
    }

    #[inline]
    pub fn max(self, other: Self) -> Self {
        Self::new(self.x.max(other.x), self.y.max(other.y))
    }

    #[inline]
    pub fn maxi(self, scalar: i32) -> Self {
        Self::new(self.x.max(scalar), self.y.max(scalar))
    }

    #[inline]
    pub fn length_squared(self) -> i64 {
        self.x as i64 * self.x as i64 + self.y as i64 * self.y as i64
    }

    #[inline]
    pub fn length(self) -> f64 {
        (self.length_squared() as f64).sqrt()
    }

    #[inline]
    pub fn zero(&mut self) {
        self.x = 0;
        self.y = 0;
    }

    #[inline]
    pub fn abs(self) -> Self {
        Self::new(self.x.abs(), self.y.abs())
    }

    #[inline]
    pub fn sign(self) -> Self {
        fn s(v: i32) -> i32 {
            if v < 0 {
                -1
            } else {
                1
            }
        }
        Self::new(s(self.x), s(self.y))
    }

    #[inline]
    pub fn clamp(self, min: Self, max: Self) -> Self {
        Self::new(self.x.clamp(min.x, max.x), self.y.clamp(min.y, max.y))
    }

    #[inline]
    pub fn clampi(self, min: i32, max: i32) -> Self {
        Self::new(self.x.clamp(min, max), self.y.clamp(min, max))
    }

    /// `Math::snapped`: `floor(v / step + 0.5) * step`, so negative halves round
    /// up instead of away from zero.
    #[inline]
    pub fn snapped(self, step: Self) -> Self {
        Self::new(
            crate::math::vector4i::snapped_scalar(self.x, step.x),
            crate::math::vector4i::snapped_scalar(self.y, step.y),
        )
    }

    #[inline]
    pub fn snappedi(self, step: i32) -> Self {
        Self::new(
            crate::math::vector4i::snapped_scalar(self.x, step),
            crate::math::vector4i::snapped_scalar(self.y, step),
        )
    }

    #[inline]
    pub fn distance_squared_to(self, to: Self) -> i64 {
        (to - self).length_squared()
    }

    #[inline]
    pub fn distance_to(self, to: Self) -> f64 {
        (self.distance_squared_to(to) as f64).sqrt()
    }

    #[inline]
    pub fn to_vector2(self) -> Vector2 {
        Vector2::new(self.x as f32, self.y as f32)
    }

    #[inline]
    pub fn hash(self) -> u32 {
        let h = hashfuncs::hash_murmur3_one_32(self.x as u32, hashfuncs::HASH_MURMUR3_SEED);
        let h = hashfuncs::hash_murmur3_one_32(self.y as u32, h);
        hashfuncs::hash_fmix32(h)
    }
}

impl From<Vector2> for Vector2i {
    #[inline]
    fn from(v: Vector2) -> Self {
        Self::new(v.x as i32, v.y as i32)
    }
}

impl std::ops::Index<usize> for Vector2i {
    type Output = i32;
    #[inline]
    fn index(&self, i: usize) -> &i32 {
        debug_assert!(i < 2);
        match i {
            0 => &self.x,
            _ => &self.y,
        }
    }
}

macro_rules! vec2i_binop {
    ($trait:ident, $method:ident, $op:tt) => {
        impl std::ops::$trait for Vector2i {
            type Output = Vector2i;
            #[inline]
            fn $method(self, o: Vector2i) -> Vector2i {
                Vector2i::new(self.x $op o.x, self.y $op o.y)
            }
        }
        impl std::ops::$trait<i32> for Vector2i {
            type Output = Vector2i;
            #[inline]
            fn $method(self, s: i32) -> Vector2i {
                Vector2i::new(self.x $op s, self.y $op s)
            }
        }
    };
}

vec2i_binop!(Add, add, +);
vec2i_binop!(Sub, sub, -);
vec2i_binop!(Mul, mul, *);
vec2i_binop!(Div, div, /);
vec2i_binop!(Rem, rem, %);

impl std::ops::Neg for Vector2i {
    type Output = Vector2i;
    #[inline]
    fn neg(self) -> Vector2i {
        Vector2i::new(-self.x, -self.y)
    }
}

impl std::ops::AddAssign for Vector2i {
    #[inline]
    fn add_assign(&mut self, o: Vector2i) {
        self.x += o.x;
        self.y += o.y;
    }
}

impl std::ops::SubAssign for Vector2i {
    #[inline]
    fn sub_assign(&mut self, o: Vector2i) {
        self.x -= o.x;
        self.y -= o.y;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arithmetic() {
        let a = Vector2i::new(3, -4);
        assert_eq!(a + Vector2i::new(1, 1), Vector2i::new(4, -3));
        assert_eq!(a.length_squared(), 25);
        assert_eq!(a.length(), 5.0);
        assert_eq!(a.abs(), Vector2i::new(3, 4));
        assert_eq!(Vector2i::new(0, -2).sign(), Vector2i::new(1, -1));
    }

    #[test]
    fn axis_indices_and_clamp() {
        assert_eq!(Vector2i::new(1, 1).min_axis_index(), 1);
        assert_eq!(Vector2i::new(1, 1).max_axis_index(), 0);
        assert_eq!(Vector2i::new(5, 9).clampi(0, 6), Vector2i::new(5, 6));
    }

    #[test]
    fn snapped_matches_math_snapped() {
        // -5/4 = -1.25 -> -1 * 4 = -4; 3/4 = 0.75 -> 1 * 4 = 4.
        assert_eq!(Vector2i::new(-5, 3).snappedi(4), Vector2i::new(-4, 4));
        // -1/2 = -0.5, floor(0.0) = 0: rounds towards +inf, not away from zero.
        assert_eq!(Vector2i::new(-1, -1).snappedi(2), Vector2i::ZERO);
        assert_eq!(Vector2i::new(3, 7).snapped(Vector2i::new(2, 5)), Vector2i::new(4, 5));
        assert_eq!(Vector2i::new(3, 7).snappedi(0), Vector2i::new(3, 7));
    }
}
