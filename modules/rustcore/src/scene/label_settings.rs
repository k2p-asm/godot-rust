//! `scene/resources/label_settings.*`: label typography settings.
//!
//! Defaults: line spacing 3, paragraph spacing 0, font size 16
//! (`Font::DEFAULT_FONT_SIZE`), white font/outline colours, no outline, shadow
//! size 1 with a transparent black colour and offset `(1, 1)`.
//!
//! The stacked outlines and shadows (added in 4.7) are ported as ordered lists
//! with the engine's index rules: negative add index appends, move/remove/set
//! validate their indices, `set_*_count` rejects negative counts and resizes.
//! The font is reduced to a `Rid` handle; text layout/rendering is not ported.

use crate::math::color::Color;
use crate::math::vector2::Vector2;
use crate::servers::rid::Rid;

#[derive(Clone, Copy, Debug, Default)]
pub struct StackedOutlineData {
    pub size: i32,
    pub color: Color,
}

#[derive(Clone, Copy, Debug)]
pub struct StackedShadowData {
    pub offset: Vector2,
    pub color: Color,
    pub outline_size: i32,
}

impl Default for StackedShadowData {
    fn default() -> Self {
        Self {
            offset: Vector2::new(1.0, 1.0),
            color: Color::ZERO,
            outline_size: 0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct LabelSettingsData {
    line_spacing: f32,
    paragraph_spacing: f32,
    font: Option<Rid>,
    font_size: i32,
    font_color: Color,
    outline_size: i32,
    outline_color: Color,
    shadow_size: i32,
    shadow_color: Color,
    shadow_offset: Vector2,
    stacked_outlines: Vec<StackedOutlineData>,
    stacked_shadows: Vec<StackedShadowData>,
}

impl Default for LabelSettingsData {
    fn default() -> Self {
        Self::new()
    }
}

impl LabelSettingsData {
    pub fn new() -> Self {
        Self {
            line_spacing: 3.0,
            paragraph_spacing: 0.0,
            font: None,
            font_size: 16,
            font_color: Color::rgb(1.0, 1.0, 1.0),
            outline_size: 0,
            outline_color: Color::rgb(1.0, 1.0, 1.0),
            shadow_size: 1,
            shadow_color: Color::ZERO,
            shadow_offset: Vector2::new(1.0, 1.0),
            stacked_outlines: Vec::new(),
            stacked_shadows: Vec::new(),
        }
    }

    pub fn set_line_spacing(&mut self, spacing: f32) {
        self.line_spacing = spacing;
    }

    pub fn get_line_spacing(&self) -> f32 {
        self.line_spacing
    }

    pub fn set_paragraph_spacing(&mut self, spacing: f32) {
        self.paragraph_spacing = spacing;
    }

    pub fn get_paragraph_spacing(&self) -> f32 {
        self.paragraph_spacing
    }

    pub fn set_font(&mut self, font: Option<Rid>) {
        self.font = font;
    }

    pub fn get_font(&self) -> Option<Rid> {
        self.font
    }

    pub fn set_font_size(&mut self, size: i32) {
        self.font_size = size;
    }

    pub fn get_font_size(&self) -> i32 {
        self.font_size
    }

    pub fn set_font_color(&mut self, color: Color) {
        self.font_color = color;
    }

    pub fn get_font_color(&self) -> Color {
        self.font_color
    }

    pub fn set_outline_size(&mut self, size: i32) {
        self.outline_size = size;
    }

    pub fn get_outline_size(&self) -> i32 {
        self.outline_size
    }

    pub fn set_outline_color(&mut self, color: Color) {
        self.outline_color = color;
    }

    pub fn get_outline_color(&self) -> Color {
        self.outline_color
    }

    pub fn set_shadow_size(&mut self, size: i32) {
        self.shadow_size = size;
    }

    pub fn get_shadow_size(&self) -> i32 {
        self.shadow_size
    }

    pub fn set_shadow_color(&mut self, color: Color) {
        self.shadow_color = color;
    }

    pub fn get_shadow_color(&self) -> Color {
        self.shadow_color
    }

    pub fn set_shadow_offset(&mut self, offset: Vector2) {
        self.shadow_offset = offset;
    }

    pub fn get_shadow_offset(&self) -> Vector2 {
        self.shadow_offset
    }

    // -- stacked outlines ---------------------------------------------------

    pub fn get_stacked_outline_count(&self) -> usize {
        self.stacked_outlines.len()
    }

    /// Negative counts are rejected; otherwise the list is resized.
    pub fn set_stacked_outline_count(&mut self, count: i32) -> bool {
        if count < 0 {
            return false;
        }
        self.stacked_outlines
            .resize(count as usize, StackedOutlineData::default());
        true
    }

    /// `index < 0` appends; otherwise it must be in `0..=len`.
    pub fn add_stacked_outline(&mut self, index: i32) -> bool {
        let index = if index < 0 {
            self.stacked_outlines.len()
        } else {
            index as usize
        };
        if index > self.stacked_outlines.len() {
            return false;
        }
        self.stacked_outlines
            .insert(index, StackedOutlineData::default());
        true
    }

    pub fn move_stacked_outline(&mut self, from: usize, to: usize) -> bool {
        if from >= self.stacked_outlines.len() || to > self.stacked_outlines.len() {
            return false;
        }
        let item = self.stacked_outlines[from];
        self.stacked_outlines.insert(to, item);
        let remove_at = if to < from { from + 1 } else { from };
        self.stacked_outlines.remove(remove_at);
        true
    }

    pub fn remove_stacked_outline(&mut self, index: usize) -> bool {
        if index >= self.stacked_outlines.len() {
            return false;
        }
        self.stacked_outlines.remove(index);
        true
    }

    pub fn set_stacked_outline_size(&mut self, index: usize, size: i32) -> bool {
        match self.stacked_outlines.get_mut(index) {
            Some(o) => {
                o.size = size;
                true
            }
            None => false,
        }
    }

    pub fn get_stacked_outline_size(&self, index: usize) -> Option<i32> {
        self.stacked_outlines.get(index).map(|o| o.size)
    }

    pub fn set_stacked_outline_color(&mut self, index: usize, color: Color) -> bool {
        match self.stacked_outlines.get_mut(index) {
            Some(o) => {
                o.color = color;
                true
            }
            None => false,
        }
    }

    pub fn get_stacked_outline_color(&self, index: usize) -> Option<Color> {
        self.stacked_outlines.get(index).map(|o| o.color)
    }

    pub fn get_stacked_outlines(&self) -> &[StackedOutlineData] {
        &self.stacked_outlines
    }

    pub fn get_stacked_outline(&self, index: usize) -> Option<StackedOutlineData> {
        self.stacked_outlines.get(index).copied()
    }

    // -- stacked shadows ----------------------------------------------------

    pub fn get_stacked_shadow_count(&self) -> usize {
        self.stacked_shadows.len()
    }

    pub fn set_stacked_shadow_count(&mut self, count: i32) -> bool {
        if count < 0 {
            return false;
        }
        self.stacked_shadows
            .resize(count as usize, StackedShadowData::default());
        true
    }

    pub fn add_stacked_shadow(&mut self, index: i32) -> bool {
        let index = if index < 0 {
            self.stacked_shadows.len()
        } else {
            index as usize
        };
        if index > self.stacked_shadows.len() {
            return false;
        }
        self.stacked_shadows
            .insert(index, StackedShadowData::default());
        true
    }

    pub fn move_stacked_shadow(&mut self, from: usize, to: usize) -> bool {
        if from >= self.stacked_shadows.len() || to > self.stacked_shadows.len() {
            return false;
        }
        let item = self.stacked_shadows[from];
        self.stacked_shadows.insert(to, item);
        let remove_at = if to < from { from + 1 } else { from };
        self.stacked_shadows.remove(remove_at);
        true
    }

    pub fn remove_stacked_shadow(&mut self, index: usize) -> bool {
        if index >= self.stacked_shadows.len() {
            return false;
        }
        self.stacked_shadows.remove(index);
        true
    }

    pub fn set_stacked_shadow_offset(&mut self, index: usize, offset: Vector2) -> bool {
        match self.stacked_shadows.get_mut(index) {
            Some(s) => {
                s.offset = offset;
                true
            }
            None => false,
        }
    }

    pub fn get_stacked_shadow_offset(&self, index: usize) -> Option<Vector2> {
        self.stacked_shadows.get(index).map(|s| s.offset)
    }

    pub fn set_stacked_shadow_color(&mut self, index: usize, color: Color) -> bool {
        match self.stacked_shadows.get_mut(index) {
            Some(s) => {
                s.color = color;
                true
            }
            None => false,
        }
    }

    pub fn get_stacked_shadow_color(&self, index: usize) -> Option<Color> {
        self.stacked_shadows.get(index).map(|s| s.color)
    }

    pub fn set_stacked_shadow_outline_size(&mut self, index: usize, size: i32) -> bool {
        match self.stacked_shadows.get_mut(index) {
            Some(s) => {
                s.outline_size = size;
                true
            }
            None => false,
        }
    }

    pub fn get_stacked_shadow_outline_size(&self, index: usize) -> Option<i32> {
        self.stacked_shadows.get(index).map(|s| s.outline_size)
    }

    pub fn get_stacked_shadows(&self) -> &[StackedShadowData] {
        &self.stacked_shadows
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults() {
        let l = LabelSettingsData::new();
        assert_eq!(l.get_line_spacing(), 3.0);
        assert_eq!(l.get_paragraph_spacing(), 0.0);
        assert!(l.get_font().is_none());
        assert_eq!(l.get_font_size(), 16);
        assert_eq!(l.get_font_color(), Color::rgb(1.0, 1.0, 1.0));
        assert_eq!(l.get_outline_size(), 0);
        assert_eq!(l.get_outline_color(), Color::rgb(1.0, 1.0, 1.0));
        assert_eq!(l.get_shadow_size(), 1);
        assert_eq!(l.get_shadow_color(), Color::ZERO);
        assert_eq!(l.get_shadow_offset(), Vector2::new(1.0, 1.0));
        assert_eq!(l.get_stacked_outline_count(), 0);
        assert_eq!(l.get_stacked_shadow_count(), 0);
    }

    #[test]
    fn core_setters_round_trip() {
        let mut l = LabelSettingsData::new();
        l.set_line_spacing(5.0);
        l.set_paragraph_spacing(2.0);
        l.set_font(Some(Rid::from_u64(7)));
        l.set_font_size(32);
        l.set_font_color(Color::new(0.5, 0.25, 0.125, 1.0));
        l.set_outline_size(4);
        l.set_outline_color(Color::rgb(0.0, 0.0, 0.0));
        l.set_shadow_size(3);
        l.set_shadow_color(Color::new(0.0, 0.0, 0.0, 0.5));
        l.set_shadow_offset(Vector2::new(-1.0, 2.0));

        assert_eq!(l.get_line_spacing(), 5.0);
        assert_eq!(l.get_paragraph_spacing(), 2.0);
        assert_eq!(l.get_font(), Some(Rid::from_u64(7)));
        assert_eq!(l.get_font_size(), 32);
        assert_eq!(l.get_font_color(), Color::new(0.5, 0.25, 0.125, 1.0));
        assert_eq!(l.get_outline_size(), 4);
        assert_eq!(l.get_outline_color(), Color::rgb(0.0, 0.0, 0.0));
        assert_eq!(l.get_shadow_size(), 3);
        assert_eq!(l.get_shadow_color(), Color::new(0.0, 0.0, 0.0, 0.5));
        assert_eq!(l.get_shadow_offset(), Vector2::new(-1.0, 2.0));

        assert!(!l.set_stacked_outline_count(-1));
        assert!(!l.set_stacked_shadow_count(-1));
    }

    #[test]
    fn stacked_outlines_ordering_and_validation() {
        let mut l = LabelSettingsData::new();
        assert!(l.add_stacked_outline(-1));
        assert!(l.add_stacked_outline(-1));
        assert_eq!(l.get_stacked_outline_count(), 2);

        assert!(l.set_stacked_outline_size(0, 3));
        assert!(l.set_stacked_outline_size(1, 5));
        assert!(l.set_stacked_outline_color(1, Color::rgb(1.0, 0.0, 0.0)));
        assert_eq!(l.get_stacked_outline_size(0), Some(3));
        assert_eq!(l.get_stacked_outline_size(1), Some(5));
        assert_eq!(l.get_stacked_outline_color(1), Some(Color::rgb(1.0, 0.0, 0.0)));

        // Move 0 to the end: sizes become [5, 3].
        assert!(l.move_stacked_outline(0, 2));
        assert_eq!(l.get_stacked_outline_size(0), Some(5));
        assert_eq!(l.get_stacked_outline_size(1), Some(3));

        assert!(!l.move_stacked_outline(2, 0));
        assert!(!l.move_stacked_outline(0, 3));
        assert!(!l.set_stacked_outline_size(2, 1));
        assert_eq!(l.get_stacked_outline_size(9), None);

        assert!(l.remove_stacked_outline(0));
        assert_eq!(l.get_stacked_outline_count(), 1);
        assert!(!l.remove_stacked_outline(1));

        // Count resize keeps the existing entries and pads with defaults.
        assert!(l.set_stacked_outline_count(3));
        assert_eq!(l.get_stacked_outline_count(), 3);
        assert_eq!(l.get_stacked_outline_size(0), Some(3));
        assert_eq!(l.get_stacked_outline_size(1), Some(0));
        assert_eq!(l.get_stacked_outline(2).map(|o| o.size), Some(0));
    }

    #[test]
    fn stacked_shadows_ordering_and_validation() {
        let mut l = LabelSettingsData::new();
        assert!(l.add_stacked_shadow(-1));
        assert!(l.add_stacked_shadow(-1));
        assert_eq!(l.get_stacked_shadow_count(), 2);

        assert!(l.set_stacked_shadow_offset(0, Vector2::new(2.0, 3.0)));
        assert!(l.set_stacked_shadow_color(0, Color::rgb(0.0, 0.0, 1.0)));
        assert!(l.set_stacked_shadow_outline_size(0, 4));
        assert_eq!(l.get_stacked_shadow_offset(0), Some(Vector2::new(2.0, 3.0)));
        assert_eq!(l.get_stacked_shadow_color(0), Some(Color::rgb(0.0, 0.0, 1.0)));
        assert_eq!(l.get_stacked_shadow_outline_size(0), Some(4));

        assert!(l.move_stacked_shadow(1, 0));
        assert!(!l.move_stacked_shadow(0, 3));
        assert!(!l.set_stacked_shadow_offset(5, Vector2::ZERO));
        assert!(l.remove_stacked_shadow(0));
        assert_eq!(l.get_stacked_shadow_count(), 1);
    }
}
