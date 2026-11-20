//! `scene/gui/base_button.*` and `button.*` (state half).
//!
//! `BaseButton` keeps the engine's interaction status (`pressed`, `hovering`,
//! `press_attempt`, `pressing_inside`, `pressed_down_with_focus`, `disabled`,
//! `touch_index`) plus the configuration (`button_mask` defaulting to left,
//! toggle mode off, action mode release, shortcut flags on, keep-pressed-inside
//! off) and reproduces the derived rules:
//!
//! - `is_pressed()` is the toggle state in toggle mode and the press attempt
//!   otherwise;
//! - `set_pressed` only works in toggle mode and reports whether the toggled
//!   state actually changed;
//! - enabling `disabled` clears the attempt/inside flags and drops a non-toggle
//!   `pressed`, reporting `button_up` when the button was held by focus;
//! - `set_toggle_mode(false)` first clears `pressed`;
//! - `get_draw_mode()` follows the engine's branch order.
//!
//! `ButtonGroup` (shared unpress) is **not** ported: it needs a shared object
//! graph. `Button`'s text/icon/alignment state is ported.

use crate::servers::rid::Rid;

/// `MouseButtonMask` values used by `BaseButton::button_mask`.
pub mod mouse_mask {
    pub const NONE: u32 = 0;
    pub const LEFT: u32 = 1;
    pub const RIGHT: u32 = 1 << 1;
    pub const MIDDLE: u32 = 1 << 2;
    pub const MB_XBUTTON1: u32 = 1 << 3;
    pub const MB_XBUTTON2: u32 = 1 << 4;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum ActionMode {
    Press,
    Release,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum DrawMode {
    Normal,
    Pressed,
    Hover,
    Disabled,
    HoverPressed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum HorizontalAlignment {
    Left,
    Center,
    Right,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum VerticalAlignment {
    Top,
    Center,
    Bottom,
}

#[derive(Clone, Copy, Debug)]
pub struct ButtonStatus {
    pub pressed: bool,
    pub hovering: bool,
    pub press_attempt: bool,
    pub pressing_inside: bool,
    pub pressed_down_with_focus: bool,
    pub disabled: bool,
    pub touch_index: i32,
}

impl Default for ButtonStatus {
    fn default() -> Self {
        Self {
            pressed: false,
            hovering: false,
            press_attempt: false,
            pressing_inside: false,
            pressed_down_with_focus: false,
            disabled: false,
            touch_index: -1,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct BaseButtonData {
    button_mask: u32,
    toggle_mode: bool,
    shortcut_in_tooltip: bool,
    keep_pressed_outside: bool,
    shortcut_feedback: bool,
    action_mode: ActionMode,
    in_shortcut_feedback: bool,
    pub status: ButtonStatus,
}

impl Default for BaseButtonData {
    fn default() -> Self {
        Self::new()
    }
}

impl BaseButtonData {
    pub fn new() -> Self {
        Self {
            button_mask: mouse_mask::LEFT,
            toggle_mode: false,
            shortcut_in_tooltip: true,
            keep_pressed_outside: false,
            shortcut_feedback: true,
            action_mode: ActionMode::Release,
            in_shortcut_feedback: false,
            status: ButtonStatus::default(),
        }
    }

    pub fn set_button_mask(&mut self, mask: u32) {
        self.button_mask = mask;
    }

    pub fn get_button_mask(&self) -> u32 {
        self.button_mask
    }

    pub fn set_toggle_mode(&mut self, on: bool) {
        // The engine clears `pressed` before switching the mode off.
        if !on {
            self.set_pressed(false);
        }
        self.toggle_mode = on;
    }

    pub fn is_toggle_mode(&self) -> bool {
        self.toggle_mode
    }

    pub fn set_shortcut_in_tooltip(&mut self, on: bool) {
        self.shortcut_in_tooltip = on;
    }

    pub fn is_shortcut_in_tooltip_enabled(&self) -> bool {
        self.shortcut_in_tooltip
    }

    pub fn set_keep_pressed_outside(&mut self, on: bool) {
        self.keep_pressed_outside = on;
    }

    pub fn is_keep_pressed_outside(&self) -> bool {
        self.keep_pressed_outside
    }

    pub fn set_shortcut_feedback(&mut self, on: bool) {
        self.shortcut_feedback = on;
    }

    pub fn is_shortcut_feedback(&self) -> bool {
        self.shortcut_feedback
    }

    pub fn set_action_mode(&mut self, mode: ActionMode) {
        self.action_mode = mode;
    }

    pub fn get_action_mode(&self) -> ActionMode {
        self.action_mode
    }

    pub fn set_in_shortcut_feedback(&mut self, feedback: bool) {
        self.in_shortcut_feedback = feedback;
    }

    /// `set_pressed_no_signal`: toggle mode only, no change report.
    pub fn set_pressed_no_signal(&mut self, pressed: bool) {
        if !self.toggle_mode || self.status.pressed == pressed {
            return;
        }
        self.status.pressed = pressed;
    }

    /// `set_pressed`: applies only in toggle mode and returns whether the
    /// toggled state changed (the engine then emits `toggled`).
    pub fn set_pressed(&mut self, pressed: bool) -> bool {
        let previous = self.status.pressed;
        self.set_pressed_no_signal(pressed);
        self.status.pressed != previous
    }

    /// `is_pressed`: toggle state, or the press attempt outside toggle mode.
    pub fn is_pressed(&self) -> bool {
        if self.toggle_mode {
            self.status.pressed
        } else {
            self.status.press_attempt
        }
    }

    pub fn is_pressing(&self) -> bool {
        self.status.press_attempt
    }

    pub fn is_hovered(&self) -> bool {
        self.status.hovering
    }

    pub fn set_hovering(&mut self, hovering: bool) {
        self.status.hovering = hovering;
    }

    pub fn set_press_attempt(&mut self, attempt: bool) {
        self.status.press_attempt = attempt;
    }

    pub fn set_pressing_inside(&mut self, inside: bool) {
        self.status.pressing_inside = inside;
    }

    /// Setting `disabled` clears the interaction flags; returns whether a
    /// `button_up` would be emitted.
    pub fn set_disabled(&mut self, disabled: bool) -> bool {
        if self.status.disabled == disabled {
            return false;
        }
        self.status.disabled = disabled;
        let mut released = false;
        if disabled {
            if !self.toggle_mode {
                self.status.pressed = false;
            }
            self.status.press_attempt = false;
            self.status.pressing_inside = false;
            if self.status.pressed_down_with_focus {
                self.status.pressed_down_with_focus = false;
                released = true;
            }
        }
        released
    }

    pub fn is_disabled(&self) -> bool {
        self.status.disabled
    }

    pub fn set_pressed_down_with_focus(&mut self, pressed: bool) {
        self.status.pressed_down_with_focus = pressed;
    }

    pub fn is_pressed_down_with_focus(&self) -> bool {
        self.status.pressed_down_with_focus
    }

    pub fn set_touch_index(&mut self, index: i32) {
        self.status.touch_index = index;
    }

    pub fn get_touch_index(&self) -> i32 {
        self.status.touch_index
    }

    /// `get_draw_mode`, branch for branch.
    pub fn get_draw_mode(&self) -> DrawMode {
        if self.status.disabled {
            return DrawMode::Disabled;
        }
        if self.in_shortcut_feedback {
            return DrawMode::HoverPressed;
        }
        if !self.status.press_attempt && self.status.hovering {
            return if self.status.pressed {
                DrawMode::HoverPressed
            } else {
                DrawMode::Hover
            };
        }

        let pressing = if self.status.press_attempt {
            let inside = self.status.pressing_inside || self.keep_pressed_outside;
            if self.status.pressed {
                !inside
            } else {
                inside
            }
        } else {
            self.status.pressed
        };

        if pressing {
            DrawMode::Pressed
        } else {
            DrawMode::Normal
        }
    }
}

/// `Button`'s own state on top of `BaseButton`.
#[derive(Clone, Debug)]
pub struct ButtonData {
    pub base: BaseButtonData,
    flat: bool,
    text: String,
    xl_text: String,
    language: String,
    text_direction: i32,
    autowrap_mode: i32,
    autowrap_flags_trim: i32,
    overrun_behavior: i32,
    icon: Option<Rid>,
    expand_icon: bool,
    clip_text: bool,
    alignment: HorizontalAlignment,
    horizontal_icon_alignment: HorizontalAlignment,
    vertical_icon_alignment: VerticalAlignment,
}

impl Default for ButtonData {
    fn default() -> Self {
        Self::new()
    }
}

impl ButtonData {
    pub fn new() -> Self {
        Self {
            base: BaseButtonData::new(),
            flat: false,
            text: String::new(),
            xl_text: String::new(),
            language: String::new(),
            text_direction: 0,
            autowrap_mode: 0,
            autowrap_flags_trim: 0,
            overrun_behavior: 0,
            icon: None,
            expand_icon: false,
            clip_text: false,
            alignment: HorizontalAlignment::Center,
            horizontal_icon_alignment: HorizontalAlignment::Left,
            vertical_icon_alignment: VerticalAlignment::Center,
        }
    }

    pub fn set_flat(&mut self, flat: bool) {
        self.flat = flat;
    }

    pub fn is_flat(&self) -> bool {
        self.flat
    }

    pub fn set_text(&mut self, text: &str) {
        self.text = text.to_string();
    }

    pub fn get_text(&self) -> &str {
        &self.text
    }

    /// The translation key; empty falls back to `text` in the engine.
    pub fn set_xl_text(&mut self, text: &str) {
        self.xl_text = text.to_string();
    }

    pub fn get_xl_text(&self) -> &str {
        &self.xl_text
    }

    pub fn set_language(&mut self, language: &str) {
        self.language = language.to_string();
    }

    pub fn get_language(&self) -> &str {
        &self.language
    }

    pub fn set_text_direction(&mut self, direction: i32) {
        self.text_direction = direction;
    }

    pub fn get_text_direction(&self) -> i32 {
        self.text_direction
    }

    pub fn set_autowrap_mode(&mut self, mode: i32) {
        self.autowrap_mode = mode;
    }

    pub fn get_autowrap_mode(&self) -> i32 {
        self.autowrap_mode
    }

    pub fn set_autowrap_trim_flags(&mut self, flags: i32) {
        self.autowrap_flags_trim = flags;
    }

    pub fn get_autowrap_trim_flags(&self) -> i32 {
        self.autowrap_flags_trim
    }

    pub fn set_text_overrun_behavior(&mut self, behavior: i32) {
        self.overrun_behavior = behavior;
    }

    pub fn get_text_overrun_behavior(&self) -> i32 {
        self.overrun_behavior
    }

    pub fn set_icon(&mut self, icon: Option<Rid>) {
        self.icon = icon;
    }

    pub fn get_icon(&self) -> Option<Rid> {
        self.icon
    }

    pub fn set_expand_icon(&mut self, expand: bool) {
        self.expand_icon = expand;
    }

    pub fn is_expand_icon(&self) -> bool {
        self.expand_icon
    }

    pub fn set_clip_text(&mut self, clip: bool) {
        self.clip_text = clip;
    }

    pub fn is_clipping_text(&self) -> bool {
        self.clip_text
    }

    pub fn set_alignment(&mut self, alignment: HorizontalAlignment) {
        self.alignment = alignment;
    }

    pub fn get_alignment(&self) -> HorizontalAlignment {
        self.alignment
    }

    pub fn set_icon_alignment(&mut self, alignment: HorizontalAlignment) {
        self.horizontal_icon_alignment = alignment;
    }

    pub fn get_icon_alignment(&self) -> HorizontalAlignment {
        self.horizontal_icon_alignment
    }

    pub fn set_vertical_icon_alignment(&mut self, alignment: VerticalAlignment) {
        self.vertical_icon_alignment = alignment;
    }

    pub fn get_vertical_icon_alignment(&self) -> VerticalAlignment {
        self.vertical_icon_alignment
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults() {
        let b = BaseButtonData::new();
        assert_eq!(b.get_button_mask(), mouse_mask::LEFT);
        assert!(!b.is_toggle_mode());
        assert!(b.is_shortcut_in_tooltip_enabled());
        assert!(!b.is_keep_pressed_outside());
        assert!(b.is_shortcut_feedback());
        assert_eq!(b.get_action_mode(), ActionMode::Release);
        assert!(!b.is_pressed());
        assert!(!b.is_pressing());
        assert!(!b.is_hovered());
        assert!(!b.is_disabled());
        assert_eq!(b.get_touch_index(), -1);
        assert_eq!(b.get_draw_mode(), DrawMode::Normal);
    }

    #[test]
    fn pressed_only_works_in_toggle_mode() {
        let mut b = BaseButtonData::new();
        assert!(!b.set_pressed(true), "no toggle mode, no change");
        assert!(!b.is_pressed());

        b.set_toggle_mode(true);
        assert!(b.set_pressed(true));
        assert!(b.is_pressed());
        // Setting the same value reports no change.
        assert!(!b.set_pressed(true));
        assert!(b.set_pressed(false));
        assert!(!b.is_pressed());

        // set_pressed_no_signal applies silently.
        b.set_pressed_no_signal(true);
        assert!(b.is_pressed());

        // Turning toggle mode off clears `pressed`.
        b.set_toggle_mode(false);
        assert!(!b.is_pressed());
        assert!(!b.status.pressed);
    }

    #[test]
    fn press_attempt_is_reported_outside_toggle_mode() {
        let mut b = BaseButtonData::new();
        b.set_press_attempt(true);
        assert!(b.is_pressed());
        assert!(b.is_pressing());
        b.set_press_attempt(false);
        assert!(!b.is_pressed());
    }

    #[test]
    fn disabling_clears_interaction_state() {
        let mut b = BaseButtonData::new();
        b.set_press_attempt(true);
        b.set_pressing_inside(true);
        b.set_hovering(true);
        b.set_pressed_down_with_focus(true);
        assert!(b.set_disabled(true), "held by focus emits button_up");
        assert!(b.is_disabled());
        assert!(!b.is_pressing());
        assert!(!b.status.pressing_inside);
        assert!(!b.status.pressed_down_with_focus);
        // Hovering is untouched by the engine.
        assert!(b.is_hovered());
        assert!(!b.set_disabled(true), "already disabled");

        // In toggle mode `pressed` survives disabling.
        let mut t = BaseButtonData::new();
        t.set_toggle_mode(true);
        t.set_pressed(true);
        assert!(!t.set_disabled(true));
        assert!(t.is_pressed());
        // Re-enabling returns false: the flag means "emit button_up", not
        // "state changed".
        assert!(!t.set_disabled(false));
        assert!(!t.is_disabled());
    }

    #[test]
    fn draw_mode_branches() {
        let mut b = BaseButtonData::new();
        assert_eq!(b.get_draw_mode(), DrawMode::Normal);

        b.set_hovering(true);
        assert_eq!(b.get_draw_mode(), DrawMode::Hover);

        // Hovering while toggled shows the hover-pressed style.
        b.set_toggle_mode(true);
        b.set_pressed(true);
        assert_eq!(b.get_draw_mode(), DrawMode::HoverPressed);

        // With `pressed` set the pressing-inside test is inverted: an attempt
        // inside the button reads as unpressed, and vice versa.
        b.set_press_attempt(true);
        assert_eq!(b.get_draw_mode(), DrawMode::Pressed);
        b.set_pressing_inside(true);
        assert_eq!(b.get_draw_mode(), DrawMode::Normal);

        // Shortcut feedback wins over everything but disabled.
        b.set_in_shortcut_feedback(true);
        assert_eq!(b.get_draw_mode(), DrawMode::HoverPressed);
        b.set_disabled(true);
        assert_eq!(b.get_draw_mode(), DrawMode::Disabled);
    }

    #[test]
    fn pressed_and_keep_outside_rules() {
        let mut b = BaseButtonData::new();
        b.set_press_attempt(true);
        assert_eq!(b.get_draw_mode(), DrawMode::Normal, "outside the button");
        b.set_pressing_inside(true);
        assert_eq!(b.get_draw_mode(), DrawMode::Pressed);

        // keep_pressed_outside makes an outside attempt count as pressed.
        let mut k = BaseButtonData::new();
        k.set_keep_pressed_outside(true);
        k.set_press_attempt(true);
        assert_eq!(k.get_draw_mode(), DrawMode::Pressed);

        // With `pressed` set, a pressing-inside attempt is inverted.
        let mut t = BaseButtonData::new();
        t.set_toggle_mode(true);
        t.set_pressed(true);
        t.set_press_attempt(true);
        assert_eq!(t.get_draw_mode(), DrawMode::Pressed);
        t.set_pressing_inside(true);
        assert_eq!(t.get_draw_mode(), DrawMode::Normal);
    }

    #[test]
    fn button_text_and_icon_state() {
        let mut b = ButtonData::new();
        assert!(!b.is_flat());
        assert_eq!(b.get_text(), "");
        assert_eq!(b.get_alignment(), HorizontalAlignment::Center);
        assert_eq!(b.get_icon_alignment(), HorizontalAlignment::Left);
        assert_eq!(b.get_vertical_icon_alignment(), VerticalAlignment::Center);
        assert!(b.get_icon().is_none());

        b.set_flat(true);
        b.set_text("Play");
        b.set_xl_text("PLAY");
        b.set_language("en");
        b.set_text_direction(2);
        b.set_autowrap_mode(3);
        b.set_autowrap_trim_flags(5);
        b.set_text_overrun_behavior(1);
        b.set_icon(Some(Rid::from_u64(4)));
        b.set_expand_icon(true);
        b.set_clip_text(true);
        b.set_alignment(HorizontalAlignment::Right);
        b.set_icon_alignment(HorizontalAlignment::Center);
        b.set_vertical_icon_alignment(VerticalAlignment::Top);

        assert!(b.is_flat());
        assert_eq!(b.get_text(), "Play");
        assert_eq!(b.get_xl_text(), "PLAY");
        assert_eq!(b.get_language(), "en");
        assert_eq!(b.get_text_direction(), 2);
        assert_eq!(b.get_autowrap_mode(), 3);
        assert_eq!(b.get_autowrap_trim_flags(), 5);
        assert_eq!(b.get_text_overrun_behavior(), 1);
        assert_eq!(b.get_icon(), Some(Rid::from_u64(4)));
        assert!(b.is_expand_icon());
        assert!(b.is_clipping_text());
        assert_eq!(b.get_alignment(), HorizontalAlignment::Right);
        assert_eq!(b.get_icon_alignment(), HorizontalAlignment::Center);
        assert_eq!(b.get_vertical_icon_alignment(), VerticalAlignment::Top);

        assert_eq!(HorizontalAlignment::Left as i32, 0);
        assert_eq!(HorizontalAlignment::Center as i32, 1);
        assert_eq!(HorizontalAlignment::Right as i32, 2);
        assert_eq!(VerticalAlignment::Top as i32, 0);
        assert_eq!(VerticalAlignment::Center as i32, 1);
        assert_eq!(VerticalAlignment::Bottom as i32, 2);
        assert_eq!(ActionMode::Press as i32, 0);
        assert_eq!(ActionMode::Release as i32, 1);
        assert_eq!(DrawMode::Normal as i32, 0);
        assert_eq!(DrawMode::Pressed as i32, 1);
        assert_eq!(DrawMode::Hover as i32, 2);
        assert_eq!(DrawMode::Disabled as i32, 3);
        assert_eq!(DrawMode::HoverPressed as i32, 4);
    }
}
