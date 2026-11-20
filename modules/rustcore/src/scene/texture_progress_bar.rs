//! `scene/gui/texture_progress_bar.*`: the nine fill modes and texture state.
//!
//! Header defaults: left-to-right fill, zero progress offset and radial centre
//! offset, radial initial angle 0 with 360 degrees of fill, nine-patch stretch
//! off with four zero margins, and white under/progress/over tints.
//!
//! `set_fill_mode` rejects `FILL_MODE_MAX` (and negatives) via `ERR_FAIL_INDEX`;
//! `set_stretch_margin` rejects side indices outside `0..4`. Everything else
//! assigns directly.
//!
//! Drawing (`unit_val_to_uv`, the nine-patch slicing) and the texture sizes are
//! not ported; textures are `Rid` handles.

use crate::math::color::Color;
use crate::math::vector2::Vector2;
use crate::servers::rid::Rid;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum FillMode {
    LeftToRight,
    RightToLeft,
    TopToBottom,
    BottomToTop,
    Clockwise,
    CounterClockwise,
    BilinearLeftAndRight,
    BilinearTopAndBottom,
    ClockwiseAndCounterClockwise,
    Max,
}

pub const FILL_MODE_MAX: i32 = 9;

#[derive(Clone, Debug)]
pub struct TextureProgressBarData {
    mode: FillMode,
    progress_offset: Vector2,
    rad_init_angle: f32,
    rad_max_degrees: f32,
    rad_center_off: Vector2,
    under_texture: Option<Rid>,
    progress_texture: Option<Rid>,
    over_texture: Option<Rid>,
    nine_patch_stretch: bool,
    stretch_margin: [i32; 4],
    tint_under: Color,
    tint_progress: Color,
    tint_over: Color,
}

impl Default for TextureProgressBarData {
    fn default() -> Self {
        Self::new()
    }
}

impl TextureProgressBarData {
    pub fn new() -> Self {
        Self {
            mode: FillMode::LeftToRight,
            progress_offset: Vector2::ZERO,
            rad_init_angle: 0.0,
            rad_max_degrees: 360.0,
            rad_center_off: Vector2::ZERO,
            under_texture: None,
            progress_texture: None,
            over_texture: None,
            nine_patch_stretch: false,
            stretch_margin: [0; 4],
            tint_under: Color::new(1.0, 1.0, 1.0, 1.0),
            tint_progress: Color::new(1.0, 1.0, 1.0, 1.0),
            tint_over: Color::new(1.0, 1.0, 1.0, 1.0),
        }
    }

    /// `ERR_FAIL_INDEX(p_fill, FILL_MODE_MAX)`; negatives are rejected too.
    pub fn set_fill_mode(&mut self, mode: i32) -> bool {
        if mode < 0 || mode >= FILL_MODE_MAX {
            return false;
        }
        self.mode = match mode {
            0 => FillMode::LeftToRight,
            1 => FillMode::RightToLeft,
            2 => FillMode::TopToBottom,
            3 => FillMode::BottomToTop,
            4 => FillMode::Clockwise,
            5 => FillMode::CounterClockwise,
            6 => FillMode::BilinearLeftAndRight,
            7 => FillMode::BilinearTopAndBottom,
            _ => FillMode::ClockwiseAndCounterClockwise,
        };
        true
    }

    pub fn get_fill_mode(&self) -> FillMode {
        self.mode
    }

    pub fn set_progress_offset(&mut self, offset: Vector2) {
        self.progress_offset = offset;
    }

    pub fn get_progress_offset(&self) -> Vector2 {
        self.progress_offset
    }

    pub fn set_radial_initial_angle(&mut self, angle: f32) {
        self.rad_init_angle = angle;
    }

    pub fn get_radial_initial_angle(&self) -> f32 {
        self.rad_init_angle
    }

    pub fn set_fill_degrees(&mut self, degrees: f32) {
        self.rad_max_degrees = degrees;
    }

    pub fn get_fill_degrees(&self) -> f32 {
        self.rad_max_degrees
    }

    pub fn set_radial_center_offset(&mut self, offset: Vector2) {
        self.rad_center_off = offset;
    }

    pub fn get_radial_center_offset(&self) -> Vector2 {
        self.rad_center_off
    }

    pub fn set_under_texture(&mut self, texture: Option<Rid>) {
        self.under_texture = texture;
    }

    pub fn get_under_texture(&self) -> Option<Rid> {
        self.under_texture
    }

    pub fn set_progress_texture(&mut self, texture: Option<Rid>) {
        self.progress_texture = texture;
    }

    pub fn get_progress_texture(&self) -> Option<Rid> {
        self.progress_texture
    }

    pub fn set_over_texture(&mut self, texture: Option<Rid>) {
        self.over_texture = texture;
    }

    pub fn get_over_texture(&self) -> Option<Rid> {
        self.over_texture
    }

    /// Sides are `[left, top, right, bottom]`; 4 and up are rejected.
    pub fn set_stretch_margin(&mut self, side: usize, size: i32) -> bool {
        if side >= 4 {
            return false;
        }
        self.stretch_margin[side] = size;
        true
    }

    pub fn get_stretch_margin(&self, side: usize) -> i32 {
        self.stretch_margin.get(side).copied().unwrap_or(0)
    }

    pub fn set_nine_patch_stretch(&mut self, stretch: bool) {
        self.nine_patch_stretch = stretch;
    }

    pub fn get_nine_patch_stretch(&self) -> bool {
        self.nine_patch_stretch
    }

    pub fn set_tint_under(&mut self, tint: Color) {
        self.tint_under = tint;
    }

    pub fn get_tint_under(&self) -> Color {
        self.tint_under
    }

    pub fn set_tint_progress(&mut self, tint: Color) {
        self.tint_progress = tint;
    }

    pub fn get_tint_progress(&self) -> Color {
        self.tint_progress
    }

    pub fn set_tint_over(&mut self, tint: Color) {
        self.tint_over = tint;
    }

    pub fn get_tint_over(&self) -> Color {
        self.tint_over
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults() {
        let p = TextureProgressBarData::new();
        assert_eq!(p.get_fill_mode(), FillMode::LeftToRight);
        assert_eq!(p.get_progress_offset(), Vector2::ZERO);
        assert_eq!(p.get_radial_initial_angle(), 0.0);
        assert_eq!(p.get_fill_degrees(), 360.0);
        assert_eq!(p.get_radial_center_offset(), Vector2::ZERO);
        assert!(p.get_under_texture().is_none());
        assert!(p.get_progress_texture().is_none());
        assert!(p.get_over_texture().is_none());
        assert!(!p.get_nine_patch_stretch());
        for side in 0..4 {
            assert_eq!(p.get_stretch_margin(side), 0);
        }
        assert_eq!(p.get_tint_under(), Color::new(1.0, 1.0, 1.0, 1.0));
        assert_eq!(p.get_tint_progress(), Color::new(1.0, 1.0, 1.0, 1.0));
        assert_eq!(p.get_tint_over(), Color::new(1.0, 1.0, 1.0, 1.0));
    }

    #[test]
    fn fill_modes_are_validated() {
        let mut p = TextureProgressBarData::new();
        assert!(!p.set_fill_mode(-1));
        assert!(!p.set_fill_mode(FILL_MODE_MAX));
        assert_eq!(p.get_fill_mode(), FillMode::LeftToRight);

        for mode in 0..FILL_MODE_MAX {
            assert!(p.set_fill_mode(mode), "{mode}");
        }
        assert_eq!(p.get_fill_mode(), FillMode::ClockwiseAndCounterClockwise);

        assert_eq!(FillMode::LeftToRight as i32, 0);
        assert_eq!(FillMode::RightToLeft as i32, 1);
        assert_eq!(FillMode::TopToBottom as i32, 2);
        assert_eq!(FillMode::BottomToTop as i32, 3);
        assert_eq!(FillMode::Clockwise as i32, 4);
        assert_eq!(FillMode::CounterClockwise as i32, 5);
        assert_eq!(FillMode::BilinearLeftAndRight as i32, 6);
        assert_eq!(FillMode::BilinearTopAndBottom as i32, 7);
        assert_eq!(FillMode::ClockwiseAndCounterClockwise as i32, 8);
        assert_eq!(FillMode::Max as i32, 9);
    }

    #[test]
    fn margins_and_textures() {
        let mut p = TextureProgressBarData::new();
        p.set_nine_patch_stretch(true);
        assert!(p.set_stretch_margin(0, 4));
        assert!(p.set_stretch_margin(3, 8));
        assert_eq!(p.get_stretch_margin(0), 4);
        assert_eq!(p.get_stretch_margin(3), 8);
        assert_eq!(p.get_stretch_margin(1), 0);
        assert!(!p.set_stretch_margin(4, 2));
        assert_eq!(p.get_stretch_margin(9), 0);
        assert!(p.get_nine_patch_stretch());

        p.set_under_texture(Some(Rid::from_u64(1)));
        p.set_progress_texture(Some(Rid::from_u64(2)));
        p.set_over_texture(Some(Rid::from_u64(3)));
        assert_eq!(p.get_under_texture(), Some(Rid::from_u64(1)));
        assert_eq!(p.get_progress_texture(), Some(Rid::from_u64(2)));
        assert_eq!(p.get_over_texture(), Some(Rid::from_u64(3)));

        p.set_progress_offset(Vector2::new(1.0, 2.0));
        p.set_radial_initial_angle(45.0);
        p.set_fill_degrees(180.0);
        p.set_radial_center_offset(Vector2::new(-1.0, 0.5));
        assert_eq!(p.get_progress_offset(), Vector2::new(1.0, 2.0));
        assert_eq!(p.get_radial_initial_angle(), 45.0);
        assert_eq!(p.get_fill_degrees(), 180.0);
        assert_eq!(p.get_radial_center_offset(), Vector2::new(-1.0, 0.5));

        p.set_tint_under(Color::new(0.5, 0.5, 0.5, 1.0));
        p.set_tint_progress(Color::new(0.0, 1.0, 0.0, 1.0));
        p.set_tint_over(Color::new(0.0, 0.0, 1.0, 0.5));
        assert_eq!(p.get_tint_under(), Color::new(0.5, 0.5, 0.5, 1.0));
        assert_eq!(p.get_tint_progress(), Color::new(0.0, 1.0, 0.0, 1.0));
        assert_eq!(p.get_tint_over(), Color::new(0.0, 0.0, 1.0, 0.5));
    }
}
