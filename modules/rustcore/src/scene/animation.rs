//! `scene/resources/animation.*`: tracks, keys and value interpolation.
//!
//! Ported: track creation/removal, keys sorted by time (an insert at an
//! existing time replaces that key), `track_find_key` in exact/approx/nearest
//! modes, key value/time/transition access, interpolation type and the linear
//! interpolation of numeric value tracks.
//!
//! Not ported: position/rotation/scale/bezier/method/audio/animation tracks,
//! compression, and the `Resource` plumbing.

use crate::string::string_name::StringName;
use crate::variant::{Variant, VariantType};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrackType {
    Value,
    Position3D,
    Rotation3D,
    Scale3D,
    BlendShape,
    Method,
    Bezier,
    Audio,
    Animation,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InterpolationType {
    Nearest,
    Linear,
    Cubic,
    LinearAngle,
    CubicAngle,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UpdateMode {
    Continuous,
    Discrete,
    Capture,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LoopMode {
    None,
    Linear,
    PingPong,
}

/// `Animation::FindMode`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FindMode {
    Nearest,
    Approx,
    Exact,
}

#[derive(Clone, Debug)]
pub struct TrackKey {
    pub time: f64,
    pub value: Variant,
    pub transition: f32,
}

#[derive(Clone, Debug)]
pub struct Track {
    pub track_type: TrackType,
    pub keys: Vec<TrackKey>,
    pub interpolation: InterpolationType,
    pub loop_wrap: bool,
    pub path: StringName,
}

pub struct Animation {
    tracks: Vec<Track>,
    length: f64,
    step: f64,
    loop_mode: LoopMode,
}

impl Default for Animation {
    fn default() -> Self {
        Self::new()
    }
}

impl Animation {
    pub fn new() -> Self {
        Self {
            tracks: Vec::new(),
            length: 1.0,
            step: 0.1,
            loop_mode: LoopMode::None,
        }
    }

    pub fn set_length(&mut self, length: f64) {
        self.length = length;
    }

    pub fn get_length(&self) -> f64 {
        self.length
    }

    pub fn set_step(&mut self, step: f64) {
        self.step = step;
    }

    pub fn get_step(&self) -> f64 {
        self.step
    }

    pub fn set_loop_mode(&mut self, mode: LoopMode) {
        self.loop_mode = mode;
    }

    pub fn get_loop_mode(&self) -> LoopMode {
        self.loop_mode
    }

    /// `add_track`; a negative position appends.
    pub fn add_track(&mut self, track_type: TrackType, at_pos: i32) -> usize {
        let track = Track {
            track_type,
            keys: Vec::new(),
            interpolation: InterpolationType::Linear,
            loop_wrap: true,
            path: StringName::new(""),
        };
        if at_pos < 0 || at_pos as usize >= self.tracks.len() {
            self.tracks.push(track);
            self.tracks.len() - 1
        } else {
            self.tracks.insert(at_pos as usize, track);
            at_pos as usize
        }
    }

    pub fn remove_track(&mut self, index: usize) -> bool {
        if index >= self.tracks.len() {
            return false;
        }
        self.tracks.remove(index);
        true
    }

    pub fn get_track_count(&self) -> usize {
        self.tracks.len()
    }

    pub fn get_track(&self, index: usize) -> Option<&Track> {
        self.tracks.get(index)
    }

    pub fn track_set_path(&mut self, track: usize, path: &str) -> bool {
        match self.tracks.get_mut(track) {
            Some(t) => {
                t.path = StringName::new(path);
                true
            }
            None => false,
        }
    }

    pub fn track_set_interpolation_type(
        &mut self,
        track: usize,
        interpolation: InterpolationType,
    ) -> bool {
        match self.tracks.get_mut(track) {
            Some(t) => {
                t.interpolation = interpolation;
                true
            }
            None => false,
        }
    }

    /// `track_insert_key`: keys stay sorted; an existing key at the same time is
    /// replaced and its index returned.
    pub fn track_insert_key(
        &mut self,
        track: usize,
        time: f64,
        value: Variant,
        transition: f32,
    ) -> Option<usize> {
        let t = self.tracks.get_mut(track)?;
        let key = TrackKey {
            time,
            value,
            transition,
        };

        // Binary search for the insertion point / existing key.
        let mut lo = 0usize;
        let mut hi = t.keys.len();
        while lo < hi {
            let mid = (lo + hi) / 2;
            if t.keys[mid].time < time {
                lo = mid + 1;
            } else {
                hi = mid;
            }
        }

        if lo < t.keys.len() && (t.keys[lo].time - time).abs() < f64::EPSILON {
            t.keys[lo] = key;
            return Some(lo);
        }
        t.keys.insert(lo, key);
        Some(lo)
    }

    pub fn track_get_key_count(&self, track: usize) -> usize {
        self.tracks.get(track).map(|t| t.keys.len()).unwrap_or(0)
    }

    pub fn track_get_key_time(&self, track: usize, key: usize) -> Option<f64> {
        self.tracks.get(track)?.keys.get(key).map(|k| k.time)
    }

    pub fn track_get_key_value(&self, track: usize, key: usize) -> Option<Variant> {
        self.tracks
            .get(track)?
            .keys
            .get(key)
            .map(|k| k.value.clone())
    }

    pub fn track_get_key_transition(&self, track: usize, key: usize) -> Option<f32> {
        self.tracks.get(track)?.keys.get(key).map(|k| k.transition)
    }

    pub fn track_remove_key(&mut self, track: usize, key: usize) -> bool {
        match self.tracks.get_mut(track) {
            Some(t) if key < t.keys.len() => {
                t.keys.remove(key);
                true
            }
            _ => false,
        }
    }

    pub fn track_remove_key_at_time(&mut self, track: usize, time: f64) -> bool {
        if let Some(idx) = self.track_find_key(track, time, FindMode::Approx) {
            self.track_remove_key(track, idx as usize)
        } else {
            false
        }
    }

    /// `track_find_key`. `Nearest` returns the closest key, `Approx` only a key
    /// within `CMP_EPSILON`, `Exact` only an exact time match.
    pub fn track_find_key(&self, track: usize, time: f64, mode: FindMode) -> Option<i32> {
        let t = self.tracks.get(track)?;
        if t.keys.is_empty() {
            return None;
        }

        match mode {
            FindMode::Exact => t
                .keys
                .iter()
                .position(|k| k.time == time)
                .map(|i| i as i32),
            FindMode::Approx => t
                .keys
                .iter()
                .position(|k| (k.time - time).abs() < f64::EPSILON)
                .map(|i| i as i32),
            FindMode::Nearest => {
                let mut best = 0usize;
                let mut best_dist = (t.keys[0].time - time).abs();
                for (i, k) in t.keys.iter().enumerate().skip(1) {
                    let d = (k.time - time).abs();
                    if d < best_dist {
                        best_dist = d;
                        best = i;
                    }
                }
                Some(best as i32)
            }
        }
    }

    /// `value_track_interpolate` for numeric tracks: nearest picks the previous
    /// key, linear lerps between the surrounding keys.
    pub fn value_track_interpolate(&self, track: usize, time: f64) -> Option<Variant> {
        let t = self.tracks.get(track)?;
        if t.keys.is_empty() {
            return None;
        }
        if time <= t.keys[0].time {
            return Some(t.keys[0].value.clone());
        }
        let last = t.keys.len() - 1;
        if time >= t.keys[last].time {
            return Some(t.keys[last].value.clone());
        }

        // Find the segment [i, i+1] containing `time`.
        let mut i = 0usize;
        while i + 1 < t.keys.len() && t.keys[i + 1].time <= time {
            i += 1;
        }

        match t.interpolation {
            InterpolationType::Nearest => Some(t.keys[i].value.clone()),
            InterpolationType::Linear
            | InterpolationType::LinearAngle
            | InterpolationType::Cubic
            | InterpolationType::CubicAngle => {
                let a = t.keys[i].time;
                let b = t.keys[i + 1].time;
                let w = if b > a { (time - a) / (b - a) } else { 0.0 };

                match (&t.keys[i].value, &t.keys[i + 1].value) {
                    (Variant::Float(x), Variant::Float(y)) => {
                        Some(Variant::Float(x + (y - x) * w))
                    }
                    (Variant::Int(x), Variant::Int(y)) => {
                        Some(Variant::Int((*x as f64 + (*y as f64 - *x as f64) * w) as i64))
                    }
                    (Variant::Float(x), Variant::Int(y)) => {
                        Some(Variant::Float(x + (*y as f64 - x) * w))
                    }
                    (Variant::Int(x), Variant::Float(y)) => {
                        Some(Variant::Float(*x as f64 + (y - *x as f64) * w))
                    }
                    _ => Some(t.keys[i].value.clone()),
                }
            }
        }
    }

    /// Segment index and weight for `time`, assuming the keys surround it.
    fn segment(&self, track: usize, time: f64) -> Option<(usize, f64)> {
        let t = self.tracks.get(track)?;
        if t.keys.len() < 2 {
            return None;
        }
        if time <= t.keys[0].time {
            return Some((0, 0.0));
        }
        let last = t.keys.len() - 1;
        if time >= t.keys[last].time {
            return Some((last.min(1), 1.0));
        }
        let mut i = 0usize;
        while i + 1 < t.keys.len() && t.keys[i + 1].time <= time {
            i += 1;
        }
        let a = t.keys[i].time;
        let b = t.keys[i + 1].time;
        let w = if b > a { (time - a) / (b - a) } else { 0.0 };
        Some((i, w))
    }

    pub fn position_track_insert_key(
        &mut self,
        track: usize,
        time: f64,
        position: crate::math::vector3::Vector3,
    ) -> Option<usize> {
        self.track_insert_key(track, time, Variant::Vector3(position), 1.0)
    }

    /// `position_track_interpolate`: component-wise lerp.
    pub fn position_track_interpolate(
        &self,
        track: usize,
        time: f64,
    ) -> Option<crate::math::vector3::Vector3> {
        let t = self.tracks.get(track)?;
        if t.keys.is_empty() {
            return None;
        }
        let first = match &t.keys[0].value {
            Variant::Vector3(v) => *v,
            _ => return None,
        };
        if t.keys.len() == 1 || time <= t.keys[0].time {
            return Some(first);
        }
        let (i, w) = self.segment(track, time)?;
        let last_value = match &t.keys[t.keys.len() - 1].value {
            Variant::Vector3(v) => *v,
            _ => return None,
        };
        if time >= t.keys[t.keys.len() - 1].time {
            return Some(last_value);
        }
        let a = match &t.keys[i].value {
            Variant::Vector3(v) => *v,
            _ => return None,
        };
        let b = match &t.keys[i + 1].value {
            Variant::Vector3(v) => *v,
            _ => return None,
        };
        Some(a + (b - a) * w as f32)
    }

    pub fn scale_track_insert_key(
        &mut self,
        track: usize,
        time: f64,
        scale: crate::math::vector3::Vector3,
    ) -> Option<usize> {
        self.track_insert_key(track, time, Variant::Vector3(scale), 1.0)
    }

    pub fn scale_track_interpolate(
        &self,
        track: usize,
        time: f64,
    ) -> Option<crate::math::vector3::Vector3> {
        self.position_track_interpolate(track, time)
    }

    pub fn rotation_track_insert_key(
        &mut self,
        track: usize,
        time: f64,
        rotation: crate::math::quaternion::Quaternion,
    ) -> Option<usize> {
        self.track_insert_key(track, time, Variant::Quaternion(rotation), 1.0)
    }

    /// `rotation_track_interpolate`: shortest-path slerp between the keys.
    pub fn rotation_track_interpolate(
        &self,
        track: usize,
        time: f64,
    ) -> Option<crate::math::quaternion::Quaternion> {
        let t = self.tracks.get(track)?;
        if t.keys.is_empty() {
            return None;
        }
        let first = match &t.keys[0].value {
            Variant::Quaternion(q) => *q,
            _ => return None,
        };
        if t.keys.len() == 1 || time <= t.keys[0].time {
            return Some(first);
        }
        let (i, w) = self.segment(track, time)?;
        let last = t.keys.len() - 1;
        let last_q = match &t.keys[last].value {
            Variant::Quaternion(q) => *q,
            _ => return None,
        };
        if time >= t.keys[last].time {
            return Some(last_q);
        }
        let a = match &t.keys[i].value {
            Variant::Quaternion(q) => *q,
            _ => return None,
        };
        let b = match &t.keys[i + 1].value {
            Variant::Quaternion(q) => *q,
            _ => return None,
        };
        Some(a.slerp(&b, w as f32))
    }

    /// Numeric value type check used by the interpolator above.
    pub fn track_is_numeric(&self, track: usize) -> bool {
        match self.tracks.get(track) {
            Some(t) => t.keys.iter().all(|k| {
                matches!(
                    k.value.vtype(),
                    VariantType::Int | VariantType::Float
                )
            }),
            None => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tracks_are_added_and_removed() {
        let mut a = Animation::new();
        let t0 = a.add_track(TrackType::Value, -1);
        let t1 = a.add_track(TrackType::Method, -1);
        assert_eq!((t0, t1), (0, 1));
        assert_eq!(a.get_track_count(), 2);
        assert!(a.track_set_path(0, "Sprite:modulate"));
        assert_eq!(a.get_track(0).unwrap().path.as_string(), "Sprite:modulate");

        assert!(a.remove_track(0));
        assert_eq!(a.get_track_count(), 1);
        assert!(!a.remove_track(5));
    }

    #[test]
    fn keys_stay_sorted_and_same_time_replaces() {
        let mut a = Animation::new();
        let t = a.add_track(TrackType::Value, -1);
        assert_eq!(a.track_insert_key(t, 1.0, Variant::Float(1.0), 1.0), Some(0));
        assert_eq!(a.track_insert_key(t, 0.0, Variant::Float(0.0), 1.0), Some(0));
        assert_eq!(a.track_insert_key(t, 2.0, Variant::Float(2.0), 1.0), Some(2));

        assert_eq!(a.track_get_key_time(t, 0), Some(0.0));
        assert_eq!(a.track_get_key_time(t, 1), Some(1.0));
        assert_eq!(a.track_get_key_count(t), 3);

        // Same time replaces instead of inserting a duplicate.
        assert_eq!(a.track_insert_key(t, 1.0, Variant::Float(9.0), 0.5), Some(1));
        assert_eq!(a.track_get_key_count(t), 3);
        assert_eq!(a.track_get_key_value(t, 1), Some(Variant::Float(9.0)));
        assert_eq!(a.track_get_key_transition(t, 1), Some(0.5));
    }

    #[test]
    fn find_modes() {
        let mut a = Animation::new();
        let t = a.add_track(TrackType::Value, -1);
        a.track_insert_key(t, 0.0, Variant::Float(0.0), 1.0);
        a.track_insert_key(t, 1.0, Variant::Float(1.0), 1.0);
        a.track_insert_key(t, 2.0, Variant::Float(2.0), 1.0);

        assert_eq!(a.track_find_key(t, 1.0, FindMode::Exact), Some(1));
        assert_eq!(a.track_find_key(t, 1.5, FindMode::Exact), None);
        assert_eq!(a.track_find_key(t, 1.0, FindMode::Approx), Some(1));
        assert_eq!(a.track_find_key(t, 0.7, FindMode::Nearest), Some(1));
        assert_eq!(a.track_find_key(t, 0.2, FindMode::Nearest), Some(0));
        assert_eq!(a.track_find_key(9, 0.0, FindMode::Nearest), None);
    }

    #[test]
    fn removal_by_index_and_time() {
        let mut a = Animation::new();
        let t = a.add_track(TrackType::Value, -1);
        a.track_insert_key(t, 0.0, Variant::Float(0.0), 1.0);
        a.track_insert_key(t, 1.0, Variant::Float(1.0), 1.0);

        assert!(a.track_remove_key_at_time(t, 0.0));
        assert_eq!(a.track_get_key_count(t), 1);
        assert!(!a.track_remove_key_at_time(t, 5.0));
        assert!(a.track_remove_key(t, 0));
        assert!(!a.track_remove_key(t, 0));
    }

    #[test]
    fn linear_and_nearest_interpolation() {
        let mut a = Animation::new();
        let t = a.add_track(TrackType::Value, -1);
        a.track_insert_key(t, 0.0, Variant::Float(0.0), 1.0);
        a.track_insert_key(t, 2.0, Variant::Float(10.0), 1.0);

        // Linear by default.
        assert_eq!(a.value_track_interpolate(t, 1.0), Some(Variant::Float(5.0)));
        assert_eq!(a.value_track_interpolate(t, 0.5), Some(Variant::Float(2.5)));
        // Clamped outside the key range.
        assert_eq!(a.value_track_interpolate(t, -1.0), Some(Variant::Float(0.0)));
        assert_eq!(a.value_track_interpolate(t, 9.0), Some(Variant::Float(10.0)));

        a.track_set_interpolation_type(t, InterpolationType::Nearest);
        assert_eq!(a.value_track_interpolate(t, 1.0), Some(Variant::Float(0.0)));
        assert_eq!(a.value_track_interpolate(t, 2.0), Some(Variant::Float(10.0)));
    }

    #[test]
    fn int_tracks_interpolate_and_stay_numeric() {
        let mut a = Animation::new();
        let t = a.add_track(TrackType::Value, -1);
        a.track_insert_key(t, 0.0, Variant::Int(0), 1.0);
        a.track_insert_key(t, 1.0, Variant::Int(10), 1.0);
        assert!(a.track_is_numeric(t));
        assert_eq!(a.value_track_interpolate(t, 0.5), Some(Variant::Int(5)));

        let s = a.add_track(TrackType::Value, -1);
        a.track_insert_key(s, 0.0, Variant::String(crate::string::gstring::GoString::from_str("x")), 1.0);
        assert!(!a.track_is_numeric(s));
    }

    #[test]
    fn position_and_scale_tracks_lerp() {
        use crate::math::vector3::Vector3;
        let mut a = Animation::new();
        let t = a.add_track(TrackType::Position3D, -1);
        a.position_track_insert_key(t, 0.0, Vector3::ZERO);
        a.position_track_insert_key(t, 2.0, Vector3::new(10.0, -4.0, 6.0));
        let p = a.position_track_interpolate(t, 1.0).unwrap();
        assert!((p - Vector3::new(5.0, -2.0, 3.0)).length() < 1e-5, "{p:?}");
        // Clamped ends.
        assert_eq!(a.position_track_interpolate(t, -5.0), Some(Vector3::ZERO));
        assert!((a.position_track_interpolate(t, 99.0).unwrap() - Vector3::new(10.0, -4.0, 6.0)).length() < 1e-5);

        let s = a.add_track(TrackType::Scale3D, -1);
        a.scale_track_insert_key(s, 0.0, Vector3::new(1.0, 1.0, 1.0));
        a.scale_track_insert_key(s, 1.0, Vector3::new(3.0, 5.0, 7.0));
        let sc = a.scale_track_interpolate(s, 0.5).unwrap();
        assert!((sc - Vector3::new(2.0, 3.0, 4.0)).length() < 1e-5, "{sc:?}");
    }

    #[test]
    fn rotation_track_slerps() {
        use crate::math::quaternion::Quaternion;
        use crate::math::vector3::Vector3;
        let mut a = Animation::new();
        let t = a.add_track(TrackType::Rotation3D, -1);
        a.rotation_track_insert_key(t, 0.0, Quaternion::IDENTITY);
        let q90 = Quaternion::from_axis_angle(Vector3::UP, std::f32::consts::FRAC_PI_2);
        a.rotation_track_insert_key(t, 1.0, q90);

        let mid = a.rotation_track_interpolate(t, 0.5).unwrap();
        let expected = Quaternion::from_axis_angle(Vector3::UP, std::f32::consts::FRAC_PI_4);
        assert!(mid.is_equal_approx(&expected), "{mid:?}");
        // 45 degrees from identity.
        let angle = mid.angle_to(&Quaternion::IDENTITY);
        assert!((angle - std::f32::consts::FRAC_PI_4).abs() < 1e-3, "{angle}");
    }

    #[test]
    fn length_step_and_loop_defaults() {
        let mut a = Animation::new();
        assert_eq!(a.get_length(), 1.0);
        assert_eq!(a.get_step(), 0.1);
        assert_eq!(a.get_loop_mode(), LoopMode::None);
        a.set_length(3.5);
        a.set_step(0.05);
        a.set_loop_mode(LoopMode::PingPong);
        assert_eq!(a.get_length(), 3.5);
        assert_eq!(a.get_step(), 0.05);
        assert_eq!(a.get_loop_mode(), LoopMode::PingPong);
    }
}
