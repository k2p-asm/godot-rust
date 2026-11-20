//! `scene/2d/bone_2d.*`: bone rest data (part of the Skeleton2D family).
//!
//! Defaults from the header: identity rest transform, automatic length/angle
//! calculation on, length 16, bone angle 0, no skeleton index (-1), an identity
//! cached transform and `copy_transform_to_cache` on.
//!
//! `set_rest` stores the transform; `set_length`/`set_bone_angle` are stored
//! unvalidated. `get_skeleton_rest` composes the parent chain in the engine; the
//! port returns the local rest (hierarchy composition is not modelled) and says
//! so.
//!
//! Skeleton registration, `apply_rest`'s node transform write and the editor
//! gizmo are not ported.

use crate::math::transform_2d::Transform2D;

#[derive(Clone, Debug)]
pub struct Bone2DData {
    rest: Transform2D,
    autocalculate_length_and_angle: bool,
    length: f32,
    bone_angle: f32,
    skeleton_index: i32,
    cache_transform: Transform2D,
    copy_transform_to_cache: bool,
}

impl Default for Bone2DData {
    fn default() -> Self {
        Self::new()
    }
}

impl Bone2DData {
    pub fn new() -> Self {
        Self {
            rest: Transform2D::IDENTITY,
            autocalculate_length_and_angle: true,
            length: 16.0,
            bone_angle: 0.0,
            skeleton_index: -1,
            cache_transform: Transform2D::IDENTITY,
            copy_transform_to_cache: true,
        }
    }

    pub fn set_rest(&mut self, rest: Transform2D) {
        self.rest = rest;
    }

    pub fn get_rest(&self) -> Transform2D {
        self.rest
    }

    /// `apply_rest` writes `rest` into the node transform; the port stores it in
    /// the cache like the engine's `cache_transform` path.
    pub fn apply_rest(&mut self) {
        if self.copy_transform_to_cache {
            self.cache_transform = self.rest;
        }
    }

    /// The skeleton-space rest. Without a modelled parent chain this is the local
    /// rest (documented deviation).
    pub fn get_skeleton_rest(&self) -> Transform2D {
        self.rest
    }

    pub fn set_autocalculate_length_and_angle(&mut self, autocalculate: bool) {
        self.autocalculate_length_and_angle = autocalculate;
    }

    pub fn get_autocalculate_length_and_angle(&self) -> bool {
        self.autocalculate_length_and_angle
    }

    pub fn set_length(&mut self, length: f32) {
        self.length = length;
    }

    pub fn get_length(&self) -> f32 {
        self.length
    }

    pub fn set_bone_angle(&mut self, angle: f32) {
        self.bone_angle = angle;
    }

    pub fn get_bone_angle(&self) -> f32 {
        self.bone_angle
    }

    pub fn get_index_in_skeleton(&self) -> i32 {
        self.skeleton_index
    }

    /// Set by `Skeleton2D` when the bone is registered.
    pub fn set_index_in_skeleton(&mut self, index: i32) {
        self.skeleton_index = index;
    }

    pub fn get_cache_transform(&self) -> Transform2D {
        self.cache_transform
    }

    pub fn set_copy_transform_to_cache(&mut self, copy: bool) {
        self.copy_transform_to_cache = copy;
    }

    pub fn is_copy_transform_to_cache_enabled(&self) -> bool {
        self.copy_transform_to_cache
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::vector2::Vector2;

    #[test]
    fn defaults() {
        let b = Bone2DData::new();
        assert_eq!(b.get_rest(), Transform2D::IDENTITY);
        assert!(b.get_autocalculate_length_and_angle());
        assert_eq!(b.get_length(), 16.0);
        assert_eq!(b.get_bone_angle(), 0.0);
        assert_eq!(b.get_index_in_skeleton(), -1);
        assert_eq!(b.get_cache_transform(), Transform2D::IDENTITY);
        assert!(b.is_copy_transform_to_cache_enabled());
    }

    #[test]
    fn rest_cache_and_parameters() {
        let mut b = Bone2DData::new();
        let rest = Transform2D {
            columns: [
                Vector2::new(0.0, 1.0),
                Vector2::new(-1.0, 0.0),
                Vector2::new(5.0, 6.0),
            ],
        };
        b.set_rest(rest);
        assert_eq!(b.get_rest(), rest);
        assert_eq!(b.get_cache_transform(), Transform2D::IDENTITY);
        b.apply_rest();
        assert_eq!(b.get_cache_transform(), rest);
        assert_eq!(b.get_skeleton_rest(), rest);

        // With caching disabled, apply_rest leaves the cache alone.
        b.set_copy_transform_to_cache(false);
        b.set_rest(Transform2D::IDENTITY);
        b.apply_rest();
        assert_eq!(b.get_cache_transform(), rest);

        b.set_autocalculate_length_and_angle(false);
        b.set_length(24.0);
        b.set_bone_angle(0.5);
        b.set_index_in_skeleton(3);
        assert!(!b.get_autocalculate_length_and_angle());
        assert_eq!(b.get_length(), 24.0);
        assert_eq!(b.get_bone_angle(), 0.5);
        assert_eq!(b.get_index_in_skeleton(), 3);
    }
}
