//! `scene/resources/curve.*`: the numeric core of `Curve` (1D).
//!
//! Points are `(offset, value)` pairs with a left/right tangent and tangent
//! mode. `sample` evaluates the cubic Bézier between two points with control
//! points placed at a third of the interval, exactly as the engine builds them;
//! `sample_baked` reads a uniformly sampled cache with linear interpolation.
//!
//! Not ported: the `Resource` plumbing, signals, the editor's property list, and
//! `Curve2D`/`Curve3D`. Non-differential: `curve.cpp` is a `Resource` and pulls
//! `ClassDB`, so the parity harness cannot link it. The tests pin the properties
//! that define the curve instead.

use crate::math::funcs;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TangentMode {
    Free,
    Linear,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CurvePoint {
    pub position: crate::math::vector2::Vector2,
    pub left_tangent: f32,
    pub right_tangent: f32,
    pub left_mode: TangentMode,
    pub right_mode: TangentMode,
}

impl CurvePoint {
    pub fn new(position: crate::math::vector2::Vector2) -> Self {
        Self {
            position,
            left_tangent: 0.0,
            right_tangent: 0.0,
            left_mode: TangentMode::Free,
            right_mode: TangentMode::Free,
        }
    }
}

#[derive(Debug)]
pub struct Curve {
    points: Vec<CurvePoint>,
    min_value: f32,
    max_value: f32,
    min_domain: f32,
    max_domain: f32,
    bake_resolution: i32,
    baked: Vec<f32>,
    baked_dirty: bool,
}

impl Default for Curve {
    fn default() -> Self {
        Self::new()
    }
}

impl Curve {
    pub fn new() -> Self {
        Self {
            points: Vec::new(),
            min_value: 0.0,
            max_value: 1.0,
            min_domain: 0.0,
            max_domain: 1.0,
            bake_resolution: 100,
            baked: Vec::new(),
            baked_dirty: true,
        }
    }

    pub fn get_point_count(&self) -> usize {
        self.points.len()
    }

    pub fn get_point(&self, index: usize) -> Option<CurvePoint> {
        self.points.get(index).copied()
    }

    pub fn clear_points(&mut self) {
        self.points.clear();
        self.mark_dirty();
    }

    fn mark_dirty(&mut self) {
        self.baked_dirty = true;
    }

    /// `get_index`: lower-bound binary search over the point offsets. The
    /// out-of-bounds case returns the last index, like the engine.
    pub fn get_index(&self, offset: f32) -> usize {
        let n = self.points.len();
        let mut imin = 0usize;
        let mut imax = n - 1;

        while imax - imin > 1 {
            let m = (imin + imax) / 2;
            let a = self.points[m].position.x;
            let b = self.points[m + 1].position.x;

            if a < offset && b < offset {
                imin = m;
            } else if a > offset {
                imax = m;
            } else {
                return m;
            }
        }

        if offset > self.points[imax].position.x {
            imax
        } else {
            imin
        }
    }

    /// `_add_point`: clamps into the value/domain ranges and keeps the points
    /// ordered by offset.
    pub fn add_point(&mut self, position: crate::math::vector2::Vector2) -> usize {
        let mut position = position;
        position.x = position.x.clamp(self.min_domain, self.max_domain);
        position.y = position.y.clamp(self.min_value, self.max_value);

        let ret = if self.points.is_empty() {
            self.points.push(CurvePoint::new(position));
            0
        } else if self.points.len() == 1 {
            if position.x - self.points[0].position.x > 0.0 {
                self.points.push(CurvePoint::new(position));
                1
            } else {
                self.points.insert(0, CurvePoint::new(position));
                0
            }
        } else {
            let i = self.get_index(position.x);
            if i == 0 && position.x < self.points[0].position.x {
                self.points.insert(0, CurvePoint::new(position));
                0
            } else {
                self.points.insert(i + 1, CurvePoint::new(position));
                i + 1
            }
        };

        self.update_auto_tangents(ret);
        self.mark_dirty();
        ret
    }

    pub fn remove_point(&mut self, index: usize) {
        if index < self.points.len() {
            self.points.remove(index);
            self.mark_dirty();
        }
    }

    pub fn set_point_value(&mut self, index: usize, value: f32) {
        if let Some(p) = self.points.get_mut(index) {
            p.position.y = value.clamp(self.min_value, self.max_value);
            self.mark_dirty();
        }
    }

    /// `set_point_offset` re-sorts the point, which can move its index;
    /// the new index is returned.
    pub fn set_point_offset(&mut self, index: usize, offset: f32) -> usize {
        if index >= self.points.len() {
            return index;
        }
        let mut p = self.points.remove(index);
        p.position.x = offset.clamp(self.min_domain, self.max_domain);
        let new_index = if self.points.is_empty() {
            self.points.push(p);
            0
        } else if self.points.len() == 1 {
            if p.position.x - self.points[0].position.x > 0.0 {
                self.points.push(p);
                1
            } else {
                self.points.insert(0, p);
                0
            }
        } else {
            let i = self.get_index(p.position.x);
            let at = if i == 0 && p.position.x < self.points[0].position.x {
                0
            } else {
                i + 1
            };
            self.points.insert(at, p);
            at
        };
        self.update_auto_tangents(new_index);
        self.mark_dirty();
        new_index
    }

    pub fn set_point_left_tangent(&mut self, index: usize, tangent: f32) {
        if let Some(p) = self.points.get_mut(index) {
            p.left_tangent = tangent;
            p.left_mode = TangentMode::Free;
            self.mark_dirty();
        }
    }

    pub fn set_point_right_tangent(&mut self, index: usize, tangent: f32) {
        if let Some(p) = self.points.get_mut(index) {
            p.right_tangent = tangent;
            p.right_mode = TangentMode::Free;
            self.mark_dirty();
        }
    }

    pub fn set_point_left_mode(&mut self, index: usize, mode: TangentMode) {
        if index >= self.points.len() {
            return;
        }
        self.points[index].left_mode = mode;
        if index > 0 && mode == TangentMode::Linear {
            let v = (self.points[index - 1].position - self.points[index].position).normalized();
            self.points[index].left_tangent = v.y / v.x;
        }
        self.mark_dirty();
    }

    pub fn set_point_right_mode(&mut self, index: usize, mode: TangentMode) {
        if index >= self.points.len() {
            return;
        }
        self.points[index].right_mode = mode;
        if index + 1 < self.points.len() && mode == TangentMode::Linear {
            let v = (self.points[index + 1].position - self.points[index].position).normalized();
            self.points[index].right_tangent = v.y / v.x;
        }
        self.mark_dirty();
    }

    /// `update_auto_tangents`: linear tangents depend on the neighbours, so a
    /// point insertion or move updates up to four tangents.
    pub fn update_auto_tangents(&mut self, index: usize) {
        let n = self.points.len();
        if index >= n {
            return;
        }

        if index > 0 {
            if self.points[index].left_mode == TangentMode::Linear {
                let v = (self.points[index - 1].position - self.points[index].position).normalized();
                self.points[index].left_tangent = v.y / v.x;
            }
            if self.points[index - 1].right_mode == TangentMode::Linear {
                let v = (self.points[index - 1].position - self.points[index].position).normalized();
                self.points[index - 1].right_tangent = v.y / v.x;
            }
        }

        if index + 1 < n {
            if self.points[index].right_mode == TangentMode::Linear {
                let v = (self.points[index + 1].position - self.points[index].position).normalized();
                self.points[index].right_tangent = v.y / v.x;
            }
            if self.points[index + 1].left_mode == TangentMode::Linear {
                let v = (self.points[index + 1].position - self.points[index].position).normalized();
                self.points[index + 1].left_tangent = v.y / v.x;
            }
        }
    }

    pub fn get_min_value(&self) -> f32 {
        self.min_value
    }

    pub fn get_max_value(&self) -> f32 {
        self.max_value
    }

    pub fn get_min_domain(&self) -> f32 {
        self.min_domain
    }

    pub fn get_max_domain(&self) -> f32 {
        self.max_domain
    }

    pub fn get_domain_range(&self) -> f32 {
        self.max_domain - self.min_domain
    }

    /// `set_min_value` also pulls the bound down to the lowest point, so the
    /// points always stay inside the value range.
    pub fn set_min_value(&mut self, min: f32) {
        let mut v = min.min(self.max_value - 0.01);
        for p in &self.points {
            v = v.min(p.position.y);
        }
        self.min_value = v;
    }

    pub fn set_max_value(&mut self, max: f32) {
        let mut v = max.max(self.min_value + 0.01);
        for p in &self.points {
            v = v.max(p.position.y);
        }
        self.max_value = v;
    }

    pub fn set_min_domain(&mut self, min: f32) {
        let mut v = min.min(self.max_domain - 0.01);
        if let Some(first) = self.points.first() {
            v = v.min(first.position.x);
        }
        self.min_domain = v;
        self.mark_dirty();
    }

    pub fn set_max_domain(&mut self, max: f32) {
        let mut v = max.max(self.min_domain + 0.01);
        if let Some(last) = self.points.last() {
            v = v.max(last.position.x);
        }
        self.max_domain = v;
        self.mark_dirty();
    }

    /// `sample`: clamp outside the domain, evaluate the segment otherwise.
    pub fn sample(&self, offset: f32) -> f32 {
        if self.points.is_empty() {
            return 0.0;
        }
        if self.points.len() == 1 {
            return self.points[0].position.y;
        }

        let i = self.get_index(offset);
        if i == self.points.len() - 1 {
            return self.points[i].position.y;
        }

        let local = offset - self.points[i].position.x;
        if i == 0 && local <= 0.0 {
            return self.points[0].position.y;
        }

        self.sample_local_nocheck(i, local)
    }

    pub fn sample_local_nocheck(&self, index: usize, local_offset: f32) -> f32 {
        let a = self.points[index];
        let b = self.points[index + 1];

        // Control points sit one third of the interval away.
        let mut d = b.position.x - a.position.x;
        if funcs::is_zero_approx(d) {
            return b.position.y;
        }
        let t = local_offset / d;
        d /= 3.0;
        let yac = a.position.y + d * a.right_tangent;
        let ybc = b.position.y - d * b.left_tangent;

        funcs::bezier_interpolate(a.position.y, yac, ybc, b.position.y, t)
    }

    pub fn set_bake_resolution(&mut self, resolution: i32) -> bool {
        if !(1..=1000).contains(&resolution) {
            return false;
        }
        self.bake_resolution = resolution;
        self.baked_dirty = true;
        true
    }

    pub fn get_bake_resolution(&self) -> i32 {
        self.bake_resolution
    }

    /// `_bake`: uniform samples, with the first and last replaced by the exact
    /// endpoint values.
    pub fn bake(&mut self) {
        self.baked.clear();
        self.baked.resize(self.bake_resolution.max(1) as usize, 0.0);

        let res = self.bake_resolution.max(1) as usize;
        for i in 1..res.saturating_sub(1) {
            let x = self.get_domain_range() * i as f32 / (res - 1) as f32 + self.min_domain;
            self.baked[i] = self.sample(x);
        }

        if !self.points.is_empty() {
            self.baked[0] = self.points[0].position.y;
            let last = self.points.len() - 1;
            self.baked[res - 1] = self.points[last].position.y;
        }

        self.baked_dirty = false;
    }

    pub fn sample_baked(&mut self, offset: f32) -> f32 {
        if !offset.is_finite() {
            return 0.0;
        }
        if self.baked_dirty {
            self.bake();
        }

        if self.baked.is_empty() {
            if self.points.is_empty() {
                return 0.0;
            }
            return self.points[0].position.y;
        }
        if self.baked.len() == 1 {
            return self.baked[0];
        }

        let range = self.get_domain_range();
        let fi = (offset - self.min_domain) / range * (self.baked.len() - 1) as f32;
        let mut i = fi.floor();
        let mut fi = fi;
        if i < 0.0 {
            i = 0.0;
            fi = 0.0;
        } else if i >= self.baked.len() as f32 {
            i = (self.baked.len() - 1) as f32;
            fi = 0.0;
        }

        let idx = i as usize;
        if idx + 1 < self.baked.len() {
            let t = fi - i;
            funcs::lerp(self.baked[idx], self.baked[idx + 1], t)
        } else {
            self.baked[self.baked.len() - 1]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::vector2::Vector2;

    fn line_curve() -> Curve {
        let mut c = Curve::new();
        c.add_point(Vector2::new(0.0, 0.0));
        c.add_point(Vector2::new(1.0, 1.0));
        c
    }

    #[test]
    fn empty_and_single_point() {
        let mut c = Curve::new();
        assert_eq!(c.sample(0.5), 0.0);
        assert_eq!(c.sample_baked(0.5), 0.0);

        c.add_point(Vector2::new(0.5, 0.25));
        assert_eq!(c.sample(0.0), 0.25);
        assert_eq!(c.sample(0.9), 0.25);
        assert_eq!(c.sample_baked(0.1), 0.25);
    }

    #[test]
    fn points_are_kept_in_offset_order() {
        let mut c = Curve::new();
        assert_eq!(c.add_point(Vector2::new(0.5, 1.0)), 0);
        assert_eq!(c.add_point(Vector2::new(0.0, 0.0)), 0);
        assert_eq!(c.add_point(Vector2::new(1.0, 2.0)), 2);
        assert_eq!(c.add_point(Vector2::new(0.25, 0.5)), 1);

        let xs: Vec<f32> = (0..c.get_point_count()).map(|i| c.get_point(i).unwrap().position.x).collect();
        assert_eq!(xs, vec![0.0, 0.25, 0.5, 1.0]);
    }

    #[test]
    fn zero_tangents_give_the_bezier_s_flat_ends() {
        let c = line_curve();
        // With zero tangents the curve is an ease-in/ease-out, not a straight
        // line: the midpoint is still 0.5 by symmetry, but quarter points lag.
        assert!((c.sample(0.5) - 0.5).abs() < 1e-6);
        let q = c.sample(0.25);
        assert!(q < 0.25, "ease-in expected, got {q}");

        // Values at the endpoints are exact.
        assert_eq!(c.sample(0.0), 0.0);
        assert_eq!(c.sample(1.0), 1.0);
    }

    #[test]
    fn linear_tangents_make_it_straight() {
        let mut c = line_curve();
        // Slope 1 on both ends of the segment.
        c.set_point_right_mode(0, TangentMode::Linear);
        c.set_point_left_mode(1, TangentMode::Linear);
        let p0 = c.get_point(0).unwrap();
        assert!((p0.right_tangent - 1.0).abs() < 1e-6);

        for i in 0..=10 {
            let x = i as f32 / 10.0;
            assert!((c.sample(x) - x).abs() < 1e-5, "sample({x}) = {}", c.sample(x));
        }
    }

    #[test]
    fn baked_matches_the_exact_curve_at_the_samples() {
        let mut c = line_curve();
        c.set_bake_resolution(11);
        c.bake();

        // Resolution 11 => nodes every 0.1; the midpoint must interpolate
        // between two baked samples of the same curve.
        let idx_mid = 5;
        let x = idx_mid as f32 / 10.0;
        assert!((c.sample_baked(x) - c.sample(x)).abs() < 1e-5);
        // The endpoints use the exact point values.
        assert_eq!(c.sample_baked(0.0), 0.0);
        assert_eq!(c.sample_baked(1.0), 1.0);
    }

    #[test]
    fn baked_cache_is_invalidated_by_edits() {
        let mut c = line_curve();
        c.set_bake_resolution(11);
        let before = c.sample_baked(0.5);

        c.set_point_value(0, 0.5);
        let after = c.sample_baked(0.5);
        assert!(after > before, "edit was not baked: {before} -> {after}");
    }

    #[test]
    fn domain_and_value_ranges_clamp_points() {
        let mut c = Curve::new();
        c.set_min_domain(0.0);
        c.set_max_domain(1.0);
        c.set_min_value(0.0);
        c.set_max_value(1.0);

        c.add_point(Vector2::new(-5.0, 5.0));
        let p = c.get_point(0).unwrap();
        assert_eq!(p.position.x, 0.0);
        assert_eq!(p.position.y, 1.0);

        // add_point clamps before insertion, so the point lands at (0,1).
        let mut c2 = Curve::new();
        c2.add_point(Vector2::new(0.0, 3.0));
        let p2 = c2.get_point(0).unwrap();
        assert_eq!(p2.position.y, 1.0);

        // Widening the value range is allowed...
        c2.set_min_value(-2.0);
        assert!((c2.get_min_value() + 2.0).abs() < 1e-6);
        // ...and narrowing is pulled back up to the lowest/highest point.
        c2.set_max_value(0.5);
        assert!((c2.get_max_value() - 1.0).abs() < 1e-6);
    }

    #[test]
    fn out_of_domain_offsets_clamp() {
        let c = line_curve();
        assert_eq!(c.sample(-1.0), 0.0);
        assert_eq!(c.sample(2.0), 1.0);
        let mut c = c;
        assert_eq!(c.sample_baked(-1.0), 0.0);
        assert_eq!(c.sample_baked(2.0), 1.0);
        assert_eq!(c.sample_baked(f32::NAN), 0.0);
    }

    #[test]
    fn moving_a_point_reorders_and_returns_the_new_index() {
        let mut c = Curve::new();
        c.add_point(Vector2::new(0.0, 0.0));
        c.add_point(Vector2::new(0.5, 1.0));
        c.add_point(Vector2::new(1.0, 2.0));

        let idx = c.set_point_offset(0, 0.75);
        // The point lands between 0.5 and 1.0, so its new index is 1.
        assert_eq!(idx, 1);
        let xs: Vec<f32> = (0..3).map(|i| c.get_point(i).unwrap().position.x).collect();
        assert_eq!(xs, vec![0.5, 0.75, 1.0]);
    }
}
