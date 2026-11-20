//! `scene/3d/spring_bone_simulator_3d.*` (state): the spring-bone simulator.
//!
//! Ported configuration: the external force, mutable bone axes, and per-setting
//! root/end bones with the virtual end joint (extension/direction/length), the
//! centre source (world origin/node/bone), the individual-config flag, the
//! default radius/stiffness/drag/gravity and their damping curves, the rotation
//! axis (all) and custom vector, the per-joint overrides (bone name/index, axis,
//! radius, stiffness, drag, gravity, direction), the child-collision toggle and
//! the explicit/excluded collision path lists.
//!
//! The Verlet integration, collision resolution and joint initialisation from a
//! real `Skeleton3D` are not ported.

use crate::math::vector3::Vector3;
use crate::servers::rid::Rid;
use crate::string::node_path::NodePath;

pub const CENTER_FROM_WORLD_ORIGIN: i32 = 0;
pub const CENTER_FROM_NODE: i32 = 1;
pub const CENTER_FROM_BONE: i32 = 2;

pub const ROTATION_AXIS_X: i32 = 0;
pub const ROTATION_AXIS_Y: i32 = 1;
pub const ROTATION_AXIS_Z: i32 = 2;
pub const ROTATION_AXIS_ALL: i32 = 3;
pub const ROTATION_AXIS_CUSTOM: i32 = 4;

pub const BONE_DIRECTION_FROM_PARENT: i32 = 6;

#[derive(Clone, Debug)]
pub struct SpringBoneJoint {
    pub bone_name: String,
    pub bone: i32,
    pub rotation_axis: i32,
    pub rotation_axis_vector: Vector3,
    pub radius: f32,
    pub stiffness: f32,
    pub drag: f32,
    pub gravity: f32,
    pub gravity_direction: Vector3,
}

impl Default for SpringBoneJoint {
    fn default() -> Self {
        Self {
            bone_name: String::new(),
            bone: -1,
            rotation_axis: ROTATION_AXIS_ALL,
            rotation_axis_vector: Vector3::new(1.0, 0.0, 0.0),
            radius: 0.1,
            stiffness: 1.0,
            drag: 0.0,
            gravity: 0.0,
            gravity_direction: Vector3::new(0.0, -1.0, 0.0),
        }
    }
}

#[derive(Clone, Debug)]
pub struct SpringBoneSetting {
    pub root_bone_name: String,
    pub root_bone: i32,
    pub end_bone_name: String,
    pub end_bone: i32,
    pub extend_end_bone: bool,
    pub end_bone_direction: i32,
    pub end_bone_length: f32,
    pub center_from: i32,
    pub center_node: NodePath,
    pub center_bone_name: String,
    pub center_bone: i32,
    pub individual_config: bool,
    pub radius: f32,
    pub radius_damping_curve: Option<Rid>,
    pub stiffness: f32,
    pub stiffness_damping_curve: Option<Rid>,
    pub drag: f32,
    pub drag_damping_curve: Option<Rid>,
    pub gravity: f32,
    pub gravity_damping_curve: Option<Rid>,
    pub gravity_direction: Vector3,
    pub rotation_axis: i32,
    pub rotation_axis_vector: Vector3,
    pub joints: Vec<SpringBoneJoint>,
    pub enable_all_child_collisions: bool,
    pub collisions: Vec<NodePath>,
    pub exclude_collisions: Vec<NodePath>,
}

impl Default for SpringBoneSetting {
    fn default() -> Self {
        Self {
            root_bone_name: String::new(),
            root_bone: -1,
            end_bone_name: String::new(),
            end_bone: -1,
            extend_end_bone: false,
            end_bone_direction: BONE_DIRECTION_FROM_PARENT,
            end_bone_length: 0.0,
            center_from: CENTER_FROM_WORLD_ORIGIN,
            center_node: NodePath::new(),
            center_bone_name: String::new(),
            center_bone: -1,
            individual_config: false,
            radius: 0.02,
            radius_damping_curve: None,
            stiffness: 1.0,
            stiffness_damping_curve: None,
            drag: 0.4,
            drag_damping_curve: None,
            gravity: 0.0,
            gravity_damping_curve: None,
            gravity_direction: Vector3::new(0.0, -1.0, 0.0),
            rotation_axis: ROTATION_AXIS_ALL,
            rotation_axis_vector: Vector3::new(1.0, 0.0, 0.0),
            joints: Vec::new(),
            enable_all_child_collisions: true,
            collisions: Vec::new(),
            exclude_collisions: Vec::new(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct SpringBoneSimulator3DData {
    external_force: Vector3,
    mutable_bone_axes: bool,
    settings: Vec<SpringBoneSetting>,
}

impl Default for SpringBoneSimulator3DData {
    fn default() -> Self {
        Self::new()
    }
}

impl SpringBoneSimulator3DData {
    pub fn new() -> Self {
        Self {
            external_force: Vector3::ZERO,
            mutable_bone_axes: true,
            settings: Vec::new(),
        }
    }

    pub fn set_external_force(&mut self, force: Vector3) {
        self.external_force = force;
    }

    pub fn get_external_force(&self) -> Vector3 {
        self.external_force
    }

    pub fn set_mutable_bone_axes(&mut self, enabled: bool) {
        self.mutable_bone_axes = enabled;
    }

    pub fn are_bone_axes_mutable(&self) -> bool {
        self.mutable_bone_axes
    }

    pub fn get_setting_count(&self) -> usize {
        self.settings.len()
    }

    pub fn set_setting_count(&mut self, count: usize) {
        self.settings.resize_with(count, SpringBoneSetting::default);
    }

    pub fn clear_settings(&mut self) {
        self.settings.clear();
    }

    pub fn get_setting(&self, index: usize) -> Option<&SpringBoneSetting> {
        self.settings.get(index)
    }

    pub fn set_root_bone_name(&mut self, index: usize, name: &str) {
        if let Some(setting) = self.settings.get_mut(index) {
            setting.root_bone_name = name.to_string();
        }
    }

    pub fn get_root_bone_name(&self, index: usize) -> &str {
        self.settings
            .get(index)
            .map(|setting| setting.root_bone_name.as_str())
            .unwrap_or("")
    }

    pub fn set_root_bone(&mut self, index: usize, bone: i32) {
        if let Some(setting) = self.settings.get_mut(index) {
            setting.root_bone = bone;
        }
    }

    pub fn get_root_bone(&self, index: usize) -> i32 {
        self.settings
            .get(index)
            .map(|setting| setting.root_bone)
            .unwrap_or(-1)
    }

    pub fn set_end_bone_name(&mut self, index: usize, name: &str) {
        if let Some(setting) = self.settings.get_mut(index) {
            setting.end_bone_name = name.to_string();
        }
    }

    pub fn get_end_bone_name(&self, index: usize) -> &str {
        self.settings
            .get(index)
            .map(|setting| setting.end_bone_name.as_str())
            .unwrap_or("")
    }

    pub fn set_end_bone(&mut self, index: usize, bone: i32) {
        if let Some(setting) = self.settings.get_mut(index) {
            setting.end_bone = bone;
        }
    }

    pub fn get_end_bone(&self, index: usize) -> i32 {
        self.settings
            .get(index)
            .map(|setting| setting.end_bone)
            .unwrap_or(-1)
    }

    pub fn set_extend_end_bone(&mut self, index: usize, enabled: bool) {
        if let Some(setting) = self.settings.get_mut(index) {
            setting.extend_end_bone = enabled;
        }
    }

    pub fn is_end_bone_extended(&self, index: usize) -> bool {
        self.settings
            .get(index)
            .map(|setting| setting.extend_end_bone)
            .unwrap_or(false)
    }

    pub fn set_end_bone_direction(&mut self, index: usize, direction: i32) {
        if let Some(setting) = self.settings.get_mut(index) {
            setting.end_bone_direction = direction;
        }
    }

    pub fn get_end_bone_direction(&self, index: usize) -> i32 {
        self.settings
            .get(index)
            .map(|setting| setting.end_bone_direction)
            .unwrap_or(BONE_DIRECTION_FROM_PARENT)
    }

    pub fn set_end_bone_length(&mut self, index: usize, length: f32) {
        if let Some(setting) = self.settings.get_mut(index) {
            setting.end_bone_length = length;
        }
    }

    pub fn get_end_bone_length(&self, index: usize) -> f32 {
        self.settings
            .get(index)
            .map(|setting| setting.end_bone_length)
            .unwrap_or(0.0)
    }

    pub fn set_center_from(&mut self, index: usize, center_from: i32) {
        if let Some(setting) = self.settings.get_mut(index) {
            setting.center_from = center_from;
        }
    }

    pub fn get_center_from(&self, index: usize) -> i32 {
        self.settings
            .get(index)
            .map(|setting| setting.center_from)
            .unwrap_or(CENTER_FROM_WORLD_ORIGIN)
    }

    pub fn set_center_node(&mut self, index: usize, path: NodePath) {
        if let Some(setting) = self.settings.get_mut(index) {
            setting.center_node = path;
        }
    }

    pub fn get_center_node(&self, index: usize) -> NodePath {
        self.settings
            .get(index)
            .map(|setting| setting.center_node.clone())
            .unwrap_or_default()
    }

    pub fn set_center_bone_name(&mut self, index: usize, name: &str) {
        if let Some(setting) = self.settings.get_mut(index) {
            setting.center_bone_name = name.to_string();
        }
    }

    pub fn get_center_bone_name(&self, index: usize) -> &str {
        self.settings
            .get(index)
            .map(|setting| setting.center_bone_name.as_str())
            .unwrap_or("")
    }

    pub fn set_center_bone(&mut self, index: usize, bone: i32) {
        if let Some(setting) = self.settings.get_mut(index) {
            setting.center_bone = bone;
        }
    }

    pub fn get_center_bone(&self, index: usize) -> i32 {
        self.settings
            .get(index)
            .map(|setting| setting.center_bone)
            .unwrap_or(-1)
    }

    pub fn set_individual_config(&mut self, index: usize, enabled: bool) {
        if let Some(setting) = self.settings.get_mut(index) {
            setting.individual_config = enabled;
        }
    }

    pub fn is_config_individual(&self, index: usize) -> bool {
        self.settings
            .get(index)
            .map(|setting| setting.individual_config)
            .unwrap_or(false)
    }

    pub fn set_radius(&mut self, index: usize, radius: f32) {
        if let Some(setting) = self.settings.get_mut(index) {
            setting.radius = radius;
        }
    }

    pub fn get_radius(&self, index: usize) -> f32 {
        self.settings
            .get(index)
            .map(|setting| setting.radius)
            .unwrap_or(0.02)
    }

    pub fn set_radius_damping_curve(&mut self, index: usize, curve: Option<Rid>) {
        if let Some(setting) = self.settings.get_mut(index) {
            setting.radius_damping_curve = curve;
        }
    }

    pub fn get_radius_damping_curve(&self, index: usize) -> Option<Rid> {
        self.settings
            .get(index)
            .and_then(|setting| setting.radius_damping_curve)
    }

    pub fn set_stiffness(&mut self, index: usize, stiffness: f32) {
        if let Some(setting) = self.settings.get_mut(index) {
            setting.stiffness = stiffness;
        }
    }

    pub fn get_stiffness(&self, index: usize) -> f32 {
        self.settings
            .get(index)
            .map(|setting| setting.stiffness)
            .unwrap_or(1.0)
    }

    pub fn set_stiffness_damping_curve(&mut self, index: usize, curve: Option<Rid>) {
        if let Some(setting) = self.settings.get_mut(index) {
            setting.stiffness_damping_curve = curve;
        }
    }

    pub fn get_stiffness_damping_curve(&self, index: usize) -> Option<Rid> {
        self.settings
            .get(index)
            .and_then(|setting| setting.stiffness_damping_curve)
    }

    pub fn set_drag(&mut self, index: usize, drag: f32) {
        if let Some(setting) = self.settings.get_mut(index) {
            setting.drag = drag;
        }
    }

    pub fn get_drag(&self, index: usize) -> f32 {
        self.settings
            .get(index)
            .map(|setting| setting.drag)
            .unwrap_or(0.4)
    }

    pub fn set_drag_damping_curve(&mut self, index: usize, curve: Option<Rid>) {
        if let Some(setting) = self.settings.get_mut(index) {
            setting.drag_damping_curve = curve;
        }
    }

    pub fn get_drag_damping_curve(&self, index: usize) -> Option<Rid> {
        self.settings
            .get(index)
            .and_then(|setting| setting.drag_damping_curve)
    }

    pub fn set_gravity(&mut self, index: usize, gravity: f32) {
        if let Some(setting) = self.settings.get_mut(index) {
            setting.gravity = gravity;
        }
    }

    pub fn get_gravity(&self, index: usize) -> f32 {
        self.settings
            .get(index)
            .map(|setting| setting.gravity)
            .unwrap_or(0.0)
    }

    pub fn set_gravity_damping_curve(&mut self, index: usize, curve: Option<Rid>) {
        if let Some(setting) = self.settings.get_mut(index) {
            setting.gravity_damping_curve = curve;
        }
    }

    pub fn get_gravity_damping_curve(&self, index: usize) -> Option<Rid> {
        self.settings
            .get(index)
            .and_then(|setting| setting.gravity_damping_curve)
    }

    pub fn set_gravity_direction(&mut self, index: usize, direction: Vector3) {
        if let Some(setting) = self.settings.get_mut(index) {
            setting.gravity_direction = direction;
        }
    }

    pub fn get_gravity_direction(&self, index: usize) -> Vector3 {
        self.settings
            .get(index)
            .map(|setting| setting.gravity_direction)
            .unwrap_or(Vector3::new(0.0, -1.0, 0.0))
    }

    pub fn set_rotation_axis(&mut self, index: usize, axis: i32) {
        if let Some(setting) = self.settings.get_mut(index) {
            setting.rotation_axis = axis;
        }
    }

    pub fn get_rotation_axis(&self, index: usize) -> i32 {
        self.settings
            .get(index)
            .map(|setting| setting.rotation_axis)
            .unwrap_or(ROTATION_AXIS_ALL)
    }

    pub fn set_rotation_axis_vector(&mut self, index: usize, vector: Vector3) {
        if let Some(setting) = self.settings.get_mut(index) {
            setting.rotation_axis_vector = vector;
        }
    }

    pub fn get_rotation_axis_vector(&self, index: usize) -> Vector3 {
        self.settings
            .get(index)
            .map(|setting| setting.rotation_axis_vector)
            .unwrap_or(Vector3::new(1.0, 0.0, 0.0))
    }

    pub fn set_joint_count(&mut self, index: usize, count: usize) {
        if let Some(setting) = self.settings.get_mut(index) {
            setting.joints.resize_with(count, SpringBoneJoint::default);
        }
    }

    pub fn get_joint_count(&self, index: usize) -> usize {
        self.settings
            .get(index)
            .map(|setting| setting.joints.len())
            .unwrap_or(0)
    }

    pub fn get_joint_bone_name(&self, index: usize, joint: usize) -> &str {
        self.settings
            .get(index)
            .and_then(|setting| setting.joints.get(joint))
            .map(|joint| joint.bone_name.as_str())
            .unwrap_or("")
    }

    pub fn get_joint_bone(&self, index: usize, joint: usize) -> i32 {
        self.settings
            .get(index)
            .and_then(|setting| setting.joints.get(joint))
            .map(|joint| joint.bone)
            .unwrap_or(-1)
    }

    pub fn set_joint_rotation_axis(&mut self, index: usize, joint: usize, axis: i32) {
        if let Some(setting) = self.settings.get_mut(index) {
            if let Some(entry) = setting.joints.get_mut(joint) {
                entry.rotation_axis = axis;
            }
        }
    }

    pub fn get_joint_rotation_axis(&self, index: usize, joint: usize) -> i32 {
        self.settings
            .get(index)
            .and_then(|setting| setting.joints.get(joint))
            .map(|joint| joint.rotation_axis)
            .unwrap_or(ROTATION_AXIS_ALL)
    }

    pub fn set_joint_rotation_axis_vector(&mut self, index: usize, joint: usize, vector: Vector3) {
        if let Some(setting) = self.settings.get_mut(index) {
            if let Some(entry) = setting.joints.get_mut(joint) {
                entry.rotation_axis_vector = vector;
            }
        }
    }

    pub fn get_joint_rotation_axis_vector(&self, index: usize, joint: usize) -> Vector3 {
        self.settings
            .get(index)
            .and_then(|setting| setting.joints.get(joint))
            .map(|joint| joint.rotation_axis_vector)
            .unwrap_or(Vector3::new(1.0, 0.0, 0.0))
    }

    pub fn set_joint_radius(&mut self, index: usize, joint: usize, radius: f32) {
        if let Some(setting) = self.settings.get_mut(index) {
            if let Some(entry) = setting.joints.get_mut(joint) {
                entry.radius = radius;
            }
        }
    }

    pub fn get_joint_radius(&self, index: usize, joint: usize) -> f32 {
        self.settings
            .get(index)
            .and_then(|setting| setting.joints.get(joint))
            .map(|joint| joint.radius)
            .unwrap_or(0.1)
    }

    pub fn set_joint_stiffness(&mut self, index: usize, joint: usize, stiffness: f32) {
        if let Some(setting) = self.settings.get_mut(index) {
            if let Some(entry) = setting.joints.get_mut(joint) {
                entry.stiffness = stiffness;
            }
        }
    }

    pub fn get_joint_stiffness(&self, index: usize, joint: usize) -> f32 {
        self.settings
            .get(index)
            .and_then(|setting| setting.joints.get(joint))
            .map(|joint| joint.stiffness)
            .unwrap_or(1.0)
    }

    pub fn set_joint_drag(&mut self, index: usize, joint: usize, drag: f32) {
        if let Some(setting) = self.settings.get_mut(index) {
            if let Some(entry) = setting.joints.get_mut(joint) {
                entry.drag = drag;
            }
        }
    }

    pub fn get_joint_drag(&self, index: usize, joint: usize) -> f32 {
        self.settings
            .get(index)
            .and_then(|setting| setting.joints.get(joint))
            .map(|joint| joint.drag)
            .unwrap_or(0.0)
    }

    pub fn set_joint_gravity(&mut self, index: usize, joint: usize, gravity: f32) {
        if let Some(setting) = self.settings.get_mut(index) {
            if let Some(entry) = setting.joints.get_mut(joint) {
                entry.gravity = gravity;
            }
        }
    }

    pub fn get_joint_gravity(&self, index: usize, joint: usize) -> f32 {
        self.settings
            .get(index)
            .and_then(|setting| setting.joints.get(joint))
            .map(|joint| joint.gravity)
            .unwrap_or(0.0)
    }

    pub fn set_joint_gravity_direction(&mut self, index: usize, joint: usize, direction: Vector3) {
        if let Some(setting) = self.settings.get_mut(index) {
            if let Some(entry) = setting.joints.get_mut(joint) {
                entry.gravity_direction = direction;
            }
        }
    }

    pub fn get_joint_gravity_direction(&self, index: usize, joint: usize) -> Vector3 {
        self.settings
            .get(index)
            .and_then(|setting| setting.joints.get(joint))
            .map(|joint| joint.gravity_direction)
            .unwrap_or(Vector3::new(0.0, -1.0, 0.0))
    }

    pub fn set_enable_all_child_collisions(&mut self, index: usize, enabled: bool) {
        if let Some(setting) = self.settings.get_mut(index) {
            setting.enable_all_child_collisions = enabled;
        }
    }

    pub fn are_all_child_collisions_enabled(&self, index: usize) -> bool {
        self.settings
            .get(index)
            .map(|setting| setting.enable_all_child_collisions)
            .unwrap_or(true)
    }

    pub fn set_collision_count(&mut self, index: usize, count: usize) {
        if let Some(setting) = self.settings.get_mut(index) {
            setting.collisions.resize_with(count, NodePath::new);
        }
    }

    pub fn get_collision_count(&self, index: usize) -> usize {
        self.settings
            .get(index)
            .map(|setting| setting.collisions.len())
            .unwrap_or(0)
    }

    pub fn set_collision_path(&mut self, index: usize, collision: usize, path: NodePath) {
        if let Some(setting) = self.settings.get_mut(index) {
            if let Some(slot) = setting.collisions.get_mut(collision) {
                *slot = path;
            }
        }
    }

    pub fn get_collision_path(&self, index: usize, collision: usize) -> NodePath {
        self.settings
            .get(index)
            .and_then(|setting| setting.collisions.get(collision))
            .cloned()
            .unwrap_or_default()
    }

    pub fn clear_collisions(&mut self, index: usize) {
        if let Some(setting) = self.settings.get_mut(index) {
            setting.collisions.clear();
        }
    }

    pub fn set_exclude_collision_count(&mut self, index: usize, count: usize) {
        if let Some(setting) = self.settings.get_mut(index) {
            setting.exclude_collisions.resize_with(count, NodePath::new);
        }
    }

    pub fn get_exclude_collision_count(&self, index: usize) -> usize {
        self.settings
            .get(index)
            .map(|setting| setting.exclude_collisions.len())
            .unwrap_or(0)
    }

    pub fn set_exclude_collision_path(&mut self, index: usize, collision: usize, path: NodePath) {
        if let Some(setting) = self.settings.get_mut(index) {
            if let Some(slot) = setting.exclude_collisions.get_mut(collision) {
                *slot = path;
            }
        }
    }

    pub fn get_exclude_collision_path(&self, index: usize, collision: usize) -> NodePath {
        self.settings
            .get(index)
            .and_then(|setting| setting.exclude_collisions.get(collision))
            .cloned()
            .unwrap_or_default()
    }

    pub fn clear_exclude_collisions(&mut self, index: usize) {
        if let Some(setting) = self.settings.get_mut(index) {
            setting.exclude_collisions.clear();
        }
    }

    /// `reset`: clears the simulation caches in the engine; the port keeps the
    /// configuration and only notes the call.
    pub fn reset(&mut self) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    fn joint() -> SpringBoneJoint {
        SpringBoneJoint::default()
    }

    #[test]
    fn defaults() {
        let mut simulator = SpringBoneSimulator3DData::new();
        assert_eq!(simulator.get_external_force(), Vector3::ZERO);
        assert!(simulator.are_bone_axes_mutable());
        assert_eq!(simulator.get_setting_count(), 0);
        simulator.set_setting_count(1);
        assert_eq!(simulator.get_setting_count(), 1);
        assert_eq!(simulator.get_root_bone_name(0), "");
        assert_eq!(simulator.get_root_bone(0), -1);
        assert_eq!(simulator.get_end_bone(0), -1);
        assert!(!simulator.is_end_bone_extended(0));
        assert_eq!(simulator.get_end_bone_direction(0), BONE_DIRECTION_FROM_PARENT);
        assert_eq!(simulator.get_end_bone_length(0), 0.0);
        assert_eq!(simulator.get_center_from(0), CENTER_FROM_WORLD_ORIGIN);
        assert!(simulator.get_center_node(0).is_empty());
        assert_eq!(simulator.get_center_bone(0), -1);
        assert!(!simulator.is_config_individual(0));
        assert_eq!(simulator.get_radius(0), 0.02);
        assert_eq!(simulator.get_stiffness(0), 1.0);
        assert_eq!(simulator.get_drag(0), 0.4);
        assert_eq!(simulator.get_gravity(0), 0.0);
        assert_eq!(
            simulator.get_gravity_direction(0),
            Vector3::new(0.0, -1.0, 0.0)
        );
        assert_eq!(simulator.get_rotation_axis(0), ROTATION_AXIS_ALL);
        assert!(simulator.are_all_child_collisions_enabled(0));
        assert_eq!(simulator.get_joint_count(0), 0);
        assert_eq!(simulator.get_collision_count(0), 0);
        assert_eq!(simulator.get_exclude_collision_count(0), 0);
        let default_joint = joint();
        assert_eq!(default_joint.radius, 0.1);
        assert_eq!(default_joint.rotation_axis, ROTATION_AXIS_ALL);
        assert_eq!(default_joint.rotation_axis_vector, Vector3::new(1.0, 0.0, 0.0));
    }

    #[test]
    fn setting_accessors() {
        let mut simulator = SpringBoneSimulator3DData::new();
        simulator.set_setting_count(1);
        simulator.set_external_force(Vector3::new(0.0, 1.0, 0.0));
        simulator.set_mutable_bone_axes(false);

        simulator.set_root_bone_name(0, "Hips");
        simulator.set_root_bone(0, 2);
        simulator.set_end_bone_name(0, "Chest");
        simulator.set_end_bone(0, 4);
        simulator.set_extend_end_bone(0, true);
        simulator.set_end_bone_direction(0, 4);
        simulator.set_end_bone_length(0, 0.25);
        simulator.set_center_from(0, CENTER_FROM_BONE);
        let node = NodePath::parse("/root/Pivot").expect("path");
        simulator.set_center_node(0, node.clone());
        simulator.set_center_bone_name(0, "Root");
        simulator.set_center_bone(0, 0);
        simulator.set_individual_config(0, true);
        simulator.set_radius(0, 0.05);
        simulator.set_radius_damping_curve(0, Some(Rid::from_u64(1)));
        simulator.set_stiffness(0, 0.5);
        simulator.set_stiffness_damping_curve(0, Some(Rid::from_u64(2)));
        simulator.set_drag(0, 0.25);
        simulator.set_drag_damping_curve(0, Some(Rid::from_u64(3)));
        simulator.set_gravity(0, 9.8);
        simulator.set_gravity_damping_curve(0, Some(Rid::from_u64(4)));
        simulator.set_gravity_direction(0, Vector3::new(0.0, -1.0, 0.5));
        simulator.set_rotation_axis(0, ROTATION_AXIS_CUSTOM);
        simulator.set_rotation_axis_vector(0, Vector3::new(0.0, 0.0, 1.0));

        assert_eq!(simulator.get_external_force(), Vector3::new(0.0, 1.0, 0.0));
        assert!(!simulator.are_bone_axes_mutable());
        assert_eq!(simulator.get_root_bone_name(0), "Hips");
        assert_eq!(simulator.get_root_bone(0), 2);
        assert_eq!(simulator.get_end_bone_name(0), "Chest");
        assert_eq!(simulator.get_end_bone(0), 4);
        assert!(simulator.is_end_bone_extended(0));
        assert_eq!(simulator.get_end_bone_direction(0), 4);
        assert_eq!(simulator.get_end_bone_length(0), 0.25);
        assert_eq!(simulator.get_center_from(0), CENTER_FROM_BONE);
        assert_eq!(simulator.get_center_node(0), node);
        assert_eq!(simulator.get_center_bone_name(0), "Root");
        assert_eq!(simulator.get_center_bone(0), 0);
        assert!(simulator.is_config_individual(0));
        assert_eq!(simulator.get_radius(0), 0.05);
        assert_eq!(simulator.get_radius_damping_curve(0), Some(Rid::from_u64(1)));
        assert_eq!(simulator.get_stiffness(0), 0.5);
        assert_eq!(simulator.get_stiffness_damping_curve(0), Some(Rid::from_u64(2)));
        assert_eq!(simulator.get_drag(0), 0.25);
        assert_eq!(simulator.get_drag_damping_curve(0), Some(Rid::from_u64(3)));
        assert_eq!(simulator.get_gravity(0), 9.8);
        assert_eq!(simulator.get_gravity_damping_curve(0), Some(Rid::from_u64(4)));
        assert_eq!(simulator.get_gravity_direction(0), Vector3::new(0.0, -1.0, 0.5));
        assert_eq!(simulator.get_rotation_axis(0), ROTATION_AXIS_CUSTOM);
        assert_eq!(simulator.get_rotation_axis_vector(0), Vector3::new(0.0, 0.0, 1.0));
        // Out-of-range settings keep defaults.
        assert_eq!(simulator.get_radius(4), 0.02);
    }

    #[test]
    fn joints_and_collisions() {
        let mut simulator = SpringBoneSimulator3DData::new();
        simulator.set_setting_count(1);
        simulator.set_joint_count(0, 2);
        assert_eq!(simulator.get_joint_count(0), 2);
        assert_eq!(simulator.get_joint_bone(0, 0), -1);
        assert_eq!(simulator.get_joint_radius(0, 0), 0.1);
        assert_eq!(simulator.get_joint_stiffness(0, 0), 1.0);
        assert_eq!(simulator.get_joint_drag(0, 0), 0.0);
        assert_eq!(simulator.get_joint_gravity(0, 0), 0.0);
        assert_eq!(simulator.get_joint_bone(0, 9), -1);

        simulator.set_joint_rotation_axis(0, 1, ROTATION_AXIS_Z);
        simulator.set_joint_rotation_axis_vector(0, 1, Vector3::new(0.0, 1.0, 0.0));
        simulator.set_joint_radius(0, 1, 0.2);
        simulator.set_joint_stiffness(0, 1, 0.7);
        simulator.set_joint_drag(0, 1, 0.3);
        simulator.set_joint_gravity(0, 1, 2.5);
        simulator.set_joint_gravity_direction(0, 1, Vector3::new(1.0, -1.0, 0.0));
        assert_eq!(simulator.get_joint_rotation_axis(0, 1), ROTATION_AXIS_Z);
        assert_eq!(
            simulator.get_joint_rotation_axis_vector(0, 1),
            Vector3::new(0.0, 1.0, 0.0)
        );
        assert_eq!(simulator.get_joint_radius(0, 1), 0.2);
        assert_eq!(simulator.get_joint_stiffness(0, 1), 0.7);
        assert_eq!(simulator.get_joint_drag(0, 1), 0.3);
        assert_eq!(simulator.get_joint_gravity(0, 1), 2.5);
        assert_eq!(
            simulator.get_joint_gravity_direction(0, 1),
            Vector3::new(1.0, -1.0, 0.0)
        );

        // Collisions.
        simulator.set_collision_count(0, 2);
        simulator.set_collision_path(0, 0, NodePath::parse("ColliderA").expect("path"));
        simulator.set_collision_path(0, 1, NodePath::parse("ColliderB").expect("path"));
        assert_eq!(simulator.get_collision_count(0), 2);
        assert_eq!(
            simulator.get_collision_path(0, 0).as_string(),
            "ColliderA"
        );
        simulator.set_exclude_collision_count(0, 1);
        simulator.set_exclude_collision_path(0, 0, NodePath::parse("Skip").expect("path"));
        assert_eq!(simulator.get_exclude_collision_count(0), 1);
        assert_eq!(
            simulator.get_exclude_collision_path(0, 0).as_string(),
            "Skip"
        );
        simulator.set_enable_all_child_collisions(0, false);
        assert!(!simulator.are_all_child_collisions_enabled(0));
        simulator.clear_collisions(0);
        simulator.clear_exclude_collisions(0);
        assert_eq!(simulator.get_collision_count(0), 0);
        assert_eq!(simulator.get_exclude_collision_count(0), 0);

        simulator.reset();
        simulator.clear_settings();
        assert_eq!(simulator.get_setting_count(), 0);
        assert_eq!(ROTATION_AXIS_CUSTOM, 4);
        assert_eq!(CENTER_FROM_NODE, 1);
    }
}
