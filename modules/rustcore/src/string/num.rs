//! Integer formatting from `String::num_int64` / `num_uint64` (bases 2..=36).
//!
//! These are **not** differential tests: the reference implementation lives in
//! `ustring.cpp`, which cannot be linked into the parity harness without the
//! whole `String`/`CowData` chain. The tests here encode the documented
//! behaviour instead, and say so.

use super::gstring::GoString;

/// `String::num_int64(p_num, base, capitalize_hex)`.
/// Returns `None` for a base outside 2..=36 (the engine errors out there).
pub fn num_int64(num: i64, base: i32, capitalize_hex: bool) -> Option<GoString> {
    if !(2..=36).contains(&base) {
        return None;
    }
    let base = base as i64;
    let sign = num < 0;

    // Digit count, exactly as the engine walks it.
    let mut n = num;
    let mut chars = 0usize;
    loop {
        n /= base;
        chars += 1;
        if n == 0 {
            break;
        }
    }
    if sign {
        chars += 1;
    }

    let mut out = vec![0u32; chars];
    n = num;
    let mut idx = chars;
    loop {
        // `Math::abs` on the remainder, so INT64_MIN does not overflow.
        let mut rem = n % base;
        if rem < 0 {
            rem = -rem;
        }
        idx -= 1;
        out[idx] = if rem >= 10 {
            let a = if capitalize_hex { b'A' } else { b'a' } as u32;
            a + (rem as u32 - 10)
        } else {
            b'0' as u32 + rem as u32
        };
        n /= base;
        if n == 0 {
            break;
        }
    }

    if sign {
        out[0] = b'-' as u32;
    }

    Some(GoString::from_units(&out))
}

/// `String::num_uint64(p_num, base, capitalize_hex)`.
pub fn num_uint64(num: u64, base: i32, capitalize_hex: bool) -> Option<GoString> {
    if !(2..=36).contains(&base) {
        return None;
    }
    let base = base as u64;

    let mut n = num;
    let mut chars = 0usize;
    loop {
        n /= base;
        chars += 1;
        if n == 0 {
            break;
        }
    }

    let mut out = vec![0u32; chars];
    n = num;
    let mut idx = chars;
    loop {
        let rem = (n % base) as u32;
        idx -= 1;
        out[idx] = if rem >= 10 {
            let a = if capitalize_hex { b'A' } else { b'a' } as u32;
            a + (rem - 10)
        } else {
            b'0' as u32 + rem
        };
        n /= base;
        if n == 0 {
            break;
        }
    }

    Some(GoString::from_units(&out))
}

/// Floating-point formatting, approximating `String::num(p_num, decimals)`.
///
/// Semantics implemented: `decimals >= 0` prints exactly that many fractional
/// digits; `decimals < 0` prints up to 6 and trims trailing zeros (and the dot).
///
/// This is **not** the engine's algorithm (which routes through `String::num`
/// and its `%f`-style path in `ustring.cpp`) and it is not differential: treat
/// it as a placeholder with defined behaviour, not as a verified port.
pub fn num_float(value: f64, decimals: i32) -> GoString {
    if !value.is_finite() {
        // The engine prints inf/-inf/nan through the same path.
        let text = if value.is_nan() {
            "nan"
        } else if value > 0.0 {
            "inf"
        } else {
            "-inf"
        };
        return GoString::from_str(text);
    }

    let dec = if decimals < 0 { 6 } else { decimals.min(16) } as usize;
    let mut text = format!("{value:.dec$}");

    if decimals < 0 && text.contains('.') {
        while text.ends_with('0') {
            text.pop();
        }
        if text.ends_with('.') {
            text.pop();
        }
    }

    // Avoid "-0" for values that rounded to zero.
    if text == "-0" {
        text = "0".to_string();
    }

    GoString::from_str(&text)
}
#[cfg(test)]
mod tests {
    use super::*;

    fn n(v: i64, base: i32) -> String {
        num_int64(v, base, false).unwrap().as_string()
    }

    #[test]
    fn decimal_basics() {
        assert_eq!(n(0, 10), "0");
        assert_eq!(n(7, 10), "7");
        assert_eq!(n(-7, 10), "-7");
        assert_eq!(n(1234567, 10), "1234567");
        assert_eq!(n(-1234567, 10), "-1234567");
    }

    #[test]
    fn int64_extremes() {
        assert_eq!(n(i64::MAX, 10), "9223372036854775807");
        // The engine takes abs() of the remainder, so this does not overflow.
        assert_eq!(n(i64::MIN, 10), "-9223372036854775808");
    }

    #[test]
    fn power_of_two_and_hex_bases() {
        assert_eq!(n(255, 16), "ff");
        assert_eq!(num_int64(255, 16, true).unwrap().as_string(), "FF");
        assert_eq!(n(8, 2), "1000");
        assert_eq!(n(64, 8), "100");
        assert_eq!(n(35, 36), "z");
        assert_eq!(n(-255, 16), "-ff");
    }

    #[test]
    fn invalid_bases_are_rejected() {
        assert!(num_int64(1, 1, false).is_none());
        assert!(num_int64(1, 37, false).is_none());
        assert!(num_uint64(1, 0, false).is_none());
    }

    #[test]
    fn float_formatting_rules() {
        assert_eq!(num_float(1.5, 3).as_string(), "1.500");
        assert_eq!(num_float(1.23456789, 2).as_string(), "1.23");
        assert_eq!(num_float(-2.0, 0).as_string(), "-2");
        // Negative decimals trim trailing zeros.
        assert_eq!(num_float(1.500000, -1).as_string(), "1.5");
        assert_eq!(num_float(2.0, -1).as_string(), "2");
        assert_eq!(num_float(0.0, -1).as_string(), "0");
        // -0 must not print as "-0".
        assert_eq!(num_float(-0.0, -1).as_string(), "0");
        // Non-finite values.
        assert_eq!(num_float(f64::INFINITY, -1).as_string(), "inf");
        assert_eq!(num_float(f64::NEG_INFINITY, -1).as_string(), "-inf");
        assert_eq!(num_float(f64::NAN, -1).as_string(), "nan");
        // Decimals are capped at 16.
        assert_eq!(num_float(0.5, 40).as_string().len(), 18);

        // Rounding follows the formatter (half to even at the last digit).
        assert_eq!(num_float(2.5, 0).as_string(), "2");
        assert_eq!(num_float(3.5, 0).as_string(), "4");
        assert_eq!(num_float(0.1 + 0.2, 3).as_string(), "0.300");

        // Very small values collapse to zero at the default precision.
        assert_eq!(num_float(1e-9, -1).as_string(), "0");
        // Large magnitudes keep every integer digit (no exponent form).
        assert_eq!(num_float(123456789.0, -1).as_string(), "123456789");
        // Negative zero with fixed decimals keeps the sign of the format.
        assert_eq!(num_float(-0.0, 2).as_string(), "-0.00");
    }

    #[test]
    fn unsigned_wraps_above_int64_max() {
        assert_eq!(
            num_uint64(u64::MAX, 10, false).unwrap().as_string(),
            "18446744073709551615"
        );
        assert_eq!(num_uint64(0, 10, false).unwrap().as_string(), "0");
    }
}
