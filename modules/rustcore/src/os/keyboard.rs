//! `core/os/keyboard.h`: `Key`, `KeyModifierMask` and `KeyLocation` constants.
//!
//! Generated from the engine header by the porting script; values keep the
//! engine's `SPECIAL` (`1 << 22`) and modifier (`1 << 24..30`) encoding.
//! The macOS-only `CMD_OR_CTRL` branch was resolved to the non-macOS one.

use crate::input::input_event::KeyLocation;

pub const KEY_NONE: u32 = 0;
pub const KEY_SPECIAL: u32 = 4194304;
pub const KEY_ESCAPE: u32 = 4194305;
pub const KEY_TAB: u32 = 4194306;
pub const KEY_BACKTAB: u32 = 4194307;
pub const KEY_BACKSPACE: u32 = 4194308;
pub const KEY_ENTER: u32 = 4194309;
pub const KEY_KP_ENTER: u32 = 4194310;
pub const KEY_INSERT: u32 = 4194311;
pub const KEY_KEY_DELETE: u32 = 4194312;
pub const KEY_PAUSE: u32 = 4194313;
pub const KEY_PRINT: u32 = 4194314;
pub const KEY_SYSREQ: u32 = 4194315;
pub const KEY_CLEAR: u32 = 4194316;
pub const KEY_HOME: u32 = 4194317;
pub const KEY_END: u32 = 4194318;
pub const KEY_LEFT: u32 = 4194319;
pub const KEY_UP: u32 = 4194320;
pub const KEY_RIGHT: u32 = 4194321;
pub const KEY_DOWN: u32 = 4194322;
pub const KEY_PAGEUP: u32 = 4194323;
pub const KEY_PAGEDOWN: u32 = 4194324;
pub const KEY_SHIFT: u32 = 4194325;
pub const KEY_CTRL: u32 = 4194326;
pub const KEY_META: u32 = 4194327;
pub const KEY_CMD_OR_CTRL: u32 = 4194326;
pub const KEY_ALT: u32 = 4194328;
pub const KEY_CAPSLOCK: u32 = 4194329;
pub const KEY_NUMLOCK: u32 = 4194330;
pub const KEY_SCROLLLOCK: u32 = 4194331;
pub const KEY_F1: u32 = 4194332;
pub const KEY_F2: u32 = 4194333;
pub const KEY_F3: u32 = 4194334;
pub const KEY_F4: u32 = 4194335;
pub const KEY_F5: u32 = 4194336;
pub const KEY_F6: u32 = 4194337;
pub const KEY_F7: u32 = 4194338;
pub const KEY_F8: u32 = 4194339;
pub const KEY_F9: u32 = 4194340;
pub const KEY_F10: u32 = 4194341;
pub const KEY_F11: u32 = 4194342;
pub const KEY_F12: u32 = 4194343;
pub const KEY_F13: u32 = 4194344;
pub const KEY_F14: u32 = 4194345;
pub const KEY_F15: u32 = 4194346;
pub const KEY_F16: u32 = 4194347;
pub const KEY_F17: u32 = 4194348;
pub const KEY_F18: u32 = 4194349;
pub const KEY_F19: u32 = 4194350;
pub const KEY_F20: u32 = 4194351;
pub const KEY_F21: u32 = 4194352;
pub const KEY_F22: u32 = 4194353;
pub const KEY_F23: u32 = 4194354;
pub const KEY_F24: u32 = 4194355;
pub const KEY_F25: u32 = 4194356;
pub const KEY_F26: u32 = 4194357;
pub const KEY_F27: u32 = 4194358;
pub const KEY_F28: u32 = 4194359;
pub const KEY_F29: u32 = 4194360;
pub const KEY_F30: u32 = 4194361;
pub const KEY_F31: u32 = 4194362;
pub const KEY_F32: u32 = 4194363;
pub const KEY_F33: u32 = 4194364;
pub const KEY_F34: u32 = 4194365;
pub const KEY_F35: u32 = 4194366;
pub const KEY_KP_MULTIPLY: u32 = 4194433;
pub const KEY_KP_DIVIDE: u32 = 4194434;
pub const KEY_KP_SUBTRACT: u32 = 4194435;
pub const KEY_KP_PERIOD: u32 = 4194436;
pub const KEY_KP_ADD: u32 = 4194437;
pub const KEY_KP_0: u32 = 4194438;
pub const KEY_KP_1: u32 = 4194439;
pub const KEY_KP_2: u32 = 4194440;
pub const KEY_KP_3: u32 = 4194441;
pub const KEY_KP_4: u32 = 4194442;
pub const KEY_KP_5: u32 = 4194443;
pub const KEY_KP_6: u32 = 4194444;
pub const KEY_KP_7: u32 = 4194445;
pub const KEY_KP_8: u32 = 4194446;
pub const KEY_KP_9: u32 = 4194447;
pub const KEY_MENU: u32 = 4194370;
pub const KEY_HYPER: u32 = 4194371;
pub const KEY_HELP: u32 = 4194373;
pub const KEY_BACK: u32 = 4194376;
pub const KEY_FORWARD: u32 = 4194377;
pub const KEY_STOP: u32 = 4194378;
pub const KEY_REFRESH: u32 = 4194379;
pub const KEY_VOLUMEDOWN: u32 = 4194380;
pub const KEY_VOLUMEMUTE: u32 = 4194381;
pub const KEY_VOLUMEUP: u32 = 4194382;
pub const KEY_MEDIAPLAY: u32 = 4194388;
pub const KEY_MEDIASTOP: u32 = 4194389;
pub const KEY_MEDIAPREVIOUS: u32 = 4194390;
pub const KEY_MEDIANEXT: u32 = 4194391;
pub const KEY_MEDIARECORD: u32 = 4194392;
pub const KEY_HOMEPAGE: u32 = 4194393;
pub const KEY_FAVORITES: u32 = 4194394;
pub const KEY_SEARCH: u32 = 4194395;
pub const KEY_STANDBY: u32 = 4194396;
pub const KEY_OPENURL: u32 = 4194397;
pub const KEY_LAUNCHMAIL: u32 = 4194398;
pub const KEY_LAUNCHMEDIA: u32 = 4194399;
pub const KEY_LAUNCH0: u32 = 4194400;
pub const KEY_LAUNCH1: u32 = 4194401;
pub const KEY_LAUNCH2: u32 = 4194402;
pub const KEY_LAUNCH3: u32 = 4194403;
pub const KEY_LAUNCH4: u32 = 4194404;
pub const KEY_LAUNCH5: u32 = 4194405;
pub const KEY_LAUNCH6: u32 = 4194406;
pub const KEY_LAUNCH7: u32 = 4194407;
pub const KEY_LAUNCH8: u32 = 4194408;
pub const KEY_LAUNCH9: u32 = 4194409;
pub const KEY_LAUNCHA: u32 = 4194410;
pub const KEY_LAUNCHB: u32 = 4194411;
pub const KEY_LAUNCHC: u32 = 4194412;
pub const KEY_LAUNCHD: u32 = 4194413;
pub const KEY_LAUNCHE: u32 = 4194414;
pub const KEY_LAUNCHF: u32 = 4194415;
pub const KEY_GLOBE: u32 = 4194416;
pub const KEY_KEYBOARD: u32 = 4194417;
pub const KEY_JIS_EISU: u32 = 4194418;
pub const KEY_JIS_KANA: u32 = 4194419;
pub const KEY_UNKNOWN: u32 = 8388607;
pub const KEY_SPACE: u32 = 32;
pub const KEY_EXCLAM: u32 = 33;
pub const KEY_QUOTEDBL: u32 = 34;
pub const KEY_NUMBERSIGN: u32 = 35;
pub const KEY_DOLLAR: u32 = 36;
pub const KEY_PERCENT: u32 = 37;
pub const KEY_AMPERSAND: u32 = 38;
pub const KEY_APOSTROPHE: u32 = 39;
pub const KEY_PARENLEFT: u32 = 40;
pub const KEY_PARENRIGHT: u32 = 41;
pub const KEY_ASTERISK: u32 = 42;
pub const KEY_PLUS: u32 = 43;
pub const KEY_COMMA: u32 = 44;
pub const KEY_MINUS: u32 = 45;
pub const KEY_PERIOD: u32 = 46;
pub const KEY_SLASH: u32 = 47;
pub const KEY_KEY_0: u32 = 48;
pub const KEY_KEY_1: u32 = 49;
pub const KEY_KEY_2: u32 = 50;
pub const KEY_KEY_3: u32 = 51;
pub const KEY_KEY_4: u32 = 52;
pub const KEY_KEY_5: u32 = 53;
pub const KEY_KEY_6: u32 = 54;
pub const KEY_KEY_7: u32 = 55;
pub const KEY_KEY_8: u32 = 56;
pub const KEY_KEY_9: u32 = 57;
pub const KEY_COLON: u32 = 58;
pub const KEY_SEMICOLON: u32 = 59;
pub const KEY_LESS: u32 = 60;
pub const KEY_EQUAL: u32 = 61;
pub const KEY_GREATER: u32 = 62;
pub const KEY_QUESTION: u32 = 63;
pub const KEY_AT: u32 = 64;
pub const KEY_A: u32 = 65;
pub const KEY_B: u32 = 66;
pub const KEY_C: u32 = 67;
pub const KEY_D: u32 = 68;
pub const KEY_E: u32 = 69;
pub const KEY_F: u32 = 70;
pub const KEY_G: u32 = 71;
pub const KEY_H: u32 = 72;
pub const KEY_I: u32 = 73;
pub const KEY_J: u32 = 74;
pub const KEY_K: u32 = 75;
pub const KEY_L: u32 = 76;
pub const KEY_M: u32 = 77;
pub const KEY_N: u32 = 78;
pub const KEY_O: u32 = 79;
pub const KEY_P: u32 = 80;
pub const KEY_Q: u32 = 81;
pub const KEY_R: u32 = 82;
pub const KEY_S: u32 = 83;
pub const KEY_T: u32 = 84;
pub const KEY_U: u32 = 85;
pub const KEY_V: u32 = 86;
pub const KEY_W: u32 = 87;
pub const KEY_X: u32 = 88;
pub const KEY_Y: u32 = 89;
pub const KEY_Z: u32 = 90;
pub const KEY_BRACKETLEFT: u32 = 91;
pub const KEY_BACKSLASH: u32 = 92;
pub const KEY_BRACKETRIGHT: u32 = 93;
pub const KEY_ASCIICIRCUM: u32 = 94;
pub const KEY_UNDERSCORE: u32 = 95;
pub const KEY_QUOTELEFT: u32 = 96;
pub const KEY_BRACELEFT: u32 = 123;
pub const KEY_BAR: u32 = 124;
pub const KEY_BRACERIGHT: u32 = 125;
pub const KEY_ASCIITILDE: u32 = 126;
pub const KEY_YEN: u32 = 165;
pub const KEY_SECTION: u32 = 167;

pub const KEY_MASK_CODE_MASK: u32 = 8388607;
pub const KEY_MASK_MODIFIER_MASK: u32 = 2130706432;
pub const KEY_MASK_CMD_OR_CTRL: u32 = 16777216;
pub const KEY_MASK_SHIFT: u32 = 33554432;
pub const KEY_MASK_ALT: u32 = 67108864;
pub const KEY_MASK_META: u32 = 134217728;
pub const KEY_MASK_CTRL: u32 = 268435456;
pub const KEY_MASK_KPAD: u32 = 536870912;
pub const KEY_MASK_GROUP_SWITCH: u32 = 1073741824;

/// The printable-key payload after removing the special/modifier bits.
pub fn key_code(key: u32) -> u32 {
    key & KEY_MASK_CODE_MASK
}

/// Whether the value is one of the modifier/lock keys.
pub fn is_modifier_key(key: u32) -> bool {
    matches!(
        key,
        KEY_SHIFT | KEY_CTRL | KEY_ALT | KEY_META | KEY_CAPSLOCK | KEY_NUMLOCK | KEY_SCROLLLOCK
    )
}

pub const KEY_LOCATION_UNSPECIFIED: i32 = 0;
pub const KEY_LOCATION_LEFT: i32 = 1;
pub const KEY_LOCATION_RIGHT: i32 = 2;

pub fn key_location_from_i32(value: i32) -> Option<KeyLocation> {
    match value {
        0 => Some(KeyLocation::Unspecified),
        1 => Some(KeyLocation::Left),
        2 => Some(KeyLocation::Right),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_values() {
        assert_eq!(KEY_NONE, 0);
        assert_eq!(KEY_SPECIAL, 1 << 22);
        assert_eq!(KEY_ESCAPE, (1 << 22) | 0x01);
        assert_eq!(KEY_MASK_SHIFT, 1 << 25);
        assert_eq!(KEY_MASK_CTRL, 1 << 28);
        assert_eq!(KEY_MASK_CODE_MASK, (1 << 23) - 1);
    }

    #[test]
    fn helpers() {
        assert_eq!(key_code(KEY_A), KEY_A & KEY_MASK_CODE_MASK);
        assert!(is_modifier_key(KEY_SHIFT));
        assert!(!is_modifier_key(KEY_A));
        assert_eq!(key_location_from_i32(1), Some(KeyLocation::Left));
        assert_eq!(key_location_from_i32(3), None);
    }
}
