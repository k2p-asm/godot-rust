//! `scene/2d/path_2d.*` (Path2D) and `audio_listener_2d.*`.
//!
//! `Path2D` only owns a `Curve2D`; the debug drawing and the change
//! notifications are not ported. `PathFollow2D` is deliberately left pending:
//! its rotation comes from `Curve2D::sample_baked_with_rotation`, which needs
//! the baked forward-vector cache (`_calculate_tangent`) that the ported
//! `Curve2D` does not keep yet; approximating it from neighbouring points would
//! change the rotation.
//!
//! `AudioListener2D` tracks the `current` flag set through `make_current` and
//! `clear_current`.

use crate::math::funcs;
use crate::math::vector2::Vector2;
use crate::scene::curve_2d::Curve2D;

#[derive(Clone, Copy, Debug)]
pub struct PathFollow2DData {
    progress: f32,
    h_offset: f32,
    v_offset: f32,
    cubic: bool,
    loop_enabled: bool,
    rotates: bool,
}

impl Default for PathFollow2DData {
    fn default() -> Self {
        Self::new()
    }
}

impl PathFollow2DData {
    pub fn new() -> Self {
        Self {
            progress: 0.0,
            h_offset: 0.0,
            v_offset: 0.0,
            cubic: true,
            loop_enabled: true,
            rotates: true,
        }
    }

    /// `set_progress`: wraps when looping (a non-zero input that wraps to zero
    /// becomes the full length) and clamps otherwise. Rejects non-finite input.
    pub fn set_progress(&mut self, curve: &mut Curve2D, progress: f32) -> bool {
        if !progress.is_finite() {
            return false;
        }
        let length = curve.get_baked_length();
        self.progress = if self.loop_enabled && length != 0.0 {
            let wrapped = funcs::fposmod(progress, length);
            if progress != 0.0 && wrapped == 0.0 {
                length
            } else {
                wrapped
            }
        } else {
            progress.clamp(0.0, length)
        };
        true
    }

    pub fn get_progress(&self) -> f32 {
        self.progress
    }

    pub fn set_progress_ratio(&mut self, curve: &mut Curve2D, ratio: f32) -> bool {
        let length = curve.get_baked_length();
        if length == 0.0 {
            return false;
        }
        self.set_progress(curve, ratio * length)
    }

    pub fn get_progress_ratio(&self, curve: &mut Curve2D) -> f32 {
        let length = curve.get_baked_length();
        if length == 0.0 {
            0.0
        } else {
            self.progress / length
        }
    }

    pub fn set_h_offset(&mut self, offset: f32) {
        self.h_offset = offset;
    }

    pub fn get_h_offset(&self) -> f32 {
        self.h_offset
    }

    pub fn set_v_offset(&mut self, offset: f32) {
        self.v_offset = offset;
    }

    pub fn get_v_offset(&self) -> f32 {
        self.v_offset
    }

    pub fn set_cubic_interpolation_enabled(&mut self, enabled: bool) {
        self.cubic = enabled;
    }

    pub fn is_cubic_interpolation_enabled(&self) -> bool {
        self.cubic
    }

    pub fn set_loop(&mut self, looped: bool) {
        self.loop_enabled = looped;
    }

    pub fn has_loop(&self) -> bool {
        self.loop_enabled
    }

    pub fn set_rotation_enabled(&mut self, enabled: bool) {
        self.rotates = enabled;
    }

    pub fn is_rotation_enabled(&self) -> bool {
        self.rotates
    }

    /// `_update_transform`: returns the position and, when rotation is enabled,
    /// the new rotation (otherwise rotation is left untouched).
    pub fn update_transform(&self, curve: &mut Curve2D) -> Option<(Vector2, Option<f32>)> {
        if curve.get_baked_length() == 0.0 {
            return None;
        }
        if self.rotates {
            let mut xform = curve.sample_baked_with_rotation(self.progress, self.cubic);
            xform.columns[2] += xform.columns[0] * self.h_offset + xform.columns[1] * self.v_offset;
            Some((xform.columns[2], Some(xform.columns[0].angle())))
        } else {
            let mut pos = curve.sample_baked(self.progress, self.cubic);
            pos.x += self.h_offset;
            pos.y += self.v_offset;
            Some((pos, None))
        }
    }
}

#[derive(Debug, Default)]
pub struct Path2DData {
    curve: Option<Curve2D>,
}

impl Path2DData {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_curve(&mut self, curve: Option<Curve2D>) {
        self.curve = curve;
    }

    pub fn get_curve(&self) -> Option<&Curve2D> {
        self.curve.as_ref()
    }

    pub fn has_curve(&self) -> bool {
        self.curve.is_some()
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct AudioListener2DData {
    current: bool,
}

impl AudioListener2DData {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn make_current(&mut self) {
        self.current = true;
    }

    pub fn clear_current(&mut self) {
        self.current = false;
    }

    pub fn is_current(&self) -> bool {
        self.current
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::vector2::Vector2;

    #[test]
    fn path_2d_holds_a_curve() {
        let mut p = Path2DData::new();
        assert!(!p.has_curve());
        assert!(p.get_curve().is_none());

        let mut curve = Curve2D::new();
        curve.add_point(Vector2::ZERO);
        curve.add_point(Vector2::new(10.0, 0.0));
        p.set_curve(Some(curve));

        assert!(p.has_curve());
        assert_eq!(p.get_curve().unwrap().get_point_count(), 2);

        p.set_curve(None);
        assert!(!p.has_curve());
    }

    #[test]
    fn audio_listener_current_flag() {
        let mut l = AudioListener2DData::new();
        assert!(!l.is_current());
        l.make_current();
        assert!(l.is_current());
        // Making it current twice is idempotent.
        l.make_current();
        assert!(l.is_current());
        l.clear_current();
        assert!(!l.is_current());
        l.clear_current();
        assert!(!l.is_current());
    }

    fn straight(len: f32) -> Curve2D {
        let mut c = Curve2D::new();
        c.add_point(Vector2::ZERO);
        c.add_point(Vector2::new(len, 0.0));
        c
    }

    #[test]
    fn follow_progress_wraps_and_clamps() {
        let mut curve = straight(10.0);
        let mut f = PathFollow2DData::new();
        assert!(f.has_loop());
        assert!(f.set_progress(&mut curve, 12.0));
        assert!((f.get_progress() - 2.0).abs() < 1e-4);
        assert!(f.set_progress(&mut curve, -2.0));
        assert!((f.get_progress() - 8.0).abs() < 1e-4);
        // A non-zero input that wraps exactly to zero becomes the full length.
        assert!(f.set_progress(&mut curve, 10.0));
        assert!((f.get_progress() - 10.0).abs() < 1e-4);
        assert!(f.set_progress(&mut curve, 0.0));
        assert_eq!(f.get_progress(), 0.0);

        // Non-looping clamps.
        f.set_loop(false);
        assert!(f.set_progress(&mut curve, 12.0));
        assert!((f.get_progress() - 10.0).abs() < 1e-4);
        assert!(f.set_progress(&mut curve, -5.0));
        assert_eq!(f.get_progress(), 0.0);

        assert!(!f.set_progress(&mut curve, f32::NAN));
        assert!(!f.set_progress(&mut curve, f32::INFINITY));
    }

    #[test]
    fn follow_ratio_requires_a_curve() {
        let mut curve = straight(10.0);
        let mut f = PathFollow2DData::new();
        assert!(f.set_progress_ratio(&mut curve, 0.25));
        assert!((f.get_progress() - 2.5).abs() < 1e-4);
        assert!((f.get_progress_ratio(&mut curve) - 0.25).abs() < 1e-4);

        let mut empty = Curve2D::new();
        assert!(!f.set_progress_ratio(&mut empty, 0.5));
        assert_eq!(f.get_progress_ratio(&mut empty), 0.0);
    }

    #[test]
    fn follow_transform_with_offsets() {
        let mut curve = straight(10.0);
        let mut f = PathFollow2DData::new();
        f.set_progress(&mut curve, 5.0);
        f.set_h_offset(2.0);
        f.set_v_offset(3.0);

        let (pos, rotation) = f.update_transform(&mut curve).unwrap();
        // Forward is +X and side is +Y: pos = (5,0) + 2*(1,0) + 3*(0,1).
        assert!((pos - Vector2::new(7.0, 3.0)).length() < 1e-4, "{pos:?}");
        assert!(rotation.unwrap().abs() < 1e-5);

        // With rotation disabled the node rotation is left untouched (None).
        f.set_rotation_enabled(false);
        let (pos, rotation) = f.update_transform(&mut curve).unwrap();
        assert!((pos - Vector2::new(7.0, 3.0)).length() < 1e-4);
        assert!(rotation.is_none());

        // Zero-length curves produce no transform.
        let mut empty = Curve2D::new();
        assert!(f.update_transform(&mut empty).is_none());
    }

    #[test]
    fn follow_flags_round_trip() {
        let mut f = PathFollow2DData::new();
        assert!(f.is_cubic_interpolation_enabled());
        assert!(f.is_rotation_enabled());
        f.set_cubic_interpolation_enabled(false);
        f.set_rotation_enabled(false);
        f.set_h_offset(-1.0);
        f.set_v_offset(2.0);
        assert!(!f.is_cubic_interpolation_enabled());
        assert!(!f.is_rotation_enabled());
        assert_eq!(f.get_h_offset(), -1.0);
        assert_eq!(f.get_v_offset(), 2.0);
    }
}
