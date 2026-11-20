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

use crate::scene::curve_2d::Curve2D;

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
}
