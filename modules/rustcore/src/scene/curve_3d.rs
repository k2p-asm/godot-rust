//! `scene/resources/curve.cpp`: the numeric core of `Curve3D`.
//!
//! Same machinery as `Curve2D` with `Vector3` and one extra feature: `closed`
//! makes the last point connect back to the first. Tilts, up vectors and
//! `sample_baked_with_rotation` are not ported.
//!
//! Non-differential: `curve.cpp` is a `Resource` and pulls `ClassDB`.

use crate::math::vector3::Vector3;
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Curve3DPoint {
    pub position: Vector3,
    pub in_: Vector3,
    pub out: Vector3,
}

impl Curve3DPoint {
    pub fn new(position: Vector3) -> Self {
        Self {
            position,
            in_: Vector3::ZERO,
            out: Vector3::ZERO,
        }
    }
}

fn bezier(p0: Vector3, p1: Vector3, p2: Vector3, p3: Vector3, t: f32) -> Vector3 {
    let u = 1.0 - t;
    p0 * (u * u * u) + p1 * (3.0 * u * u * t) + p2 * (3.0 * u * t * t) + p3 * (t * t * t)
}

#[derive(Clone, Copy, Debug)]
struct Interval {
    idx: usize,
    frac: f32,
}

pub struct Curve3D {
    points: Vec<Curve3DPoint>,
    closed: bool,
    bake_interval: f32,
    baked_points: Vec<Vector3>,
    baked_dist: Vec<f32>,
    baked_max_ofs: f32,
    baked_dirty: bool,
}

impl Default for Curve3D {
    fn default() -> Self {
        Self::new()
    }
}

impl Curve3D {
    pub fn new() -> Self {
        Self {
            points: Vec::new(),
            closed: false,
            bake_interval: 0.2,
            baked_points: Vec::new(),
            baked_dist: Vec::new(),
            baked_max_ofs: 0.0,
            baked_dirty: true,
        }
    }

    pub fn get_point_count(&self) -> usize {
        self.points.len()
    }

    pub fn get_point(&self, index: usize) -> Option<Curve3DPoint> {
        self.points.get(index).copied()
    }

    pub fn add_point(&mut self, position: Vector3) -> usize {
        self.points.push(Curve3DPoint::new(position));
        self.baked_dirty = true;
        self.points.len() - 1
    }

    pub fn set_point_position(&mut self, index: usize, position: Vector3) {
        if let Some(p) = self.points.get_mut(index) {
            p.position = position;
            self.baked_dirty = true;
        }
    }

    pub fn set_point_in(&mut self, index: usize, control: Vector3) {
        if let Some(p) = self.points.get_mut(index) {
            p.in_ = control;
            self.baked_dirty = true;
        }
    }

    pub fn set_point_out(&mut self, index: usize, control: Vector3) {
        if let Some(p) = self.points.get_mut(index) {
            p.out = control;
            self.baked_dirty = true;
        }
    }

    pub fn set_closed(&mut self, closed: bool) {
        self.closed = closed;
        self.baked_dirty = true;
    }

    pub fn is_closed(&self) -> bool {
        self.closed
    }

    pub fn set_bake_interval(&mut self, interval: f32) {
        self.bake_interval = interval;
        self.baked_dirty = true;
    }

    pub fn get_bake_interval(&self) -> f32 {
        self.bake_interval
    }

    /// `Curve3D::sample`. With `closed`, the last index wraps to point 0.
    pub fn sample(&self, index: i32, offset: f32) -> Vector3 {
        let pc = self.points.len();
        if pc == 0 {
            return Vector3::ZERO;
        }

        let mut i = index;
        if i >= pc as i32 - 1 {
            if !self.closed {
                return self.points[pc - 1].position;
            }
            i = pc as i32 - 1;
        } else if i < 0 {
            return self.points[0].position;
        }

        let iu = i as usize;
        let p0 = self.points[iu].position;
        let p1 = p0 + self.points[iu].out;
        let (p3, p2) = if !self.closed || iu < pc - 1 {
            let p3 = self.points[iu + 1].position;
            (p3, p3 + self.points[iu + 1].in_)
        } else {
            let p3 = self.points[0].position;
            (p3, p3 + self.points[0].in_)
        };

        bezier(p0, p1, p2, p3, offset)
    }

    pub fn samplef(&self, findex: f32) -> Vector3 {
        let pc = self.points.len();
        if pc == 0 {
            return Vector3::ZERO;
        }
        let mut f = findex;
        if f < 0.0 {
            f = 0.0;
        } else if f >= pc as f32 {
            f = pc as f32;
        }
        self.sample(f as i32, f % 1.0)
    }

    #[allow(clippy::too_many_arguments)]
    fn bake_segment_even(
        out: &mut BTreeMap<u32, Vector3>,
        begin: f32,
        end: f32,
        a: Vector3,
        a_out: Vector3,
        b: Vector3,
        b_in: Vector3,
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

        let n = self.points.len();
        let num_intervals = if self.closed { n } else { n - 1 };

        let mut baked = vec![self.points[0].position];

        for i in 0..num_intervals {
            let a = self.points[i].position;
            let a_out = self.points[i].out;
            let j = (i + 1) % n;
            let b = self.points[j].position;
            let b_in = self.points[j].in_;

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

    pub fn get_baked_length(&mut self) -> f32 {
        if self.baked_dirty {
            self.bake();
        }
        self.baked_max_ofs
    }

    pub fn get_baked_points(&mut self) -> Vec<Vector3> {
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

    fn sample_interval(&self, interval: Interval, cubic: bool) -> Vector3 {
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

    pub fn sample_baked(&mut self, offset: f32, cubic: bool) -> Vector3 {
        if !offset.is_finite() {
            return Vector3::ZERO;
        }
        if self.baked_dirty {
            self.bake();
        }
        if self.baked_points.is_empty() {
            return Vector3::ZERO;
        }
        if self.baked_points.len() == 1 {
            return self.baked_points[0];
        }

        let offset = offset.clamp(0.0, self.get_baked_length());
        match self.find_interval(offset) {
            Some(i) => self.sample_interval(i, cubic),
            None => Vector3::ZERO,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn straight(len: f32) -> Curve3D {
        let mut c = Curve3D::new();
        c.add_point(Vector3::ZERO);
        c.add_point(Vector3::new(len, 0.0, 0.0));
        c
    }

    #[test]
    fn sampling_endpoints_and_clamping() {
        let c = straight(10.0);
        assert_eq!(c.sample(0, 0.0), Vector3::ZERO);
        assert_eq!(c.sample(0, 1.0), Vector3::new(10.0, 0.0, 0.0));
        assert_eq!(c.sample(-1, 0.5), Vector3::ZERO);
        assert_eq!(c.sample(1, 0.5), Vector3::new(10.0, 0.0, 0.0));

        let mid = c.sample(0, 0.5);
        assert!((mid - Vector3::new(5.0, 0.0, 0.0)).length() < 1e-5, "{mid:?}");
    }

    #[test]
    fn fractional_sampling() {
        let mut c = Curve3D::new();
        c.add_point(Vector3::ZERO);
        c.add_point(Vector3::new(10.0, 0.0, 0.0));
        c.add_point(Vector3::new(10.0, 0.0, 10.0));

        assert!((c.samplef(0.5) - Vector3::new(5.0, 0.0, 0.0)).length() < 1e-5);
        assert!((c.samplef(1.5) - Vector3::new(10.0, 0.0, 5.0)).length() < 1e-5);
        assert_eq!(c.samplef(-1.0), Vector3::ZERO);
    }

    #[test]
    fn baked_length_matches_the_polyline() {
        let mut c = straight(10.0);
        let len = c.get_baked_length();
        assert!((len - 10.0).abs() < 1e-3, "{len}");
        // bake_interval 0.2 splits the segment into many pieces.
        assert!(c.get_baked_points().len() > 10);
    }

    #[test]
    fn closed_curve_adds_the_return_segment() {
        // A 10x10 square path.
        let mut c = Curve3D::new();
        c.add_point(Vector3::ZERO);
        c.add_point(Vector3::new(10.0, 0.0, 0.0));
        c.add_point(Vector3::new(10.0, 0.0, 10.0));
        c.add_point(Vector3::new(0.0, 0.0, 10.0));

        assert!(!c.is_closed());
        let open_len = c.get_baked_length();
        assert!((open_len - 30.0).abs() < 1e-3, "{open_len}");

        c.set_closed(true);
        let closed_len = c.get_baked_length();
        assert!((closed_len - 40.0).abs() < 1e-2, "{closed_len}");

        // The closing segment wraps: sampling the last interval ends at the
        // first point.
        let end = c.sample(3, 1.0);
        assert!((end - Vector3::ZERO).length() < 1e-5, "{end:?}");
    }

    #[test]
    fn baked_sampling_is_monotone_on_a_straight_line() {
        let mut c = straight(10.0);
        let mut prev = -1.0;
        for i in 0..=50 {
            let p = c.sample_baked(i as f32 * 0.2, false);
            assert!(p.x >= prev - 1e-4, "not monotone at {i}");
            prev = p.x;
            assert!(p.y.abs() < 1e-5 && p.z.abs() < 1e-5);
        }
    }

    #[test]
    fn control_points_bulge_the_curve() {
        let mut c = Curve3D::new();
        c.add_point(Vector3::ZERO);
        c.add_point(Vector3::new(10.0, 0.0, 0.0));
        c.set_point_out(0, Vector3::new(0.0, 10.0, 0.0));

        let mid = c.sample(0, 0.5);
        assert!(mid.y > 1.0, "{mid:?}");
        let len = c.get_baked_length();
        assert!(len > 10.1, "{len}");
    }

    #[test]
    fn empty_and_single_point() {
        let mut c = Curve3D::new();
        assert_eq!(c.get_baked_length(), 0.0);
        assert_eq!(c.sample_baked(1.0, false), Vector3::ZERO);

        c.add_point(Vector3::new(1.0, 2.0, 3.0));
        assert_eq!(c.get_baked_length(), 0.0);
        assert_eq!(c.sample_baked(5.0, true), Vector3::new(1.0, 2.0, 3.0));
    }
}
