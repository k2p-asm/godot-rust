//! `core/os/time.*`: proleptic-Gregorian date/time conversions.
//!
//! All conversions use the same civil-date algorithms as the engine (proleptic
//! Gregorian, year 0 for 1 BC, no leap seconds). Unix time is seconds since
//! 1970-01-01T00:00:00; negative values floor correctly. Weekdays follow the
//! engine's `Weekday` enum: Sunday = 0 ... Saturday = 6.
//!
//! The system-clock methods (`get_*_from_system`, ticks) are not ported because
//! they need the platform layer; the pure conversion/format/parse API is.

/// Broken-down date and time.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DateTime {
    pub year: i32,
    pub month: i32,
    pub day: i32,
    /// Sunday = 0 ... Saturday = 6.
    pub weekday: i32,
    pub hour: i32,
    pub minute: i32,
    pub second: i32,
}

fn floor_div(a: i64, b: i64) -> i64 {
    let q = a / b;
    if (a % b != 0) && ((a < 0) != (b < 0)) {
        q - 1
    } else {
        q
    }
}

fn floor_mod(a: i64, b: i64) -> i64 {
    a - floor_div(a, b) * b
}

/// Days since 1970-01-01 from a civil date (Howard Hinnant's algorithm).
fn days_from_civil(year: i32, month: i32, day: i32) -> i64 {
    let y = if month <= 2 { year - 1 } else { year } as i64;
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400; // [0, 399]
    let m = month as i64;
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + day as i64 - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy; // [0, 146096]
    era * 146097 + doe - 719468
}

/// Civil date from days since 1970-01-01.
fn civil_from_days(days: i64) -> (i32, i32, i32) {
    let z = days + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = z - era * 146097; // [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365; // [0, 399]
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
    let mp = (5 * doy + 2) / 153; // [0, 11]
    let d = doy - (153 * mp + 2) / 5 + 1; // [1, 31]
    let m = if mp < 10 { mp + 3 } else { mp - 9 }; // [1, 12]
    let year = if m <= 2 { y + 1 } else { y };
    (year as i32, m as i32, d as i32)
}

/// Sunday-based weekday for days since the epoch (1970-01-01 was Thursday).
fn weekday_from_days(days: i64) -> i32 {
    floor_mod(days + 4, 7) as i32
}

pub fn datetime_from_unix_time(unix_time: i64) -> DateTime {
    let days = floor_div(unix_time, 86400);
    let secs = floor_mod(unix_time, 86400);
    let (year, month, day) = civil_from_days(days);
    DateTime {
        year,
        month,
        day,
        weekday: weekday_from_days(days),
        hour: (secs / 3600) as i32,
        minute: ((secs % 3600) / 60) as i32,
        second: (secs % 60) as i32,
    }
}

/// `get_datetime_string_from_unix_time`: `YYYY-MM-DD[T ]HH:MM:SS`.
pub fn datetime_string_from_unix_time(unix_time: i64, use_space: bool) -> String {
    let d = datetime_from_unix_time(unix_time);
    format!(
        "{:04}-{:02}-{:02}{}{:02}:{:02}:{:02}",
        d.year,
        d.month,
        d.day,
        if use_space { ' ' } else { 'T' },
        d.hour,
        d.minute,
        d.second
    )
}

pub fn date_string_from_unix_time(unix_time: i64) -> String {
    let d = datetime_from_unix_time(unix_time);
    format!("{:04}-{:02}-{:02}", d.year, d.month, d.day)
}

pub fn time_string_from_unix_time(unix_time: i64) -> String {
    let d = datetime_from_unix_time(unix_time);
    format!("{:02}:{:02}:{:02}", d.hour, d.minute, d.second)
}

/// Parses `YYYY-MM-DD[T ]HH:MM:SS` (the engine's canonical form) and returns
/// the Unix timestamp, or `None` on malformed input.
pub fn unix_time_from_datetime_string(text: &str) -> Option<i64> {
    let bytes = text.as_bytes();
    if bytes.len() < 19 {
        return None;
    }
    let part = |start: usize, len: usize| -> Option<i32> {
        let slice = bytes.get(start..start + len)?;
        std::str::from_utf8(slice).ok()?.parse::<i32>().ok()
    };
    let year = part(0, 4)?;
    if bytes[4] != b'-' || bytes[7] != b'-' {
        return None;
    }
    let month = part(5, 2)?;
    let day = part(8, 2)?;
    if bytes[10] != b'T' && bytes[10] != b' ' {
        return None;
    }
    if bytes[13] != b':' || bytes[16] != b':' {
        return None;
    }
    let hour = part(11, 2)?;
    let minute = part(14, 2)?;
    let second = part(17, 2)?;
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return None;
    }
    if !(0..=23).contains(&hour) || !(0..=59).contains(&minute) || !(0..=60).contains(&second) {
        return None;
    }
    Some(
        days_from_civil(year, month, day) * 86400
            + hour as i64 * 3600
            + minute as i64 * 60
            + second as i64,
    )
}

/// `get_offset_string_from_offset_minutes`: `+HH:MM` / `-HH:MM`.
pub fn offset_string_from_offset_minutes(minutes: i64) -> String {
    let sign = if minutes < 0 { '-' } else { '+' };
    let abs = minutes.abs();
    format!("{}{:02}:{:02}", sign, abs / 60, abs % 60)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn epoch_and_known_dates() {
        let epoch = datetime_from_unix_time(0);
        assert_eq!(epoch.year, 1970);
        assert_eq!(epoch.month, 1);
        assert_eq!(epoch.day, 1);
        assert_eq!(epoch.weekday, 4); // Thursday
        assert_eq!((epoch.hour, epoch.minute, epoch.second), (0, 0, 0));

        // 1_000_000_000 = 2001-09-09T01:46:40Z.
        let d = datetime_from_unix_time(1_000_000_000);
        assert_eq!((d.year, d.month, d.day), (2001, 9, 9));
        assert_eq!((d.hour, d.minute, d.second), (1, 46, 40));
        assert_eq!(d.weekday, 0); // Sunday

        // Negative times floor: -1 is the last second of 1969.
        let before = datetime_from_unix_time(-1);
        assert_eq!((before.year, before.month, before.day), (1969, 12, 31));
        assert_eq!((before.hour, before.minute, before.second), (23, 59, 59));

        // 2000-02-29 (leap year, century divisible by 400).
        let leap = unix_time_from_datetime_string("2000-02-29T12:34:56").unwrap();
        let back = datetime_from_unix_time(leap);
        assert_eq!((back.year, back.month, back.day), (2000, 2, 29));
        assert_eq!((back.hour, back.minute, back.second), (12, 34, 56));
    }

    #[test]
    fn strings_and_round_trips() {
        assert_eq!(datetime_string_from_unix_time(0, false), "1970-01-01T00:00:00");
        assert_eq!(datetime_string_from_unix_time(0, true), "1970-01-01 00:00:00");
        assert_eq!(date_string_from_unix_time(1_000_000_000), "2001-09-09");
        assert_eq!(time_string_from_unix_time(1_000_000_000), "01:46:40");

        for stamp in [0, 1_000_000_000, -1, 1_700_000_000, 4_102_444_800] {
            let text = datetime_string_from_unix_time(stamp, false);
            assert_eq!(unix_time_from_datetime_string(&text), Some(stamp), "{text}");
        }

        assert_eq!(
            unix_time_from_datetime_string("2001-09-09 01:46:40"),
            Some(1_000_000_000)
        );
    }

    #[test]
    fn malformed_datetimes_are_rejected() {
        assert_eq!(unix_time_from_datetime_string(""), None);
        assert_eq!(unix_time_from_datetime_string("2001-09-09"), None);
        assert_eq!(unix_time_from_datetime_string("2001-13-09T00:00:00"), None);
        assert_eq!(unix_time_from_datetime_string("2001-09-09T24:00:00"), None);
        assert_eq!(unix_time_from_datetime_string("2001-09-09T00:60:00"), None);
        assert_eq!(unix_time_from_datetime_string("2001/09/09T00:00:00"), None);
        assert_eq!(unix_time_from_datetime_string("abcd-09-09T00:00:00"), None);
    }

    #[test]
    fn offset_strings() {
        assert_eq!(offset_string_from_offset_minutes(0), "+00:00");
        assert_eq!(offset_string_from_offset_minutes(330), "+05:30");
        assert_eq!(offset_string_from_offset_minutes(-480), "-08:00");
        assert_eq!(offset_string_from_offset_minutes(-45), "-00:45");
    }
}
