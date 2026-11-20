//! `scene/gui/virtual_joystick.*`: parameters and the stick vector math.
//!
//! Defaults from the header: fixed mode, 100 px joystick, 50 px tip, deadzone 0,
//! clamp zone 1, initial offset ratio `(0.5, 0.5)`, the four `ui_*` actions and
//! always-visible.
//!
//! `_update_joystick` is ported exactly: in following mode the base is dragged so
//! the tip stays inside the clamp radius (only while the pointer is inside the
//! control), the offset is clamped to that radius, the raw vector is
//! `offset / radius`, and the input vector rescales through
//! `inverse_lerp(deadzone_radius, radius, length)` — reporting the one-shot
//! `flick_canceled` transition.
//!
//! The `Input` singleton calls and drawing are not ported; the caller receives
//! the vector and decides which actions to press.

use crate::math::vector2::Vector2;
use crate::string::string_name::StringName;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum JoystickMode {
    Fixed,
    Dynamic,
    Following,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum VisibilityMode {
    Always,
    WhenTouched,
}

#[derive(Clone, Debug)]
pub struct VirtualJoystickData {
    joystick_mode: JoystickMode,
    joystick_size: f32,
    tip_size: f32,
    deadzone_ratio: f32,
    clampzone_ratio: f32,
    initial_offset_ratio: Vector2,
    action_left: StringName,
    action_right: StringName,
    action_up: StringName,
    action_down: StringName,
    visibility: VisibilityMode,
    // Interaction state.
    is_pressed: bool,
    has_input: bool,
    has_moved: bool,
    raw_input_vector: Vector2,
    input_vector: Vector2,
    is_flick_canceled: bool,
    touch_index: i32,
    joystick_pos: Vector2,
    tip_pos: Vector2,
}

impl Default for VirtualJoystickData {
    fn default() -> Self {
        Self::new()
    }
}

impl VirtualJoystickData {
    pub fn new() -> Self {
        Self {
            joystick_mode: JoystickMode::Fixed,
            joystick_size: 100.0,
            tip_size: 50.0,
            deadzone_ratio: 0.0,
            clampzone_ratio: 1.0,
            initial_offset_ratio: Vector2::new(0.5, 0.5),
            action_left: StringName::new("ui_left"),
            action_right: StringName::new("ui_right"),
            action_up: StringName::new("ui_up"),
            action_down: StringName::new("ui_down"),
            visibility: VisibilityMode::Always,
            is_pressed: false,
            has_input: false,
            has_moved: false,
            raw_input_vector: Vector2::ZERO,
            input_vector: Vector2::ZERO,
            is_flick_canceled: false,
            touch_index: -1,
            joystick_pos: Vector2::ZERO,
            tip_pos: Vector2::ZERO,
        }
    }

    pub fn set_joystick_mode(&mut self, mode: JoystickMode) {
        self.joystick_mode = mode;
    }

    pub fn get_joystick_mode(&self) -> JoystickMode {
        self.joystick_mode
    }

    pub fn set_joystick_size(&mut self, size: f32) {
        self.joystick_size = size;
    }

    pub fn get_joystick_size(&self) -> f32 {
        self.joystick_size
    }

    pub fn set_tip_size(&mut self, size: f32) {
        self.tip_size = size;
    }

    pub fn get_tip_size(&self) -> f32 {
        self.tip_size
    }

    pub fn set_deadzone_ratio(&mut self, ratio: f32) {
        self.deadzone_ratio = ratio;
    }

    pub fn get_deadzone_ratio(&self) -> f32 {
        self.deadzone_ratio
    }

    pub fn set_clampzone_ratio(&mut self, ratio: f32) {
        self.clampzone_ratio = ratio;
    }

    pub fn get_clampzone_ratio(&self) -> f32 {
        self.clampzone_ratio
    }

    pub fn set_initial_offset_ratio(&mut self, ratio: Vector2) {
        self.initial_offset_ratio = ratio;
    }

    pub fn get_initial_offset_ratio(&self) -> Vector2 {
        self.initial_offset_ratio
    }

    pub fn set_action_left(&mut self, action: &str) {
        self.action_left = StringName::new(action);
    }

    pub fn get_action_left(&self) -> StringName {
        self.action_left
    }

    pub fn set_action_right(&mut self, action: &str) {
        self.action_right = StringName::new(action);
    }

    pub fn get_action_right(&self) -> StringName {
        self.action_right
    }

    pub fn set_action_up(&mut self, action: &str) {
        self.action_up = StringName::new(action);
    }

    pub fn get_action_up(&self) -> StringName {
        self.action_up
    }

    pub fn set_action_down(&mut self, action: &str) {
        self.action_down = StringName::new(action);
    }

    pub fn get_action_down(&self) -> StringName {
        self.action_down
    }

    pub fn set_visibility_mode(&mut self, mode: VisibilityMode) {
        self.visibility = mode;
    }

    pub fn get_visibility_mode(&self) -> VisibilityMode {
        self.visibility
    }

    // -- interaction state --------------------------------------------------

    pub fn get_joystick_position(&self) -> Vector2 {
        self.joystick_pos
    }

    pub fn get_tip_position(&self) -> Vector2 {
        self.tip_pos
    }

    pub fn get_input_vector(&self) -> Vector2 {
        self.input_vector
    }

    pub fn get_raw_input_vector(&self) -> Vector2 {
        self.raw_input_vector
    }

    pub fn has_input(&self) -> bool {
        self.has_input
    }

    pub fn is_pressed(&self) -> bool {
        self.is_pressed
    }

    pub fn set_pressed(&mut self, pressed: bool) {
        self.is_pressed = pressed;
    }

    pub fn has_moved(&self) -> bool {
        self.has_moved
    }

    pub fn set_moved(&mut self, moved: bool) {
        self.has_moved = moved;
    }

    pub fn is_flick_canceled(&self) -> bool {
        self.is_flick_canceled
    }

    pub fn get_touch_index(&self) -> i32 {
        self.touch_index
    }

    pub fn set_touch_index(&mut self, index: i32) {
        self.touch_index = index;
    }

    /// `_reset`: clears the interaction state and re-centres the stick from the
    /// control size and the initial offset ratio.
    pub fn reset(&mut self, control_size: Vector2) {
        self.is_pressed = false;
        self.has_input = false;
        self.has_moved = false;
        self.raw_input_vector = Vector2::ZERO;
        self.input_vector = Vector2::ZERO;
        self.is_flick_canceled = false;
        self.touch_index = -1;
        self.joystick_pos = control_size * self.initial_offset_ratio;
        self.tip_pos = self.joystick_pos;
    }

    /// `_update_joystick`. `point_inside` is the control's `has_point` result,
    /// which only matters for following mode. Returns whether `flick_canceled`
    /// fired on this call.
    pub fn update_joystick(&mut self, pos: Vector2, point_inside: bool) -> bool {
        let mut offset = pos - self.joystick_pos;
        let mut length = offset.length();
        let direction = offset.normalized();

        let clampzone_radius = self.joystick_size * 0.5 * self.clampzone_ratio;

        if self.joystick_mode == JoystickMode::Following
            && length > clampzone_radius
            && point_inside
        {
            self.joystick_pos = pos - direction * clampzone_radius;
        }

        if length > clampzone_radius {
            length = clampzone_radius;
            offset = direction * length;
        }

        self.tip_pos = self.joystick_pos + offset;
        let was_pressed = self.has_input;
        self.raw_input_vector = offset / clampzone_radius;

        if length > self.deadzone_ratio * clampzone_radius {
            self.has_input = true;
            let scaled = crate::math::funcs::inverse_lerp(
                self.deadzone_ratio * clampzone_radius,
                clampzone_radius,
                length,
            );
            self.input_vector = direction * scaled;
        } else {
            self.has_input = false;
            self.input_vector = Vector2::ZERO;
        }

        let mut flick_canceled = false;
        if !self.is_flick_canceled && was_pressed && !self.has_input {
            self.is_flick_canceled = true;
            flick_canceled = true;
        } else if self.is_flick_canceled && !was_pressed && self.has_input {
            self.is_flick_canceled = false;
        }
        flick_canceled
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults() {
        let j = VirtualJoystickData::new();
        assert_eq!(j.get_joystick_mode(), JoystickMode::Fixed);
        assert_eq!(j.get_joystick_size(), 100.0);
        assert_eq!(j.get_tip_size(), 50.0);
        assert_eq!(j.get_deadzone_ratio(), 0.0);
        assert_eq!(j.get_clampzone_ratio(), 1.0);
        assert_eq!(j.get_initial_offset_ratio(), Vector2::new(0.5, 0.5));
        assert_eq!(j.get_action_left().as_string(), "ui_left");
        assert_eq!(j.get_action_right().as_string(), "ui_right");
        assert_eq!(j.get_action_up().as_string(), "ui_up");
        assert_eq!(j.get_action_down().as_string(), "ui_down");
        assert_eq!(j.get_visibility_mode(), VisibilityMode::Always);
        assert_eq!(j.get_touch_index(), -1);
        assert!(!j.is_pressed());
        assert!(!j.has_input());
        assert!(!j.is_flick_canceled());
    }

    #[test]
    fn reset_centres_from_the_offset_ratio() {
        let mut j = VirtualJoystickData::new();
        j.reset(Vector2::new(200.0, 100.0));
        assert_eq!(j.get_joystick_position(), Vector2::new(100.0, 50.0));
        assert_eq!(j.get_tip_position(), Vector2::new(100.0, 50.0));

        j.set_initial_offset_ratio(Vector2::new(0.25, 1.0));
        j.reset(Vector2::new(200.0, 100.0));
        assert_eq!(j.get_joystick_position(), Vector2::new(50.0, 100.0));
    }

    #[test]
    fn fixed_mode_clamps_the_offset() {
        let mut j = VirtualJoystickData::new();
        j.reset(Vector2::new(0.0, 0.0));

        let flick = j.update_joystick(Vector2::new(75.0, 0.0), false);
        assert!(!flick);
        // Clamp radius is 100 * 0.5 * 1 = 50.
        assert_eq!(j.get_tip_position(), Vector2::new(50.0, 0.0));
        assert_eq!(j.get_raw_input_vector(), Vector2::new(1.0, 0.0));
        assert!((j.get_input_vector() - Vector2::new(1.0, 0.0)).length() < 1e-5);
        assert!(j.has_input());
        assert_eq!(j.get_joystick_position(), Vector2::ZERO, "fixed mode never moves");

        // Inside the clamp radius the vector scales linearly.
        j.update_joystick(Vector2::new(25.0, 0.0), false);
        assert!((j.get_input_vector() - Vector2::new(0.5, 0.0)).length() < 1e-5);
        assert_eq!(j.get_raw_input_vector(), Vector2::new(0.5, 0.0));
    }

    #[test]
    fn deadzone_scales_and_cancels_the_flick() {
        let mut j = VirtualJoystickData::new();
        j.set_deadzone_ratio(0.5);
        j.reset(Vector2::ZERO);

        // At the deadzone edge there is no input and no flick yet.
        j.update_joystick(Vector2::new(25.0, 0.0), false);
        assert!(!j.has_input());
        assert!(!j.is_flick_canceled());

        // Moving past it starts input, with the vector rescaled from the deadzone.
        j.update_joystick(Vector2::new(50.0, 0.0), false);
        assert!(j.has_input());
        assert!((j.get_input_vector() - Vector2::new(1.0, 0.0)).length() < 1e-5);
        j.update_joystick(Vector2::new(37.5, 0.0), false);
        assert!((j.get_input_vector() - Vector2::new(0.5, 0.0)).length() < 1e-5);

        // Dropping back inside the deadzone cancels the flick exactly once.
        assert!(j.update_joystick(Vector2::new(10.0, 0.0), false));
        assert!(j.is_flick_canceled());
        assert!(!j.has_input());
        assert!(!j.update_joystick(Vector2::new(10.0, 0.0), false));

        // Pressing again clears the flag.
        j.update_joystick(Vector2::new(50.0, 0.0), false);
        assert!(!j.is_flick_canceled());
    }

    #[test]
    fn following_mode_drags_the_base() {
        let mut j = VirtualJoystickData::new();
        j.set_joystick_mode(JoystickMode::Following);
        j.reset(Vector2::ZERO);

        j.update_joystick(Vector2::new(75.0, 0.0), true);
        // The base moves so the tip sits exactly under the pointer.
        assert_eq!(j.get_joystick_position(), Vector2::new(25.0, 0.0));
        assert_eq!(j.get_tip_position(), Vector2::new(75.0, 0.0));
        assert_eq!(j.get_raw_input_vector(), Vector2::new(1.0, 0.0));

        // Outside the control the base must not follow.
        let mut k = VirtualJoystickData::new();
        k.set_joystick_mode(JoystickMode::Following);
        k.reset(Vector2::ZERO);
        k.update_joystick(Vector2::new(75.0, 0.0), false);
        assert_eq!(k.get_joystick_position(), Vector2::ZERO);
        assert_eq!(k.get_tip_position(), Vector2::new(50.0, 0.0));
    }

    #[test]
    fn setters_and_enum_values() {
        let mut j = VirtualJoystickData::new();
        j.set_joystick_size(120.0);
        j.set_tip_size(60.0);
        j.set_deadzone_ratio(0.2);
        j.set_clampzone_ratio(0.8);
        j.set_action_left("move_left");
        j.set_action_down("move_down");
        j.set_visibility_mode(VisibilityMode::WhenTouched);
        j.set_pressed(true);
        j.set_moved(true);
        j.set_touch_index(3);

        assert_eq!(j.get_joystick_size(), 120.0);
        assert_eq!(j.get_tip_size(), 60.0);
        assert_eq!(j.get_deadzone_ratio(), 0.2);
        assert_eq!(j.get_clampzone_ratio(), 0.8);
        assert_eq!(j.get_action_left().as_string(), "move_left");
        assert_eq!(j.get_action_down().as_string(), "move_down");
        assert_eq!(j.get_visibility_mode(), VisibilityMode::WhenTouched);
        assert!(j.is_pressed());
        assert!(j.has_moved());
        assert_eq!(j.get_touch_index(), 3);

        assert_eq!(JoystickMode::Fixed as i32, 0);
        assert_eq!(JoystickMode::Dynamic as i32, 1);
        assert_eq!(JoystickMode::Following as i32, 2);
        assert_eq!(VisibilityMode::Always as i32, 0);
        assert_eq!(VisibilityMode::WhenTouched as i32, 1);
    }
}
