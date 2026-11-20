//! `scene/3d/retarget_modifier_3d.*` (state): the skeleton retarget modifier.
//!
//! Ported state: the profile handle (none), the global-pose flag (off) and the
//! enabled transform flags (`TRANSFORM_FLAG_ALL` by default) with the per-mode
//! toggles. The flag bit values match the engine (`position = 1`, `rotation = 2`,
//! `scale = 4`).
//!
//! The child-skeleton rest capture and the retarget transform are not ported.

use crate::servers::rid::Rid;

pub const TRANSFORM_FLAG_POSITION: i32 = 1;
pub const TRANSFORM_FLAG_ROTATION: i32 = 2;
pub const TRANSFORM_FLAG_SCALE: i32 = 4;
pub const TRANSFORM_FLAG_ALL: i32 =
    TRANSFORM_FLAG_POSITION | TRANSFORM_FLAG_ROTATION | TRANSFORM_FLAG_SCALE;

#[derive(Clone, Debug, Default)]
pub struct RetargetModifier3DData {
    profile: Option<Rid>,
    use_global_pose: bool,
    enable_flags: i32,
}

impl RetargetModifier3DData {
    pub fn new() -> Self {
        Self {
            profile: None,
            use_global_pose: false,
            enable_flags: TRANSFORM_FLAG_ALL,
        }
    }

    pub fn set_use_global_pose(&mut self, enabled: bool) {
        self.use_global_pose = enabled;
    }

    pub fn is_using_global_pose(&self) -> bool {
        self.use_global_pose
    }

    pub fn set_enable_flags(&mut self, flags: i32) {
        self.enable_flags = flags;
    }

    pub fn get_enable_flags(&self) -> i32 {
        self.enable_flags
    }

    pub fn set_position_enabled(&mut self, enabled: bool) {
        if enabled {
            self.enable_flags |= TRANSFORM_FLAG_POSITION;
        } else {
            self.enable_flags &= !TRANSFORM_FLAG_POSITION;
        }
    }

    pub fn is_position_enabled(&self) -> bool {
        self.enable_flags & TRANSFORM_FLAG_POSITION != 0
    }

    pub fn set_rotation_enabled(&mut self, enabled: bool) {
        if enabled {
            self.enable_flags |= TRANSFORM_FLAG_ROTATION;
        } else {
            self.enable_flags &= !TRANSFORM_FLAG_ROTATION;
        }
    }

    pub fn is_rotation_enabled(&self) -> bool {
        self.enable_flags & TRANSFORM_FLAG_ROTATION != 0
    }

    pub fn set_scale_enabled(&mut self, enabled: bool) {
        if enabled {
            self.enable_flags |= TRANSFORM_FLAG_SCALE;
        } else {
            self.enable_flags &= !TRANSFORM_FLAG_SCALE;
        }
    }

    pub fn is_scale_enabled(&self) -> bool {
        self.enable_flags & TRANSFORM_FLAG_SCALE != 0
    }

    pub fn set_profile(&mut self, profile: Option<Rid>) {
        self.profile = profile;
    }

    pub fn get_profile(&self) -> Option<Rid> {
        self.profile
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults() {
        let retarget = RetargetModifier3DData::new();
        assert!(retarget.get_profile().is_none());
        assert!(!retarget.is_using_global_pose());
        assert_eq!(retarget.get_enable_flags(), TRANSFORM_FLAG_ALL);
        assert!(retarget.is_position_enabled());
        assert!(retarget.is_rotation_enabled());
        assert!(retarget.is_scale_enabled());
        assert_eq!(TRANSFORM_FLAG_POSITION, 1);
        assert_eq!(TRANSFORM_FLAG_ROTATION, 2);
        assert_eq!(TRANSFORM_FLAG_SCALE, 4);
        assert_eq!(TRANSFORM_FLAG_ALL, 7);
    }

    #[test]
    fn flags_and_profile() {
        let mut retarget = RetargetModifier3DData::new();
        retarget.set_use_global_pose(true);
        assert!(retarget.is_using_global_pose());

        retarget.set_position_enabled(false);
        assert!(!retarget.is_position_enabled());
        assert_eq!(retarget.get_enable_flags(), 6);
        retarget.set_scale_enabled(false);
        assert_eq!(retarget.get_enable_flags(), 2);
        assert!(retarget.is_rotation_enabled());

        retarget.set_enable_flags(TRANSFORM_FLAG_POSITION | TRANSFORM_FLAG_SCALE);
        assert_eq!(retarget.get_enable_flags(), 5);
        assert!(retarget.is_position_enabled());
        assert!(!retarget.is_rotation_enabled());
        assert!(retarget.is_scale_enabled());

        retarget.set_rotation_enabled(true);
        assert_eq!(retarget.get_enable_flags(), 7);

        retarget.set_profile(Some(Rid::from_u64(4)));
        assert_eq!(retarget.get_profile(), Some(Rid::from_u64(4)));
        retarget.set_profile(None);
        assert!(retarget.get_profile().is_none());
    }
}
