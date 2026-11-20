//! `core/input/input_map.*`: action bindings (state only).
//!
//! An action has a deadzone (0.2 by default, 0.5 for the built-in toggle
//! actions) and a list of input events. Ported: `has_action`, `add_action`,
//! `erase_action`, the deadzone accessors, `action_add_event`,
//! `action_has_event`, `action_erase_event`, `action_erase_events`,
//! `action_get_events` and `event_is_action`, plus the sorted `get_actions`.
//!
//! `load_from_project_settings`/`load_default`, the built-in UI actions, action
//! descriptions and the Input singleton are not ported; callers add actions and
//! events explicitly.

use std::collections::BTreeMap;

use crate::input::input_event::{
    InputEventAction, InputEventKey, InputEventMouseButton, InputEventMouseMotion,
};
use crate::input::input_event_extra::{InputEventJoypadButton, InputEventJoypadMotion};

pub const ALL_DEVICES: i32 = -1;
pub const DEFAULT_DEADZONE: f32 = 0.2;
pub const DEFAULT_TOGGLE_DEADZONE: f32 = 0.5;

/// A bound event, mirroring the engine's `Ref<InputEvent>` list entries.
#[derive(Clone, Debug, PartialEq)]
pub enum InputEventRef {
    Key(InputEventKey),
    MouseButton(InputEventMouseButton),
    MouseMotion(InputEventMouseMotion),
    Action(InputEventAction),
    JoypadButton(InputEventJoypadButton),
    JoypadMotion(InputEventJoypadMotion),
}

impl InputEventRef {
    /// The engine's `is_match` for the payload comparison.
    pub fn matches(&self, other: &InputEventRef) -> bool {
        match (self, other) {
            (InputEventRef::Key(a), InputEventRef::Key(b)) => a.matches(b),
            (InputEventRef::MouseButton(a), InputEventRef::MouseButton(b)) => a.matches(b),
            (InputEventRef::MouseMotion(_), InputEventRef::MouseMotion(_)) => false,
            (InputEventRef::Action(a), InputEventRef::Action(b)) => a.matches(b),
            (InputEventRef::JoypadButton(a), InputEventRef::JoypadButton(b)) => a.matches(b),
            (InputEventRef::JoypadMotion(a), InputEventRef::JoypadMotion(b)) => a.matches(b),
            _ => false,
        }
    }
}

#[derive(Clone, Debug)]
pub struct InputAction {
    pub deadzone: f32,
    pub events: Vec<InputEventRef>,
}

impl InputAction {
    pub fn new(deadzone: f32) -> Self {
        Self {
            deadzone,
            events: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct InputMapData {
    actions: BTreeMap<String, InputAction>,
}

impl InputMapData {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn has_action(&self, action: &str) -> bool {
        self.actions.contains_key(action)
    }

    /// `add_action`: refuses duplicates and negative deadzones.
    pub fn add_action(&mut self, action: &str, deadzone: f32) -> bool {
        if action.is_empty() || deadzone < 0.0 || self.actions.contains_key(action) {
            return false;
        }
        self.actions
            .insert(action.to_string(), InputAction::new(deadzone));
        true
    }

    pub fn erase_action(&mut self, action: &str) -> bool {
        self.actions.remove(action).is_some()
    }

    pub fn action_get_deadzone(&self, action: &str) -> f32 {
        self.actions
            .get(action)
            .map(|entry| entry.deadzone)
            .unwrap_or(0.0)
    }

    pub fn action_set_deadzone(&mut self, action: &str, deadzone: f32) -> bool {
        match self.actions.get_mut(action) {
            Some(entry) => {
                entry.deadzone = deadzone;
                true
            }
            None => false,
        }
    }

    pub fn action_add_event(&mut self, action: &str, event: InputEventRef) -> bool {
        match self.actions.get_mut(action) {
            Some(entry) => {
                entry.events.push(event);
                true
            }
            None => false,
        }
    }

    pub fn action_has_event(&self, action: &str, event: &InputEventRef) -> bool {
        match self.actions.get(action) {
            Some(entry) => entry.events.iter().any(|existing| existing.matches(event)),
            None => false,
        }
    }

    pub fn action_erase_event(&mut self, action: &str, event: &InputEventRef) -> bool {
        match self.actions.get_mut(action) {
            Some(entry) => match entry.events.iter().position(|existing| existing.matches(event)) {
                Some(index) => {
                    entry.events.remove(index);
                    true
                }
                None => false,
            },
            None => false,
        }
    }

    pub fn action_erase_events(&mut self, action: &str) -> bool {
        match self.actions.get_mut(action) {
            Some(entry) => {
                entry.events.clear();
                true
            }
            None => false,
        }
    }

    pub fn action_get_events(&self, action: &str) -> Option<&[InputEventRef]> {
        self.actions.get(action).map(|entry| entry.events.as_slice())
    }

    pub fn action_get_event_count(&self, action: &str) -> usize {
        self.actions
            .get(action)
            .map(|entry| entry.events.len())
            .unwrap_or(0)
    }

    /// `event_is_action`: any bound event matches the incoming one.
    pub fn event_is_action(&self, event: &InputEventRef, action: &str) -> bool {
        match self.actions.get(action) {
            Some(entry) => entry.events.iter().any(|bound| bound.matches(event)),
            None => false,
        }
    }

    /// The action names in sorted order.
    pub fn get_actions(&self) -> Vec<String> {
        self.actions.keys().cloned().collect()
    }

    pub fn get_action_count(&self) -> usize {
        self.actions.len()
    }

    pub fn clear(&mut self) {
        self.actions.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key_event(keycode: u32) -> InputEventRef {
        let mut key = InputEventKey::new();
        key.set_keycode(keycode);
        InputEventRef::Key(key)
    }

    #[test]
    fn action_lifecycle_and_deadzone() {
        let mut map = InputMapData::new();
        assert!(!map.has_action("jump"));
        assert!(map.add_action("jump", DEFAULT_DEADZONE));
        assert!(map.has_action("jump"));
        assert!(!map.add_action("jump", DEFAULT_DEADZONE), "duplicate");
        assert!(!map.add_action("", DEFAULT_DEADZONE), "empty name");
        assert!(!map.add_action("bad", -1.0), "negative deadzone");
        assert_eq!(map.action_get_deadzone("jump"), 0.2);
        assert!(map.action_set_deadzone("jump", 0.5));
        assert_eq!(map.action_get_deadzone("jump"), 0.5);
        assert!(!map.action_set_deadzone("missing", 0.5));
        assert_eq!(map.action_get_deadzone("missing"), 0.0);
        assert_eq!(map.get_action_count(), 1);
        assert!(map.erase_action("jump"));
        assert!(!map.erase_action("jump"));
    }

    #[test]
    fn events_add_match_and_erase() {
        let mut map = InputMapData::new();
        map.add_action("jump", DEFAULT_DEADZONE);
        assert!(!map.action_add_event("missing", key_event(32)));
        assert!(map.action_add_event("jump", key_event(32)));
        assert_eq!(map.action_get_event_count("jump"), 1);

        // A different keycode does not match.
        assert!(!map.action_has_event("jump", &key_event(33)));
        assert!(map.action_has_event("jump", &key_event(32)));
        assert!(map.event_is_action(&key_event(32), "jump"));
        assert!(!map.event_is_action(&key_event(32), "missing"));

        // Mouse buttons are matched by index.
        let mut button = InputEventMouseButton::new();
        button.set_button_index(crate::input::input_event::MouseButton::Left);
        let mouse_event = InputEventRef::MouseButton(button);
        assert!(map.action_add_event("jump", mouse_event.clone()));
        assert!(map.action_has_event("jump", &mouse_event));
        assert_eq!(map.action_get_event_count("jump"), 2);
        assert!(map.action_erase_event("jump", &key_event(32)));
        assert_eq!(map.action_get_event_count("jump"), 1);
        assert!(!map.action_erase_event("jump", &key_event(32)));
        assert!(map.action_erase_events("jump"));
        assert_eq!(map.action_get_event_count("jump"), 0);

        assert!(map.action_get_events("jump").is_some());
        assert!(map.action_get_events("missing").is_none());
    }

    #[test]
    fn actions_are_sorted() {
        let mut map = InputMapData::new();
        map.add_action("ui_right", DEFAULT_DEADZONE);
        map.add_action("jump", DEFAULT_DEADZONE);
        map.add_action("ui_left", DEFAULT_DEADZONE);
        assert_eq!(map.get_actions(), vec!["jump", "ui_left", "ui_right"]);
        map.clear();
        assert!(map.get_actions().is_empty());
    }

    #[test]
    fn joypad_events_take_part_in_actions() {
        use crate::input::input_event_extra::{InputEventJoypadButton, InputEventJoypadMotion};
        use crate::input::input_event_extra::{JoyAxis, JoyButton};

        let mut map = InputMapData::new();
        map.add_action("jump", DEFAULT_DEADZONE);
        map.add_action("move_right", DEFAULT_DEADZONE);

        let mut button = InputEventJoypadButton::new();
        button.set_button_index(JoyButton::A as i32);
        let bound = InputEventRef::JoypadButton(button.clone());
        assert!(map.action_add_event("jump", bound.clone()));
        assert!(map.event_is_action(&bound, "jump"));

        // The same button on another index does not match.
        let mut other = button.clone();
        other.set_button_index(JoyButton::B as i32);
        assert!(!map.event_is_action(&InputEventRef::JoypadButton(other), "jump"));

        // Axis bindings compare the axis and the direction sign.
        let mut motion = InputEventJoypadMotion::new();
        motion.set_axis(JoyAxis::LeftX as i32);
        motion.set_axis_value(1.0);
        let bound = InputEventRef::JoypadMotion(motion.clone());
        assert!(map.action_add_event("move_right", bound.clone()));
        assert!(map.event_is_action(&bound, "move_right"));

        let mut opposite = motion.clone();
        opposite.set_axis_value(-1.0);
        assert!(!map.event_is_action(&InputEventRef::JoypadMotion(opposite), "move_right"));
        let mut other_axis = motion.clone();
        other_axis.set_axis(JoyAxis::LeftY as i32);
        assert!(!map.event_is_action(&InputEventRef::JoypadMotion(other_axis), "move_right"));
    }
}
