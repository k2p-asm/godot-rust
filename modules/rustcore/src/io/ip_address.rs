//! `core/io/ip_address.*`: IPv4/IPv6 addresses.
//!
//! Ported faithfully from `ip_address.cpp`:
//!
//! - `_parse_ipv4` (three dots, each octet 0..=255, no leading/trailing junk);
//! - `_parse_ipv6` (up to four hex digits per group, one `::` shortening, the
//!   engine's field-shifting for the shortened part, and the `FFFF:`-mapped IPv4
//!   tail which sets `field16[5] = 0xFFFF` and parses the rest as IPv4);
//! - the string constructor (`*` wildcard, `:` → IPv6 under 40 characters,
//!   otherwise IPv4 under 16 characters);
//! - `operator String()` (`*`, empty for invalid, dotted quad for mapped IPv4,
//!   lowercase colon-separated hex groups otherwise);
//! - `is_valid_ip_address`, `set_ipv4`/`set_ipv6` and equality (both valid and
//!   the same 16 bytes).
//!
//! `IPV4_MAX_STRING_LENGTH` is 16 and `IPV6_MAX_STRING_LENGTH` is 40, as in the
//! header.

pub const IPV4_MAX_STRING_LENGTH: usize = 16;
pub const IPV6_MAX_STRING_LENGTH: usize = 40;

#[derive(Clone, Copy, Debug)]
pub struct IpAddress {
    bytes: [u8; 16],
    valid: bool,
    wildcard: bool,
}

/// operator==: two addresses are equal only when both are valid and share
/// all 16 bytes. Invalid addresses never compare equal (not even to themselves),
/// exactly like the C++.
impl PartialEq for IpAddress {
    fn eq(&self, other: &Self) -> bool {
        if self.valid != other.valid || !self.valid {
            return false;
        }
        self.bytes == other.bytes
    }
}

impl Default for IpAddress {
    fn default() -> Self {
        Self::new()
    }
}

impl IpAddress {
    /// The default constructor zeroes the fields (`clear`).
    pub fn new() -> Self {
        Self {
            bytes: [0; 16],
            valid: false,
            wildcard: false,
        }
    }

    /// `IPAddress(const String &)`.
    pub fn parse(text: &str) -> Self {
        let length = text.chars().count();
        if text == "*" {
            return Self {
                bytes: [0; 16],
                valid: false,
                wildcard: true,
            };
        }
        if length < IPV6_MAX_STRING_LENGTH && text.contains(':') {
            return match parse_ipv6(text) {
                Some(bytes) => Self {
                    bytes,
                    valid: true,
                    wildcard: false,
                },
                None => Self::new(),
            };
        }
        if length < IPV4_MAX_STRING_LENGTH {
            let mut bytes = [0u8; 16];
            if let Some(ipv4) = parse_ipv4(text, 0) {
                bytes[10] = 0xFF;
                bytes[11] = 0xFF;
                bytes[12..16].copy_from_slice(&ipv4);
                return Self {
                    bytes,
                    valid: true,
                    wildcard: false,
                };
            }
        }
        Self::new()
    }

    /// `IPAddress(a, b, c, d, is_v6)`.
    pub fn from_u32s(a: u32, b: u32, c: u32, d: u32, is_v6: bool) -> Self {
        let mut bytes = [0u8; 16];
        if !is_v6 {
            bytes[10] = 0xFF;
            bytes[11] = 0xFF;
            bytes[12] = a as u8;
            bytes[13] = b as u8;
            bytes[14] = c as u8;
            bytes[15] = d as u8;
        } else {
            bytes[0..4].copy_from_slice(&a.to_be_bytes());
            bytes[4..8].copy_from_slice(&b.to_be_bytes());
            bytes[8..12].copy_from_slice(&c.to_be_bytes());
            bytes[12..16].copy_from_slice(&d.to_be_bytes());
        }
        Self {
            bytes,
            valid: true,
            wildcard: false,
        }
    }

    /// `is_valid_ip_address`: no wildcard, no construction side effects.
    pub fn is_valid_ip_address(text: &str) -> bool {
        let length = text.chars().count();
        if length < IPV6_MAX_STRING_LENGTH && text.contains(':') {
            return parse_ipv6(text).is_some();
        }
        if length < IPV4_MAX_STRING_LENGTH {
            return parse_ipv4(text, 0).is_some();
        }
        false
    }

    pub fn clear(&mut self) {
        self.bytes = [0; 16];
        self.valid = false;
        self.wildcard = false;
    }

    pub fn is_wildcard(&self) -> bool {
        self.wildcard
    }

    pub fn is_valid(&self) -> bool {
        self.valid
    }

    pub fn is_ipv4(&self) -> bool {
        self.bytes[0..10].iter().all(|&byte| byte == 0)
            && self.bytes[10] == 0xFF
            && self.bytes[11] == 0xFF
    }

    pub fn is_ipv6(&self) -> bool {
        self.valid && !self.is_ipv4()
    }

    /// The four IPv4 bytes (also for an IPv4-mapped address).
    pub fn get_ipv4(&self) -> Option<[u8; 4]> {
        if !self.is_ipv4() {
            return None;
        }
        Some([
            self.bytes[12],
            self.bytes[13],
            self.bytes[14],
            self.bytes[15],
        ])
    }

    /// `set_ipv4`: clears first, then maps into IPv6.
    pub fn set_ipv4(&mut self, ip: [u8; 4]) {
        self.clear();
        self.valid = true;
        self.bytes[10] = 0xFF;
        self.bytes[11] = 0xFF;
        self.bytes[12..16].copy_from_slice(&ip);
    }

    pub fn get_ipv6(&self) -> [u8; 16] {
        self.bytes
    }

    pub fn set_ipv6(&mut self, buffer: [u8; 16]) {
        self.clear();
        self.valid = true;
        self.bytes = buffer;
    }

    /// `operator String()`.
    pub fn to_string(&self) -> String {
        if self.wildcard {
            return "*".to_string();
        }
        if !self.valid {
            return String::new();
        }
        if self.is_ipv4() {
            return format!(
                "{}.{}.{}.{}",
                self.bytes[12], self.bytes[13], self.bytes[14], self.bytes[15]
            );
        }
        (0..8)
            .map(|index| {
                let value = ((self.bytes[index * 2] as u16) << 8) + self.bytes[index * 2 + 1] as u16;
                format!("{value:x}")
            })
            .collect::<Vec<_>>()
            .join(":")
    }
}

/// `_parse_ipv4`, starting at character `start`.
fn parse_ipv4(text: &str, start: usize) -> Option<[u8; 4]> {
    let chars: Vec<char> = text.chars().collect();
    let length = chars.len();
    let mut dest = [0u8; 4];
    let mut cur = 0usize;
    let mut next: u16 = 0;
    let mut parsed = false;

    for index in start..length {
        let c = chars[index];
        if c == '.' {
            if !parsed {
                return None;
            }
            parsed = false;
            dest[cur] = next as u8;
            next = 0;
            cur += 1;
            if cur > 3 {
                return None;
            }
        } else if c.is_ascii_digit() {
            parsed = true;
            next = next * 10 + (c as u16 - '0' as u16);
            if next > 255 {
                return None;
            }
        } else {
            return None;
        }
    }
    if !parsed {
        return None;
    }
    dest[cur] = next as u8;
    if cur == 3 {
        Some(dest)
    } else {
        None
    }
}

/// `_parse_ipv6`.
fn parse_ipv6(text: &str) -> Option<[u8; 16]> {
    let chars: Vec<char> = text.chars().collect();
    let length = chars.len();
    let mut groups = [0u16; 8];
    let mut cur = 0usize;
    let mut shift: isize = -1;
    let mut i = 0usize;

    while i < length {
        let mut j = i;
        while j < length {
            let c = chars[j];
            if c == ':' {
                if j + 1 == length {
                    // Can't end with a colon (unless it is the shortening pair).
                    return None;
                }
                if chars[j + 1] == ':' {
                    if shift > -1 {
                        return None; // Only one shortening.
                    } else if j == 0 {
                        shift = cur as isize;
                    } else {
                        shift = cur as isize + 1;
                    }
                    j += 1;
                } else if i == j {
                    return None; // Stray colon.
                }
                i = j;
                break;
            }
            if j - i > 3 {
                return None;
            }
            if c == '.' {
                // IPv4-mapped IPv6 (e.g. "::FFFF:127.0.0.1").
                if cur < 1 || groups[cur - 1] != 0xFFFF {
                    return None; // The IPv6 part must end with FFFF.
                }
                if shift < 0 && cur != 6 {
                    return None; // Needs five zero groups and FFFF.
                }
                groups[cur] = 0;
                groups[cur - 1] = 0;
                while cur > 0 {
                    cur -= 1;
                    if groups[cur] != 0 {
                        return None;
                    }
                }
                groups[5] = 0xFFFF;
                let ipv4 = parse_ipv4(text, i)?;
                let mut bytes = groups_to_bytes(&groups);
                bytes[12..16].copy_from_slice(&ipv4);
                return Some(bytes);
            }
            let digit = c.to_digit(16)? as u16;
            if cur >= 8 {
                return None;
            }
            groups[cur] = (groups[cur] << 4) | digit;
            j += 1;
        }

        cur += 1;
        if cur > 8 || (cur == 8 && i + 1 != length) {
            return None;
        }
        i += 1;
    }

    if shift < 0 {
        // Must have parsed exactly eight groups.
        return if cur == 8 { Some(groups_to_bytes(&groups)) } else { None };
    }
    if shift > 7 {
        return None;
    }
    if shift as usize == cur || cur == 8 {
        return Some(groups_to_bytes(&groups));
    }

    // Shift the groups to the end, zeroing the shortened range.
    let pad = 8 - cur;
    let blank_end = shift as usize + pad;
    for index in (shift as usize + 1..8).rev() {
        if index < blank_end {
            groups[index] = 0;
        } else {
            groups[index] = groups[index - pad];
            groups[index - pad] = 0;
        }
    }
    Some(groups_to_bytes(&groups))
}

fn groups_to_bytes(groups: &[u16; 8]) -> [u8; 16] {
    let mut bytes = [0u8; 16];
    for (index, group) in groups.iter().enumerate() {
        bytes[index * 2..index * 2 + 2].copy_from_slice(&group.to_be_bytes());
    }
    bytes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ipv4_parsing_and_printing() {
        let ip = IpAddress::parse("127.0.0.1");
        assert!(ip.is_valid());
        assert!(ip.is_ipv4());
        assert!(!ip.is_wildcard());
        assert_eq!(ip.get_ipv4(), Some([127, 0, 0, 1]));
        assert_eq!(ip.to_string(), "127.0.0.1");

        let ip = IpAddress::parse("255.255.255.255");
        assert_eq!(ip.to_string(), "255.255.255.255");
        assert_eq!(IpAddress::parse("0.0.0.0").to_string(), "0.0.0.0");

        // Rejections.
        assert!(!IpAddress::parse("256.0.0.1").is_valid());
        assert!(!IpAddress::parse("1.2.3").is_valid());
        assert!(!IpAddress::parse("1.2.3.4.5").is_valid());
        assert!(!IpAddress::parse("1..3.4").is_valid());
        assert!(!IpAddress::parse("a.b.c.d").is_valid());
        assert!(!IpAddress::parse("1.2.3.4 ").is_valid());
        assert_eq!(IpAddress::parse("256.0.0.1").to_string(), "");
        assert!(!IpAddress::is_valid_ip_address("1.2.3"));
        assert!(IpAddress::is_valid_ip_address("1.2.3.4"));
    }

    #[test]
    fn ipv6_parsing_and_printing() {
        let full = IpAddress::parse("2001:db8:0:0:0:0:0:1");
        assert!(full.is_valid());
        assert!(full.is_ipv6());
        assert!(!full.is_ipv4());
        assert_eq!(full.to_string(), "2001:db8:0:0:0:0:0:1");

        assert_eq!(IpAddress::parse("::1").to_string(), "0:0:0:0:0:0:0:1");
        assert_eq!(IpAddress::parse("1::2").to_string(), "1:0:0:0:0:0:0:2");
        assert_eq!(IpAddress::parse("::").to_string(), "0:0:0:0:0:0:0:0");
        assert_eq!(IpAddress::parse("fe80::1").to_string(), "fe80:0:0:0:0:0:0:1");

        // Rejections.
        assert!(!IpAddress::parse("1:2:3:4:5:6:7:8:9").is_valid());
        assert!(!IpAddress::parse("1:::2").is_valid());
        assert!(!IpAddress::parse("1:2:3:4:5:6:7").is_valid());
        assert!(!IpAddress::parse("1:2:3:4:5:6:7:8:9:10").is_valid());
        assert!(!IpAddress::parse("gggg::1").is_valid());
        assert!(!IpAddress::parse("12345::1").is_valid());
        assert!(!IpAddress::parse("1:2:").is_valid());
        assert!(!IpAddress::parse("::1::2").is_valid());
    }

    #[test]
    fn ipv4_mapped_ipv6() {
        let ip = IpAddress::parse("::FFFF:127.0.0.1");
        assert!(ip.is_valid());
        assert!(ip.is_ipv4(), "the engine treats the mapped form as IPv4");
        assert_eq!(ip.get_ipv4(), Some([127, 0, 0, 1]));
        assert_eq!(ip.to_string(), "127.0.0.1");

        let ip = IpAddress::parse("0:0:0:0:0:FFFF:192.168.1.1");
        assert_eq!(ip.to_string(), "192.168.1.1");
        assert!(!IpAddress::parse("::1:127.0.0.1").is_valid());
        assert!(!IpAddress::parse("::FFFF:127.0.0").is_valid());
    }

    #[test]
    fn wildcard_and_constructors() {
        let wildcard = IpAddress::parse("*");
        assert!(wildcard.is_wildcard());
        assert!(!wildcard.is_valid());
        assert_eq!(wildcard.to_string(), "*");

        let ip = IpAddress::from_u32s(10, 0, 0, 5, false);
        assert_eq!(ip.to_string(), "10.0.0.5");
        let ip = IpAddress::from_u32s(0x20010db8, 0, 0, 1, true);
        assert_eq!(ip.to_string(), "2001:db8:0:0:0:0:0:1");

        let mut ip = IpAddress::new();
        assert!(!ip.is_valid());
        assert_eq!(ip.to_string(), "");
        ip.set_ipv4([1, 2, 3, 4]);
        assert_eq!(ip.to_string(), "1.2.3.4");
        ip.set_ipv6([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1]);
        assert_eq!(ip.to_string(), "0:0:0:0:0:0:0:1");
        ip.clear();
        assert!(!ip.is_valid() && !ip.is_wildcard());
    }

    #[test]
    fn equality() {
        let a = IpAddress::parse("1.2.3.4");
        let b = IpAddress::parse("1.2.3.4");
        let c = IpAddress::parse("1.2.3.5");
        assert_eq!(a, b);
        assert_ne!(a, c);
        // Invalid addresses never compare equal, even to themselves.
        let invalid = IpAddress::new();
        assert_ne!(invalid, invalid);
        assert_ne!(a, invalid);
    }
}
