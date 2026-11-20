//! `scene/2d/path_2d.*` and `scene/3d/path_3d.*` (state only).
//!
//! `Path2D`/`Path3D` hold the curve handle and the editor debug colour
//! (transparent black for `Path3D`).
//!
//! `PathFollow2D`/`PathFollow3D` default to zero progress, zero offsets, cubic
//! interpolation, looping and (2D) rotation on; the 3D follower also enables
//! tilt, uses the non-model front and the `ROTATION_XYZ` mode.
//!
//! `progress_ratio` needs the baked curve length in the engine; the port keeps a
//! caller-supplied `curve_length` so the conversion is still testable.
//! Transform sampling, rotation correction and the update timer are not ported.

use crate::math::color::Color;
use crate::servers::rid::Rid;

// -- Path2D -----------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct Path2DData {
    curve: Option<Rid>,
}

impl Default for Path2DData {
    fn default() -> Self {
        Self::new()
    }
}

impl Path2DData {
    pub fn new() -> Self {
        Self { curve: None }
    }

    pub fn set_curve(&mut self, curve: Option<Rid>) {
        self.curve = curve;
    }

    pub fn get_curve(&self) -> Option<Rid> {
        self.curve
    }
}

// -- PathFollow2D -----------------------------------------------------------

#[derive(Clone, Debug)]
pub struct PathFollow2DData {
    progress: f32,
    curve_length: f32,
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
            curve_length: 0.0,
            h_offset: 0.0,
            v_offset: 0.0,
            cubic: true,
            loop_enabled: true,
            rotates: true,
        }
    }

    pub fn set_progress(&mut self, progress: f32) {
        self.progress = progress;
    }

    pub fn get_progress(&self) -> f32 {
        self.progress
    }

    /// Caller-supplied baked curve length (the engine reads it from `Curve2D`).
    pub fn set_curve_length(&mut self, length: f32) {
        self.curve_length = length;
    }

    pub fn get_curve_length(&self) -> f32 {
        self.curve_length
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

    /// `set_progress_ratio`: scales by the baked length.
    pub fn set_progress_ratio(&mut self, ratio: f32) {
        self.progress = ratio * self.curve_length;
    }

    pub fn get_progress_ratio(&self) -> f32 {
        if self.curve_length == 0.0 {
            0.0
        } else {
            self.progress / self.curve_length
        }
    }

    pub fn set_loop(&mut self, loop_enabled: bool) {
        self.loop_enabled = loop_enabled;
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

    pub fn set_cubic_interpolation_enabled(&mut self, enabled: bool) {
        self.cubic = enabled;
    }

    pub fn is_cubic_interpolation_enabled(&self) -> bool {
        self.cubic
    }
}

// -- Path3D -----------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct Path3DData {
    curve: Option<Rid>,
    debug_custom_color: Color,
}

impl Default for Path3DData {
    fn default() -> Self {
        Self::new()
    }
}

impl Path3DData {
    pub fn new() -> Self {
        Self {
            curve: None,
            // The engine's `Color()` default is opaque black.
            debug_custom_color: Color::new(0.0, 0.0, 0.0, 1.0),
        }
    }

    pub fn set_curve(&mut self, curve: Option<Rid>) {
        self.curve = curve;
    }

    pub fn get_curve(&self) -> Option<Rid> {
        self.curve
    }

    pub fn set_debug_custom_color(&mut self, color: Color) {
        self.debug_custom_color = color;
    }

    pub fn get_debug_custom_color(&self) -> Color {
        self.debug_custom_color
    }
}

// -- PathFollow3D -----------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum RotationMode {
    None,
    Y,
    Xy,
    Xyz,
    Oriented,
}

#[derive(Clone, Debug)]
pub struct PathFollow3DData {
    progress: f32,
    curve_length: f32,
    h_offset: f32,
    v_offset: f32,
    cubic: bool,
    loop_enabled: bool,
    tilt_enabled: bool,
    use_model_front: bool,
    rotation_mode: RotationMode,
}

impl Default for PathFollow3DData {
    fn default() -> Self {
        Self::new()
    }
}

impl PathFollow3DData {
    pub fn new() -> Self {
        Self {
            progress: 0.0,
            curve_length: 0.0,
            h_offset: 0.0,
            v_offset: 0.0,
            cubic: true,
            loop_enabled: true,
            tilt_enabled: true,
            use_model_front: false,
            rotation_mode: RotationMode::Xyz,
        }
    }

    pub fn set_progress(&mut self, progress: f32) {
        self.progress = progress;
    }

    pub fn get_progress(&self) -> f32 {
        self.progress
    }

    pub fn set_curve_length(&mut self, length: f32) {
        self.curve_length = length;
    }

    pub fn get_curve_length(&self) -> f32 {
        self.curve_length
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

    pub fn set_progress_ratio(&mut self, ratio: f32) {
        self.progress = ratio * self.curve_length;
    }

    pub fn get_progress_ratio(&self) -> f32 {
        if self.curve_length == 0.0 {
            0.0
        } else {
            self.progress / self.curve_length
        }
    }

    pub fn set_loop(&mut self, loop_enabled: bool) {
        self.loop_enabled = loop_enabled;
    }

    pub fn has_loop(&self) -> bool {
        self.loop_enabled
    }

    pub fn set_tilt_enabled(&mut self, enabled: bool) {
        self.tilt_enabled = enabled;
    }

    pub fn is_tilt_enabled(&self) -> bool {
        self.tilt_enabled
    }

    /// `ERR_FAIL_INDEX(mode, ROTATION_ORIENTED + 1)`.
    pub fn set_rotation_mode(&mut self, mode: i32) -> bool {
        let parsed = match mode {
            0 => RotationMode::None,
            1 => RotationMode::Y,
            2 => RotationMode::Xy,
            3 => RotationMode::Xyz,
            4 => RotationMode::Oriented,
            _ => return false,
        };
        self.rotation_mode = parsed;
        true
    }

    pub fn get_rotation_mode(&self) -> RotationMode {
        self.rotation_mode
    }

    pub fn set_use_model_front(&mut self, use_model_front: bool) {
        self.use_model_front = use_model_front;
    }

    pub fn is_using_model_front(&self) -> bool {
        self.use_model_front
    }

    pub fn set_cubic_interpolation_enabled(&mut self, enabled: bool) {
        self.cubic = enabled;
    }

    pub fn is_cubic_interpolation_enabled(&self) -> bool {
        self.cubic
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn path_2d_and_follow() {
        let mut path = Path2DData::new();
        assert!(path.get_curve().is_none());
        path.set_curve(Some(Rid::from_u64(1)));
        assert_eq!(path.get_curve(), Some(Rid::from_u64(1)));

        let mut follow = PathFollow2DData::new();
        assert_eq!(follow.get_progress(), 0.0);
        assert_eq!(follow.get_progress_ratio(), 0.0);
        assert_eq!(follow.get_h_offset(), 0.0);
        assert_eq!(follow.get_v_offset(), 0.0);
        assert!(follow.has_loop());
        assert!(follow.is_rotation_enabled());
        assert!(follow.is_cubic_interpolation_enabled());

        follow.set_curve_length(200.0);
        follow.set_progress_ratio(0.25);
        assert_eq!(follow.get_progress(), 50.0);
        assert_eq!(follow.get_progress_ratio(), 0.25);
        follow.set_progress(100.0);
        assert_eq!(follow.get_progress_ratio(), 0.5);
        follow.set_h_offset(1.5);
        follow.set_v_offset(-2.0);
        follow.set_loop(false);
        follow.set_rotation_enabled(false);
        follow.set_cubic_interpolation_enabled(false);
        assert_eq!(follow.get_h_offset(), 1.5);
        assert_eq!(follow.get_v_offset(), -2.0);
        assert!(!follow.has_loop());
        assert!(!follow.is_rotation_enabled());
        assert!(!follow.is_cubic_interpolation_enabled());
    }

    #[test]
    fn path_3d_and_follow() {
        let mut path = Path3DData::new();
        assert!(path.get_curve().is_none());
        assert_eq!(path.get_debug_custom_color(), Color::new(0.0, 0.0, 0.0, 1.0));
        path.set_curve(Some(Rid::from_u64(3)));
        path.set_debug_custom_color(Color::new(1.0, 0.0, 0.0, 1.0));
        assert_eq!(path.get_curve(), Some(Rid::from_u64(3)));
        assert_eq!(path.get_debug_custom_color(), Color::new(1.0, 0.0, 0.0, 1.0));

        let mut follow = PathFollow3DData::new();
        assert_eq!(follow.get_rotation_mode(), RotationMode::Xyz);
        assert!(follow.is_tilt_enabled());
        assert!(!follow.is_using_model_front());
        assert!(follow.has_loop());
        assert!(follow.is_cubic_interpolation_enabled());
        assert!(!follow.set_rotation_mode(5));
        assert!(follow.set_rotation_mode(4));
        assert_eq!(follow.get_rotation_mode(), RotationMode::Oriented);
        assert_eq!(RotationMode::None as i32, 0);
        assert_eq!(RotationMode::Oriented as i32, 4);

        follow.set_curve_length(10.0);
        follow.set_progress_ratio(0.5);
        assert_eq!(follow.get_progress(), 5.0);
        follow.set_progress(2.5);
        assert_eq!(follow.get_progress_ratio(), 0.25);
        follow.set_tilt_enabled(false);
        follow.set_use_model_front(true);
        follow.set_loop(false);
        follow.set_cubic_interpolation_enabled(false);
        follow.set_h_offset(0.5);
        follow.set_v_offset(1.5);
        assert!(!follow.is_tilt_enabled());
        assert!(follow.is_using_model_front());
        assert!(!follow.has_loop());
        assert!(!follow.is_cubic_interpolation_enabled());
        assert_eq!(follow.get_h_offset(), 0.5);
        assert_eq!(follow.get_v_offset(), 1.5);
    }
}
