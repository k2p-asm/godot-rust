//! `scene/3d/bone_attachment_3d.*`: bone attachment state.
//!
//! Stores the selected bone name (empty) and index (-1), whether the attachment
//! overrides the bone pose (off), and the optional external skeleton path
//! (empty). Binding to the skeleton, pose overrides and the transform updates
//! are not ported.

#[derive(Clone, Debug, Default)]
pub struct BoneAttachment3DData {
    bone_name: String,
    bone_idx: i32,
    override_pose: bool,
    use_external_skeleton: bool,
    external_skeleton: String,
}

impl BoneAttachment3DData {
    pub fn new() -> Self {
        Self {
            bone_name: String::new(),
            bone_idx: -1,
            override_pose: false,
            use_external_skeleton: false,
            external_skeleton: String::new(),
        }
    }

    pub fn set_bone_name(&mut self, name: &str) {
        self.bone_name = name.to_string();
    }

    pub fn get_bone_name(&self) -> &str {
        &self.bone_name
    }

    pub fn set_bone_idx(&mut self, index: i32) {
        self.bone_idx = index;
    }

    pub fn get_bone_idx(&self) -> i32 {
        self.bone_idx
    }

    pub fn set_override_pose(&mut self, override_pose: bool) {
        self.override_pose = override_pose;
    }

    pub fn get_override_pose(&self) -> bool {
        self.override_pose
    }

    pub fn set_use_external_skeleton(&mut self, use_external: bool) {
        self.use_external_skeleton = use_external;
    }

    pub fn get_use_external_skeleton(&self) -> bool {
        self.use_external_skeleton
    }

    pub fn set_external_skeleton(&mut self, path: &str) {
        self.external_skeleton = path.to_string();
    }

    pub fn get_external_skeleton(&self) -> &str {
        &self.external_skeleton
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults() {
        let b = BoneAttachment3DData::new();
        assert_eq!(b.get_bone_name(), "");
        assert_eq!(b.get_bone_idx(), -1);
        assert!(!b.get_override_pose());
        assert!(!b.get_use_external_skeleton());
        assert_eq!(b.get_external_skeleton(), "");
    }

    #[test]
    fn setters_round_trip() {
        let mut b = BoneAttachment3DData::new();
        b.set_bone_name("hand.R");
        b.set_bone_idx(7);
        b.set_override_pose(true);
        b.set_use_external_skeleton(true);
        b.set_external_skeleton("../Armature/Skeleton3D");

        assert_eq!(b.get_bone_name(), "hand.R");
        assert_eq!(b.get_bone_idx(), 7);
        assert!(b.get_override_pose());
        assert!(b.get_use_external_skeleton());
        assert_eq!(b.get_external_skeleton(), "../Armature/Skeleton3D");

        // No validation: negative indices and empty paths are stored.
        b.set_bone_idx(-5);
        b.set_external_skeleton("");
        assert_eq!(b.get_bone_idx(), -5);
        assert_eq!(b.get_external_skeleton(), "");
    }
}
