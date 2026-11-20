//! `scene/3d/physics/collision_object_3d.*` (state half) and its 2D sibling.
//!
//! Defaults: collision layer and mask 1, collision priority 1, disable mode
//! `REMOVE`, static body mode, ray pickable on and input-capture-on-drag off.
//!
//! Layer/mask bit helpers use the engine's `1..=32` range with its two error
//! messages ("Collision layer number must be between 1 and 32 inclusive."), and
//! reads of out-of-range layers report `false`.
//!
//! The physics server calls, shape owners, debug shapes, input picking and the
//! transform sync are not ported; shape owners are modelled as a simple table so
//! the node-side bookkeeping can be exercised.

use std::collections::HashMap;

use crate::math::transform_3d::Transform3D;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum DisableMode {
    Remove,
    MakeStatic,
    KeepActive,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum BodyMode {
    Static,
    Kinematic,
    Rigid,
    RigidLinear,
}

#[derive(Clone, Debug)]
pub struct ShapeOwner {
    pub transform: Transform3D,
    pub disabled: bool,
}

#[derive(Clone, Debug)]
pub struct CollisionObject3DData {
    collision_layer: u32,
    collision_mask: u32,
    collision_priority: f32,
    disable_mode: DisableMode,
    body_mode: BodyMode,
    area: bool,
    ray_pickable: bool,
    capture_input_on_drag: bool,
    shape_owners: HashMap<u32, ShapeOwner>,
    next_owner: u32,
}

impl Default for CollisionObject3DData {
    fn default() -> Self {
        Self::new(false)
    }
}

impl CollisionObject3DData {
    /// `CollisionObject3D(rid, area)`; the 2D object passes `area` too.
    pub fn new(area: bool) -> Self {
        Self {
            collision_layer: 1,
            collision_mask: 1,
            collision_priority: 1.0,
            disable_mode: DisableMode::Remove,
            body_mode: BodyMode::Static,
            area,
            ray_pickable: true,
            capture_input_on_drag: false,
            shape_owners: HashMap::new(),
            next_owner: 0,
        }
    }

    pub fn is_area(&self) -> bool {
        self.area
    }

    pub fn set_collision_layer(&mut self, layer: u32) {
        self.collision_layer = layer;
    }

    pub fn get_collision_layer(&self) -> u32 {
        self.collision_layer
    }

    pub fn set_collision_mask(&mut self, mask: u32) {
        self.collision_mask = mask;
    }

    pub fn get_collision_mask(&self) -> u32 {
        self.collision_mask
    }

    /// Layers are `1..=32`.
    pub fn set_collision_layer_value(&mut self, layer_number: i32, value: bool) -> bool {
        if !(1..=32).contains(&layer_number) {
            return false;
        }
        let bit = 1u32 << (layer_number - 1);
        if value {
            self.collision_layer |= bit;
        } else {
            self.collision_layer &= !bit;
        }
        true
    }

    /// Invalid layers read as `false`.
    pub fn get_collision_layer_value(&self, layer_number: i32) -> bool {
        if !(1..=32).contains(&layer_number) {
            return false;
        }
        (self.collision_layer & (1u32 << (layer_number - 1))) != 0
    }

    pub fn set_collision_mask_value(&mut self, layer_number: i32, value: bool) -> bool {
        if !(1..=32).contains(&layer_number) {
            return false;
        }
        let bit = 1u32 << (layer_number - 1);
        if value {
            self.collision_mask |= bit;
        } else {
            self.collision_mask &= !bit;
        }
        true
    }

    pub fn get_collision_mask_value(&self, layer_number: i32) -> bool {
        if !(1..=32).contains(&layer_number) {
            return false;
        }
        (self.collision_mask & (1u32 << (layer_number - 1))) != 0
    }

    pub fn set_collision_priority(&mut self, priority: f32) {
        self.collision_priority = priority;
    }

    pub fn get_collision_priority(&self) -> f32 {
        self.collision_priority
    }

    pub fn set_disable_mode(&mut self, mode: DisableMode) {
        self.disable_mode = mode;
    }

    pub fn get_disable_mode(&self) -> DisableMode {
        self.disable_mode
    }

    pub fn set_body_mode(&mut self, mode: BodyMode) {
        self.body_mode = mode;
    }

    pub fn get_body_mode(&self) -> BodyMode {
        self.body_mode
    }

    pub fn set_ray_pickable(&mut self, pickable: bool) {
        self.ray_pickable = pickable;
    }

    pub fn is_ray_pickable(&self) -> bool {
        self.ray_pickable
    }

    pub fn set_capture_input_on_drag(&mut self, capture: bool) {
        self.capture_input_on_drag = capture;
    }

    pub fn get_capture_input_on_drag(&self) -> bool {
        self.capture_input_on_drag
    }

    // -- shape owners --------------------------------------------------------

    /// `create_shape_owner`: the returned owner starts enabled and unshifted.
    pub fn create_shape_owner(&mut self) -> u32 {
        let owner = self.next_owner;
        self.next_owner += 1;
        self.shape_owners.insert(
            owner,
            ShapeOwner {
                transform: Transform3D::IDENTITY,
                disabled: false,
            },
        );
        owner
    }

    pub fn remove_shape_owner(&mut self, owner: u32) -> bool {
        self.shape_owners.remove(&owner).is_some()
    }

    pub fn get_shape_owners(&self) -> Vec<u32> {
        let mut owners: Vec<u32> = self.shape_owners.keys().copied().collect();
        owners.sort_unstable();
        owners
    }

    pub fn shape_owner_set_transform(&mut self, owner: u32, transform: Transform3D) -> bool {
        match self.shape_owners.get_mut(&owner) {
            Some(data) => {
                data.transform = transform;
                true
            }
            None => false,
        }
    }

    pub fn shape_owner_get_transform(&self, owner: u32) -> Option<Transform3D> {
        self.shape_owners.get(&owner).map(|data| data.transform)
    }

    pub fn shape_owner_set_disabled(&mut self, owner: u32, disabled: bool) -> bool {
        match self.shape_owners.get_mut(&owner) {
            Some(data) => {
                data.disabled = disabled;
                true
            }
            None => false,
        }
    }

    pub fn is_shape_owner_disabled(&self, owner: u32) -> bool {
        self.shape_owners
            .get(&owner)
            .map(|data| data.disabled)
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::vector3::Vector3;

    #[test]
    fn defaults() {
        let c = CollisionObject3DData::new(false);
        assert!(!c.is_area());
        assert_eq!(c.get_collision_layer(), 1);
        assert_eq!(c.get_collision_mask(), 1);
        assert_eq!(c.get_collision_priority(), 1.0);
        assert_eq!(c.get_disable_mode(), DisableMode::Remove);
        assert_eq!(c.get_body_mode(), BodyMode::Static);
        assert!(c.is_ray_pickable());
        assert!(!c.get_capture_input_on_drag());
        assert!(c.get_shape_owners().is_empty());

        assert!(CollisionObject3DData::default().get_shape_owners().is_empty());
        let area = CollisionObject3DData::new(true);
        assert!(area.is_area());
    }

    #[test]
    fn layer_and_mask_bits() {
        let mut c = CollisionObject3DData::new(false);
        c.set_collision_layer(0);
        assert!(c.set_collision_layer_value(1, true));
        assert!(c.set_collision_layer_value(32, true));
        assert_eq!(c.get_collision_layer(), 1 | (1 << 31));
        assert!(c.get_collision_layer_value(32));
        assert!(!c.get_collision_layer_value(2));

        assert!(!c.set_collision_layer_value(0, true));
        assert!(!c.set_collision_layer_value(33, true));
        assert!(!c.get_collision_layer_value(0));
        assert!(!c.get_collision_layer_value(33));

        c.set_collision_mask(0);
        assert!(c.set_collision_mask_value(3, true));
        assert_eq!(c.get_collision_mask(), 4);
        assert!(c.get_collision_mask_value(3));
        assert!(!c.set_collision_mask_value(32, true) == false);
        assert!(c.set_collision_mask_value(32, false));
    }

    #[test]
    fn disable_and_interaction_flags() {
        let mut c = CollisionObject3DData::new(false);
        c.set_disable_mode(DisableMode::KeepActive);
        c.set_body_mode(BodyMode::Rigid);
        c.set_collision_priority(2.5);
        c.set_ray_pickable(false);
        c.set_capture_input_on_drag(true);

        assert_eq!(c.get_disable_mode(), DisableMode::KeepActive);
        assert_eq!(c.get_body_mode(), BodyMode::Rigid);
        assert_eq!(c.get_collision_priority(), 2.5);
        assert!(!c.is_ray_pickable());
        assert!(c.get_capture_input_on_drag());

        assert_eq!(DisableMode::Remove as i32, 0);
        assert_eq!(DisableMode::MakeStatic as i32, 1);
        assert_eq!(DisableMode::KeepActive as i32, 2);
        assert_eq!(BodyMode::Static as i32, 0);
        assert_eq!(BodyMode::Kinematic as i32, 1);
        assert_eq!(BodyMode::Rigid as i32, 2);
        assert_eq!(BodyMode::RigidLinear as i32, 3);
    }

    #[test]
    fn shape_owner_bookkeeping() {
        let mut c = CollisionObject3DData::new(true);
        let first = c.create_shape_owner();
        let second = c.create_shape_owner();
        assert_eq!(c.get_shape_owners(), vec![first, second]);
        assert_eq!(c.shape_owner_get_transform(first), Some(Transform3D::IDENTITY));
        assert!(!c.is_shape_owner_disabled(first));

        let moved = Transform3D {
            basis: crate::math::basis::Basis::IDENTITY,
            origin: Vector3::new(1.0, 2.0, 3.0),
        };
        assert!(c.shape_owner_set_transform(first, moved));
        assert_eq!(c.shape_owner_get_transform(first), Some(moved));
        assert!(c.shape_owner_set_disabled(first, true));
        assert!(c.is_shape_owner_disabled(first));

        assert!(!c.shape_owner_set_transform(99, Transform3D::IDENTITY));
        assert!(!c.shape_owner_set_disabled(99, true));
        assert_eq!(c.shape_owner_get_transform(99), None);
        assert!(!c.is_shape_owner_disabled(99));

        assert!(c.remove_shape_owner(first));
        assert!(!c.remove_shape_owner(first));
        assert_eq!(c.get_shape_owners(), vec![second]);
    }
}
