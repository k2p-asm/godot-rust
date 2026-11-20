//! `scene/2d/physics/shape_cast_2d.*` and `scene/3d/physics/shape_cast_3d.*`
//! (state only).
//!
//! Both casts default to enabled, no shape, a target of `(0,50)` (2D) or
//! `(0,-1,0)` (3D), margin 0, collision mask 1, parent-body exclusion on, areas
//! off, bodies on and 32 max results; no hit is recorded and both safe/unsafe
//! fractions are 1.
//!
//! Ported: every setting, the `1..=32` collision-mask bit helpers with the
//! engine's layer rules, `set_max_results` clamping to at least 1, and the hit
//! list (the physics query is replaced by `set_results`).
//!
//! The space query, exclusions and debug drawing are not ported.

use crate::math::color::Color;
use crate::math::vector2::Vector2;
use crate::math::vector3::Vector3;
use crate::servers::rid::Rid;

#[derive(Clone, Debug, Default)]
pub struct ShapeCastHit2D {
    pub collider: Option<Rid>,
    pub collider_shape: i32,
    pub point: Vector2,
    pub normal: Vector2,
}

#[derive(Clone, Debug, Default)]
pub struct ShapeCastHit3D {
    pub collider: Option<Rid>,
    pub collider_shape: i32,
    pub point: Vector3,
    pub normal: Vector3,
    pub face_index: i32,
}

fn set_mask_bit(mask: &mut u32, layer: i32, value: bool) -> bool {
    if !(1..=32).contains(&layer) {
        return false;
    }
    let bit = 1u32 << (layer - 1);
    if value {
        *mask |= bit;
    } else {
        *mask &= !bit;
    }
    true
}

fn mask_bit(mask: u32, layer: i32) -> bool {
    if !(1..=32).contains(&layer) {
        return false;
    }
    mask & (1u32 << (layer - 1)) != 0
}

#[derive(Clone, Debug)]
pub struct ShapeCast2DData {
    enabled: bool,
    shape: Option<Rid>,
    target_position: Vector2,
    margin: f32,
    collision_mask: u32,
    exclude_parent_body: bool,
    collide_with_areas: bool,
    collide_with_bodies: bool,
    max_results: i32,
    collided: bool,
    collision_safe_fraction: f32,
    collision_unsafe_fraction: f32,
    results: Vec<ShapeCastHit2D>,
}

impl Default for ShapeCast2DData {
    fn default() -> Self {
        Self::new()
    }
}

impl ShapeCast2DData {
    pub fn new() -> Self {
        Self {
            enabled: true,
            shape: None,
            target_position: Vector2::new(0.0, 50.0),
            margin: 0.0,
            collision_mask: 1,
            exclude_parent_body: true,
            collide_with_areas: false,
            collide_with_bodies: true,
            max_results: 32,
            collided: false,
            collision_safe_fraction: 1.0,
            collision_unsafe_fraction: 1.0,
            results: Vec::new(),
        }
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        if !enabled {
            self.collided = false;
            self.results.clear();
        }
        self.enabled = enabled;
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn set_shape(&mut self, shape: Option<Rid>) {
        self.shape = shape;
    }

    pub fn get_shape(&self) -> Option<Rid> {
        self.shape
    }

    pub fn set_target_position(&mut self, position: Vector2) {
        self.target_position = position;
    }

    pub fn get_target_position(&self) -> Vector2 {
        self.target_position
    }

    pub fn set_margin(&mut self, margin: f32) {
        self.margin = margin;
    }

    pub fn get_margin(&self) -> f32 {
        self.margin
    }

    pub fn set_collision_mask(&mut self, mask: u32) {
        self.collision_mask = mask;
    }

    pub fn get_collision_mask(&self) -> u32 {
        self.collision_mask
    }

    pub fn set_collision_mask_value(&mut self, layer: i32, value: bool) -> bool {
        set_mask_bit(&mut self.collision_mask, layer, value)
    }

    pub fn get_collision_mask_value(&self, layer: i32) -> bool {
        mask_bit(self.collision_mask, layer)
    }

    pub fn set_exclude_parent_body(&mut self, exclude: bool) {
        self.exclude_parent_body = exclude;
    }

    pub fn get_exclude_parent_body(&self) -> bool {
        self.exclude_parent_body
    }

    pub fn set_collide_with_areas(&mut self, enabled: bool) {
        self.collide_with_areas = enabled;
    }

    pub fn is_collide_with_areas_enabled(&self) -> bool {
        self.collide_with_areas
    }

    pub fn set_collide_with_bodies(&mut self, enabled: bool) {
        self.collide_with_bodies = enabled;
    }

    pub fn is_collide_with_bodies_enabled(&self) -> bool {
        self.collide_with_bodies
    }

    /// `set_max_results`: the engine clamps to at least one.
    pub fn set_max_results(&mut self, max_results: i32) {
        self.max_results = max_results.max(1);
    }

    pub fn get_max_results(&self) -> i32 {
        self.max_results
    }

    /// The query result, in place of the space state.
    pub fn set_results(&mut self, results: Vec<ShapeCastHit2D>) {
        self.collided = !results.is_empty();
        self.results = results;
    }

    pub fn get_collision_count(&self) -> usize {
        self.results.len()
    }

    pub fn is_colliding(&self) -> bool {
        self.collided
    }

    pub fn get_collider(&self, index: usize) -> Option<Rid> {
        self.results.get(index).and_then(|hit| hit.collider)
    }

    pub fn get_collider_shape(&self, index: usize) -> i32 {
        self.results
            .get(index)
            .map(|hit| hit.collider_shape)
            .unwrap_or(0)
    }

    pub fn get_collision_point(&self, index: usize) -> Vector2 {
        self.results
            .get(index)
            .map(|hit| hit.point)
            .unwrap_or(Vector2::ZERO)
    }

    pub fn get_collision_normal(&self, index: usize) -> Vector2 {
        self.results
            .get(index)
            .map(|hit| hit.normal)
            .unwrap_or(Vector2::ZERO)
    }

    pub fn set_collision_safe_fraction(&mut self, fraction: f32) {
        self.collision_safe_fraction = fraction;
    }

    pub fn get_collision_safe_fraction(&self) -> f32 {
        self.collision_safe_fraction
    }

    pub fn set_collision_unsafe_fraction(&mut self, fraction: f32) {
        self.collision_unsafe_fraction = fraction;
    }

    pub fn get_collision_unsafe_fraction(&self) -> f32 {
        self.collision_unsafe_fraction
    }
}

#[derive(Clone, Debug)]
pub struct ShapeCast3DData {
    enabled: bool,
    shape: Option<Rid>,
    target_position: Vector3,
    margin: f32,
    collision_mask: u32,
    exclude_parent_body: bool,
    collide_with_areas: bool,
    collide_with_bodies: bool,
    max_results: i32,
    collided: bool,
    collision_safe_fraction: f32,
    collision_unsafe_fraction: f32,
    debug_shape_custom_color: Color,
    results: Vec<ShapeCastHit3D>,
}

impl Default for ShapeCast3DData {
    fn default() -> Self {
        Self::new()
    }
}

impl ShapeCast3DData {
    pub fn new() -> Self {
        Self {
            enabled: true,
            shape: None,
            target_position: Vector3::new(0.0, -1.0, 0.0),
            margin: 0.0,
            collision_mask: 1,
            exclude_parent_body: true,
            collide_with_areas: false,
            collide_with_bodies: true,
            max_results: 32,
            collided: false,
            collision_safe_fraction: 1.0,
            collision_unsafe_fraction: 1.0,
            debug_shape_custom_color: Color::new(0.0, 0.0, 0.0, 1.0),
            results: Vec::new(),
        }
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        if !enabled {
            self.collided = false;
            self.results.clear();
        }
        self.enabled = enabled;
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn set_shape(&mut self, shape: Option<Rid>) {
        self.shape = shape;
    }

    pub fn get_shape(&self) -> Option<Rid> {
        self.shape
    }

    pub fn set_target_position(&mut self, position: Vector3) {
        self.target_position = position;
    }

    pub fn get_target_position(&self) -> Vector3 {
        self.target_position
    }

    pub fn set_margin(&mut self, margin: f32) {
        self.margin = margin;
    }

    pub fn get_margin(&self) -> f32 {
        self.margin
    }

    pub fn set_collision_mask(&mut self, mask: u32) {
        self.collision_mask = mask;
    }

    pub fn get_collision_mask(&self) -> u32 {
        self.collision_mask
    }

    pub fn set_collision_mask_value(&mut self, layer: i32, value: bool) -> bool {
        set_mask_bit(&mut self.collision_mask, layer, value)
    }

    pub fn get_collision_mask_value(&self, layer: i32) -> bool {
        mask_bit(self.collision_mask, layer)
    }

    pub fn set_exclude_parent_body(&mut self, exclude: bool) {
        self.exclude_parent_body = exclude;
    }

    pub fn get_exclude_parent_body(&self) -> bool {
        self.exclude_parent_body
    }

    pub fn set_collide_with_areas(&mut self, enabled: bool) {
        self.collide_with_areas = enabled;
    }

    pub fn is_collide_with_areas_enabled(&self) -> bool {
        self.collide_with_areas
    }

    pub fn set_collide_with_bodies(&mut self, enabled: bool) {
        self.collide_with_bodies = enabled;
    }

    pub fn is_collide_with_bodies_enabled(&self) -> bool {
        self.collide_with_bodies
    }

    pub fn set_max_results(&mut self, max_results: i32) {
        self.max_results = max_results.max(1);
    }

    pub fn get_max_results(&self) -> i32 {
        self.max_results
    }

    pub fn set_debug_shape_custom_color(&mut self, color: Color) {
        self.debug_shape_custom_color = color;
    }

    pub fn get_debug_shape_custom_color(&self) -> Color {
        self.debug_shape_custom_color
    }

    pub fn set_results(&mut self, results: Vec<ShapeCastHit3D>) {
        self.collided = !results.is_empty();
        self.results = results;
    }

    pub fn get_collision_count(&self) -> usize {
        self.results.len()
    }

    pub fn is_colliding(&self) -> bool {
        self.collided
    }

    pub fn get_collider(&self, index: usize) -> Option<Rid> {
        self.results.get(index).and_then(|hit| hit.collider)
    }

    pub fn get_collider_shape(&self, index: usize) -> i32 {
        self.results
            .get(index)
            .map(|hit| hit.collider_shape)
            .unwrap_or(0)
    }

    pub fn get_collision_point(&self, index: usize) -> Vector3 {
        self.results
            .get(index)
            .map(|hit| hit.point)
            .unwrap_or(Vector3::ZERO)
    }

    pub fn get_collision_normal(&self, index: usize) -> Vector3 {
        self.results
            .get(index)
            .map(|hit| hit.normal)
            .unwrap_or(Vector3::ZERO)
    }

    pub fn get_collision_face_index(&self, index: usize) -> i32 {
        self.results
            .get(index)
            .map(|hit| hit.face_index)
            .unwrap_or(-1)
    }

    pub fn set_collision_safe_fraction(&mut self, fraction: f32) {
        self.collision_safe_fraction = fraction;
    }

    pub fn get_collision_safe_fraction(&self) -> f32 {
        self.collision_safe_fraction
    }

    pub fn set_collision_unsafe_fraction(&mut self, fraction: f32) {
        self.collision_unsafe_fraction = fraction;
    }

    pub fn get_collision_unsafe_fraction(&self) -> f32 {
        self.collision_unsafe_fraction
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cast_2d_defaults_and_mask() {
        let mut cast = ShapeCast2DData::new();
        assert!(cast.is_enabled());
        assert!(cast.get_shape().is_none());
        assert_eq!(cast.get_target_position(), Vector2::new(0.0, 50.0));
        assert_eq!(cast.get_margin(), 0.0);
        assert_eq!(cast.get_collision_mask(), 1);
        assert!(cast.get_exclude_parent_body());
        assert!(!cast.is_collide_with_areas_enabled());
        assert!(cast.is_collide_with_bodies_enabled());
        assert_eq!(cast.get_max_results(), 32);
        assert!(!cast.is_colliding());
        assert_eq!(cast.get_collision_safe_fraction(), 1.0);
        assert_eq!(cast.get_collision_unsafe_fraction(), 1.0);

        assert!(cast.set_collision_mask_value(3, true));
        assert_eq!(cast.get_collision_mask(), 1 | 4);
        assert!(cast.get_collision_mask_value(3));
        assert!(!cast.set_collision_mask_value(0, true));
        assert!(!cast.set_collision_mask_value(33, true));
        assert!(!cast.get_collision_mask_value(33));

        cast.set_max_results(0);
        assert_eq!(cast.get_max_results(), 1, "clamped to at least one");
        cast.set_max_results(4);
        assert_eq!(cast.get_max_results(), 4);

        cast.set_results(vec![ShapeCastHit2D {
            collider: Some(Rid::from_u64(2)),
            collider_shape: 1,
            point: Vector2::new(1.0, 2.0),
            normal: Vector2::UP,
        }]);
        assert!(cast.is_colliding());
        assert_eq!(cast.get_collision_count(), 1);
        assert_eq!(cast.get_collider(0), Some(Rid::from_u64(2)));
        assert_eq!(cast.get_collider_shape(0), 1);
        assert_eq!(cast.get_collision_point(0), Vector2::new(1.0, 2.0));
        assert_eq!(cast.get_collision_normal(0), Vector2::UP);
        cast.set_enabled(false);
        assert!(!cast.is_colliding());
        assert_eq!(cast.get_collision_count(), 0);
    }

    #[test]
    fn cast_3d_defaults_and_results() {
        let mut cast = ShapeCast3DData::new();
        assert_eq!(cast.get_target_position(), Vector3::new(0.0, -1.0, 0.0));
        assert_eq!(cast.get_debug_shape_custom_color(), Color::new(0.0, 0.0, 0.0, 1.0));
        assert!(cast.is_collide_with_bodies_enabled());
        assert!(!cast.is_collide_with_areas_enabled());
        assert_eq!(cast.get_max_results(), 32);
        assert!(!cast.is_colliding());

        cast.set_target_position(Vector3::new(1.0, 0.0, 0.0));
        cast.set_margin(0.25);
        cast.set_collision_mask_value(32, true);
        cast.set_collide_with_areas(true);
        cast.set_collide_with_bodies(false);
        cast.set_exclude_parent_body(false);
        cast.set_max_results(-3);
        cast.set_debug_shape_custom_color(Color::new(1.0, 0.0, 0.0, 1.0));
        assert_eq!(cast.get_target_position(), Vector3::new(1.0, 0.0, 0.0));
        assert_eq!(cast.get_margin(), 0.25);
        assert!(cast.get_collision_mask_value(32));
        assert!(cast.is_collide_with_areas_enabled());
        assert!(!cast.is_collide_with_bodies_enabled());
        assert!(!cast.get_exclude_parent_body());
        assert_eq!(cast.get_max_results(), 1);

        cast.set_results(vec![ShapeCastHit3D {
            collider: Some(Rid::from_u64(5)),
            collider_shape: 2,
            point: Vector3::new(0.0, -1.0, 0.0),
            normal: Vector3::UP,
            face_index: 7,
        }]);
        assert!(cast.is_colliding());
        assert_eq!(cast.get_collider(0), Some(Rid::from_u64(5)));
        assert_eq!(cast.get_collider_shape(0), 2);
        assert_eq!(cast.get_collision_point(0), Vector3::new(0.0, -1.0, 0.0));
        assert_eq!(cast.get_collision_normal(0), Vector3::UP);
        assert_eq!(cast.get_collision_face_index(0), 7);
        assert_eq!(cast.get_collision_face_index(4), -1);
    }
}
