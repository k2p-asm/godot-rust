//! `scene/resources/animation_library.*`: the named animation store.
//!
//! Names are validated exactly like the engine: an animation name cannot be
//! empty nor contain `/`, `:`, `,` or `[`; a library name allows the empty
//! string but forbids the same four characters, and `validate_library_name`
//! replaces them with `_`.
//!
//! The engine keeps the animations in an `RBMap` ordered by `StringName`'s
//! alphabetical comparator, so `get_animation_list` comes out sorted; that is
//! reproduced here explicitly.
//!
//! Not ported: the `Resource` signals (`animation_added`/`removed`/`renamed`)
//! and the changed-notification forwarding.

use crate::scene::animation::Animation;
use crate::string::string_name::StringName;
use std::collections::HashMap;

pub struct AnimationLibrary {
    animations: HashMap<StringName, Animation>,
}

impl Default for AnimationLibrary {
    fn default() -> Self {
        Self::new()
    }
}

impl AnimationLibrary {
    pub fn new() -> Self {
        Self {
            animations: HashMap::new(),
        }
    }

    pub fn is_valid_animation_name(name: &str) -> bool {
        !(name.is_empty()
            || name.contains('/')
            || name.contains(':')
            || name.contains(',')
            || name.contains('['))
    }

    pub fn is_valid_library_name(name: &str) -> bool {
        !(name.contains('/')
            || name.contains(':')
            || name.contains(',')
            || name.contains('['))
    }

    /// Replaces the four forbidden characters with underscores.
    pub fn validate_library_name(name: &str) -> String {
        name.replace(['/', ':', ',', '['], "_")
    }

    /// Adds or replaces an animation. Returns false for an invalid name; the
    /// engine also rejects a null animation, which the by-value signature
    /// cannot express.
    pub fn add_animation(&mut self, name: &str, animation: Animation) -> bool {
        if !Self::is_valid_animation_name(name) {
            return false;
        }
        self.animations.insert(StringName::new(name), animation);
        true
    }

    /// The engine errors on a missing name; here it reports false.
    pub fn remove_animation(&mut self, name: &str) -> bool {
        self.animations.remove(&StringName::new(name)).is_some()
    }

    /// Fails when the source is missing, the new name is invalid, or the new
    /// name already exists (renaming to the same name included).
    pub fn rename_animation(&mut self, name: &str, new_name: &str) -> bool {
        let key = StringName::new(name);
        if !self.animations.contains_key(&key) {
            return false;
        }
        if !Self::is_valid_animation_name(new_name) {
            return false;
        }
        let new_key = StringName::new(new_name);
        if self.animations.contains_key(&new_key) {
            return false;
        }
        let animation = self.animations.remove(&key).expect("checked above");
        self.animations.insert(new_key, animation);
        true
    }

    pub fn has_animation(&self, name: &str) -> bool {
        self.animations.contains_key(&StringName::new(name))
    }

    pub fn get_animation(&self, name: &str) -> Option<&Animation> {
        self.animations.get(&StringName::new(name))
    }

    /// Sorted alphabetically by name, matching the engine's `RBMap` order.
    pub fn get_animation_list(&self) -> Vec<StringName> {
        let mut names: Vec<StringName> = self.animations.keys().copied().collect();
        names.sort_by(|a, b| a.as_string().cmp(&b.as_string()));
        names
    }

    pub fn get_animation_list_size(&self) -> usize {
        self.animations.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scene::animation::{Animation, LoopMode};

    fn anim(length: f64) -> Animation {
        let mut a = Animation::new();
        a.set_length(length);
        a.set_loop_mode(LoopMode::Linear);
        a
    }

    #[test]
    fn name_validation_matches_the_engine() {
        assert!(AnimationLibrary::is_valid_animation_name("run"));
        assert!(!AnimationLibrary::is_valid_animation_name(""));
        for bad in ["a/b", "a:b", "a,b", "a[b"] {
            assert!(!AnimationLibrary::is_valid_animation_name(bad), "{bad}");
        }

        // Library names may be empty, but not contain the four characters.
        assert!(AnimationLibrary::is_valid_library_name(""));
        assert!(AnimationLibrary::is_valid_library_name("Hero"));
        assert!(!AnimationLibrary::is_valid_library_name("a/b"));

        assert_eq!(AnimationLibrary::validate_library_name("a/b:c,d[e"), "a_b_c_d_e");
        assert_eq!(AnimationLibrary::validate_library_name("ok"), "ok");
    }

    #[test]
    fn add_replace_and_list() {
        let mut lib = AnimationLibrary::new();
        assert_eq!(lib.get_animation_list_size(), 0);

        assert!(lib.add_animation("run", anim(1.0)));
        assert!(lib.add_animation("idle", anim(2.0)));
        assert!(lib.add_animation("walk", anim(3.0)));
        assert_eq!(lib.get_animation_list_size(), 3);
        assert!(lib.has_animation("run"));
        assert!(!lib.has_animation("jump"));

        // Sorted by name.
        let list: Vec<String> = lib
            .get_animation_list()
            .iter()
            .map(|n| n.as_string())
            .collect();
        assert_eq!(list, vec!["idle", "run", "walk"]);
        assert_eq!(lib.get_animation("walk").unwrap().get_length(), 3.0);

        // Adding an existing name replaces it.
        assert!(lib.add_animation("run", anim(9.0)));
        assert_eq!(lib.get_animation_list_size(), 3);
        assert_eq!(lib.get_animation("run").unwrap().get_length(), 9.0);

        // Invalid names are refused and leave the store untouched.
        assert!(!lib.add_animation("a/b", anim(1.0)));
        assert!(!lib.add_animation("", anim(1.0)));
        assert_eq!(lib.get_animation_list_size(), 3);
    }

    #[test]
    fn rename_rules() {
        let mut lib = AnimationLibrary::new();
        lib.add_animation("run", anim(1.0));
        lib.add_animation("walk", anim(2.0));

        assert!(!lib.rename_animation("missing", "jump"));
        assert!(!lib.rename_animation("run", "a/b"));
        // Renaming onto an existing name fails, and so does renaming to itself.
        assert!(!lib.rename_animation("run", "walk"));
        assert!(!lib.rename_animation("run", "run"));

        assert!(lib.rename_animation("run", "sprint"));
        assert!(!lib.has_animation("run"));
        assert!(lib.has_animation("sprint"));
        assert_eq!(lib.get_animation("sprint").unwrap().get_length(), 1.0);
        assert_eq!(lib.get_animation_list_size(), 2);
    }

    #[test]
    fn remove_reports_missing_animations() {
        let mut lib = AnimationLibrary::new();
        lib.add_animation("run", anim(1.0));
        assert!(lib.remove_animation("run"));
        assert!(!lib.remove_animation("run"));
        assert_eq!(lib.get_animation_list_size(), 0);
        assert!(lib.get_animation("run").is_none());
    }
}
