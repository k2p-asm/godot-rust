//! `core/version.h` (and `version.py`): the engine version constants.
//!
//! The C++ header is macro-only; this port exposes the same values as
//! `const`s plus a parser/comparator for version strings (the branch/patch
//! formatting rule included: a zero patch is omitted from the pretty number,
//! the hexadecimal form is `major << 16 | minor << 8 | patch`).

pub const GODOT_VERSION_MAJOR: u32 = 4;
pub const GODOT_VERSION_MINOR: u32 = 7;
pub const GODOT_VERSION_PATCH: u32 = 2;
pub const GODOT_VERSION_STATUS: &str = "stable";
pub const GODOT_VERSION_MODULE_CONFIG: &str = "";

/// `GODOT_VERSION_BRANCH` (`major.minor`).
pub const GODOT_VERSION_BRANCH: &str = "4.7";
/// `GODOT_VERSION_NUMBER` (`major.minor[.patch]`).
pub const GODOT_VERSION_NUMBER: &str = "4.7.2";
/// `GODOT_VERSION_HEX`.
pub const GODOT_VERSION_HEX: u32 = (GODOT_VERSION_MAJOR << 16)
    | (GODOT_VERSION_MINOR << 8)
    | GODOT_VERSION_PATCH;
/// `GODOT_VERSION_FULL_CONFIG`.
pub const GODOT_VERSION_FULL_CONFIG: &str = "4.7.2.stable";
/// `GODOT_VERSION_NAME`.
pub const GODOT_VERSION_NAME: &str = "Godot";

/// `GODOT_VERSION_HEX` for any triple.
pub const fn version_hex(major: u32, minor: u32, patch: u32) -> u32 {
    (major << 16) | (minor << 8) | patch
}

/// The pretty number: `major.minor` when the patch is zero.
pub fn version_number(major: u32, minor: u32, patch: u32) -> String {
    if patch == 0 {
        format!("{major}.{minor}")
    } else {
        format!("{major}.{minor}.{patch}")
    }
}

/// A parsed version (`4.7.2.stable.official` and friends).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VersionInfo {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub status: String,
    pub build: String,
}

impl VersionInfo {
    /// Parses `major.minor[.patch][.status][.build]`; `None` when the numeric
    /// fields are missing or not numbers.
    pub fn parse(text: &str) -> Option<Self> {
        let mut parts = text.split('.');
        let major = parts.next()?.parse::<u32>().ok()?;
        let minor = parts.next()?.parse::<u32>().ok()?;
        let mut patch = 0;
        let mut status = String::new();
        let mut build = String::new();
        let remaining: Vec<&str> = parts.collect();
        let mut index = 0;
        if let Some(candidate) = remaining.first() {
            if let Ok(value) = candidate.parse::<u32>() {
                patch = value;
                index = 1;
            }
        }
        if let Some(candidate) = remaining.get(index) {
            status = (*candidate).to_string();
            index += 1;
        }
        if let Some(candidate) = remaining.get(index) {
            build = (*candidate).to_string();
        }
        Some(Self {
            major,
            minor,
            patch,
            status,
            build,
        })
    }

    pub fn hex(&self) -> u32 {
        version_hex(self.major, self.minor, self.patch)
    }

    pub fn branch(&self) -> String {
        format!("{}.{}", self.major, self.minor)
    }

    pub fn number(&self) -> String {
        version_number(self.major, self.minor, self.patch)
    }

    pub fn full_config(&self) -> String {
        if self.status.is_empty() {
            self.number()
        } else {
            format!("{}.{}", self.number(), self.status)
        }
    }

    pub fn full_build(&self) -> String {
        let config = self.full_config();
        if self.build.is_empty() {
            config
        } else {
            format!("{config}.{}", self.build)
        }
    }

    pub fn full_name(&self) -> String {
        format!("{GODOT_VERSION_NAME} v{}", self.full_build())
    }
}

impl PartialOrd for VersionInfo {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for VersionInfo {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (self.major, self.minor, self.patch).cmp(&(other.major, other.minor, other.patch))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constants_match_version_py() {
        assert_eq!(GODOT_VERSION_MAJOR, 4);
        assert_eq!(GODOT_VERSION_MINOR, 7);
        assert_eq!(GODOT_VERSION_PATCH, 2);
        assert_eq!(GODOT_VERSION_BRANCH, "4.7");
        assert_eq!(GODOT_VERSION_NUMBER, "4.7.2");
        assert_eq!(GODOT_VERSION_HEX, 0x040702);
        assert_eq!(GODOT_VERSION_FULL_CONFIG, "4.7.2.stable");
        assert_eq!(GODOT_VERSION_NAME, "Godot");
        assert_eq!(version_hex(3, 1, 4), 0x030104);
        assert_eq!(version_number(4, 7, 0), "4.7");
        assert_eq!(version_number(4, 7, 2), "4.7.2");
    }

    #[test]
    fn parsing_and_formatting() {
        let version = VersionInfo::parse("4.7.2.stable.official").expect("parsed");
        assert_eq!(version.major, 4);
        assert_eq!(version.minor, 7);
        assert_eq!(version.patch, 2);
        assert_eq!(version.status, "stable");
        assert_eq!(version.build, "official");
        assert_eq!(version.hex(), 0x040702);
        assert_eq!(version.branch(), "4.7");
        assert_eq!(version.number(), "4.7.2");
        assert_eq!(version.full_config(), "4.7.2.stable");
        assert_eq!(version.full_build(), "4.7.2.stable.official");
        assert_eq!(version.full_name(), "Godot v4.7.2.stable.official");

        // A missing patch defaults to zero and is omitted from the number.
        let version = VersionInfo::parse("4.7.stable").expect("parsed");
        assert_eq!(version.patch, 0);
        assert_eq!(version.number(), "4.7");
        assert_eq!(version.full_config(), "4.7.stable");

        let version = VersionInfo::parse("4.7.2").expect("parsed");
        assert_eq!(version.full_config(), "4.7.2");
        assert_eq!(version.full_build(), "4.7.2");
        assert_eq!(version.full_name(), "Godot v4.7.2");

        // An unparsable status is treated as the status field.
        let version = VersionInfo::parse("4.7.beta").expect("parsed");
        assert_eq!(version.patch, 0);
        assert_eq!(version.status, "beta");

        assert!(VersionInfo::parse("").is_none());
        assert!(VersionInfo::parse("4").is_none());
        assert!(VersionInfo::parse("x.y").is_none());
        assert!(VersionInfo::parse("4.x").is_none());
    }

    #[test]
    fn ordering_and_equality() {
        let a = VersionInfo::parse("4.7.2").unwrap();
        let b = VersionInfo::parse("4.7.3").unwrap();
        let c = VersionInfo::parse("4.8").unwrap();
        let d = VersionInfo::parse("3.9.9").unwrap();
        assert!(a < b);
        assert!(b < c);
        assert!(d < a);
        assert_eq!(a.cmp(&a), std::cmp::Ordering::Equal);
        // The status/build do not participate in ordering.
        let mut with_status = a.clone();
        with_status.status = "rc1".to_string();
        assert_eq!(a.cmp(&with_status), std::cmp::Ordering::Equal);
        assert_ne!(a, with_status);
    }
}
