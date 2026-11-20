//! `scene/2d/physics/touch_screen_button.*` (state only).
//!
//! Defaults: no textures/bitmask/shape, shape centred and visible, no action,
//! pass-by press off, no pressed finger (-1) and always visible.
//!
//! Input handling, the bitmask/shape hit test and the action emission are not
//! ported; `press`/`release` only move the pressed-finger state.

use crate::servers::rid::Rid;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum VisibilityMode {
    Always,
    TouchscreenOnly,
}

#[derive(Clone, Debug)]
pub struct TouchScreenButtonData {
    texture_normal: Option<Rid>,
    texture_pressed: Option<Rid>,
    bitmask: Option<Rid>,
    shape: Option<Rid>,
    shape_centered: bool,
    shape_visible: bool,
    action: String,
    passby_press: bool,
    finger_pressed: i32,
    visibility: VisibilityMode,
}

impl Default for TouchScreenButtonData {
    fn default() -> Self {
        Self::new()
    }
}

impl TouchScreenButtonData {
    pub fn new() -> Self {
        Self {
            texture_normal: None,
            texture_pressed: None,
            bitmask: None,
            shape: None,
            shape_centered: true,
            shape_visible: true,
            action: String::new(),
            passby_press: false,
            finger_pressed: -1,
            visibility: VisibilityMode::Always,
        }
    }

    pub fn set_texture_normal(&mut self, texture: Option<Rid>) {
        self.texture_normal = texture;
    }

    pub fn get_texture_normal(&self) -> Option<Rid> {
        self.texture_normal
    }

    pub fn set_texture_pressed(&mut self, texture: Option<Rid>) {
        self.texture_pressed = texture;
    }

    pub fn get_texture_pressed(&self) -> Option<Rid> {
        self.texture_pressed
    }

    pub fn set_bitmask(&mut self, bitmask: Option<Rid>) {
        self.bitmask = bitmask;
    }

    pub fn get_bitmask(&self) -> Option<Rid> {
        self.bitmask
    }

    pub fn set_shape(&mut self, shape: Option<Rid>) {
        self.shape = shape;
    }

    pub fn get_shape(&self) -> Option<Rid> {
        self.shape
    }

    pub fn set_shape_centered(&mut self, centered: bool) {
        self.shape_centered = centered;
    }

    pub fn is_shape_centered(&self) -> bool {
        self.shape_centered
    }

    pub fn set_shape_visible(&mut self, visible: bool) {
        self.shape_visible = visible;
    }

    pub fn is_shape_visible(&self) -> bool {
        self.shape_visible
    }

    pub fn set_action(&mut self, action: &str) {
        self.action = action.to_string();
    }

    pub fn get_action(&self) -> &str {
        &self.action
    }

    pub fn set_passby_press(&mut self, enabled: bool) {
        self.passby_press = enabled;
    }

    pub fn is_passby_press_enabled(&self) -> bool {
        self.passby_press
    }

    pub fn set_visibility_mode(&mut self, mode: i32) -> bool {
        let parsed = match mode {
            0 => VisibilityMode::Always,
            1 => VisibilityMode::TouchscreenOnly,
            _ => return false,
        };
        self.visibility = parsed;
        true
    }

    pub fn get_visibility_mode(&self) -> VisibilityMode {
        self.visibility
    }

    /// `_press`.
    pub fn press(&mut self, finger: i32) {
        self.finger_pressed = finger;
    }

    /// `_release`.
    pub fn release(&mut self) {
        self.finger_pressed = -1;
    }

    pub fn is_pressed(&self) -> bool {
        self.finger_pressed != -1
    }

    pub fn get_pressed_finger(&self) -> i32 {
        self.finger_pressed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_and_state() {
        let mut button = TouchScreenButtonData::new();
        assert!(button.get_texture_normal().is_none());
        assert!(button.get_texture_pressed().is_none());
        assert!(button.get_bitmask().is_none());
        assert!(button.get_shape().is_none());
        assert!(button.is_shape_centered());
        assert!(button.is_shape_visible());
        assert_eq!(button.get_action(), "");
        assert!(!button.is_passby_press_enabled());
        assert!(!button.is_pressed());
        assert_eq!(button.get_pressed_finger(), -1);
        assert_eq!(button.get_visibility_mode(), VisibilityMode::Always);

        assert!(button.set_visibility_mode(1));
        assert!(!button.set_visibility_mode(2));
        assert_eq!(
            button.get_visibility_mode(),
            VisibilityMode::TouchscreenOnly
        );

        button.press(2);
        assert!(button.is_pressed());
        assert_eq!(button.get_pressed_finger(), 2);
        button.release();
        assert!(!button.is_pressed());

        button.set_texture_normal(Some(Rid::from_u64(1)));
        button.set_texture_pressed(Some(Rid::from_u64(2)));
        button.set_bitmask(Some(Rid::from_u64(3)));
        button.set_shape(Some(Rid::from_u64(4)));
        button.set_shape_centered(false);
        button.set_shape_visible(false);
        button.set_action("jump");
        button.set_passby_press(true);
        assert_eq!(button.get_texture_normal(), Some(Rid::from_u64(1)));
        assert_eq!(button.get_texture_pressed(), Some(Rid::from_u64(2)));
        assert_eq!(button.get_bitmask(), Some(Rid::from_u64(3)));
        assert_eq!(button.get_shape(), Some(Rid::from_u64(4)));
        assert!(!button.is_shape_centered());
        assert!(!button.is_shape_visible());
        assert_eq!(button.get_action(), "jump");
        assert!(button.is_passby_press_enabled());
    }
}
