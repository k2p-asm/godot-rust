//! `core/input/input_event.*`: the input event payloads (subset).
//!
//! Ported: the base `InputEvent` state (device, canceled, pressed), window-scoped
//! and modifier events, key events (keycode/physical/label/unicode/location/echo),
//! mouse events (button mask, position, global position), mouse buttons
//! (factor/index/double click), mouse motion (tilt/pressure/relative/screen
//! relative/velocity/pen inversion) and action events (action name/strength/
//! event index).
//!
//! The engine's `is_action`/`action_match` need a live `InputMap`; here each
//! event exposes `matches` (the payload comparison used by
//! [`crate::input::input_map::InputMap`]) and the map performs the action lookup.
//! Joypad, screen-touch/drag and MIDI events are not ported.

use crate::math::vector2::Vector2;
use crate::string::string_name::StringName;

pub const DEVICE_ID_EMULATION: i32 = -1;
pub const DEVICE_ID_INTERNAL: i32 = -2;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum KeyLocation {
    #[default]
    Unspecified,
    Left,
    Right,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum MouseButton {
    None,
    Left,
    Right,
    Middle,
    WheelUp,
    WheelDown,
    WheelLeft,
    WheelRight,
    Xbutton1,
    Xbutton2,
    Max,
}

pub const MOUSE_BUTTON_MASK_LEFT: u32 = 1 << 0;
pub const MOUSE_BUTTON_MASK_RIGHT: u32 = 1 << 1;
pub const MOUSE_BUTTON_MASK_MIDDLE: u32 = 1 << 2;
pub const MOUSE_BUTTON_MASK_XBUTTON1: u32 = 1 << 3;
pub const MOUSE_BUTTON_MASK_XBUTTON2: u32 = 1 << 4;

// -- base -------------------------------------------------------------------

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct InputEvent {
    pub device: i32,
    pub canceled: bool,
    pub pressed: bool,
}

impl InputEvent {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_device(&mut self, device: i32) {
        self.device = device;
    }

    pub fn get_device(&self) -> i32 {
        self.device
    }

    pub fn set_canceled(&mut self, canceled: bool) {
        self.canceled = canceled;
    }

    pub fn is_canceled(&self) -> bool {
        self.canceled
    }

    pub fn set_pressed(&mut self, pressed: bool) {
        self.pressed = pressed;
    }

    pub fn is_pressed(&self) -> bool {
        self.pressed
    }

    pub fn is_released(&self) -> bool {
        !self.pressed
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct InputEventWithModifiers {
    pub event: InputEvent,
    pub window_id: i64,
    pub command_or_control_autoremap: bool,
    pub shift_pressed: bool,
    pub alt_pressed: bool,
    pub meta_pressed: bool,
    pub ctrl_pressed: bool,
}

impl InputEventWithModifiers {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_window_id(&mut self, id: i64) {
        self.window_id = id;
    }

    pub fn get_window_id(&self) -> i64 {
        self.window_id
    }

    pub fn set_command_or_control_autoremap(&mut self, enabled: bool) {
        self.command_or_control_autoremap = enabled;
    }

    pub fn is_command_or_control_autoremap(&self) -> bool {
        self.command_or_control_autoremap
    }

    /// On the port's Windows target the command key is Ctrl.
    pub fn is_command_or_control_pressed(&self) -> bool {
        if self.command_or_control_autoremap {
            self.meta_pressed
        } else {
            self.ctrl_pressed
        }
    }

    pub fn set_shift_pressed(&mut self, pressed: bool) {
        self.shift_pressed = pressed;
    }

    pub fn is_shift_pressed(&self) -> bool {
        self.shift_pressed
    }

    pub fn set_alt_pressed(&mut self, pressed: bool) {
        self.alt_pressed = pressed;
    }

    pub fn is_alt_pressed(&self) -> bool {
        self.alt_pressed
    }

    pub fn set_ctrl_pressed(&mut self, pressed: bool) {
        self.ctrl_pressed = pressed;
    }

    pub fn is_ctrl_pressed(&self) -> bool {
        self.ctrl_pressed
    }

    pub fn set_meta_pressed(&mut self, pressed: bool) {
        self.meta_pressed = pressed;
    }

    pub fn is_meta_pressed(&self) -> bool {
        self.meta_pressed
    }

    /// `set_modifiers_from_event`.
    pub fn set_modifiers_from_event(&mut self, other: &InputEventWithModifiers) {
        self.shift_pressed = other.shift_pressed;
        self.alt_pressed = other.alt_pressed;
        self.meta_pressed = other.meta_pressed;
        self.ctrl_pressed = other.ctrl_pressed;
        self.command_or_control_autoremap = other.command_or_control_autoremap;
    }
}

// -- key --------------------------------------------------------------------

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct InputEventKey {
    pub modifiers: InputEventWithModifiers,
    pub keycode: u32,
    pub physical_keycode: u32,
    pub key_label: u32,
    pub unicode: u32,
    pub location: KeyLocation,
    pub echo: bool,
}

impl InputEventKey {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_keycode(&mut self, keycode: u32) {
        self.keycode = keycode;
    }

    pub fn get_keycode(&self) -> u32 {
        self.keycode
    }

    pub fn set_physical_keycode(&mut self, keycode: u32) {
        self.physical_keycode = keycode;
    }

    pub fn get_physical_keycode(&self) -> u32 {
        self.physical_keycode
    }

    pub fn set_key_label(&mut self, label: u32) {
        self.key_label = label;
    }

    pub fn get_key_label(&self) -> u32 {
        self.key_label
    }

    pub fn set_unicode(&mut self, unicode: char) {
        self.unicode = unicode as u32;
    }

    pub fn get_unicode(&self) -> char {
        char::from_u32(self.unicode).unwrap_or('\0')
    }

    pub fn set_location(&mut self, location: KeyLocation) {
        self.location = location;
    }

    pub fn get_location(&self) -> KeyLocation {
        self.location
    }

    pub fn set_echo(&mut self, echo: bool) {
        self.echo = echo;
    }

    pub fn is_echo(&self) -> bool {
        self.echo
    }

    /// Payload comparison used by the input map. A zero keycode falls back to
    /// the physical keycode, like the engine's action matching.
    pub fn matches(&self, other: &InputEventKey) -> bool {
        let a = if self.keycode != 0 {
            self.keycode
        } else {
            self.physical_keycode
        };
        let b = if other.keycode != 0 {
            other.keycode
        } else {
            other.physical_keycode
        };
        a == b
            && a != 0
            && self.modifiers.shift_pressed == other.modifiers.shift_pressed
            && self.modifiers.alt_pressed == other.modifiers.alt_pressed
            && self.modifiers.ctrl_pressed == other.modifiers.ctrl_pressed
            && self.modifiers.meta_pressed == other.modifiers.meta_pressed
    }
}

// -- mouse ------------------------------------------------------------------

#[derive(Clone, Debug, Default, PartialEq)]
pub struct InputEventMouse {
    pub modifiers: InputEventWithModifiers,
    pub button_mask: u32,
    pub position: Vector2,
    pub global_position: Vector2,
}

impl InputEventMouse {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_button_mask(&mut self, mask: u32) {
        self.button_mask = mask;
    }

    pub fn get_button_mask(&self) -> u32 {
        self.button_mask
    }

    pub fn set_position(&mut self, position: Vector2) {
        self.position = position;
    }

    pub fn get_position(&self) -> Vector2 {
        self.position
    }

    pub fn set_global_position(&mut self, position: Vector2) {
        self.global_position = position;
    }

    pub fn get_global_position(&self) -> Vector2 {
        self.global_position
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct InputEventMouseButton {
    pub mouse: InputEventMouse,
    pub factor: f32,
    pub button_index: MouseButton,
    pub double_click: bool,
}

impl Default for InputEventMouseButton {
    fn default() -> Self {
        Self {
            mouse: InputEventMouse::default(),
            factor: 1.0,
            button_index: MouseButton::None,
            double_click: false,
        }
    }
}

impl InputEventMouseButton {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_factor(&mut self, factor: f32) {
        self.factor = factor;
    }

    pub fn get_factor(&self) -> f32 {
        self.factor
    }

    pub fn set_button_index(&mut self, index: MouseButton) {
        self.button_index = index;
    }

    pub fn get_button_index(&self) -> MouseButton {
        self.button_index
    }

    pub fn set_double_click(&mut self, double_click: bool) {
        self.double_click = double_click;
    }

    pub fn is_double_click(&self) -> bool {
        self.double_click
    }

    pub fn matches(&self, other: &InputEventMouseButton) -> bool {
        self.button_index == other.button_index && self.button_index != MouseButton::None
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct InputEventMouseMotion {
    pub mouse: InputEventMouse,
    pub tilt: Vector2,
    pub pressure: f32,
    pub relative: Vector2,
    pub screen_relative: Vector2,
    pub velocity: Vector2,
    pub screen_velocity: Vector2,
    pub pen_inverted: bool,
}

impl InputEventMouseMotion {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_tilt(&mut self, tilt: Vector2) {
        self.tilt = tilt;
    }

    pub fn get_tilt(&self) -> Vector2 {
        self.tilt
    }

    pub fn set_pressure(&mut self, pressure: f32) {
        self.pressure = pressure;
    }

    pub fn get_pressure(&self) -> f32 {
        self.pressure
    }

    pub fn set_pen_inverted(&mut self, inverted: bool) {
        self.pen_inverted = inverted;
    }

    pub fn get_pen_inverted(&self) -> bool {
        self.pen_inverted
    }

    pub fn set_relative(&mut self, relative: Vector2) {
        self.relative = relative;
    }

    pub fn get_relative(&self) -> Vector2 {
        self.relative
    }

    pub fn set_relative_screen_position(&mut self, relative: Vector2) {
        self.screen_relative = relative;
    }

    pub fn get_relative_screen_position(&self) -> Vector2 {
        self.screen_relative
    }

    pub fn set_velocity(&mut self, velocity: Vector2) {
        self.velocity = velocity;
    }

    pub fn get_velocity(&self) -> Vector2 {
        self.velocity
    }

    pub fn set_screen_velocity(&mut self, velocity: Vector2) {
        self.screen_velocity = velocity;
    }

    pub fn get_screen_velocity(&self) -> Vector2 {
        self.screen_velocity
    }
}

// -- action -----------------------------------------------------------------

#[derive(Clone, Debug, Default, PartialEq)]
pub struct InputEventAction {
    pub event: InputEvent,
    pub action: StringName,
    pub strength: f32,
    pub event_index: i32,
}

impl InputEventAction {
    pub fn new() -> Self {
        Self {
            event: InputEvent::new(),
            action: StringName::new(""),
            strength: 1.0,
            event_index: -1,
        }
    }

    pub fn set_action(&mut self, action: &str) {
        self.action = StringName::new(action);
    }

    pub fn get_action(&self) -> &StringName {
        &self.action
    }

    pub fn set_strength(&mut self, strength: f32) {
        self.strength = strength;
    }

    pub fn get_strength(&self) -> f32 {
        self.strength
    }

    pub fn set_event_index(&mut self, index: i32) {
        self.event_index = index;
    }

    pub fn get_event_index(&self) -> i32 {
        self.event_index
    }

    pub fn matches(&self, other: &InputEventAction) -> bool {
        self.action == other.action && self.action.as_string() != ""
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base_event_state() {
        let mut event = InputEvent::new();
        assert_eq!(event.get_device(), 0);
        assert!(!event.is_pressed());
        assert!(event.is_released());
        assert!(!event.is_canceled());
        event.set_device(DEVICE_ID_EMULATION);
        event.set_pressed(true);
        event.set_canceled(true);
        assert_eq!(event.get_device(), DEVICE_ID_EMULATION);
        assert!(event.is_pressed());
        assert!(!event.is_released());
        assert!(event.is_canceled());
    }

    #[test]
    fn modifiers_and_keys() {
        let mut key = InputEventKey::new();
        assert_eq!(key.get_keycode(), 0);
        assert_eq!(key.get_location(), KeyLocation::Unspecified);
        assert!(!key.is_echo());
        key.set_keycode(65);
        key.set_physical_keycode(30);
        key.set_unicode('A');
        key.set_echo(true);
        key.modifiers.set_ctrl_pressed(true);
        key.modifiers.set_shift_pressed(true);
        assert_eq!(key.get_keycode(), 65);
        assert_eq!(key.get_physical_keycode(), 30);
        assert_eq!(key.get_unicode(), 'A');
        assert!(key.is_echo());
        assert!(key.modifiers.is_ctrl_pressed());
        assert!(key.modifiers.is_command_or_control_pressed());

        let mut other = InputEventKey::new();
        other.set_keycode(65);
        other.modifiers.set_ctrl_pressed(true);
        other.modifiers.set_shift_pressed(true);
        assert!(key.matches(&other));

        other.set_keycode(66);
        assert!(!key.matches(&other));
        // Modifiers are part of the match.
        other.set_keycode(65);
        other.modifiers.set_alt_pressed(true);
        assert!(!key.matches(&other));
    }

    #[test]
    fn mouse_events() {
        let mut button = InputEventMouseButton::new();
        assert_eq!(button.get_factor(), 1.0);
        assert_eq!(button.get_button_index(), MouseButton::None);
        assert!(!button.is_double_click());
        button.set_button_index(MouseButton::Left);
        button.set_factor(2.0);
        button.set_double_click(true);
        button.mouse.set_position(Vector2::new(3.0, 4.0));
        button.mouse.set_global_position(Vector2::new(5.0, 6.0));
        button.mouse.set_button_mask(MOUSE_BUTTON_MASK_LEFT);
        assert_eq!(button.get_button_index(), MouseButton::Left);
        assert_eq!(button.get_factor(), 2.0);
        assert!(button.is_double_click());
        assert_eq!(button.mouse.get_position(), Vector2::new(3.0, 4.0));
        assert_eq!(button.mouse.get_global_position(), Vector2::new(5.0, 6.0));
        assert_eq!(button.mouse.get_button_mask(), MOUSE_BUTTON_MASK_LEFT);
        assert_eq!(MouseButton::Left as i32, 1);
        assert_eq!(MouseButton::WheelUp as i32, 4);

        let mut motion = InputEventMouseMotion::new();
        motion.set_relative(Vector2::new(1.0, 2.0));
        motion.set_velocity(Vector2::new(3.0, 4.0));
        motion.set_pressure(0.5);
        motion.set_tilt(Vector2::new(0.1, 0.2));
        motion.set_pen_inverted(true);
        assert_eq!(motion.get_relative(), Vector2::new(1.0, 2.0));
        assert_eq!(motion.get_velocity(), Vector2::new(3.0, 4.0));
        assert_eq!(motion.get_pressure(), 0.5);
        assert_eq!(motion.get_tilt(), Vector2::new(0.1, 0.2));
        assert!(motion.get_pen_inverted());
    }

    #[test]
    fn actions_and_modifier_copy() {
        let mut action = InputEventAction::new();
        assert_eq!(action.get_strength(), 1.0);
        assert_eq!(action.get_event_index(), -1);
        assert_eq!(action.get_action().as_string(), "");
        action.set_action("jump");
        action.set_strength(0.5);
        action.set_event_index(2);
        assert_eq!(action.get_action().as_string(), "jump");
        assert_eq!(action.get_strength(), 0.5);
        assert_eq!(action.get_event_index(), 2);

        let mut a = InputEventKey::new();
        a.modifiers.set_alt_pressed(true);
        a.modifiers.set_meta_pressed(true);
        let mut b = InputEventWithModifiers::new();
        b.set_modifiers_from_event(&a.modifiers);
        assert!(b.is_alt_pressed());
        assert!(b.is_meta_pressed());
        assert!(!b.is_shift_pressed());
    }
}
