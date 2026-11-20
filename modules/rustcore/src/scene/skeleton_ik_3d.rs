//! `scene/3d/skeleton_ik_3d.*` (state): the FABRIK IK modifier.
//!
//! Ported state from `SkeletonIK3D`: the root/tip bone names, the target
//! transform and its node-path override, the tip-basis override and magnet
//! settings, the minimum distance and iteration cap, the influence
//! (`SkeletonModifier3D::get_influence`, exposed by the deprecated interpolation
//! accessors) and the active flag driven by `start`/`stop`.
//!
//! The FABRIK solver, the chain construction from a real `Skeleton3D`, the
//! modification callback and the magnet re-orientation are not ported.

use crate::math::transform_3d::Transform3D;
use crate::math::vector3::Vector3;
use crate::string::node_path::NodePath;

#[derive(Clone, Debug)]
pub struct SkeletonIk3DData {
    internal_active: bool,
    root_bone: String,
    tip_bone: String,
    target: Transform3D,
    target_node_path_override: NodePath,
    override_tip_basis: bool,
    use_magnet: bool,
    magnet_position: Vector3,
    min_distance: f32,
    max_iterations: i32,
    influence: f32,
}

impl Default for SkeletonIk3DData {
    fn default() -> Self {
        Self::new()
    }
}

impl SkeletonIk3DData {
    pub fn new() -> Self {
        Self {
            internal_active: false,
            root_bone: String::new(),
            tip_bone: String::new(),
            target: Transform3D::default(),
            target_node_path_override: NodePath::new(),
            override_tip_basis: true,
            use_magnet: false,
            magnet_position: Vector3::ZERO,
            min_distance: 0.01,
            max_iterations: 10,
            influence: 1.0,
        }
    }

    pub fn set_root_bone(&mut self, bone: &str) {
        self.root_bone = bone.to_string();
    }

    pub fn get_root_bone(&self) -> &str {
        &self.root_bone
    }

    pub fn set_tip_bone(&mut self, bone: &str) {
        self.tip_bone = bone.to_string();
    }

    pub fn get_tip_bone(&self) -> &str {
        &self.tip_bone
    }

    pub fn set_target_transform(&mut self, target: Transform3D) {
        self.target = target;
    }

    pub fn get_target_transform(&self) -> Transform3D {
        self.target
    }

    pub fn set_target_node(&mut self, path: NodePath) {
        self.target_node_path_override = path;
    }

    pub fn get_target_node(&self) -> &NodePath {
        &self.target_node_path_override
    }

    pub fn set_override_tip_basis(&mut self, enabled: bool) {
        self.override_tip_basis = enabled;
    }

    pub fn is_override_tip_basis(&self) -> bool {
        self.override_tip_basis
    }

    pub fn set_use_magnet(&mut self, enabled: bool) {
        self.use_magnet = enabled;
    }

    pub fn is_using_magnet(&self) -> bool {
        self.use_magnet
    }

    pub fn set_magnet_position(&mut self, position: Vector3) {
        self.magnet_position = position;
    }

    pub fn get_magnet_position(&self) -> Vector3 {
        self.magnet_position
    }

    pub fn set_min_distance(&mut self, distance: f32) {
        self.min_distance = distance;
    }

    pub fn get_min_distance(&self) -> f32 {
        self.min_distance
    }

    pub fn set_max_iterations(&mut self, iterations: i32) {
        self.max_iterations = iterations;
    }

    pub fn get_max_iterations(&self) -> i32 {
        self.max_iterations
    }

    /// The deprecated interpolation accessors map onto the influence.
    pub fn set_interpolation(&mut self, value: f32) -> bool {
        self.influence = value;
        true
    }

    pub fn get_interpolation(&self) -> f32 {
        self.influence
    }

    pub fn set_influence(&mut self, influence: f32) {
        self.influence = influence;
    }

    pub fn get_influence(&self) -> f32 {
        self.influence
    }

    pub fn is_running(&self) -> bool {
        self.internal_active
    }

    /// `start`: a one-time start runs and immediately deactivates.
    pub fn start(&mut self, one_time: bool) {
        if one_time {
            self.internal_active = true;
            self.internal_active = false;
        } else {
            self.internal_active = true;
        }
    }

    pub fn stop(&mut self) {
        self.internal_active = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults() {
        let ik = SkeletonIk3DData::new();
        assert_eq!(ik.get_root_bone(), "");
        assert_eq!(ik.get_tip_bone(), "");
        assert_eq!(ik.get_target_transform(), Transform3D::default());
        assert!(ik.get_target_node().is_empty());
        assert!(ik.is_override_tip_basis());
        assert!(!ik.is_using_magnet());
        assert_eq!(ik.get_magnet_position(), Vector3::ZERO);
        assert_eq!(ik.get_min_distance(), 0.01);
        assert_eq!(ik.get_max_iterations(), 10);
        assert_eq!(ik.get_influence(), 1.0);
        assert!(!ik.is_running());
    }

    #[test]
    fn bone_and_target_accessors() {
        let mut ik = SkeletonIk3DData::new();
        ik.set_root_bone("Hips");
        ik.set_tip_bone("Hand.R");
        assert_eq!(ik.get_root_bone(), "Hips");
        assert_eq!(ik.get_tip_bone(), "Hand.R");

        let target = Transform3D::default();
        ik.set_target_transform(target);
        assert_eq!(ik.get_target_transform(), target);

        let path = NodePath::parse("/root/Scene/Target").expect("path");
        ik.set_target_node(path.clone());
        assert_eq!(ik.get_target_node(), &path);

        ik.set_override_tip_basis(false);
        assert!(!ik.is_override_tip_basis());
        ik.set_use_magnet(true);
        assert!(ik.is_using_magnet());
        ik.set_magnet_position(Vector3::new(1.0, 2.0, 3.0));
        assert_eq!(ik.get_magnet_position(), Vector3::new(1.0, 2.0, 3.0));
    }

    #[test]
    fn tuning_and_influence() {
        let mut ik = SkeletonIk3DData::new();
        ik.set_min_distance(0.5);
        ik.set_max_iterations(32);
        assert_eq!(ik.get_min_distance(), 0.5);
        assert_eq!(ik.get_max_iterations(), 32);

        assert!(ik.set_interpolation(0.25));
        assert_eq!(ik.get_interpolation(), 0.25);
        assert_eq!(ik.get_influence(), 0.25);
        ik.set_influence(0.75);
        assert_eq!(ik.get_influence(), 0.75);
        assert_eq!(ik.get_interpolation(), 0.75);
    }

    #[test]
    fn start_and_stop() {
        let mut ik = SkeletonIk3DData::new();
        ik.start(false);
        assert!(ik.is_running());
        ik.stop();
        assert!(!ik.is_running());

        // A one-time start leaves the modifier inactive.
        ik.start(true);
        assert!(!ik.is_running());
    }
}
