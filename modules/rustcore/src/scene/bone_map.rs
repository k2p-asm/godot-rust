//! `scene/resources/bone_map.*`: profile-bone to skeleton-bone mapping.
//!
//! `set_profile` validates the map against the profile: missing profile bones
//! are inserted with an empty mapping, keys that are not profile bones are
//! removed, and a null profile clears the map. Setting a mapping requires the
//! profile bone to exist. Lookups return empty strings for missing entries;
//! `find_profile_bone_name` and `get_skeleton_bone_name_count` scan the map by
//! skeleton name (the engine's `HashMap` order is unspecified; this port uses
//! `HashMap` too).

use std::collections::HashMap;

use crate::scene::skeleton_profile::SkeletonProfileData;
use crate::string::string_name::StringName;

#[derive(Clone, Debug, Default)]
pub struct BoneMapData {
    profile: Option<SkeletonProfileData>,
    bone_map: HashMap<StringName, StringName>,
}

impl BoneMapData {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_profile(&self) -> Option<&SkeletonProfileData> {
        self.profile.as_ref()
    }

    /// Revalidates the map whenever the profile changes.
    pub fn set_profile(&mut self, profile: Option<SkeletonProfileData>) {
        if self.profile == profile {
            return;
        }
        self.profile = profile;
        self.validate_bone_map();
    }

    fn validate_bone_map(&mut self) {
        let Some(profile) = &self.profile else {
            self.bone_map.clear();
            return;
        };

        for bone in profile.get_bone_names() {
            self.bone_map
                .entry(bone)
                .or_insert_with(|| StringName::new(""));
        }

        let keep: Vec<StringName> = profile.get_bone_names();
        self.bone_map
            .retain(|name, _| keep.iter().any(|k| k == name));
    }

    /// Empty when the profile bone is unknown.
    pub fn get_skeleton_bone_name(&self, profile_bone: &str) -> StringName {
        self.bone_map
            .get(&StringName::new(profile_bone))
            .copied()
            .unwrap_or_else(|| StringName::new(""))
    }

    /// Only existing profile bones can be mapped.
    pub fn set_skeleton_bone_name(&mut self, profile_bone: &str, skeleton_bone: &str) -> bool {
        let key = StringName::new(profile_bone);
        if !self.bone_map.contains_key(&key) {
            return false;
        }
        self.bone_map.insert(key, StringName::new(skeleton_bone));
        true
    }

    /// First profile bone mapped to `skeleton_bone`, or empty.
    pub fn find_profile_bone_name(&self, skeleton_bone: &str) -> StringName {
        let wanted = StringName::new(skeleton_bone);
        for (profile_bone, mapped) in &self.bone_map {
            if *mapped == wanted {
                return *profile_bone;
            }
        }
        StringName::new("")
    }

    pub fn get_skeleton_bone_name_count(&self, skeleton_bone: &str) -> i32 {
        let wanted = StringName::new(skeleton_bone);
        self.bone_map.values().filter(|v| **v == wanted).count() as i32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn profile() -> SkeletonProfileData {
        let mut p = SkeletonProfileData::new();
        p.set_bone_size(2);
        p.set_bone_name(0, "Root");
        p.set_bone_name(1, "Spine");
        p
    }

    #[test]
    fn profile_validation_inserts_and_removes() {
        let mut b = BoneMapData::new();
        assert!(b.get_profile().is_none());
        assert_eq!(b.get_skeleton_bone_name("Root").as_string(), "");

        b.set_profile(Some(profile()));
        assert_eq!(b.get_profile().unwrap().get_bone_size(), 2);
        // Both profile bones exist with empty mappings.
        assert_eq!(b.get_skeleton_bone_name("Root").as_string(), "");
        assert_eq!(b.get_skeleton_bone_name("Spine").as_string(), "");

        assert!(b.set_skeleton_bone_name("Root", "SkeletonRoot"));
        assert!(b.set_skeleton_bone_name("Spine", "SkeletonSpine"));
        assert_eq!(b.get_skeleton_bone_name("Root").as_string(), "SkeletonRoot");
        assert!(!b.set_skeleton_bone_name("Unknown", "x"));

        // A smaller profile drops the missing bone and keeps the other mapping.
        let mut smaller = SkeletonProfileData::new();
        smaller.set_bone_size(1);
        smaller.set_bone_name(0, "Root");
        b.set_profile(Some(smaller));
        assert_eq!(b.get_skeleton_bone_name("Root").as_string(), "SkeletonRoot");
        assert_eq!(b.get_skeleton_bone_name("Spine").as_string(), "");
        assert!(!b.set_skeleton_bone_name("Spine", "x"));

        // A null profile clears everything.
        b.set_profile(None);
        assert_eq!(b.get_skeleton_bone_name("Root").as_string(), "");
        assert!(!b.set_skeleton_bone_name("Root", "x"));
    }

    #[test]
    fn reverse_lookup_and_counts() {
        let mut b = BoneMapData::new();
        b.set_profile(Some(profile()));
        b.set_skeleton_bone_name("Root", "Same");
        b.set_skeleton_bone_name("Spine", "Same");

        assert_eq!(b.find_profile_bone_name("Same").as_string() != "", true);
        assert_eq!(b.find_profile_bone_name("Missing").as_string(), "");
        assert_eq!(b.get_skeleton_bone_name_count("Same"), 2);
        assert_eq!(b.get_skeleton_bone_name_count("Missing"), 0);

        // Different skeleton names make the reverse lookup unique.
        b.set_skeleton_bone_name("Spine", "Other");
        assert_eq!(b.find_profile_bone_name("Same").as_string(), "Root");
        assert_eq!(b.get_skeleton_bone_name_count("Same"), 1);
        assert_eq!(b.get_skeleton_bone_name_count("Other"), 1);
    }
}
