//! Port of `core/math/rect2.h` + `rect2.cpp` (floating point variant).
//!
//! Pending from the original: `intersects_transformed` (needs `Transform2D`) and
//! `intersects_filled_polygon` (needs `Vector2.orthogonal` polygon walking).
//! `Rect2i` is not ported yet.

use crate::math::transform_2d::Transform2D;
use crate::math::vector2::Vector2;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Rect2 {
    pub position: Vector2,
    pub size: Vector2,
}

impl Rect2 {
    #[inline]
    pub const fn new(position: Vector2, size: Vector2) -> Self {
        Self { position, size }
    }

    /// `Rect2(x, y, w, h)` constructor.
    #[inline]
    pub const fn new_xywh(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self { position: Vector2::new(x, y), size: Vector2::new(w, h) }
    }

    #[inline]
    pub fn get_position(&self) -> Vector2 {
        self.position
    }

    #[inline]
    pub fn set_position(&mut self, pos: Vector2) {
        self.position = pos;
    }

    #[inline]
    pub fn get_size(&self) -> Vector2 {
        self.size
    }

    #[inline]
    pub fn set_size(&mut self, size: Vector2) {
        self.size = size;
    }

    #[inline]
    pub fn get_area(&self) -> f32 {
        self.size.x * self.size.y
    }

    #[inline]
    pub fn get_center(&self) -> Vector2 {
        self.position + (self.size * 0.5)
    }

    #[inline]
    pub fn has_area(&self) -> bool {
        self.size.x > 0.0 && self.size.y > 0.0
    }

    pub fn intersects(&self, other: &Rect2, include_borders: bool) -> bool {
        if include_borders {
            if self.position.x > other.position.x + other.size.x {
                return false;
            }
            if self.position.x + self.size.x < other.position.x {
                return false;
            }
            if self.position.y > other.position.y + other.size.y {
                return false;
            }
            if self.position.y + self.size.y < other.position.y {
                return false;
            }
        } else {
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
        }
        true
    }

    pub fn distance_to(&self, point: Vector2) -> f32 {
        let mut dist = 0.0f32;
        let mut inside = true;

        if point.x < self.position.x {
            dist = self.position.x - point.x;
            inside = false;
        }
        if point.y < self.position.y {
            let d = self.position.y - point.y;
            dist = if inside { d } else { dist.min(d) };
            inside = false;
        }
        if point.x >= self.position.x + self.size.x {
            let d = point.x - (self.position.x + self.size.x);
            dist = if inside { d } else { dist.min(d) };
            inside = false;
        }
        if point.y >= self.position.y + self.size.y {
            let d = point.y - (self.position.y + self.size.y);
            dist = if inside { d } else { dist.min(d) };
            inside = false;
        }

        if inside {
            0.0
        } else {
            dist
        }
    }

    #[inline]
    pub fn encloses(&self, other: &Rect2) -> bool {
        other.position.x >= self.position.x
            && other.position.y >= self.position.y
            && (other.position.x + other.size.x) <= (self.position.x + self.size.x)
            && (other.position.y + other.size.y) <= (self.position.y + self.size.y)
    }

    pub fn intersection(&self, other: &Rect2) -> Self {
        if !self.intersects(other, false) {
            return Self::default();
        }

        let position = other.position.max(self.position);
        let other_end = other.position + other.size;
        let end = self.position + self.size;
        let size = other_end.min(end) - position;

        Self { position, size }
    }

    pub fn merge(&self, other: &Rect2) -> Self {
        let position = other.position.min(self.position);
        let far = (other.position + other.size).max(self.position + self.size);
        Self { position, size: far - position }
    }

    #[inline]
    pub fn has_point(&self, point: Vector2) -> bool {
        if point.x < self.position.x || point.y < self.position.y {
            return false;
        }
        if point.x >= self.position.x + self.size.x || point.y >= self.position.y + self.size.y {
            return false;
        }
        true
    }

    #[inline]
    pub fn is_equal_approx(&self, other: &Rect2) -> bool {
        self.position.is_equal_approx(other.position) && self.size.is_equal_approx(other.size)
    }

    #[inline]
    pub fn is_same(&self, other: &Rect2) -> bool {
        self.position.is_same(other.position) && self.size.is_same(other.size)
    }

    #[inline]
    pub fn is_finite(&self) -> bool {
        self.position.is_finite() && self.size.is_finite()
    }

    pub fn grow(&self, amount: f32) -> Self {
        let mut g = *self;
        g.grow_by(amount);
        g
    }

    #[inline]
    pub fn grow_by(&mut self, amount: f32) {
        self.position.x -= amount;
        self.position.y -= amount;
        self.size.x += amount * 2.0;
        self.size.y += amount * 2.0;
    }

    pub fn grow_individual(&self, left: f32, top: f32, right: f32, bottom: f32) -> Self {
        let mut g = *self;
        g.position.x -= left;
        g.position.y -= top;
        g.size.x += left + right;
        g.size.y += top + bottom;
        g
    }

    pub fn expand(&self, point: Vector2) -> Self {
        let mut r = *self;
        r.expand_to(point);
        r
    }

    #[inline]
    pub fn expand_to(&mut self, point: Vector2) {
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
        Self::new(self.position + self.size.minf(0.0), self.size.abs())
    }

    #[inline]
    pub fn round(&self) -> Self {
        Self::new(self.position.round(), self.size.round())
    }

    #[inline]
    pub fn get_support(&self, dir: Vector2) -> Vector2 {
        let mut support = self.position;
        if dir.x > 0.0 {
            support.x += self.size.x;
        }
        if dir.y > 0.0 {
            support.y += self.size.y;
        }
        support
    }

    /// SAT test between this rectangle and `rect` transformed by `xform`
    /// (`Rect2::intersects_transformed`). The early rejects come first in the
    /// engine to skip the SAT work in the common separated case.
    pub fn intersects_transformed(&self, xform: &Transform2D, rect: &Rect2) -> bool {
        let p = rect.position;
        let s = rect.size;
        let xf = [
            xform.xform(p),
            xform.xform(Vector2::new(p.x + s.x, p.y)),
            xform.xform(Vector2::new(p.x, p.y + s.y)),
            xform.xform(Vector2::new(p.x + s.x, p.y + s.y)),
        ];

        // Base rect first (cheaper): bail out when every corner is beyond an edge.
        if xf.iter().all(|v| v.y <= self.position.y) {
            return false;
        }
        if xf.iter().all(|v| v.y >= self.position.y + self.size.y) {
            return false;
        }
        if xf.iter().all(|v| v.x <= self.position.x) {
            return false;
        }
        if xf.iter().all(|v| v.x >= self.position.x + self.size.x) {
            return false;
        }

        let local = [
            self.position,
            Vector2::new(self.position.x + self.size.x, self.position.y),
            Vector2::new(self.position.x, self.position.y + self.size.y),
            Vector2::new(self.position.x + self.size.x, self.position.y + self.size.y),
        ];

        // Separate axis test on both basis axes.
        for axis in 0..2 {
            let dir = xform.columns[axis];

            let mut min_a = dir.dot(local[0]);
            let mut max_a = min_a;
            for v in local.iter().skip(1) {
                let dp = dir.dot(*v);
                max_a = max_a.max(dp);
                min_a = min_a.min(dp);
            }

            let mut min_b = dir.dot(xf[0]);
            let mut max_b = min_b;
            for v in xf.iter().skip(1) {
                let dp = dir.dot(*v);
                max_b = max_b.max(dp);
                min_b = min_b.min(dp);
            }

            if min_a > max_b {
                return false;
            }
            if min_b > max_a {
                return false;
            }
        }

        true
    }
    /// True when the filled polygon overlaps the rectangle.
    /// Port of `Rect2::intersects_filled_polygon`.
    pub fn intersects_filled_polygon(&self, points: &[Vector2]) -> bool {
        let center = self.get_center();
        let mut side_plus = 0i32;
        let mut side_minus = 0i32;
        let end = self.position + self.size;

        let mut i_f = points.len().wrapping_sub(1);
        for i in 0..points.len() {
            let a = points[i_f];
            let b = points[i];
            i_f = i;

            let mut r = b - a;
            let l = r.length();
            if l == 0.0 {
                continue;
            }

            // Inside test.
            let tg = r.orthogonal();
            let s = tg.dot(center) - tg.dot(a);
            if s < 0.0 {
                side_plus += 1;
            } else {
                side_minus += 1;
            }

            // Ray-box test.
            r /= l;
            let ir = Vector2::new(1.0 / r.x, 1.0 / r.y);
            let t13 = (self.position - a) * ir;
            let t24 = (end - a) * ir;

            let tmin = t13.x.min(t24.x).max(t13.y.min(t24.y));
            let tmax = t13.x.max(t24.x).min(t13.y.max(t24.y));

            if tmax < 0.0 || tmin > tmax || tmin >= l {
                continue;
            }

            return true;
        }

        side_plus * side_minus == 0
    }
    /// Segment/rectangle clipping (Liang-Barsky), as in `rect2.cpp`.
    pub fn intersects_segment(&self, from: Vector2, to: Vector2) -> Option<Segment2Hit> {
        let mut min = 0.0f32;
        let mut max = 1.0f32;
        let mut sign = Vector2::ZERO;

        let p = self.position;
        let s = self.size;

        // X slab.
        let rel = to.x - from.x;
        if rel == 0.0 {
            if from.x < p.x || from.x > p.x + s.x {
                return None;
            }
        } else {
            let mut cmin = (p.x - from.x) / rel;
            let mut cmax = ((p.x + s.x) - from.x) / rel;
            let mut csign = Vector2::new(-1.0, 0.0);
            if cmin > cmax {
                std::mem::swap(&mut cmin, &mut cmax);
                csign = Vector2::new(1.0, 0.0);
            }
            if cmin > min {
                min = cmin;
                sign = csign;
            }
            if cmax < max {
                max = cmax;
            }
            if max < min {
                return None;
            }
        }

        // Y slab.
        let rel = to.y - from.y;
        if rel == 0.0 {
            if from.y < p.y || from.y > p.y + s.y {
                return None;
            }
        } else {
            let mut cmin = (p.y - from.y) / rel;
            let mut cmax = ((p.y + s.y) - from.y) / rel;
            let mut csign = Vector2::new(0.0, -1.0);
            if cmin > cmax {
                std::mem::swap(&mut cmin, &mut cmax);
                csign = Vector2::new(0.0, 1.0);
            }
            if cmin > min {
                min = cmin;
                sign = csign;
            }
            if cmax < max {
                max = cmax;
            }
            if max < min {
                return None;
            }
        }

        let point = from + (to - from) * min;
        Some(Segment2Hit { point, normal: sign, t: min })
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Segment2Hit {
    pub point: Vector2,
    pub normal: Vector2,
    pub t: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn r(x: f32, y: f32, w: f32, h: f32) -> Rect2 {
        Rect2::new_xywh(x, y, w, h)
    }

    #[test]
    fn intersections() {
        let a = r(0.0, 0.0, 2.0, 2.0);
        let b = r(2.0, 0.0, 2.0, 2.0);
        assert!(!a.intersects(&b, false));
        assert!(a.intersects(&b, true));
        assert!(a.intersects(&r(1.0, 1.0, 2.0, 2.0), false));
        let i = a.intersection(&r(1.0, 1.0, 2.0, 2.0));
        assert_eq!(i, r(1.0, 1.0, 1.0, 1.0));
        assert_eq!(a.intersection(&b), Rect2::default());
    }

    #[test]
    fn merge_grow_expand() {
        let a = r(0.0, 0.0, 2.0, 2.0);
        assert_eq!(a.merge(&r(3.0, 3.0, 1.0, 1.0)), r(0.0, 0.0, 4.0, 4.0));
        assert_eq!(a.grow(1.0), r(-1.0, -1.0, 4.0, 4.0));
        assert_eq!(a.grow_individual(1.0, 0.0, 0.0, 0.0), r(-1.0, 0.0, 3.0, 2.0));
        assert_eq!(a.expand(Vector2::new(-1.0, 3.0)), r(-1.0, 0.0, 3.0, 3.0));
    }

    #[test]
    fn points_and_distance() {
        let a = r(0.0, 0.0, 2.0, 2.0);
        assert!(a.has_point(Vector2::new(1.0, 1.0)));
        assert!(!a.has_point(Vector2::new(2.0, 1.0))); // right edge excluded
        assert_eq!(a.distance_to(Vector2::new(1.0, 1.0)), 0.0);
        assert_eq!(a.distance_to(Vector2::new(3.0, 1.0)), 1.0);
        assert_eq!(a.distance_to(Vector2::new(3.0, 3.0)), 1.0);
    }

    #[test]
    fn negative_size_abs() {
        let a = r(2.0, 2.0, -2.0, -2.0);
        assert_eq!(a.abs(), r(0.0, 0.0, 2.0, 2.0));
        assert!(!a.has_area());
    }
}
