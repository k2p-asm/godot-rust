//! `scene/resources/curve.cpp`: the numeric core of `Curve2D`.
//!
//! Each point carries an `in` and an `out` control offset, so a segment is the
//! cubic Bézier `p0, p0+out, p1+in, p1`. The baked cache tessellates segments
//! recursively until every piece is shorter than `bake_interval`, stores
//! cumulative arc lengths, and `sample_baked` looks the offset up by binary
//! search.
//!
//! Not ported: tilts, `sample_baked_with_rotation`/posture, `Curve3D`, and the
//! `Resource` plumbing. Non-differential: `curve.cpp` pulls `ClassDB`.

use crate::math::vector2::Vector2;
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Curve2DPoint {
    pub position: Vector2,
    pub in_: Vector2,
    pub out: Vector2,
}

impl Curve2DPoint {
    pub fn new(position: Vector2) -> Self {
        Self {
            position,
            in_: Vector2::ZERO,
            out: Vector2::ZERO,
        }
    }
}

/// Cubic Bézier with control points `p1`, `p2`.
fn bezier(p0: Vector2, p1: Vector2, p2: Vector2, p3: Vector2, t: f32) -> Vector2 {
    let u = 1.0 - t;
    p0 * (u * u * u) + p1 * (3.0 * u * u * t) + p2 * (3.0 * u * t * t) + p3 * (t * t * t)
}

#[derive(Clone, Copy, Debug)]
struct Interval {
    idx: usize,
    frac: f32,
}

#[derive(Debug)]
pub struct Curve2D {
    points: Vec<Curve2DPoint>,
    bake_interval: f32,
    baked_points: Vec<Vector2>,
    baked_dist: Vec<f32>,
    baked_max_ofs: f32,
    baked_dirty: bool,
}

impl Default for Curve2D {
    fn default() -> Self {
        Self::new()
    }
}

impl Curve2D {
    pub fn new() -> Self {
        Self {
            points: Vec::new(),
            bake_interval: 5.0,
            baked_points: Vec::new(),
            baked_dist: Vec::new(),
            baked_max_ofs: 0.0,
            baked_dirty: true,
        }
    }

    pub fn get_point_count(&self) -> usize {
        self.points.len()
    }

    pub fn get_point(&self, index: usize) -> Option<Curve2DPoint> {
        self.points.get(index).copied()
    }

    pub fn add_point(&mut self, position: Vector2) -> usize {
        self.points.push(Curve2DPoint::new(position));
        self.baked_dirty = true;
        self.points.len() - 1
    }

    pub fn set_point_position(&mut self, index: usize, position: Vector2) {
        if let Some(p) = self.points.get_mut(index) {
            p.position = position;
            self.baked_dirty = true;
        }
    }

    pub fn set_point_in(&mut self, index: usize, control: Vector2) {
        if let Some(p) = self.points.get_mut(index) {
            p.in_ = control;
            self.baked_dirty = true;
        }
    }

    pub fn set_point_out(&mut self, index: usize, control: Vector2) {
        if let Some(p) = self.points.get_mut(index) {
            p.out = control;
            self.baked_dirty = true;
        }
    }

    pub fn clear_points(&mut self) {
        self.points.clear();
        self.baked_dirty = true;
    }

    pub fn set_bake_interval(&mut self, interval: f32) {
        self.bake_interval = interval;
        self.baked_dirty = true;
    }

    pub fn get_bake_interval(&self) -> f32 {
        self.bake_interval
    }

    /// `Curve2D::sample`: out-of-range indices clamp to the first/last point.
    pub fn sample(&self, index: i32, offset: f32) -> Vector2 {
        let pc = self.points.len();
        if pc == 0 {
            return Vector2::ZERO;
        }
        if index >= pc as i32 - 1 {
            return self.points[pc - 1].position;
        }
        if index < 0 {
            return self.points[0].position;
        }

        let i = index as usize;
        let p0 = self.points[i].position;
        let p1 = p0 + self.points[i].out;
        let p3 = self.points[i + 1].position;
        let p2 = p3 + self.points[i + 1].in_;

        bezier(p0, p1, p2, p3, offset)
    }

    /// `Curve2D::samplef`: fractional index, `2.5` is halfway between points 2
    /// and 3.
    pub fn samplef(&self, findex: f32) -> Vector2 {
        let pc = self.points.len();
        if pc == 0 {
            return Vector2::ZERO;
        }
        let mut f = findex;
        if f < 0.0 {
            f = 0.0;
        } else if f >= pc as f32 {
            f = pc as f32;
        }
        self.sample(f as i32, f % 1.0)
    }

    /// `_bake_segment2d_even_length`: split until the chord is short enough or
    /// the depth budget is spent.
    #[allow(clippy::too_many_arguments)]
    fn bake_segment_even(
        out: &mut BTreeMap<u32, Vector2>,
        begin: f32,
        end: f32,
        a: Vector2,
        a_out: Vector2,
        b: Vector2,
        b_in: Vector2,
        depth: i32,
        max_depth: i32,
        length: f32,
    ) {
        let beg = bezier(a, a + a_out, b + b_in, b, begin);
        let endp = bezier(a, a + a_out, b + b_in, b, end);

        if beg.distance_to(endp) > length && depth < max_depth {
            let mp = (begin + end) * 0.5;
            let mid = bezier(a, a + a_out, b + b_in, b, mp);
            out.insert(mp.to_bits(), mid);

            Self::bake_segment_even(out, begin, mp, a, a_out, b, b_in, depth + 1, max_depth, length);
            Self::bake_segment_even(out, mp, end, a, a_out, b, b_in, depth + 1, max_depth, length);
        }
    }

    /// `_bake`: cumulative arc-length table over the tessellation.
    fn bake(&mut self) {
        if !self.baked_dirty {
            return;
        }
        self.baked_max_ofs = 0.0;
        self.baked_dirty = false;

        self.baked_points.clear();
        self.baked_dist.clear();

        if self.points.is_empty() {
            return;
        }
        if self.points.len() == 1 {
            self.baked_points.push(self.points[0].position);
            self.baked_dist.push(0.0);
            return;
        }

        // Tessellate every segment, then assemble in curve order.
        let mut baked = Vec::new();
        baked.push(self.points[0].position);

        for i in 0..self.points.len() - 1 {
            let a = self.points[i].position;
            let a_out = self.points[i].out;
            let b = self.points[i + 1].position;
            let b_in = self.points[i + 1].in_;

            let mut mid = BTreeMap::new();
            Self::bake_segment_even(
                &mut mid,
                0.0,
                1.0,
                a,
                a_out,
                b,
                b_in,
                0,
                10,
                self.bake_interval,
            );
            for (_, p) in mid {
                baked.push(p);
            }
            baked.push(b);
        }

        self.baked_dist.push(0.0);
        for i in 0..baked.len() - 1 {
            let d = self.baked_dist[i] + baked[i].distance_to(baked[i + 1]);
            self.baked_dist.push(d);
        }
        self.baked_max_ofs = *self.baked_dist.last().unwrap();
        self.baked_points = baked;
    }

    /// `_bake_segment2d`: angle-based tessellation. Inserts midpoints where the
    /// curve turns more than `tolerance` degrees, recursing `max_depth` times.
    #[allow(clippy::too_many_arguments)]
    fn tessellate_segment(
        out: &mut BTreeMap<u32, Vector2>,
        begin: f32,
        end: f32,
        a: Vector2,
        a_out: Vector2,
        b: Vector2,
        b_in: Vector2,
        depth: i32,
        max_depth: i32,
        tolerance: f32,
    ) {
        let mp = begin + (end - begin) * 0.5;
        let beg = bezier(a, a + a_out, b + b_in, b, begin);
        let mid = bezier(a, a + a_out, b + b_in, b, mp);
        let endp = bezier(a, a + a_out, b + b_in, b, end);

        let na = (mid - beg).normalized();
        let nb = (endp - mid).normalized();
        let dp = na.dot(nb);

        if dp < crate::math::funcs::deg_to_rad(tolerance as f64).cos() as f32 {
            out.insert(mp.to_bits(), mid);
        }

        if depth < max_depth {
            Self::tessellate_segment(out, begin, mp, a, a_out, b, b_in, depth + 1, max_depth, tolerance);
            Self::tessellate_segment(out, mp, end, a, a_out, b, b_in, depth + 1, max_depth, tolerance);
        }
    }

    /// `tessellate`: polyline for display; `max_stages` limits the recursion and
    /// `tolerance` is in degrees.
    pub fn tessellate(&self, max_stages: i32, tolerance: f32) -> Vec<Vector2> {
        if self.points.is_empty() {
            return Vec::new();
        }
        if self.points.len() == 1 {
            return vec![self.points[0].position];
        }

        let mut out = vec![self.points[0].position];
        for i in 0..self.points.len() - 1 {
            let a = self.points[i].position;
            let a_out = self.points[i].out;
            let b = self.points[i + 1].position;
            let b_in = self.points[i + 1].in_;

            let mut mid = BTreeMap::new();
            Self::tessellate_segment(&mut mid, 0.0, 1.0, a, a_out, b, b_in, 0, max_stages, tolerance);
            for (_, p) in mid {
                out.push(p);
            }
            out.push(b);
        }
        out
    }

    /// `get_closest_point`: brute force over the baked segments, projecting the
    /// query point onto each one.
    pub fn get_closest_point(&mut self, p: Vector2) -> Vector2 {
        if self.baked_dirty {
            self.bake();
        }
        if self.baked_points.is_empty() {
            return Vector2::ZERO;
        }
        if self.baked_points.len() == 1 {
            return self.baked_points[0];
        }
        let r = &self.baked_points;
        let dist = &self.baked_dist;
        let mut nearest = Vector2::ZERO;
        let mut nearest_dist = -1.0f32;

        for i in 0..r.len() - 1 {
            let interval = dist[i + 1] - dist[i];
            let origin = r[i];
            let direction = (r[i + 1] - origin) / interval;
            let d = (p - origin).dot(direction).clamp(0.0, interval);
            let proj = origin + direction * d;
            let dsq = proj.distance_squared_to(p);
            if nearest_dist < 0.0 || dsq < nearest_dist {
                nearest = proj;
                nearest_dist = dsq;
            }
        }
        nearest
    }

    /// `get_closest_offset`: the same projection, returning the arc length.
    pub fn get_closest_offset(&mut self, p: Vector2) -> f32 {
        if self.baked_dirty {
            self.bake();
        }
        if self.baked_points.len() < 2 {
            return 0.0;
        }
        let r = &self.baked_points;
        let dist = &self.baked_dist;
        let mut nearest = 0.0f32;
        let mut nearest_dist = -1.0f32;

        for i in 0..r.len() - 1 {
            let offset = dist[i];
            let interval = dist[i + 1] - dist[i];
            let origin = r[i];
            let direction = (r[i + 1] - origin) / interval;
            let d = (p - origin).dot(direction).clamp(0.0, interval);
            let proj = origin + direction * d;
            let dsq = proj.distance_squared_to(p);
            if nearest_dist < 0.0 || dsq < nearest_dist {
                nearest = offset + d;
                nearest_dist = dsq;
            }
        }
        nearest
    }

    pub fn get_baked_length(&mut self) -> f32 {
        if self.baked_dirty {
            self.bake();
        }
        self.baked_max_ofs
    }

    pub fn get_baked_points(&mut self) -> Vec<Vector2> {
        if self.baked_dirty {
            self.bake();
        }
        self.baked_points.clone()
    }

    fn find_interval(&self, offset: f32) -> Option<Interval> {
        let pc = self.baked_points.len();
        if pc < 2 {
            return None;
        }

        let mut start = 0usize;
        let mut end = pc;
        let mut idx = (end + start) / 2;
        while start < idx {
            if offset <= self.baked_dist[idx] {
                end = idx;
            } else {
                start = idx;
            }
            idx = (end + start) / 2;
        }

        let offset_begin = self.baked_dist[idx];
        let offset_end = self.baked_dist[idx + 1];
        if offset < offset_begin || offset > offset_end {
            return None;
        }

        let span = offset_end - offset_begin;
        if span < f32::EPSILON {
            return Some(Interval { idx, frac: 0.5 });
        }
        Some(Interval {
            idx,
            frac: (offset - offset_begin) / span,
        })
    }

    fn sample_interval(&self, interval: Interval, cubic: bool) -> Vector2 {
        let idx = interval.idx;
        let frac = interval.frac;
        let r = &self.baked_points;
        let pc = r.len();

        if cubic {
            let pre = if idx > 0 { r[idx - 1] } else { r[idx] };
            let post = if idx < pc - 2 { r[idx + 2] } else { r[idx + 1] };
            r[idx].cubic_interpolate(r[idx + 1], pre, post, frac)
        } else {
            r[idx].lerp(r[idx + 1], frac)
        }
    }

    /// `sample_baked`: clamps the offset into `[0, get_baked_length()]`.
    pub fn sample_baked(&mut self, offset: f32, cubic: bool) -> Vector2 {
        if !offset.is_finite() {
            return Vector2::ZERO;
        }
        if self.baked_dirty {
            self.bake();
        }
        if self.baked_points.is_empty() {
            return Vector2::ZERO;
        }
        if self.baked_points.len() == 1 {
            return self.baked_points[0];
        }

        let offset = offset.clamp(0.0, self.get_baked_length());
        match self.find_interval(offset) {
            Some(i) => self.sample_interval(i, cubic),
            None => Vector2::ZERO,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn straight(len: f32) -> Curve2D {
        let mut c = Curve2D::new();
        c.add_point(Vector2::ZERO);
        c.add_point(Vector2::new(len, 0.0));
        c
    }

    #[test]
    fn sampling_a_segment_with_zero_controls_is_a_line() {
        let c = straight(10.0);
        assert_eq!(c.sample(0, 0.0), Vector2::ZERO);
        assert_eq!(c.sample(0, 1.0), Vector2::new(10.0, 0.0));
        // Degenerate control points: the cubic still sweeps the chord, and the
        // midpoint is the chord midpoint by symmetry.
        let mid = c.sample(0, 0.5);
        assert!((mid - Vector2::new(5.0, 0.0)).length() < 1e-5, "{mid:?}");

        // Out-of-range indices clamp.
        assert_eq!(c.sample(-1, 0.5), Vector2::ZERO);
        assert_eq!(c.sample(1, 0.5), Vector2::new(10.0, 0.0));
    }

    #[test]
    fn fractional_sampling() {
        let mut c = Curve2D::new();
        c.add_point(Vector2::ZERO);
        c.add_point(Vector2::new(10.0, 0.0));
        c.add_point(Vector2::new(10.0, 10.0));

        let p = c.samplef(0.5);
        assert!((p - Vector2::new(5.0, 0.0)).length() < 1e-5, "{p:?}");
        let p = c.samplef(1.5);
        assert!((p - Vector2::new(10.0, 5.0)).length() < 1e-5, "{p:?}");
        // Clamped ends.
        assert_eq!(c.samplef(-3.0), Vector2::ZERO);
        assert_eq!(c.samplef(9.0), Vector2::new(10.0, 10.0));
    }

    #[test]
    fn empty_and_single_point_curves() {
        let mut c = Curve2D::new();
        assert_eq!(c.get_baked_length(), 0.0);
        assert_eq!(c.sample_baked(1.0, false), Vector2::ZERO);
        assert!(c.get_baked_points().is_empty());

        c.add_point(Vector2::new(3.0, 4.0));
        assert_eq!(c.get_baked_length(), 0.0);
        assert_eq!(c.sample_baked(5.0, false), Vector2::new(3.0, 4.0));
        assert_eq!(c.get_baked_points().len(), 1);
    }

    #[test]
    fn baked_length_of_a_straight_line() {
        let mut c = straight(10.0);
        // 10 units long; the tessellation splits until chords are <= 5, so two
        // pieces, and the polyline length is exact.
        let len = c.get_baked_length();
        assert!((len - 10.0).abs() < 1e-4, "{len}");
        assert!(c.get_baked_points().len() >= 3);
    }

    #[test]
    fn baked_sampling_is_monotone_and_covers_the_curve() {
        let mut c = straight(10.0);
        let mut prev_x = -1.0;
        for i in 0..=20 {
            let p = c.sample_baked(i as f32 * 0.5, false);
            assert!(p.x >= prev_x - 1e-5, "not monotone at {i}");
            prev_x = p.x;
            assert!(p.y.abs() < 1e-5);
        }
        // Offset 0 is the start, the baked length is the end.
        let start = c.sample_baked(0.0, false);
        let len = c.get_baked_length();
        let end = c.sample_baked(len, false);
        assert!((start - Vector2::ZERO).length() < 1e-5);
        assert!((end - Vector2::new(10.0, 0.0)).length() < 1e-4);
    }

    #[test]
    fn offsets_outside_the_baked_range_clamp() {
        let mut c = straight(10.0);
        let a = c.sample_baked(-100.0, false);
        let b = c.sample_baked(100.0, false);
        assert!((a - Vector2::ZERO).length() < 1e-5, "{a:?}");
        assert!((b - Vector2::new(10.0, 0.0)).length() < 1e-4, "{b:?}");
        assert_eq!(c.sample_baked(f32::NAN, false), Vector2::ZERO);
    }

    #[test]
    fn control_points_bend_the_curve() {
        let mut c = Curve2D::new();
        c.add_point(Vector2::ZERO);
        c.add_point(Vector2::new(10.0, 0.0));
        c.set_point_out(0, Vector2::new(0.0, 10.0));

        // The curve bulges upwards; the straight chord would stay at y = 0.
        let mid = c.sample(0, 0.5);
        assert!(mid.y > 1.0, "{mid:?}");
        // Baking follows the bulge, so the arc is longer than the chord.
        let len = c.get_baked_length();
        assert!(len > 10.1, "{len}");
    }

    #[test]
    fn tessellate_straight_line_is_two_points() {
        let c = straight(10.0);
        assert_eq!(c.tessellate(5, 4.0).len(), 2);
    }

    #[test]
    fn tessellate_curved_segment_adds_points() {
        let mut c = Curve2D::new();
        c.add_point(Vector2::ZERO);
        c.add_point(Vector2::new(10.0, 0.0));
        c.set_point_out(0, Vector2::new(0.0, 10.0));
        c.set_point_in(1, Vector2::new(0.0, 10.0));

        let poly = c.tessellate(5, 4.0);
        assert!(poly.len() > 3, "got {} points", poly.len());
        // First and last are the control points.
        assert_eq!(poly[0], Vector2::ZERO);
        assert!((poly[poly.len() - 1] - Vector2::new(10.0, 0.0)).length() < 1e-5);
        // The polyline follows the bulge.
        assert!(poly.iter().any(|p| p.y > 1.0));
    }

    #[test]
    fn tessellate_without_stages_still_splits_once() {
        let mut c = Curve2D::new();
        c.add_point(Vector2::ZERO);
        c.add_point(Vector2::new(10.0, 0.0));
        c.set_point_out(0, Vector2::new(0.0, 10.0));
        c.set_point_in(1, Vector2::new(0.0, 10.0));

        // The top-level call inserts the midpoint if the turn is over the
        // tolerance, even with no recursion left.
        assert_eq!(c.tessellate(0, 4.0).len(), 3);
        // With a tolerance that no turn exceeds, nothing is inserted.
        assert_eq!(c.tessellate(0, 179.0).len(), 2);
    }

    #[test]
    fn closest_point_projects_onto_the_polyline() {
        let mut c = straight(10.0);
        // Beside the middle: projects to (5,0), offset 5.
        let p = c.get_closest_point(Vector2::new(5.0, 3.0));
        assert!((p - Vector2::new(5.0, 0.0)).length() < 1e-3, "{p:?}");
        assert!((c.get_closest_offset(Vector2::new(5.0, 3.0)) - 5.0).abs() < 1e-3);

        // Beyond the ends: clamps to the endpoints.
        assert!((c.get_closest_point(Vector2::new(20.0, 0.0)) - Vector2::new(10.0, 0.0)).length() < 1e-3);
        assert!((c.get_closest_offset(Vector2::new(20.0, 0.0)) - 10.0).abs() < 1e-3);
        assert!((c.get_closest_point(Vector2::new(-5.0, 0.0))).length() < 1e-3);
        assert!((c.get_closest_offset(Vector2::new(-5.0, 0.0))).abs() < 1e-3);
    }

    #[test]
    fn closest_point_on_empty_and_single_point() {
        let mut c = Curve2D::new();
        assert_eq!(c.get_closest_point(Vector2::ONE), Vector2::ZERO);
        assert_eq!(c.get_closest_offset(Vector2::ONE), 0.0);
        c.add_point(Vector2::new(3.0, 4.0));
        assert_eq!(c.get_closest_point(Vector2::ZERO), Vector2::new(3.0, 4.0));
        assert_eq!(c.get_closest_offset(Vector2::ZERO), 0.0);
    }

    #[test]
    fn more_curve_edits_invalidate_the_cache() {
        let mut c = straight(10.0);
        let before = c.get_baked_length();
        c.set_point_position(1, Vector2::new(20.0, 0.0));
        let after = c.get_baked_length();
        assert!((after - 20.0).abs() < 1e-4, "{before} -> {after}");
    }
}
