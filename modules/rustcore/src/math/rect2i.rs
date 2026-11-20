//! Port of `core/math/rect2i.h` (integer rectangle).
//!
//! Pending: `intersects_filled_polygon`, `intersects_segment` and the
//! `Rect2` conversion.

use crate::math::vector2i::Vector2i;
use crate::templates::hashfuncs;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Rect2i {
    pub position: Vector2i,
    pub size: Vector2i,
}

impl Rect2i {
    #[inline]
    pub const fn new(position: Vector2i, size: Vector2i) -> Self {
        Self { position, size }
    }

    #[inline]
    pub const fn new_xywh(x: i32, y: i32, w: i32, h: i32) -> Self {
        Self { position: Vector2i::new(x, y), size: Vector2i::new(w, h) }
    }

    #[inline]
    pub fn get_position(&self) -> Vector2i {
        self.position
    }

    #[inline]
    pub fn set_position(&mut self, position: Vector2i) {
        self.position = position;
    }

    #[inline]
    pub fn get_size(&self) -> Vector2i {
        self.size
    }

    #[inline]
    pub fn set_size(&mut self, size: Vector2i) {
        self.size = size;
    }

    #[inline]
    pub fn get_area(&self) -> i32 {
        self.size.x * self.size.y
    }

    #[inline]
    pub fn get_center(&self) -> Vector2i {
        self.position + (self.size / 2)
    }

    #[inline]
    pub fn has_area(&self) -> bool {
        self.size.x > 0 && self.size.y > 0
    }

    pub fn intersects(&self, other: &Rect2i) -> bool {
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
        true
    }

    #[inline]
    pub fn encloses(&self, other: &Rect2i) -> bool {
        other.position.x >= self.position.x
            && other.position.y >= self.position.y
            && (other.position.x + other.size.x) <= (self.position.x + self.size.x)
            && (other.position.y + other.size.y) <= (self.position.y + self.size.y)
    }

    pub fn intersection(&self, other: &Rect2i) -> Self {
        if !self.intersects(other) {
            return Self::default();
        }
        let position = other.position.max(self.position);
        let other_end = other.position + other.size;
        let end = self.position + self.size;
        Self { position, size: other_end.min(end) - position }
    }

    pub fn merge(&self, other: &Rect2i) -> Self {
        let position = other.position.min(self.position);
        let far = (other.position + other.size).max(self.position + self.size);
        Self { position, size: far - position }
    }

    #[inline]
    pub fn has_point(&self, point: Vector2i) -> bool {
        if point.x < self.position.x || point.y < self.position.y {
            return false;
        }
        if point.x >= self.position.x + self.size.x || point.y >= self.position.y + self.size.y {
            return false;
        }
        true
    }

    pub fn grow(&self, amount: i32) -> Self {
        Self {
            position: Vector2i::new(self.position.x - amount, self.position.y - amount),
            size: Vector2i::new(self.size.x + amount * 2, self.size.y + amount * 2),
        }
    }

    pub fn grow_individual(&self, left: i32, top: i32, right: i32, bottom: i32) -> Self {
        Self {
            position: Vector2i::new(self.position.x - left, self.position.y - top),
            size: Vector2i::new(self.size.x + left + right, self.size.y + top + bottom),
        }
    }

    pub fn expand(&self, point: Vector2i) -> Self {
        let mut r = *self;
        r.expand_to(point);
        r
    }

    pub fn expand_to(&mut self, point: Vector2i) {
        let mut begin = self.position;
        let mut end = self.position + self.size;

        if point.x < begin.x {
            begin.x = point.x;
        }
        if point.y < begin.y {
            begin.y = point.y;
        }
        if point.x > end.x {
            end.x = point.x;
        }
        if point.y > end.y {
            end.y = point.y;
        }

        self.position = begin;
        self.size = end - begin;
    }

    #[inline]
    pub fn abs(&self) -> Self {
        Self::new(self.position + self.size.mini(0), self.size.abs())
    }

    #[inline]
    pub fn get_support(&self, dir: Vector2i) -> Vector2i {
        let mut support = self.position;
        if dir.x > 0 {
            support.x += self.size.x;
        }
        if dir.y > 0 {
            support.y += self.size.y;
        }
        support
    }

    #[inline]
    pub fn hash(&self) -> u32 {
        let h = hashfuncs::hash_murmur3_one_32(self.position.x as u32, hashfuncs::HASH_MURMUR3_SEED);
        let h = hashfuncs::hash_murmur3_one_32(self.position.y as u32, h);
        let h = hashfuncs::hash_murmur3_one_32(self.size.x as u32, h);
        let h = hashfuncs::hash_murmur3_one_32(self.size.y as u32, h);
        hashfuncs::hash_fmix32(h)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn overlap_and_merge() {
        let a = Rect2i::new_xywh(0, 0, 4, 4);
        let b = Rect2i::new_xywh(4, 0, 4, 4);
        assert!(!a.intersects(&b));
        assert!(a.intersects(&Rect2i::new_xywh(2, 2, 4, 4)));
        assert_eq!(a.merge(&b), Rect2i::new_xywh(0, 0, 8, 4));
    }

    #[test]
    fn points_and_intersection() {
        let a = Rect2i::new_xywh(0, 0, 4, 4);
        assert!(a.has_point(Vector2i::new(3, 3)));
        assert!(!a.has_point(Vector2i::new(4, 3)));
        assert_eq!(
            a.intersection(&Rect2i::new_xywh(2, 2, 4, 4)),
            Rect2i::new_xywh(2, 2, 2, 2)
        );
        assert_eq!(a.get_center(), Vector2i::new(2, 2));
        assert_eq!(a.get_area(), 16);
    }

    #[test]
    fn grow_and_expand() {
        let a = Rect2i::new_xywh(1, 1, 2, 2);
        assert_eq!(a.grow(1), Rect2i::new_xywh(0, 0, 4, 4));
        assert_eq!(a.expand(Vector2i::new(-1, 5)), Rect2i::new_xywh(-1, 1, 4, 4));
        assert_eq!(Rect2i::new_xywh(2, 2, -2, -2).abs(), Rect2i::new_xywh(0, 0, 2, 2));
    }
}
