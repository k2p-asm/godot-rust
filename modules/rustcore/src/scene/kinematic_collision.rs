//! `scene/2d/physics/kinematic_collision_2d.*` and
//! `scene/3d/physics/kinematic_collision_3d.*` (state only).
//!
//! A collision record from `move_and_slide`: the local shape, the collider (RID +
//! object id), the collider shape and its index, the collision position/normal,
//! the travel and remainder, the depth, the collider's velocity (and angular
//! velocity in 3D) and `get_angle(up)`, which is the angle between the normal and
//! the up direction.
//!
//! The 3D variant can hold several collisions per slide; the port stores a list
//! and exposes the same index-based accessors, returning zero values for bad
//! indices like the engine.
//!
//! Creating these records is the job of `move_and_slide`, which is not ported.

use crate::math::vector2::Vector2;
use crate::math::vector3::Vector3;
use crate::servers::rid::Rid;

#[derive(Clone, Debug, Default)]
pub struct KinematicCollision2DData {
    pub position: Vector2,
    pub normal: Vector2,
    pub travel: Vector2,
    pub remainder: Vector2,
    pub depth: f32,
    pub local_shape: Option<Rid>,
    pub collider: Option<Rid>,
    pub collider_id: u64,
    pub collider_shape: Option<Rid>,
    pub collider_shape_index: i32,
    pub collider_velocity: Vector2,
}

impl KinematicCollision2DData {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_position(&self) -> Vector2 {
        self.position
    }

    pub fn get_normal(&self) -> Vector2 {
        self.normal
    }

    pub fn get_travel(&self) -> Vector2 {
        self.travel
    }

    pub fn get_remainder(&self) -> Vector2 {
        self.remainder
    }

    /// `get_angle`: the angle between the normal and `up_direction`.
    pub fn get_angle(&self, up_direction: Vector2) -> f32 {
        self.normal.angle_to(up_direction)
    }

    pub fn get_depth(&self) -> f32 {
        self.depth
    }

    pub fn get_local_shape(&self) -> Option<Rid> {
        self.local_shape
    }

    pub fn get_collider(&self) -> Option<Rid> {
        self.collider
    }

    pub fn get_collider_id(&self) -> u64 {
        self.collider_id
    }

    pub fn get_collider_rid(&self) -> Option<Rid> {
        self.collider
    }

    pub fn get_collider_shape(&self) -> Option<Rid> {
        self.collider_shape
    }

    pub fn get_collider_shape_index(&self) -> i32 {
        self.collider_shape_index
    }

    pub fn get_collider_velocity(&self) -> Vector2 {
        self.collider_velocity
    }
}

#[derive(Clone, Debug, Default)]
pub struct KinematicCollision3DData {
    pub collisions: Vec<KinematicCollision3DEntry>,
    pub travel: Vector3,
    pub remainder: Vector3,
}

#[derive(Clone, Debug, Default)]
pub struct KinematicCollision3DEntry {
    pub position: Vector3,
    pub normal: Vector3,
    pub depth: f32,
    pub local_shape: Option<Rid>,
    pub collider: Option<Rid>,
    pub collider_id: u64,
    pub collider_shape: Option<Rid>,
    pub collider_shape_index: i32,
    pub collider_velocity: Vector3,
    pub collider_angular_velocity: Vector3,
}

impl KinematicCollision3DData {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_travel(&self) -> Vector3 {
        self.travel
    }

    pub fn get_remainder(&self) -> Vector3 {
        self.remainder
    }

    pub fn get_collision_count(&self) -> usize {
        self.collisions.len()
    }

    pub fn add_collision(&mut self, entry: KinematicCollision3DEntry) -> usize {
        self.collisions.push(entry);
        self.collisions.len() - 1
    }

    fn entry(&self, index: usize) -> Option<&KinematicCollision3DEntry> {
        self.collisions.get(index)
    }

    pub fn get_position(&self, index: usize) -> Vector3 {
        self.entry(index).map(|e| e.position).unwrap_or(Vector3::ZERO)
    }

    pub fn get_normal(&self, index: usize) -> Vector3 {
        self.entry(index).map(|e| e.normal).unwrap_or(Vector3::ZERO)
    }

    /// `get_angle`: the angle between the entry's normal and `up_direction`.
    pub fn get_angle(&self, index: usize, up_direction: Vector3) -> f32 {
        self.entry(index)
            .map(|e| e.normal.angle_to(up_direction))
            .unwrap_or(0.0)
    }

    pub fn get_local_shape(&self, index: usize) -> Option<Rid> {
        self.entry(index).and_then(|e| e.local_shape)
    }

    pub fn get_collider(&self, index: usize) -> Option<Rid> {
        self.entry(index).and_then(|e| e.collider)
    }

    pub fn get_collider_id(&self, index: usize) -> u64 {
        self.entry(index).map(|e| e.collider_id).unwrap_or(0)
    }

    pub fn get_collider_rid(&self, index: usize) -> Option<Rid> {
        self.entry(index).and_then(|e| e.collider)
    }

    pub fn get_collider_shape(&self, index: usize) -> Option<Rid> {
        self.entry(index).and_then(|e| e.collider_shape)
    }

    pub fn get_collider_shape_index(&self, index: usize) -> i32 {
        self.entry(index).map(|e| e.collider_shape_index).unwrap_or(0)
    }

    pub fn get_collider_velocity(&self, index: usize) -> Vector3 {
        self.entry(index)
            .map(|e| e.collider_velocity)
            .unwrap_or(Vector3::ZERO)
    }

    pub fn get_collider_angular_velocity(&self, index: usize) -> Vector3 {
        self.entry(index)
            .map(|e| e.collider_angular_velocity)
            .unwrap_or(Vector3::ZERO)
    }

    pub fn get_depth(&self, index: usize) -> f32 {
        self.entry(index).map(|e| e.depth).unwrap_or(0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collision_2d_fields() {
        let mut c = KinematicCollision2DData::new();
        assert_eq!(c.get_position(), Vector2::ZERO);
        assert_eq!(c.get_travel(), Vector2::ZERO);
        assert_eq!(c.get_depth(), 0.0);
        assert!(c.get_collider().is_none());
        assert_eq!(c.get_collider_shape_index(), 0);

        c.position = Vector2::new(1.0, 2.0);
        c.normal = Vector2::UP;
        c.travel = Vector2::new(0.0, -1.0);
        c.remainder = Vector2::new(0.0, -0.5);
        c.depth = 0.25;
        c.local_shape = Some(Rid::from_u64(1));
        c.collider = Some(Rid::from_u64(2));
        c.collider_id = 42;
        c.collider_shape = Some(Rid::from_u64(3));
        c.collider_shape_index = 1;
        c.collider_velocity = Vector2::new(0.0, 1.0);

        assert_eq!(c.get_position(), Vector2::new(1.0, 2.0));
        assert_eq!(c.get_normal(), Vector2::UP);
        assert_eq!(c.get_travel(), Vector2::new(0.0, -1.0));
        assert_eq!(c.get_remainder(), Vector2::new(0.0, -0.5));
        assert_eq!(c.get_depth(), 0.25);
        assert_eq!(c.get_local_shape(), Some(Rid::from_u64(1)));
        assert_eq!(c.get_collider(), Some(Rid::from_u64(2)));
        assert_eq!(c.get_collider_id(), 42);
        assert_eq!(c.get_collider_rid(), Some(Rid::from_u64(2)));
        assert_eq!(c.get_collider_shape(), Some(Rid::from_u64(3)));
        assert_eq!(c.get_collider_shape_index(), 1);
        assert_eq!(c.get_collider_velocity(), Vector2::new(0.0, 1.0));

        // A flat floor has a 0 angle to up.
        assert!(c.get_angle(Vector2::UP).abs() < 1e-6);
    }

    #[test]
    fn collision_3d_entries() {
        let mut c = KinematicCollision3DData::new();
        assert_eq!(c.get_collision_count(), 0);
        assert_eq!(c.get_position(0), Vector3::ZERO);
        assert_eq!(c.get_collider_velocity(0), Vector3::ZERO);
        assert_eq!(c.get_collider_angular_velocity(5), Vector3::ZERO);
        assert_eq!(c.get_depth(0), 0.0);
        assert_eq!(c.get_angle(0, Vector3::UP), 0.0);
        assert_eq!(c.get_collider_id(0), 0);

        c.travel = Vector3::new(0.0, -2.0, 0.0);
        c.remainder = Vector3::new(0.0, -0.5, 0.0);
        let index = c.add_collision(KinematicCollision3DEntry {
            position: Vector3::new(0.0, 1.0, 0.0),
            normal: Vector3::UP,
            depth: 0.1,
            local_shape: Some(Rid::from_u64(4)),
            collider: Some(Rid::from_u64(5)),
            collider_id: 9,
            collider_shape: Some(Rid::from_u64(6)),
            collider_shape_index: 2,
            collider_velocity: Vector3::new(1.0, 0.0, 0.0),
            collider_angular_velocity: Vector3::new(0.0, 0.0, 1.0),
        });
        assert_eq!(index, 0);
        assert_eq!(c.get_collision_count(), 1);
        assert_eq!(c.get_travel(), Vector3::new(0.0, -2.0, 0.0));
        assert_eq!(c.get_remainder(), Vector3::new(0.0, -0.5, 0.0));
        assert_eq!(c.get_position(0), Vector3::new(0.0, 1.0, 0.0));
        assert_eq!(c.get_normal(0), Vector3::UP);
        assert_eq!(c.get_collider(0), Some(Rid::from_u64(5)));
        assert_eq!(c.get_collider_id(0), 9);
        assert_eq!(c.get_collider_shape(0), Some(Rid::from_u64(6)));
        assert_eq!(c.get_collider_shape_index(0), 2);
        assert_eq!(c.get_collider_velocity(0), Vector3::new(1.0, 0.0, 0.0));
        assert_eq!(
            c.get_collider_angular_velocity(0),
            Vector3::new(0.0, 0.0, 1.0)
        );
        assert_eq!(c.get_depth(0), 0.1);
        assert!(c.get_angle(0, Vector3::UP).abs() < 1e-6);
    }
}
