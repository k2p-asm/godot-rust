//! `scene/2d/physics/ray_cast_2d.*` and `scene/3d/physics/ray_cast_3d.*`
//! (state only).
//!
//! Defaults: enabled, mask 1, excluding the parent body, areas off and bodies on,
//! `hit_from_inside` off, and no collision recorded. The 2D cast starts pointing
//! at `(0, 50)`; the 3D cast points down `(0, -1, 0)` and also carries the debug
//! shape options (`thin_line` on, 1 px thickness, no custom colour).
//!
//! Ported: every setting, the `1..=32` collision-mask bit helpers with the
//! engine's layer rules, exception lists and the collision result fields, with a
//! `set_collision` helper standing in for the physics query.
//!
//! The actual space query (`force_raycast_update`) is not ported; callers fill
//! the result through `set_collision`, and `clear_collision` models a missed cast.

use crate::math::color::Color;
use crate::math::vector2::Vector2;
use crate::math::vector3::Vector3;
use crate::servers::rid::Rid;

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
    (mask & (1u32 << (layer - 1))) != 0
}

// -- RayCast2D --------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct RayCast2DData {
    enabled: bool,
    collided: bool,
    collider: Option<Rid>,
    collider_shape: i32,
    collision_point: Vector2,
    collision_normal: Vector2,
    exclude: Vec<Rid>,
    collision_mask: u32,
    exclude_parent_body: bool,
    target_position: Vector2,
    collide_with_areas: bool,
    collide_with_bodies: bool,
    hit_from_inside: bool,
}

impl Default for RayCast2DData {
    fn default() -> Self {
        Self::new()
    }
}

impl RayCast2DData {
    pub fn new() -> Self {
        Self {
            enabled: true,
            collided: false,
            collider: None,
            collider_shape: -1,
            collision_point: Vector2::ZERO,
            collision_normal: Vector2::ZERO,
            exclude: Vec::new(),
            collision_mask: 1,
            exclude_parent_body: true,
            target_position: Vector2::new(0.0, 50.0),
            collide_with_areas: false,
            collide_with_bodies: true,
            hit_from_inside: false,
        }
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        if !enabled {
            self.collided = false;
        }
        self.enabled = enabled;
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
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

    pub fn add_exception(&mut self, node: Rid) -> bool {
        if self.exclude.contains(&node) {
            return false;
        }
        self.exclude.push(node);
        true
    }

    pub fn remove_exception(&mut self, node: Rid) -> bool {
        match self.exclude.iter().position(|rid| *rid == node) {
            Some(index) => {
                self.exclude.remove(index);
                true
            }
            None => false,
        }
    }

    pub fn clear_exceptions(&mut self) {
        self.exclude.clear();
    }

    pub fn get_exception_count(&self) -> usize {
        self.exclude.len()
    }

    pub fn set_exclude_parent_body(&mut self, exclude: bool) {
        self.exclude_parent_body = exclude;
    }

    pub fn get_exclude_parent_body(&self) -> bool {
        self.exclude_parent_body
    }

    pub fn set_target_position(&mut self, position: Vector2) {
        self.target_position = position;
    }

    pub fn get_target_position(&self) -> Vector2 {
        self.target_position
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

    pub fn set_hit_from_inside(&mut self, enabled: bool) {
        self.hit_from_inside = enabled;
    }

    pub fn is_hit_from_inside_enabled(&self) -> bool {
        self.hit_from_inside
    }

    /// Records a hit (the engine's `_update_raycast_state` result).
    pub fn set_collision(
        &mut self,
        collider: Rid,
        shape: i32,
        point: Vector2,
        normal: Vector2,
    ) {
        self.collided = true;
        self.collider = Some(collider);
        self.collider_shape = shape;
        self.collision_point = point;
        self.collision_normal = normal;
    }

    pub fn clear_collision(&mut self) {
        self.collided = false;
        self.collider = None;
        self.collider_shape = -1;
        self.collision_point = Vector2::ZERO;
        self.collision_normal = Vector2::ZERO;
    }

    pub fn is_colliding(&self) -> bool {
        self.collided
    }

    pub fn get_collider(&self) -> Option<Rid> {
        self.collider
    }

    pub fn get_collider_shape(&self) -> i32 {
        if self.collided {
            self.collider_shape
        } else {
            0
        }
    }

    pub fn get_collision_point(&self) -> Vector2 {
        if self.collided {
            self.collision_point
        } else {
            Vector2::ZERO
        }
    }

    pub fn get_collision_normal(&self) -> Vector2 {
        if self.collided {
            self.collision_normal
        } else {
            Vector2::ZERO
        }
    }
}

// -- RayCast3D --------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct RayCast3DData {
    enabled: bool,
    collided: bool,
    collider: Option<Rid>,
    collider_shape: i32,
    collision_point: Vector3,
    collision_normal: Vector3,
    collision_face_index: i32,
    exclude: Vec<Rid>,
    collision_mask: u32,
    exclude_parent_body: bool,
    target_position: Vector3,
    collide_with_areas: bool,
    collide_with_bodies: bool,
    hit_from_inside: bool,
    debug_shape_thin_line: bool,
    debug_shape_custom_color: Color,
    debug_shape_thickness: f32,
}

impl Default for RayCast3DData {
    fn default() -> Self {
        Self::new()
    }
}

impl RayCast3DData {
    pub fn new() -> Self {
        Self {
            enabled: true,
            collided: false,
            collider: None,
            collider_shape: -1,
            collision_point: Vector3::ZERO,
            collision_normal: Vector3::ZERO,
            collision_face_index: -1,
            exclude: Vec::new(),
            collision_mask: 1,
            exclude_parent_body: true,
            target_position: Vector3::new(0.0, -1.0, 0.0),
            collide_with_areas: false,
            collide_with_bodies: true,
            hit_from_inside: false,
            debug_shape_thin_line: true,
            debug_shape_custom_color: Color::new(0.0, 0.0, 0.0, 0.0),
            debug_shape_thickness: 1.0,
        }
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        if !enabled {
            self.collided = false;
        }
        self.enabled = enabled;
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
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

    pub fn add_exception(&mut self, node: Rid) -> bool {
        if self.exclude.contains(&node) {
            return false;
        }
        self.exclude.push(node);
        true
    }

    pub fn remove_exception(&mut self, node: Rid) -> bool {
        match self.exclude.iter().position(|rid| *rid == node) {
            Some(index) => {
                self.exclude.remove(index);
                true
            }
            None => false,
        }
    }

    pub fn clear_exceptions(&mut self) {
        self.exclude.clear();
    }

    pub fn get_exception_count(&self) -> usize {
        self.exclude.len()
    }

    pub fn set_exclude_parent_body(&mut self, exclude: bool) {
        self.exclude_parent_body = exclude;
    }

    pub fn get_exclude_parent_body(&self) -> bool {
        self.exclude_parent_body
    }

    pub fn set_target_position(&mut self, position: Vector3) {
        self.target_position = position;
    }

    pub fn get_target_position(&self) -> Vector3 {
        self.target_position
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

    pub fn set_hit_from_inside(&mut self, enabled: bool) {
        self.hit_from_inside = enabled;
    }

    pub fn is_hit_from_inside_enabled(&self) -> bool {
        self.hit_from_inside
    }

    pub fn set_debug_shape_thickness(&mut self, thickness: f32) {
        self.debug_shape_thickness = thickness;
    }

    pub fn get_debug_shape_thickness(&self) -> f32 {
        self.debug_shape_thickness
    }

    pub fn set_debug_shape_custom_color(&mut self, color: Color) {
        self.debug_shape_custom_color = color;
    }

    pub fn get_debug_shape_custom_color(&self) -> Color {
        self.debug_shape_custom_color
    }

    pub fn is_debug_shape_custom_color_enabled(&self) -> bool {
        self.debug_shape_custom_color.a > 0.0
    }

    pub fn set_debug_shape_thin_line(&mut self, enabled: bool) {
        self.debug_shape_thin_line = enabled;
    }

    pub fn is_debug_shape_thin_line_enabled(&self) -> bool {
        self.debug_shape_thin_line
    }

    /// Records a hit (the engine's `_update_raycast_state` result).
    #[allow(clippy::too_many_arguments)]
    pub fn set_collision(
        &mut self,
        collider: Rid,
        shape: i32,
        point: Vector3,
        normal: Vector3,
        face_index: i32,
    ) {
        self.collided = true;
        self.collider = Some(collider);
        self.collider_shape = shape;
        self.collision_point = point;
        self.collision_normal = normal;
        self.collision_face_index = face_index;
    }

    pub fn clear_collision(&mut self) {
        self.collided = false;
        self.collider = None;
        self.collider_shape = -1;
        self.collision_point = Vector3::ZERO;
        self.collision_normal = Vector3::ZERO;
        self.collision_face_index = -1;
    }

    pub fn is_colliding(&self) -> bool {
        self.collided
    }

    pub fn get_collider(&self) -> Option<Rid> {
        self.collider
    }

    pub fn get_collider_shape(&self) -> i32 {
        if self.collided {
            self.collider_shape
        } else {
            0
        }
    }

    pub fn get_collision_point(&self) -> Vector3 {
        if self.collided {
            self.collision_point
        } else {
            Vector3::ZERO
        }
    }

    pub fn get_collision_normal(&self) -> Vector3 {
        if self.collided {
            self.collision_normal
        } else {
            Vector3::ZERO
        }
    }

    pub fn get_collision_face_index(&self) -> i32 {
        if self.collided {
            self.collision_face_index
        } else {
            -1
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ray_cast_2d_defaults() {
        let mut r = RayCast2DData::new();
        assert!(r.is_enabled());
        assert!(!r.is_colliding());
        assert_eq!(r.get_collision_mask(), 1);
        assert!(r.get_exclude_parent_body());
        assert_eq!(r.get_target_position(), Vector2::new(0.0, 50.0));
        assert!(!r.is_collide_with_areas_enabled());
        assert!(r.is_collide_with_bodies_enabled());
        assert!(!r.is_hit_from_inside_enabled());
        assert_eq!(r.get_exception_count(), 0);

        assert!(r.set_collision_mask_value(3, true));
        assert_eq!(r.get_collision_mask(), 1 | 4);
        assert!(r.get_collision_mask_value(3));
        assert!(!r.set_collision_mask_value(0, true));
        assert!(!r.set_collision_mask_value(33, true));
        assert!(!r.get_collision_mask_value(33));

        r.set_target_position(Vector2::new(10.0, 0.0));
        r.set_collide_with_areas(true);
        r.set_collide_with_bodies(false);
        r.set_hit_from_inside(true);
        assert_eq!(r.get_target_position(), Vector2::new(10.0, 0.0));
        assert!(r.is_collide_with_areas_enabled());
        assert!(!r.is_collide_with_bodies_enabled());
        assert!(r.is_hit_from_inside_enabled());
    }

    #[test]
    fn ray_cast_2d_collision_and_exceptions() {
        let mut r = RayCast2DData::new();
        assert_eq!(r.get_collider_shape(), 0);
        assert_eq!(r.get_collision_point(), Vector2::ZERO);

        r.set_collision(
            Rid::from_u64(7),
            2,
            Vector2::new(1.0, 2.0),
            Vector2::UP,
        );
        assert!(r.is_colliding());
        assert_eq!(r.get_collider(), Some(Rid::from_u64(7)));
        assert_eq!(r.get_collider_shape(), 2);
        assert_eq!(r.get_collision_point(), Vector2::new(1.0, 2.0));
        assert_eq!(r.get_collision_normal(), Vector2::UP);

        // Disabling the cast clears the hit.
        r.set_enabled(false);
        assert!(!r.is_colliding());
        assert!(!r.is_enabled());
        r.set_enabled(true);

        r.clear_collision();
        assert!(!r.is_colliding());
        assert_eq!(r.get_collider(), None);

        assert!(r.add_exception(Rid::from_u64(3)));
        assert!(!r.add_exception(Rid::from_u64(3)));
        assert_eq!(r.get_exception_count(), 1);
        assert!(r.remove_exception(Rid::from_u64(3)));
        assert!(!r.remove_exception(Rid::from_u64(3)));
        r.add_exception(Rid::from_u64(4));
        r.clear_exceptions();
        assert_eq!(r.get_exception_count(), 0);

        r.set_exclude_parent_body(false);
        assert!(!r.get_exclude_parent_body());
    }

    #[test]
    fn ray_cast_3d_defaults_and_collision() {
        let mut r = RayCast3DData::new();
        assert!(r.is_enabled());
        assert_eq!(r.get_collision_mask(), 1);
        assert!(r.get_exclude_parent_body());
        assert_eq!(r.get_target_position(), Vector3::new(0.0, -1.0, 0.0));
        assert!(!r.is_collide_with_areas_enabled());
        assert!(r.is_collide_with_bodies_enabled());
        assert!(!r.is_hit_from_inside_enabled());
        assert!(r.is_debug_shape_thin_line_enabled());
        assert_eq!(r.get_debug_shape_thickness(), 1.0);
        assert!(!r.is_debug_shape_custom_color_enabled());

        r.set_collision(
            Rid::from_u64(5),
            1,
            Vector3::new(0.0, -1.0, 0.0),
            Vector3::UP,
            12,
        );
        assert!(r.is_colliding());
        assert_eq!(r.get_collider(), Some(Rid::from_u64(5)));
        assert_eq!(r.get_collider_shape(), 1);
        assert_eq!(r.get_collision_point(), Vector3::new(0.0, -1.0, 0.0));
        assert_eq!(r.get_collision_normal(), Vector3::UP);
        assert_eq!(r.get_collision_face_index(), 12);

        r.clear_collision();
        assert!(!r.is_colliding());
        assert_eq!(r.get_collision_face_index(), -1);
        assert_eq!(r.get_collider_shape(), 0);

        r.set_debug_shape_custom_color(Color::new(1.0, 0.0, 0.0, 1.0));
        assert!(r.is_debug_shape_custom_color_enabled());
        r.set_debug_shape_thickness(2.0);
        r.set_debug_shape_thin_line(false);
        assert_eq!(r.get_debug_shape_thickness(), 2.0);
        assert!(!r.is_debug_shape_thin_line_enabled());

        assert!(r.set_collision_mask_value(5, true));
        assert_eq!(r.get_collision_mask(), 1 | 16);
        assert!(r.get_collision_mask_value(5));
        assert!(r.add_exception(Rid::from_u64(2)));
        assert_eq!(r.get_exception_count(), 1);
        assert!(r.remove_exception(Rid::from_u64(2)));
    }
}
