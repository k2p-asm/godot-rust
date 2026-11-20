//! `core/input/input_event.*` (joypad, touch, gesture and MIDI payloads).
//!
//! Ported from `input_event.h`/`input_event.cpp`:
//!
//! - `JoyAxis`/`JoyButton`/`MIDIMessage` with the engine's numeric values
//!   (`INVALID = -1`, `MAX` values included);
//! - `InputEventJoypadMotion` (axis/value, `ALL_DEVICES` default device),
//!   `InputEventJoypadButton` (button index, pressure 0, pressed from the base
//!   event), `InputEventScreenTouch` (index, position, double tap),
//!   `InputEventScreenDrag` (position/relative/screen-relative/velocity/tilt/
//!   pressure/pen inversion), the magnify/pan gestures (`factor = 1.0`, delta),
//!   and `InputEventMIDI` (channel/message/pitch/velocity/instrument/pressure/
//!   controller number+value);
//! - the payload `matches` rules used by the action map: joypad buttons compare
//!   the index (ignoring `INVALID`), joypad axes compare the axis and the sign of
//!   the value.
//!
//! Screen/gesture `xformed_by`, `as_text` and the `Input` singleton are not
//! ported.

use crate::math::vector2::Vector2;

/// `InputMap::ALL_DEVICES`.
pub const ALL_DEVICES: i32 = -1;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(i32)]
pub enum JoyAxis {
    Invalid = -1,
    LeftX = 0,
    LeftY = 1,
    RightX = 2,
    RightY = 3,
    TriggerLeft = 4,
    TriggerRight = 5,
    SdlMax = 6,
    Max = 10,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(i32)]
pub enum JoyButton {
    Invalid = -1,
    A = 0,
    B = 1,
    X = 2,
    Y = 3,
    Back = 4,
    Guide = 5,
    Start = 6,
    LeftStick = 7,
    RightStick = 8,
    LeftShoulder = 9,
    RightShoulder = 10,
    DpadUp = 11,
    DpadDown = 12,
    DpadLeft = 13,
    DpadRight = 14,
    Misc1 = 15,
    Paddle1 = 16,
    Paddle2 = 17,
    Paddle3 = 18,
    Paddle4 = 19,
    Touchpad = 20,
    Misc2 = 21,
    Misc3 = 22,
    Misc4 = 23,
    Misc5 = 24,
    Misc6 = 25,
    SdlMax = 26,
    Max = 128,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum MidiMessage {
    None = 0,
    NoteOff = 0x8,
    NoteOn = 0x9,
    Aftertouch = 0xA,
    ControlChange = 0xB,
    ProgramChange = 0xC,
    ChannelPressure = 0xD,
    PitchBend = 0xE,
    SystemExclusive = 0xF0,
    QuarterFrame = 0xF1,
    SongPositionPointer = 0xF2,
    SongSelect = 0xF3,
    TuneRequest = 0xF6,
    TimingClock = 0xF8,
    Start = 0xFA,
    Continue = 0xFB,
    Stop = 0xFC,
    ActiveSensing = 0xFE,
    SystemReset = 0xFF,
}

fn axis_from_i32(value: i32) -> JoyAxis {
    match value {
        -1 => JoyAxis::Invalid,
        0 => JoyAxis::LeftX,
        1 => JoyAxis::LeftY,
        2 => JoyAxis::RightX,
        3 => JoyAxis::RightY,
        4 => JoyAxis::TriggerLeft,
        5 => JoyAxis::TriggerRight,
        6..=9 => JoyAxis::SdlMax,
        _ => JoyAxis::Max,
    }
}

fn button_from_i32(value: i32) -> JoyButton {
    if value < 0 {
        return JoyButton::Invalid;
    }
    match value {
        0 => JoyButton::A,
        1 => JoyButton::B,
        2 => JoyButton::X,
        3 => JoyButton::Y,
        4 => JoyButton::Back,
        5 => JoyButton::Guide,
        6 => JoyButton::Start,
        7 => JoyButton::LeftStick,
        8 => JoyButton::RightStick,
        9 => JoyButton::LeftShoulder,
        10 => JoyButton::RightShoulder,
        11 => JoyButton::DpadUp,
        12 => JoyButton::DpadDown,
        13 => JoyButton::DpadLeft,
        14 => JoyButton::DpadRight,
        15 => JoyButton::Misc1,
        16 => JoyButton::Paddle1,
        17 => JoyButton::Paddle2,
        18 => JoyButton::Paddle3,
        19 => JoyButton::Paddle4,
        20 => JoyButton::Touchpad,
        21 => JoyButton::Misc2,
        22 => JoyButton::Misc3,
        23 => JoyButton::Misc4,
        24 => JoyButton::Misc5,
        25 => JoyButton::Misc6,
        26 => JoyButton::SdlMax,
        _ => JoyButton::Max,
    }
}

// -- joypad -----------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub struct InputEventJoypadMotion {
    /// `InputEvent::device`; joypad events default to `ALL_DEVICES`.
    pub device: i32,
    pub axis: i32,
    pub axis_value: f32,
}

impl Default for InputEventJoypadMotion {
    fn default() -> Self {
        Self::new()
    }
}

impl InputEventJoypadMotion {
    pub fn new() -> Self {
        Self {
            device: ALL_DEVICES,
            axis: JoyAxis::LeftX as i32,
            axis_value: 0.0,
        }
    }

    pub fn set_axis(&mut self, axis: i32) {
        self.axis = axis;
    }

    pub fn get_axis(&self) -> JoyAxis {
        axis_from_i32(self.axis)
    }

    pub fn set_axis_value(&mut self, value: f32) {
        self.axis_value = value;
    }

    pub fn get_axis_value(&self) -> f32 {
        self.axis_value
    }

    /// `action_match`: same axis and the same direction (zero counts as
    /// "no direction", which matches any stored direction).
    pub fn matches(&self, other: &InputEventJoypadMotion) -> bool {
        self.axis != JoyAxis::Invalid as i32
            && self.axis == other.axis
            && (self.axis_value >= 0.0) == (other.axis_value >= 0.0)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct InputEventJoypadButton {
    pub device: i32,
    pub button_index: i32,
    pub pressure: f32,
    pub pressed: bool,
}

impl Default for InputEventJoypadButton {
    fn default() -> Self {
        Self::new()
    }
}

impl InputEventJoypadButton {
    pub fn new() -> Self {
        Self {
            device: ALL_DEVICES,
            button_index: JoyButton::A as i32,
            pressure: 0.0,
            pressed: false,
        }
    }

    pub fn set_button_index(&mut self, index: i32) {
        self.button_index = index;
    }

    pub fn get_button_index(&self) -> JoyButton {
        button_from_i32(self.button_index)
    }

    pub fn set_pressure(&mut self, pressure: f32) {
        self.pressure = pressure;
    }

    pub fn get_pressure(&self) -> f32 {
        self.pressure
    }

    pub fn set_pressed(&mut self, pressed: bool) {
        self.pressed = pressed;
    }

    pub fn is_pressed(&self) -> bool {
        self.pressed
    }

    pub fn matches(&self, other: &InputEventJoypadButton) -> bool {
        self.button_index >= 0 && self.button_index == other.button_index
    }
}

// -- touch and gestures -----------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub struct InputEventScreenTouch {
    pub window_id: i64,
    pub index: i32,
    pub position: Vector2,
    pub double_tap: bool,
    pub pressed: bool,
    pub canceled: bool,
}

impl Default for InputEventScreenTouch {
    fn default() -> Self {
        Self::new()
    }
}

impl InputEventScreenTouch {
    pub fn new() -> Self {
        Self {
            window_id: 0,
            index: 0,
            position: Vector2::ZERO,
            double_tap: false,
            pressed: false,
            canceled: false,
        }
    }

    pub fn set_index(&mut self, index: i32) {
        self.index = index;
    }

    pub fn get_index(&self) -> i32 {
        self.index
    }

    pub fn set_position(&mut self, position: Vector2) {
        self.position = position;
    }

    pub fn get_position(&self) -> Vector2 {
        self.position
    }

    pub fn set_pressed(&mut self, pressed: bool) {
        self.pressed = pressed;
    }

    pub fn is_pressed(&self) -> bool {
        self.pressed
    }

    pub fn set_canceled(&mut self, canceled: bool) {
        self.canceled = canceled;
    }

    pub fn is_canceled(&self) -> bool {
        self.canceled
    }

    pub fn set_double_tap(&mut self, double_tap: bool) {
        self.double_tap = double_tap;
    }

    pub fn is_double_tap(&self) -> bool {
        self.double_tap
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct InputEventScreenDrag {
    pub window_id: i64,
    pub index: i32,
    pub position: Vector2,
    pub relative: Vector2,
    pub screen_relative: Vector2,
    pub velocity: Vector2,
    pub screen_velocity: Vector2,
    pub tilt: Vector2,
    pub pressure: f32,
    pub pen_inverted: bool,
}

impl Default for InputEventScreenDrag {
    fn default() -> Self {
        Self::new()
    }
}

impl InputEventScreenDrag {
    pub fn new() -> Self {
        Self {
            window_id: 0,
            index: 0,
            position: Vector2::ZERO,
            relative: Vector2::ZERO,
            screen_relative: Vector2::ZERO,
            velocity: Vector2::ZERO,
            screen_velocity: Vector2::ZERO,
            tilt: Vector2::ZERO,
            pressure: 0.0,
            pen_inverted: false,
        }
    }

    pub fn set_index(&mut self, index: i32) {
        self.index = index;
    }

    pub fn get_index(&self) -> i32 {
        self.index
    }

    pub fn set_position(&mut self, position: Vector2) {
        self.position = position;
    }

    pub fn get_position(&self) -> Vector2 {
        self.position
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
}

/// `InputEventGesture`'s position-bearing base.
#[derive(Clone, Debug, PartialEq)]
pub struct InputEventMagnifyGesture {
    pub position: Vector2,
    pub factor: f32,
}

impl Default for InputEventMagnifyGesture {
    fn default() -> Self {
        Self::new()
    }
}

impl InputEventMagnifyGesture {
    pub fn new() -> Self {
        Self {
            position: Vector2::ZERO,
            factor: 1.0,
        }
    }

    pub fn set_position(&mut self, position: Vector2) {
        self.position = position;
    }

    pub fn get_position(&self) -> Vector2 {
        self.position
    }

    /// The engine accepts any positive factor; zero/negative are stored as
    /// given and rejected by the GUI layer, which is not modelled here.
    pub fn set_factor(&mut self, factor: f32) {
        self.factor = factor;
    }

    pub fn get_factor(&self) -> f32 {
        self.factor
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct InputEventPanGesture {
    pub position: Vector2,
    pub delta: Vector2,
}

impl Default for InputEventPanGesture {
    fn default() -> Self {
        Self::new()
    }
}

impl InputEventPanGesture {
    pub fn new() -> Self {
        Self {
            position: Vector2::ZERO,
            delta: Vector2::ZERO,
        }
    }

    pub fn set_position(&mut self, position: Vector2) {
        self.position = position;
    }

    pub fn get_position(&self) -> Vector2 {
        self.position
    }

    pub fn set_delta(&mut self, delta: Vector2) {
        self.delta = delta;
    }

    pub fn get_delta(&self) -> Vector2 {
        self.delta
    }
}

// -- MIDI -------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InputEventMidi {
    pub channel: i32,
    pub message: i32,
    pub pitch: i32,
    pub velocity: i32,
    pub instrument: i32,
    pub pressure: i32,
    pub controller_number: i32,
    pub controller_value: i32,
}

impl Default for InputEventMidi {
    fn default() -> Self {
        Self::new()
    }
}

impl InputEventMidi {
    pub fn new() -> Self {
        Self {
            channel: 0,
            message: MidiMessage::None as i32,
            pitch: 0,
            velocity: 0,
            instrument: 0,
            pressure: 0,
            controller_number: 0,
            controller_value: 0,
        }
    }

    pub fn set_channel(&mut self, channel: i32) {
        self.channel = channel;
    }

    pub fn get_channel(&self) -> i32 {
        self.channel
    }

    pub fn set_message(&mut self, message: i32) {
        self.message = message;
    }

    pub fn get_message(&self) -> MidiMessage {
        match self.message {
            0x8 => MidiMessage::NoteOff,
            0x9 => MidiMessage::NoteOn,
            0xA => MidiMessage::Aftertouch,
            0xB => MidiMessage::ControlChange,
            0xC => MidiMessage::ProgramChange,
            0xD => MidiMessage::ChannelPressure,
            0xE => MidiMessage::PitchBend,
            0xF0 => MidiMessage::SystemExclusive,
            0xF1 => MidiMessage::QuarterFrame,
            0xF2 => MidiMessage::SongPositionPointer,
            0xF3 => MidiMessage::SongSelect,
            0xF6 => MidiMessage::TuneRequest,
            0xF8 => MidiMessage::TimingClock,
            0xFA => MidiMessage::Start,
            0xFB => MidiMessage::Continue,
            0xFC => MidiMessage::Stop,
            0xFE => MidiMessage::ActiveSensing,
            0xFF => MidiMessage::SystemReset,
            _ => MidiMessage::None,
        }
    }

    pub fn set_pitch(&mut self, pitch: i32) {
        self.pitch = pitch;
    }

    pub fn get_pitch(&self) -> i32 {
        self.pitch
    }

    pub fn set_velocity(&mut self, velocity: i32) {
        self.velocity = velocity;
    }

    pub fn get_velocity(&self) -> i32 {
        self.velocity
    }

    pub fn set_instrument(&mut self, instrument: i32) {
        self.instrument = instrument;
    }

    pub fn get_instrument(&self) -> i32 {
        self.instrument
    }

    pub fn set_pressure(&mut self, pressure: i32) {
        self.pressure = pressure;
    }

    pub fn get_pressure(&self) -> i32 {
        self.pressure
    }

    pub fn set_controller_number(&mut self, number: i32) {
        self.controller_number = number;
    }

    pub fn get_controller_number(&self) -> i32 {
        self.controller_number
    }

    pub fn set_controller_value(&mut self, value: i32) {
        self.controller_value = value;
    }

    pub fn get_controller_value(&self) -> i32 {
        self.controller_value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enum_values_match_the_engine() {
        assert_eq!(JoyAxis::Invalid as i32, -1);
        assert_eq!(JoyAxis::LeftX as i32, 0);
        assert_eq!(JoyAxis::TriggerRight as i32, 5);
        assert_eq!(JoyAxis::SdlMax as i32, 6);
        assert_eq!(JoyAxis::Max as i32, 10);
        assert_eq!(JoyButton::Invalid as i32, -1);
        assert_eq!(JoyButton::A as i32, 0);
        assert_eq!(JoyButton::DpadRight as i32, 14);
        assert_eq!(JoyButton::Misc1 as i32, 15);
        assert_eq!(JoyButton::Touchpad as i32, 20);
        assert_eq!(JoyButton::SdlMax as i32, 26);
        assert_eq!(JoyButton::Max as i32, 128);
        assert_eq!(MidiMessage::None as i32, 0);
        assert_eq!(MidiMessage::NoteOn as i32, 0x9);
        assert_eq!(MidiMessage::PitchBend as i32, 0xE);
        assert_eq!(MidiMessage::SystemReset as i32, 0xFF);
    }

    #[test]
    fn joypad_defaults_and_matching() {
        let mut motion = InputEventJoypadMotion::new();
        assert_eq!(motion.device, ALL_DEVICES);
        assert_eq!(motion.get_axis(), JoyAxis::LeftX);
        assert_eq!(motion.get_axis_value(), 0.0);
        motion.set_axis(JoyAxis::TriggerRight as i32);
        motion.set_axis_value(-0.5);
        assert_eq!(motion.get_axis(), JoyAxis::TriggerRight);
        assert_eq!(motion.get_axis_value(), -0.5);

        let mut other = InputEventJoypadMotion::new();
        other.set_axis(JoyAxis::TriggerRight as i32);
        other.set_axis_value(-0.25);
        assert!(motion.matches(&other), "same axis and direction");
        other.set_axis_value(0.25);
        assert!(!motion.matches(&other), "direction differs");
        other.set_axis(JoyAxis::LeftX as i32);
        assert!(!motion.matches(&other), "axis differs");

        let mut button = InputEventJoypadButton::new();
        assert_eq!(button.device, ALL_DEVICES);
        assert_eq!(button.get_button_index(), JoyButton::A);
        assert_eq!(button.get_pressure(), 0.0);
        assert!(!button.is_pressed());
        button.set_button_index(JoyButton::Start as i32);
        button.set_pressure(0.75);
        button.set_pressed(true);
        assert_eq!(button.get_button_index(), JoyButton::Start);
        assert_eq!(button.get_pressure(), 0.75);
        assert!(button.is_pressed());

        let mut same = InputEventJoypadButton::new();
        same.button_index = JoyButton::Start as i32;
        assert!(button.matches(&same));
        same.button_index = JoyButton::Invalid as i32;
        assert!(!button.matches(&same));
    }

    #[test]
    fn screen_events() {
        let mut touch = InputEventScreenTouch::new();
        assert_eq!(touch.get_index(), 0);
        assert_eq!(touch.get_position(), Vector2::ZERO);
        assert!(!touch.is_pressed());
        assert!(!touch.is_canceled());
        assert!(!touch.is_double_tap());
        touch.set_index(2);
        touch.set_position(Vector2::new(3.0, 4.0));
        touch.set_pressed(true);
        touch.set_canceled(true);
        touch.set_double_tap(true);
        assert_eq!(touch.get_index(), 2);
        assert_eq!(touch.get_position(), Vector2::new(3.0, 4.0));
        assert!(touch.is_pressed() && touch.is_canceled() && touch.is_double_tap());

        let mut drag = InputEventScreenDrag::new();
        assert_eq!(drag.get_index(), 0);
        assert_eq!(drag.get_tilt(), Vector2::ZERO);
        assert_eq!(drag.get_pressure(), 0.0);
        assert!(!drag.get_pen_inverted());
        drag.set_index(1);
        drag.set_position(Vector2::new(1.0, 2.0));
        drag.set_relative(Vector2::new(3.0, 4.0));
        drag.set_relative_screen_position(Vector2::new(5.0, 6.0));
        drag.set_velocity(Vector2::new(7.0, 8.0));
        drag.set_screen_velocity(Vector2::new(9.0, 10.0));
        drag.set_tilt(Vector2::new(0.1, 0.2));
        drag.set_pressure(0.5);
        drag.set_pen_inverted(true);
        assert_eq!(drag.get_position(), Vector2::new(1.0, 2.0));
        assert_eq!(drag.get_relative(), Vector2::new(3.0, 4.0));
        assert_eq!(drag.get_relative_screen_position(), Vector2::new(5.0, 6.0));
        assert_eq!(drag.get_velocity(), Vector2::new(7.0, 8.0));
        assert_eq!(drag.get_screen_velocity(), Vector2::new(9.0, 10.0));
        assert_eq!(drag.get_tilt(), Vector2::new(0.1, 0.2));
        assert_eq!(drag.get_pressure(), 0.5);
        assert!(drag.get_pen_inverted());
    }

    #[test]
    fn gestures_and_midi() {
        let mut magnify = InputEventMagnifyGesture::new();
        assert_eq!(magnify.get_factor(), 1.0);
        assert_eq!(magnify.get_position(), Vector2::ZERO);
        magnify.set_factor(2.0);
        magnify.set_position(Vector2::new(10.0, 20.0));
        assert_eq!(magnify.get_factor(), 2.0);
        assert_eq!(magnify.get_position(), Vector2::new(10.0, 20.0));

        let mut pan = InputEventPanGesture::new();
        assert_eq!(pan.get_delta(), Vector2::ZERO);
        pan.set_delta(Vector2::new(-1.0, 2.0));
        pan.set_position(Vector2::new(4.0, 5.0));
        assert_eq!(pan.get_delta(), Vector2::new(-1.0, 2.0));
        assert_eq!(pan.get_position(), Vector2::new(4.0, 5.0));

        let mut midi = InputEventMidi::new();
        assert_eq!(midi.get_channel(), 0);
        assert_eq!(midi.get_message(), MidiMessage::None);
        assert_eq!(midi.get_pitch(), 0);
        assert_eq!(midi.get_velocity(), 0);
        assert_eq!(midi.get_instrument(), 0);
        assert_eq!(midi.get_pressure(), 0);
        assert_eq!(midi.get_controller_number(), 0);
        assert_eq!(midi.get_controller_value(), 0);
        midi.set_channel(3);
        midi.set_message(0x9);
        midi.set_pitch(64);
        midi.set_velocity(100);
        midi.set_instrument(1);
        midi.set_pressure(2);
        midi.set_controller_number(7);
        midi.set_controller_value(127);
        assert_eq!(midi.get_channel(), 3);
        assert_eq!(midi.get_message(), MidiMessage::NoteOn);
        assert_eq!(midi.get_pitch(), 64);
        assert_eq!(midi.get_velocity(), 100);
        assert_eq!(midi.get_instrument(), 1);
        assert_eq!(midi.get_pressure(), 2);
        assert_eq!(midi.get_controller_number(), 7);
        assert_eq!(midi.get_controller_value(), 127);
        midi.set_message(0x42);
        assert_eq!(midi.get_message(), MidiMessage::None);
    }
}
