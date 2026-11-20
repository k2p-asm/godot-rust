//! `core/os/os.h` (platform metadata subset) and `core/templates/pair.h`.
//!
//! `OS` is a singleton over the platform layer; this port exposes the parts that
//! are pure configuration: the OS name, the build feature set from
//! `OS::has_feature` (platform, architecture, editor/template, precision,
//! threads, module flags) and the processor count query hook.
//!
//! `Pair` mirrors the engine's template: `first`/`second` with structural
//! equality and ordering by `first` then `second`.

/// `OS::get_os_name` for this build target.
pub const OS_NAME: &str = "Windows";

/// `OS::has_feature`: the feature strings the port's build configuration sets.
#[derive(Clone, Debug)]
pub struct OsFeatures {
    features: Vec<String>,
}

impl Default for OsFeatures {
    fn default() -> Self {
        Self::new()
    }
}

impl OsFeatures {
    /// The port's Windows editor build: 64-bit, threads, editor, single
    /// precision, no mono/double, module-config subset (none).
    pub fn new() -> Self {
        let mut features = vec![
            "windows".to_string(),
            "pc".to_string(),
            "x86_64".to_string(),
            "64".to_string(),
            "editor".to_string(),
            "threads".to_string(),
            "single".to_string(),
        ];
        features.sort();
        Self { features }
    }

    /// A template (export) build instead of the editor.
    pub fn template() -> Self {
        let mut features = Self::new();
        features.features.retain(|feature| feature != "editor");
        features.features.push("template".to_string());
        features.features.sort();
        features
    }

    pub fn has_feature(&self, feature: &str) -> bool {
        self.features.iter().any(|known| known == feature)
    }

    pub fn get_features(&self) -> &[String] {
        &self.features
    }
}

/// `Pair<T, U>`.
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Pair<T, U> {
    pub first: T,
    pub second: U,
}

impl<T, U> Pair<T, U> {
    pub fn new(first: T, second: U) -> Self {
        Self { first, second }
    }

    pub fn from(first: T, second: U) -> Self {
        Self::new(first, second)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn os_metadata() {
        assert_eq!(OS_NAME, "Windows");
        let features = OsFeatures::new();
        assert!(features.has_feature("windows"));
        assert!(features.has_feature("x86_64"));
        assert!(features.has_feature("64"));
        assert!(features.has_feature("editor"));
        assert!(features.has_feature("threads"));
        assert!(features.has_feature("single"));
        assert!(features.has_feature("pc"));
        assert!(!features.has_feature("macos"));
        assert!(!features.has_feature("template"));
        assert!(!features.has_feature("double"));
        assert!(!features.has_feature("mono"));
        // The list is sorted and duplicate-free.
        let list = features.get_features();
        let mut sorted = list.to_vec();
        sorted.sort();
        assert_eq!(list, sorted.as_slice());
        assert_eq!(list.len(), 7);

        let template = OsFeatures::template();
        assert!(template.has_feature("template"));
        assert!(!template.has_feature("editor"));
        assert!(template.has_feature("windows"));
        assert_eq!(template.get_features().len(), 7);
    }

    #[test]
    fn pair_ordering() {
        let pair = Pair::new(1, "a");
        assert_eq!(pair.first, 1);
        assert_eq!(pair.second, "a");
        assert_eq!(Pair::from(1, "a"), pair);

        let a = Pair::new(1, 2);
        let b = Pair::new(1, 3);
        let c = Pair::new(2, 0);
        assert!(a < b, "second breaks ties");
        assert!(b < c, "first dominates");
        assert_eq!(a.cmp(&a), std::cmp::Ordering::Equal);
        assert_ne!(a, b);
    }
}
