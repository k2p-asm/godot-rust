//! `scene/resources/3d/skin.*`: skin bindings.
//!
//! A bind stores the bone index (default -1), an optional bone name and the bind
//! pose (`Transform3D`, identity by default). `set_bind_count` rejects negative
//! counts and resizes (new entries keep the defaults), `add_bind` /
//! `add_named_bind` append, and `set_*` validate the index. `clear_binds`
//! empties the list.

use crate::math::transform_3d::Transform3D;
use crate::string::string_name::StringName;

#[derive(Clone, Debug)]
pub struct SkinBind {
    pub bone: i32,
    pub name: StringName,
    pub pose: Transform3D,
}

impl Default for SkinBind {
    fn default() -> Self {
        Self {
            bone: -1,
            name: StringName::new(""),
            pose: Transform3D::IDENTITY,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct SkinData {
    binds: Vec<SkinBind>,
}

impl SkinData {
    pub fn new() -> Self {
        Self::default()
    }

    /// Rejects negative counts; new entries are the default bind.
    pub fn set_bind_count(&mut self, count: i32) -> bool {
        if count < 0 {
            return false;
        }
        self.binds.resize(count as usize, SkinBind::default());
        true
    }

    pub fn get_bind_count(&self) -> usize {
        self.binds.len()
    }

    pub fn add_bind(&mut self, bone: i32, pose: Transform3D) {
        self.binds.push(SkinBind {
            bone,
            name: StringName::new(""),
            pose,
        });
    }

    pub fn add_named_bind(&mut self, name: &str, pose: Transform3D) {
        self.binds.push(SkinBind {
            bone: -1,
            name: StringName::new(name),
            pose,
        });
    }

    pub fn set_bind_bone(&mut self, index: usize, bone: i32) -> bool {
        match self.binds.get_mut(index) {
            Some(b) => {
                b.bone = bone;
                true
            }
            None => false,
        }
    }

    pub fn set_bind_name(&mut self, index: usize, name: &str) -> bool {
        match self.binds.get_mut(index) {
            Some(b) => {
                b.name = StringName::new(name);
                true
            }
            None => false,
        }
    }

    pub fn set_bind_pose(&mut self, index: usize, pose: Transform3D) -> bool {
        match self.binds.get_mut(index) {
            Some(b) => {
                b.pose = pose;
                true
            }
            None => false,
        }
    }

    /// Invalid indices read as -1, like `ERR_FAIL_INDEX_V`.
    pub fn get_bind_bone(&self, index: usize) -> i32 {
        self.binds.get(index).map(|b| b.bone).unwrap_or(-1)
    }

    pub fn get_bind_name(&self, index: usize) -> StringName {
        self.binds
            .get(index)
            .map(|b| b.name)
            .unwrap_or_else(|| StringName::new(""))
    }

    pub fn get_bind_pose(&self, index: usize) -> Transform3D {
        self.binds
            .get(index)
            .map(|b| b.pose)
            .unwrap_or(Transform3D::IDENTITY)
    }

    pub fn get_bind(&self, index: usize) -> Option<&SkinBind> {
        self.binds.get(index)
    }

    pub fn clear_binds(&mut self) {
        self.binds.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::vector3::Vector3;

    #[test]
    fn defaults_and_resize() {
        let mut s = SkinData::new();
        assert_eq!(s.get_bind_count(), 0);

        assert!(!s.set_bind_count(-1));
        assert!(s.set_bind_count(2));
        assert_eq!(s.get_bind_count(), 2);
        assert_eq!(s.get_bind_bone(0), -1);
        assert_eq!(s.get_bind_name(0).as_string(), "");
        assert_eq!(s.get_bind_pose(0), Transform3D::IDENTITY);

        // Resizing keeps the existing entries.
        s.set_bind_bone(0, 3);
        assert!(s.set_bind_count(4));
        assert_eq!(s.get_bind_bone(0), 3);
        assert_eq!(s.get_bind_bone(3), -1);
    }

    #[test]
    fn add_and_set_binds() {
        let mut s = SkinData::new();
        let pose = Transform3D {
            basis: crate::math::basis::Basis::IDENTITY,
            origin: Vector3::new(1.0, 2.0, 3.0),
        };
        s.add_bind(5, pose);
        s.add_named_bind("hand", pose);
        assert_eq!(s.get_bind_count(), 2);
        assert_eq!(s.get_bind_bone(0), 5);
        assert_eq!(s.get_bind_name(0).as_string(), "");
        assert_eq!(s.get_bind_pose(0).origin, Vector3::new(1.0, 2.0, 3.0));
        assert_eq!(s.get_bind_name(1).as_string(), "hand");
        assert_eq!(s.get_bind_bone(1), -1);

        assert!(s.set_bind_bone(1, 7));
        assert!(s.set_bind_name(1, "right_hand"));
        assert!(s.set_bind_pose(
            1,
            Transform3D {
                basis: crate::math::basis::Basis::IDENTITY,
                origin: Vector3::ZERO,
            }
        ));
        assert_eq!(s.get_bind_bone(1), 7);
        assert_eq!(s.get_bind_name(1).as_string(), "right_hand");
        assert_eq!(s.get_bind_pose(1).origin, Vector3::ZERO);

        assert!(!s.set_bind_bone(2, 1));
        assert!(!s.set_bind_name(2, "x"));
        assert!(!s.set_bind_pose(2, Transform3D::IDENTITY));
        assert_eq!(s.get_bind_bone(9), -1);
        assert_eq!(s.get_bind_pose(9), Transform3D::IDENTITY);

        s.clear_binds();
        assert_eq!(s.get_bind_count(), 0);
    }
}
