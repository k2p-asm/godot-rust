//! Port of `core/math/vector3i.h` (integer vector).
//!
//! Pending: `snapped` (integer snapping semantics), `posmod` and the
//! `String` conversion.

use crate::math::vector3::Vector3;
use crate::templates::hashfuncs;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Vector3i {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl Vector3i {
    pub const ZERO: Vector3i = Vector3i { x: 0, y: 0, z: 0 };
    pub const ONE: Vector3i = Vector3i { x: 1, y: 1, z: 1 };
    pub const LEFT: Vector3i = Vector3i { x: -1, y: 0, z: 0 };
    pub const RIGHT: Vector3i = Vector3i { x: 1, y: 0, z: 0 };
    pub const UP: Vector3i = Vector3i { x: 0, y: 1, z: 0 };
    pub const DOWN: Vector3i = Vector3i { x: 0, y: -1, z: 0 };
    pub const FORWARD: Vector3i = Vector3i { x: 0, y: 0, z: -1 };
    pub const BACK: Vector3i = Vector3i { x: 0, y: 0, z: 1 };

    #[inline]
    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    #[inline]
    pub const fn splat(v: i32) -> Self {
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
    pub fn min(self, other: Self) -> Self {
        Self::new(self.x.min(other.x), self.y.min(other.y), self.z.min(other.z))
    }

    #[inline]
    pub fn mini(self, scalar: i32) -> Self {
        Self::new(self.x.min(scalar), self.y.min(scalar), self.z.min(scalar))
    }

    #[inline]
    pub fn max(self, other: Self) -> Self {
        Self::new(self.x.max(other.x), self.y.max(other.y), self.z.max(other.z))
    }

    #[inline]
    pub fn maxi(self, scalar: i32) -> Self {
        Self::new(self.x.max(scalar), self.y.max(scalar), self.z.max(scalar))
    }

    /// 64-bit accumulation, as in the C++ version (int64_t return).
    #[inline]
    pub fn length_squared(self) -> i64 {
        self.x as i64 * self.x as i64
            + self.y as i64 * self.y as i64
            + self.z as i64 * self.z as i64
    }

    #[inline]
    pub fn length(self) -> f64 {
        (self.length_squared() as f64).sqrt()
    }

    #[inline]
    pub fn zero(&mut self) {
        self.x = 0;
        self.y = 0;
        self.z = 0;
    }

    #[inline]
    pub fn abs(self) -> Self {
        Self::new(self.x.abs(), self.y.abs(), self.z.abs())
    }

    /// `(x < 0) ? -1 : 1` for each component; zero counts as positive.
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
        Self::new(s(self.x), s(self.y), s(self.z))
    }

    #[inline]
    pub fn clamp(self, min: Self, max: Self) -> Self {
        Self::new(
            self.x.clamp(min.x, max.x),
            self.y.clamp(min.y, max.y),
            self.z.clamp(min.z, max.z),
        )
    }

    #[inline]
    pub fn clampi(self, min: i32, max: i32) -> Self {
        Self::new(
            self.x.clamp(min, max),
            self.y.clamp(min, max),
            self.z.clamp(min, max),
        )
    }

    #[inline]
    pub fn snapped(self, step: Self) -> Self {
        Self::new(
            crate::math::vector4i::snapped_scalar(self.x, step.x),
            crate::math::vector4i::snapped_scalar(self.y, step.y),
            crate::math::vector4i::snapped_scalar(self.z, step.z),
        )
    }

    #[inline]
    pub fn snappedi(self, step: i32) -> Self {
        Self::new(
            crate::math::vector4i::snapped_scalar(self.x, step),
            crate::math::vector4i::snapped_scalar(self.y, step),
            crate::math::vector4i::snapped_scalar(self.z, step),
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
    pub fn to_vector3(self) -> Vector3 {
        Vector3::new(self.x as f32, self.y as f32, self.z as f32)
    }

    #[inline]
    pub fn hash(self) -> u32 {
        let h = hashfuncs::hash_murmur3_one_32(self.x as u32, hashfuncs::HASH_MURMUR3_SEED);
        let h = hashfuncs::hash_murmur3_one_32(self.y as u32, h);
        let h = hashfuncs::hash_murmur3_one_32(self.z as u32, h);
        hashfuncs::hash_fmix32(h)
    }
}

impl From<Vector3> for Vector3i {
    #[inline]
    fn from(v: Vector3) -> Self {
        Self::new(v.x as i32, v.y as i32, v.z as i32)
    }
}

impl std::ops::Index<usize> for Vector3i {
    type Output = i32;
    #[inline]
    fn index(&self, i: usize) -> &i32 {
        debug_assert!(i < 3);
        match i {
            0 => &self.x,
            1 => &self.y,
            _ => &self.z,
        }
    }
}

macro_rules! vec3i_binop {
    ($trait:ident, $method:ident, $op:tt) => {
        impl std::ops::$trait for Vector3i {
            type Output = Vector3i;
            #[inline]
            fn $method(self, o: Vector3i) -> Vector3i {
                Vector3i::new(self.x $op o.x, self.y $op o.y, self.z $op o.z)
            }
        }
        impl std::ops::$trait<i32> for Vector3i {
            type Output = Vector3i;
            #[inline]
            fn $method(self, s: i32) -> Vector3i {
                Vector3i::new(self.x $op s, self.y $op s, self.z $op s)
            }
        }
    };
}

vec3i_binop!(Add, add, +);
vec3i_binop!(Sub, sub, -);
vec3i_binop!(Mul, mul, *);
vec3i_binop!(Div, div, /);
vec3i_binop!(Rem, rem, %);

impl std::ops::Neg for Vector3i {
    type Output = Vector3i;
    #[inline]
    fn neg(self) -> Vector3i {
        Vector3i::new(-self.x, -self.y, -self.z)
    }
}

impl std::ops::AddAssign for Vector3i {
    #[inline]
    fn add_assign(&mut self, o: Vector3i) {
        self.x += o.x;
        self.y += o.y;
        self.z += o.z;
    }
}

impl std::ops::SubAssign for Vector3i {
    #[inline]
    fn sub_assign(&mut self, o: Vector3i) {
        self.x -= o.x;
        self.y -= o.y;
        self.z -= o.z;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arithmetic_and_bounds() {
        let a = Vector3i::new(1, -2, 3);
        let b = Vector3i::new(4, 5, -6);
        assert_eq!(a + b, Vector3i::new(5, 3, -3));
        assert_eq!(a - b, Vector3i::new(-3, -7, 9));
        assert_eq!(a * 2, Vector3i::new(2, -4, 6));
        assert_eq!(a.abs(), Vector3i::new(1, 2, 3));
        assert_eq!(a.sign(), Vector3i::new(1, -1, 1));
        assert_eq!(Vector3i::new(5, 0, -5).sign(), Vector3i::new(1, 0, -1));
        assert_eq!(a.min(b), Vector3i::new(1, -2, -6));
        assert_eq!(a.max(b), Vector3i::new(4, 5, 3));
    }

    #[test]
    fn length_uses_64_bit() {
        // 90000^2 * 3 overflows int32 but not int64.
        let v = Vector3i::splat(90000);
        assert_eq!(v.length_squared(), 3 * 90000i64 * 90000i64);
    }

    #[test]
    fn conversion_is_truncation() {
        let v = Vector3::new(1.9, -1.9, 0.5);
        assert_eq!(Vector3i::from(v), Vector3i::new(1, -1, 0));
    }
}
