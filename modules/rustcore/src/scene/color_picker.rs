//! `scene/gui/color_picker.*`: the color picker's state.
//!
//! Ported: the `ColorModeType` (RGB/HSV/linear alias/OKHSL) and
//! `PickerShapeType` enums with the engine's compatibility index mapping
//! (`SHAPE_NONE` is 4 but maps to index -1), the default shape/mode, the
//! visibility toggles, the alpha/intensity editing flags, the current color, the
//! cached HSV/OKHSL components, and the preset/recent-preset lists with the
//! engine's de-duplicating move-to-back and 9-slot recent limit.
//!
//! Sliders, the picker control, popups, the color wheel math and presets file
//! loading are not ported.

use crate::math::color::Color;

pub const MODE_SLIDER_COUNT: i32 = 3;
/// `PRESET_COLUMN_COUNT`, the recent-preset cap.
pub const PRESET_COLUMN_COUNT: usize = 9;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum ColorMode {
    Rgb = 0,
    Hsv = 1,
    /// `MODE_LINEAR` / the deprecated `MODE_RAW`.
    Linear = 2,
    Okhsl = 3,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum PickerShape {
    HsvRectangle = 0,
    HsvWheel = 1,
    VhsCircle = 2,
    OkhslCircle = 3,
    None = 4,
    OkHsRectangle = 5,
    OkHlRectangle = 6,
    Max = 7,
}

/// `shape_to_index`: `SHAPE_NONE` becomes -1, the shapes after it shift down.
pub fn shape_to_index(shape: PickerShape) -> i32 {
    if shape == PickerShape::None {
        return -1;
    }
    if shape as i32 > PickerShape::None as i32 {
        return shape as i32 - 1;
    }
    shape as i32
}

/// `index_to_shape`.
pub fn index_to_shape(index: i32) -> PickerShape {
    if index == -1 {
        return PickerShape::None;
    }
    if index >= PickerShape::None as i32 {
        return match index + 1 {
            5 => PickerShape::OkHsRectangle,
            6 => PickerShape::OkHlRectangle,
            _ => PickerShape::Max,
        };
    }
    match index {
        0 => PickerShape::HsvRectangle,
        1 => PickerShape::HsvWheel,
        2 => PickerShape::VhsCircle,
        3 => PickerShape::OkhslCircle,
        _ => PickerShape::HsvRectangle,
    }
}

#[derive(Clone, Debug)]
pub struct ColorPickerData {
    color: Color,
    edit_alpha: bool,
    edit_intensity: bool,
    colorize_sliders: bool,
    current_shape: PickerShape,
    current_mode: ColorMode,
    presets: Vec<Color>,
    recent_presets: Vec<Color>,
    presets_visible: bool,
    color_modes_visible: bool,
    sampler_visible: bool,
    sliders_visible: bool,
    hex_visible: bool,
    can_add_swatches: bool,
    deferred_mode_enabled: bool,
    /// Cached HSV components.
    pub h: f32,
    pub s: f32,
    pub v: f32,
    /// Cached OKHSL components.
    pub ok_hsl_h: f32,
    pub ok_hsl_s: f32,
    pub ok_hsl_l: f32,
    pub intensity: f32,
}

impl Default for ColorPickerData {
    fn default() -> Self {
        Self::new()
    }
}

impl ColorPickerData {
    pub fn new() -> Self {
        Self {
            // `Color()` is opaque black in the engine.
            color: Color::new(0.0, 0.0, 0.0, 1.0),
            edit_alpha: true,
            edit_intensity: true,
            colorize_sliders: true,
            current_shape: PickerShape::HsvRectangle,
            current_mode: ColorMode::Rgb,
            presets: Vec::new(),
            recent_presets: Vec::new(),
            presets_visible: true,
            color_modes_visible: true,
            sampler_visible: true,
            sliders_visible: true,
            hex_visible: true,
            can_add_swatches: true,
            deferred_mode_enabled: false,
            h: 0.0,
            s: 0.0,
            v: 0.0,
            ok_hsl_h: 0.0,
            ok_hsl_s: 0.0,
            ok_hsl_l: 0.0,
            intensity: 0.0,
        }
    }

    pub fn set_color(&mut self, color: Color) {
        self.color = color;
    }

    pub fn get_color(&self) -> Color {
        self.color
    }

    pub fn set_edit_alpha(&mut self, enabled: bool) {
        self.edit_alpha = enabled;
    }

    pub fn is_editing_alpha(&self) -> bool {
        self.edit_alpha
    }

    pub fn set_edit_intensity(&mut self, enabled: bool) {
        self.edit_intensity = enabled;
    }

    pub fn is_editing_intensity(&self) -> bool {
        self.edit_intensity
    }

    pub fn set_colorize_sliders(&mut self, enabled: bool) {
        self.colorize_sliders = enabled;
    }

    pub fn is_colorize_sliders_enabled(&self) -> bool {
        self.colorize_sliders
    }

    /// The engine takes the enum directly; the port validates the range.
    pub fn set_picker_shape(&mut self, shape: i32) -> bool {
        let parsed = match shape {
            0 => PickerShape::HsvRectangle,
            1 => PickerShape::HsvWheel,
            2 => PickerShape::VhsCircle,
            3 => PickerShape::OkhslCircle,
            4 => PickerShape::None,
            5 => PickerShape::OkHsRectangle,
            6 => PickerShape::OkHlRectangle,
            _ => return false,
        };
        self.current_shape = parsed;
        true
    }

    pub fn get_picker_shape(&self) -> PickerShape {
        self.current_shape
    }

    pub fn get_current_shape_index(&self) -> i32 {
        shape_to_index(self.current_shape)
    }

    pub fn set_color_mode(&mut self, mode: i32) -> bool {
        let parsed = match mode {
            0 => ColorMode::Rgb,
            1 => ColorMode::Hsv,
            2 => ColorMode::Linear,
            3 => ColorMode::Okhsl,
            _ => return false,
        };
        self.current_mode = parsed;
        true
    }

    pub fn get_color_mode(&self) -> ColorMode {
        self.current_mode
    }

    /// `add_preset`: an existing preset moves to the back.
    pub fn add_preset(&mut self, color: Color) {
        if let Some(index) = self.presets.iter().position(|known| *known == color) {
            let value = self.presets.remove(index);
            self.presets.push(value);
        } else {
            self.presets.push(color);
        }
    }

    pub fn erase_preset(&mut self, color: Color) -> bool {
        match self.presets.iter().position(|known| *known == color) {
            Some(index) => {
                self.presets.remove(index);
                true
            }
            None => false,
        }
    }

    pub fn get_presets(&self) -> &[Color] {
        &self.presets
    }

    /// `add_recent_preset`: oldest entries drop out at the 9-slot cap.
    pub fn add_recent_preset(&mut self, color: Color) {
        if let Some(index) = self.recent_presets.iter().position(|known| *known == color) {
            let value = self.recent_presets.remove(index);
            self.recent_presets.push(value);
            return;
        }
        if self.recent_presets.len() >= PRESET_COLUMN_COUNT {
            self.recent_presets.remove(0);
        }
        self.recent_presets.push(color);
    }

    pub fn erase_recent_preset(&mut self, color: Color) -> bool {
        match self.recent_presets.iter().position(|known| *known == color) {
            Some(index) => {
                self.recent_presets.remove(index);
                true
            }
            None => false,
        }
    }

    pub fn get_recent_presets(&self) -> &[Color] {
        &self.recent_presets
    }

    pub fn set_presets_visible(&mut self, visible: bool) {
        self.presets_visible = visible;
    }

    pub fn are_presets_visible(&self) -> bool {
        self.presets_visible
    }

    pub fn set_color_modes_visible(&mut self, visible: bool) {
        self.color_modes_visible = visible;
    }

    pub fn are_color_modes_visible(&self) -> bool {
        self.color_modes_visible
    }

    pub fn set_sampler_visible(&mut self, visible: bool) {
        self.sampler_visible = visible;
    }

    pub fn is_sampler_visible(&self) -> bool {
        self.sampler_visible
    }

    pub fn set_sliders_visible(&mut self, visible: bool) {
        self.sliders_visible = visible;
    }

    pub fn are_sliders_visible(&self) -> bool {
        self.sliders_visible
    }

    pub fn set_hex_visible(&mut self, visible: bool) {
        self.hex_visible = visible;
    }

    pub fn is_hex_visible(&self) -> bool {
        self.hex_visible
    }

    pub fn set_can_add_swatches(&mut self, enabled: bool) {
        self.can_add_swatches = enabled;
    }

    pub fn can_add_swatches(&self) -> bool {
        self.can_add_swatches
    }

    pub fn set_deferred_mode(&mut self, enabled: bool) {
        self.deferred_mode_enabled = enabled;
    }

    pub fn is_deferred_mode(&self) -> bool {
        self.deferred_mode_enabled
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enum_mapping() {
        assert_eq!(ColorMode::Rgb as i32, 0);
        assert_eq!(ColorMode::Hsv as i32, 1);
        assert_eq!(ColorMode::Linear as i32, 2);
        assert_eq!(ColorMode::Okhsl as i32, 3);
        assert_eq!(PickerShape::HsvRectangle as i32, 0);
        assert_eq!(PickerShape::OkhslCircle as i32, 3);
        assert_eq!(PickerShape::None as i32, 4);
        assert_eq!(PickerShape::OkHsRectangle as i32, 5);
        assert_eq!(PickerShape::OkHlRectangle as i32, 6);
        assert_eq!(PickerShape::Max as i32, 7);

        assert_eq!(shape_to_index(PickerShape::HsvRectangle), 0);
        assert_eq!(shape_to_index(PickerShape::OkhslCircle), 3);
        assert_eq!(shape_to_index(PickerShape::None), -1);
        assert_eq!(shape_to_index(PickerShape::OkHsRectangle), 4);
        assert_eq!(shape_to_index(PickerShape::OkHlRectangle), 5);

        assert_eq!(index_to_shape(-1), PickerShape::None);
        assert_eq!(index_to_shape(0), PickerShape::HsvRectangle);
        assert_eq!(index_to_shape(3), PickerShape::OkhslCircle);
        assert_eq!(index_to_shape(4), PickerShape::OkHsRectangle);
        assert_eq!(index_to_shape(5), PickerShape::OkHlRectangle);
    }

    #[test]
    fn defaults_and_toggles() {
        let mut picker = ColorPickerData::new();
        assert_eq!(picker.get_color(), Color::new(0.0, 0.0, 0.0, 1.0));
        assert!(picker.is_editing_alpha());
        assert!(picker.is_editing_intensity());
        assert!(picker.is_colorize_sliders_enabled());
        assert_eq!(picker.get_picker_shape(), PickerShape::HsvRectangle);
        assert_eq!(picker.get_current_shape_index(), 0);
        assert_eq!(picker.get_color_mode(), ColorMode::Rgb);
        assert!(picker.are_presets_visible());
        assert!(picker.are_color_modes_visible());
        assert!(picker.is_sampler_visible());
        assert!(picker.are_sliders_visible());
        assert!(picker.is_hex_visible());
        assert!(picker.can_add_swatches());
        assert!(!picker.is_deferred_mode());
        assert!(picker.get_presets().is_empty());
        assert!(picker.get_recent_presets().is_empty());
        assert_eq!(MODE_SLIDER_COUNT, 3);
        assert_eq!(PRESET_COLUMN_COUNT, 9);

        picker.set_color(Color::new(0.1, 0.2, 0.3, 0.4));
        picker.set_edit_alpha(false);
        picker.set_edit_intensity(false);
        picker.set_colorize_sliders(false);
        assert!(picker.set_picker_shape(4));
        assert!(!picker.set_picker_shape(7));
        assert!(picker.set_color_mode(3));
        assert!(!picker.set_color_mode(4));
        picker.set_presets_visible(false);
        picker.set_color_modes_visible(false);
        picker.set_sampler_visible(false);
        picker.set_sliders_visible(false);
        picker.set_hex_visible(false);
        picker.set_can_add_swatches(false);
        picker.set_deferred_mode(true);

        assert_eq!(picker.get_color(), Color::new(0.1, 0.2, 0.3, 0.4));
        assert!(!picker.is_editing_alpha());
        assert!(!picker.is_editing_intensity());
        assert!(!picker.is_colorize_sliders_enabled());
        assert_eq!(picker.get_picker_shape(), PickerShape::None);
        assert_eq!(picker.get_current_shape_index(), -1);
        assert_eq!(picker.get_color_mode(), ColorMode::Okhsl);
        assert!(!picker.are_presets_visible());
        assert!(!picker.are_color_modes_visible());
        assert!(!picker.is_sampler_visible());
        assert!(!picker.are_sliders_visible());
        assert!(!picker.is_hex_visible());
        assert!(!picker.can_add_swatches());
        assert!(picker.is_deferred_mode());
    }

    #[test]
    fn presets_and_recent_history() {
        let mut picker = ColorPickerData::new();
        let red = Color::new(1.0, 0.0, 0.0, 1.0);
        let green = Color::new(0.0, 1.0, 0.0, 1.0);
        let blue = Color::new(0.0, 0.0, 1.0, 1.0);

        picker.add_preset(red);
        picker.add_preset(green);
        assert_eq!(picker.get_presets(), &[red, green]);
        // Re-adding moves the preset to the back without duplicating it.
        picker.add_preset(red);
        assert_eq!(picker.get_presets(), &[green, red]);
        assert!(picker.erase_preset(green));
        assert!(!picker.erase_preset(green));
        assert_eq!(picker.get_presets(), &[red]);

        // Recent presets de-duplicate and cap at nine entries.
        picker.add_recent_preset(red);
        picker.add_recent_preset(green);
        picker.add_recent_preset(red);
        assert_eq!(picker.get_recent_presets(), &[green, red]);
        for index in 0..PRESET_COLUMN_COUNT + 2 {
            picker.add_recent_preset(Color::new(index as f32 / 16.0, 0.5, 0.5, 1.0));
        }
        assert_eq!(picker.get_recent_presets().len(), PRESET_COLUMN_COUNT);
        // The oldest entries (red/green) were pushed out by the cap.
        assert!(!picker.erase_recent_preset(red));
        let newest = Color::new((PRESET_COLUMN_COUNT + 1) as f32 / 16.0, 0.5, 0.5, 1.0);
        assert!(picker.erase_recent_preset(newest));
        assert!(!picker.erase_recent_preset(newest));
        assert_eq!(picker.get_recent_presets().len(), PRESET_COLUMN_COUNT - 1);

        // The HSV/OKHSL caches and intensity are plain fields.
        picker.h = 0.5;
        picker.s = 0.25;
        picker.v = 0.75;
        picker.ok_hsl_h = 0.1;
        picker.ok_hsl_s = 0.2;
        picker.ok_hsl_l = 0.3;
        picker.intensity = 2.0;
        assert_eq!((picker.h, picker.s, picker.v), (0.5, 0.25, 0.75));
        assert_eq!(
            (picker.ok_hsl_h, picker.ok_hsl_s, picker.ok_hsl_l),
            (0.1, 0.2, 0.3)
        );
        assert_eq!(picker.intensity, 2.0);
        let _ = blue;
    }
}
