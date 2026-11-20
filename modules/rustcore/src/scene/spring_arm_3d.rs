//! `scene/3d/physics/spring_arm_3d.*` (state half).
//!
//! Defaults: spring length 1, current (hit) length 0, collision mask 1, margin
//! 0.01, no shape and no excluded objects. `get_hit_length` returns the current
//! length the cast computed, exactly like the C++.
//!
//! The shape-cast query, the exclusion bookkeeping with the physics server and
//! the per-frame `process_spring` are not ported; `set_current_length` stands in
//! for the cast result.

use crate::servers::rid::Rid;

#[derive(Clone, Debug)]
pub struct SpringArm3DData {
    shape: Option<Rid>,
    excluded_objects: Vec<Rid>,
    spring_length: f32,
    current_spring_length: f32,
    mask: u32,
    margin: f32,
}

impl Default for SpringArm3DData {
    fn default() -> Self {
        Self::new()
    }
}

impl SpringArm3DData {
    pub fn new() -> Self {
        Self {
            shape: None,
            excluded_objects: Vec::new(),
            spring_length: 1.0,
            current_spring_length: 0.0,
            mask: 1,
            margin: 0.01,
        }
    }

    pub fn set_length(&mut self, length: f32) {
        self.spring_length = length;
    }

    pub fn get_length(&self) -> f32 {
        self.spring_length
    }

    pub fn set_shape(&mut self, shape: Option<Rid>) {
        self.shape = shape;
    }

    pub fn get_shape(&self) -> Option<Rid> {
        self.shape
    }

    pub fn set_mask(&mut self, mask: u32) {
        self.mask = mask;
    }

    pub fn get_mask(&self) -> u32 {
        self.mask
    }

    /// Adds an object the cast must ignore; returns false when it was present.
    pub fn add_excluded_object(&mut self, rid: Rid) -> bool {
        if self.excluded_objects.contains(&rid) {
            return false;
        }
        self.excluded_objects.push(rid);
        true
    }

    pub fn remove_excluded_object(&mut self, rid: Rid) -> bool {
        match self.excluded_objects.iter().position(|other| *other == rid) {
            Some(index) => {
                self.excluded_objects.remove(index);
                true
            }
            None => false,
        }
    }

    pub fn clear_excluded_objects(&mut self) {
        self.excluded_objects.clear();
    }

    pub fn get_excluded_object_count(&self) -> usize {
        self.excluded_objects.len()
    }

    /// `get_hit_length`: the length the last cast resolved to.
    pub fn get_hit_length(&self) -> f32 {
        self.current_spring_length
    }

    /// The cast result, in place of `process_spring`'s query.
    pub fn set_current_length(&mut self, length: f32) {
        self.current_spring_length = length;
    }

    pub fn set_margin(&mut self, margin: f32) {
        self.margin = margin;
    }

    pub fn get_margin(&self) -> f32 {
        self.margin
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults() {
        let s = SpringArm3DData::new();
        assert_eq!(s.get_length(), 1.0);
        assert_eq!(s.get_hit_length(), 0.0);
        assert_eq!(s.get_mask(), 1);
        assert_eq!(s.get_margin(), 0.01);
        assert!(s.get_shape().is_none());
        assert_eq!(s.get_excluded_object_count(), 0);
    }

    #[test]
    fn setters_and_exclusions() {
        let mut s = SpringArm3DData::new();
        s.set_length(4.5);
        s.set_mask(0b1010);
        s.set_margin(0.5);
        s.set_shape(Some(Rid::from_u64(2)));
        s.set_current_length(3.25);
        assert_eq!(s.get_length(), 4.5);
        assert_eq!(s.get_mask(), 0b1010);
        assert_eq!(s.get_margin(), 0.5);
        assert_eq!(s.get_shape(), Some(Rid::from_u64(2)));
        assert_eq!(s.get_hit_length(), 3.25);

        assert!(s.add_excluded_object(Rid::from_u64(1)));
        assert!(!s.add_excluded_object(Rid::from_u64(1)));
        assert_eq!(s.get_excluded_object_count(), 1);
        assert!(s.remove_excluded_object(Rid::from_u64(1)));
        assert!(!s.remove_excluded_object(Rid::from_u64(1)));
        s.add_excluded_object(Rid::from_u64(3));
        s.clear_excluded_objects();
        assert_eq!(s.get_excluded_object_count(), 0);
    }
}
