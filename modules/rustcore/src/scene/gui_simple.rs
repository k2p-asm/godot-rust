//! Small `scene/gui` controls: `ColorRect`, `Separator`/`HSeparator`/`VSeparator`,
//! `ReferenceRect`, `NinePatchRect` and `Popup` (state only).
//!
//! Defaults: ColorRect is white; separators follow the engine's heading
//! orientation (`HSeparator` horizontal, `VSeparator` vertical); ReferenceRect has
//! a red 1 px border and is editor-only; NinePatchRect draws its centre, has zero
//! patch margins, an empty region/texture and stretch axes; a Popup is not popped
//! up and has no hide reason.
//!
//! Validation: `NinePatchRect::set_patch_margin` rejects side indices above 3
//! (`ERR_FAIL_INDEX(side, 4)`), like the C++.
//!
//! Drawing, theme lookups, window plumbing and the popup's parent bookkeeping are
//! not ported.

use crate::math::color::Color;
use crate::math::rect2::Rect2;
use crate::servers::rid::Rid;

// -- ColorRect --------------------------------------------------------------

#[derive(Clone, Copy, Debug)]
pub struct ColorRectData {
    color: Color,
}

impl Default for ColorRectData {
    fn default() -> Self {
        Self::new()
    }
}

impl ColorRectData {
    pub fn new() -> Self {
        Self {
            color: Color::new(1.0, 1.0, 1.0, 1.0),
        }
    }

    pub fn set_color(&mut self, color: Color) {
        self.color = color;
    }

    pub fn get_color(&self) -> Color {
        self.color
    }
}

// -- Separator --------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Orientation {
    Horizontal,
    Vertical,
}

#[derive(Clone, Copy, Debug)]
pub struct SeparatorData {
    orientation: Orientation,
}

impl Default for SeparatorData {
    fn default() -> Self {
        Self::new()
    }
}

impl SeparatorData {
    pub fn new() -> Self {
        Self {
            orientation: Orientation::Horizontal,
        }
    }

    /// `HSeparator`'s constructor.
    pub fn horizontal() -> Self {
        Self {
            orientation: Orientation::Horizontal,
        }
    }

    /// `VSeparator`'s constructor.
    pub fn vertical() -> Self {
        Self {
            orientation: Orientation::Vertical,
        }
    }

    pub fn get_orientation(&self) -> Orientation {
        self.orientation
    }
}

// -- ReferenceRect ----------------------------------------------------------

#[derive(Clone, Copy, Debug)]
pub struct ReferenceRectData {
    border_color: Color,
    border_width: f32,
    editor_only: bool,
}

impl Default for ReferenceRectData {
    fn default() -> Self {
        Self::new()
    }
}

impl ReferenceRectData {
    pub fn new() -> Self {
        Self {
            border_color: Color::new(1.0, 0.0, 0.0, 1.0),
            border_width: 1.0,
            editor_only: true,
        }
    }

    pub fn set_border_color(&mut self, color: Color) {
        self.border_color = color;
    }

    pub fn get_border_color(&self) -> Color {
        self.border_color
    }

    pub fn set_border_width(&mut self, width: f32) {
        self.border_width = width;
    }

    pub fn get_border_width(&self) -> f32 {
        self.border_width
    }

    pub fn set_editor_only(&mut self, enabled: bool) {
        self.editor_only = enabled;
    }

    pub fn get_editor_only(&self) -> bool {
        self.editor_only
    }
}

// -- NinePatchRect ----------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum AxisStretchMode {
    Stretch,
    Tile,
    TileFit,
}

#[derive(Clone, Debug)]
pub struct NinePatchRectData {
    draw_center: bool,
    /// `[left, top, right, bottom]`.
    margin: [i32; 4],
    region_rect: Rect2,
    texture: Option<Rid>,
    axis_h: AxisStretchMode,
    axis_v: AxisStretchMode,
}

impl Default for NinePatchRectData {
    fn default() -> Self {
        Self::new()
    }
}

impl NinePatchRectData {
    pub fn new() -> Self {
        Self {
            draw_center: true,
            margin: [0; 4],
            region_rect: Rect2::default(),
            texture: None,
            axis_h: AxisStretchMode::Stretch,
            axis_v: AxisStretchMode::Stretch,
        }
    }

    pub fn set_draw_center(&mut self, enabled: bool) {
        self.draw_center = enabled;
    }

    pub fn get_draw_center(&self) -> bool {
        self.draw_center
    }

    /// `ERR_FAIL_INDEX(side, 4)`.
    pub fn set_patch_margin(&mut self, side: usize, size: i32) -> bool {
        if side >= 4 {
            return false;
        }
        self.margin[side] = size;
        true
    }

    pub fn get_patch_margin(&self, side: usize) -> i32 {
        self.margin.get(side).copied().unwrap_or(0)
    }

    pub fn set_region_rect(&mut self, rect: Rect2) {
        self.region_rect = rect;
    }

    pub fn get_region_rect(&self) -> Rect2 {
        self.region_rect
    }

    pub fn set_texture(&mut self, texture: Option<Rid>) {
        self.texture = texture;
    }

    pub fn get_texture(&self) -> Option<Rid> {
        self.texture
    }

    /// Accepts `0..=2` (`AXIS_STRETCH_MODE_*`); other values are rejected.
    pub fn set_h_axis_stretch_mode(&mut self, mode: i32) -> bool {
        match mode {
            0 => self.axis_h = AxisStretchMode::Stretch,
            1 => self.axis_h = AxisStretchMode::Tile,
            2 => self.axis_h = AxisStretchMode::TileFit,
            _ => return false,
        }
        true
    }

    pub fn get_h_axis_stretch_mode(&self) -> AxisStretchMode {
        self.axis_h
    }

    pub fn set_v_axis_stretch_mode(&mut self, mode: i32) -> bool {
        match mode {
            0 => self.axis_v = AxisStretchMode::Stretch,
            1 => self.axis_v = AxisStretchMode::Tile,
            2 => self.axis_v = AxisStretchMode::TileFit,
            _ => return false,
        }
        true
    }

    pub fn get_v_axis_stretch_mode(&self) -> AxisStretchMode {
        self.axis_v
    }
}

// -- Popup ------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum HideReason {
    None,
    Canceled,
    Unfocused,
}

#[derive(Clone, Copy, Debug)]
pub struct PopupData {
    popped_up: bool,
    hide_reason: HideReason,
}

impl Default for PopupData {
    fn default() -> Self {
        Self::new()
    }
}

impl PopupData {
    pub fn new() -> Self {
        Self {
            popped_up: false,
            hide_reason: HideReason::None,
        }
    }

    /// `_post_popup`: the window became visible.
    pub fn popup(&mut self) {
        self.popped_up = true;
        self.hide_reason = HideReason::None;
    }

    /// `_close_pressed`.
    pub fn close(&mut self) {
        self.popped_up = false;
    }

    pub fn is_popped_up(&self) -> bool {
        self.popped_up
    }

    /// Recorded before the platform window hides.
    pub fn set_hide_reason(&mut self, reason: HideReason) {
        self.hide_reason = reason;
    }

    pub fn get_hide_reason(&self) -> HideReason {
        self.hide_reason
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::vector2::Vector2;

    #[test]
    fn color_rect_and_reference_rect() {
        let mut c = ColorRectData::new();
        assert_eq!(c.get_color(), Color::new(1.0, 1.0, 1.0, 1.0));
        c.set_color(Color::new(0.2, 0.4, 0.6, 0.5));
        assert_eq!(c.get_color(), Color::new(0.2, 0.4, 0.6, 0.5));

        let mut r = ReferenceRectData::new();
        assert_eq!(r.get_border_color(), Color::new(1.0, 0.0, 0.0, 1.0));
        assert_eq!(r.get_border_width(), 1.0);
        assert!(r.get_editor_only());
        r.set_border_color(Color::new(0.0, 1.0, 0.0, 1.0));
        r.set_border_width(3.0);
        r.set_editor_only(false);
        assert_eq!(r.get_border_color(), Color::new(0.0, 1.0, 0.0, 1.0));
        assert_eq!(r.get_border_width(), 3.0);
        assert!(!r.get_editor_only());
    }

    #[test]
    fn separator_orientations() {
        assert_eq!(SeparatorData::new().get_orientation(), Orientation::Horizontal);
        assert_eq!(
            SeparatorData::horizontal().get_orientation(),
            Orientation::Horizontal
        );
        assert_eq!(
            SeparatorData::vertical().get_orientation(),
            Orientation::Vertical
        );
        assert_eq!(Orientation::Horizontal as i32, 0);
        assert_eq!(Orientation::Vertical as i32, 1);
    }

    #[test]
    fn nine_patch_defaults_and_margins() {
        let mut n = NinePatchRectData::new();
        assert!(n.get_draw_center());
        for side in 0..4 {
            assert_eq!(n.get_patch_margin(side), 0);
        }
        assert_eq!(n.get_region_rect(), Rect2::default());
        assert!(n.get_texture().is_none());
        assert_eq!(n.get_h_axis_stretch_mode(), AxisStretchMode::Stretch);
        assert_eq!(n.get_v_axis_stretch_mode(), AxisStretchMode::Stretch);

        assert!(n.set_patch_margin(0, 4));
        assert!(n.set_patch_margin(3, 8));
        assert!(!n.set_patch_margin(4, 1));
        assert_eq!(n.get_patch_margin(0), 4);
        assert_eq!(n.get_patch_margin(3), 8);
        assert_eq!(n.get_patch_margin(9), 0);

        n.set_draw_center(false);
        n.set_region_rect(Rect2::new(Vector2::new(1.0, 2.0), Vector2::new(3.0, 4.0)));
        n.set_texture(Some(Rid::from_u64(5)));
        assert!(!n.get_draw_center());
        assert_eq!(n.get_region_rect().size, Vector2::new(3.0, 4.0));
        assert_eq!(n.get_texture(), Some(Rid::from_u64(5)));

        assert!(n.set_h_axis_stretch_mode(2));
        assert!(!n.set_h_axis_stretch_mode(3));
        assert_eq!(n.get_h_axis_stretch_mode(), AxisStretchMode::TileFit);
        assert!(n.set_v_axis_stretch_mode(1));
        assert_eq!(n.get_v_axis_stretch_mode(), AxisStretchMode::Tile);
        assert_eq!(AxisStretchMode::Stretch as i32, 0);
        assert_eq!(AxisStretchMode::Tile as i32, 1);
        assert_eq!(AxisStretchMode::TileFit as i32, 2);
    }

    #[test]
    fn popup_state() {
        let mut p = PopupData::new();
        assert!(!p.is_popped_up());
        assert_eq!(p.get_hide_reason(), HideReason::None);

        p.popup();
        assert!(p.is_popped_up());
        p.set_hide_reason(HideReason::Unfocused);
        p.close();
        assert!(!p.is_popped_up());
        assert_eq!(p.get_hide_reason(), HideReason::Unfocused);

        // Popping up again clears the previous reason.
        p.popup();
        assert_eq!(p.get_hide_reason(), HideReason::None);

        assert_eq!(HideReason::None as i32, 0);
        assert_eq!(HideReason::Canceled as i32, 1);
        assert_eq!(HideReason::Unfocused as i32, 2);
    }
}
