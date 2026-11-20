//! New 4.x bone-modifier state: `ModifierBoneTarget3D`, `AimModifier3D`,
//! `CopyTransformModifier3D` and `ConvertTransformModifier3D`.
//!
//! Each constraint modifier keeps the shared `BoneConstraint3D` settings array
//! plus its own per-setting fields (aim axes/euler/relative, copy/invert flags,
//! convert modes/ranges). The port keeps the base array and the extra arrays in
//! sync through `set_setting_count`.
//!
//! The skeleton processing/constraint evaluation is not ported.

use crate::scene::bone_constraint_3d::BoneConstraint3DData;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum BoneAxis {
    PlusX = 0,
    PlusY = 1,
    PlusZ = 2,
    MinusX = 3,
    MinusY = 4,
    MinusZ = 5,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum TransformFlag {
    Position = 1,
    Rotation = 2,
    Scale = 4,
    All = 7,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum AxisFlag {
    X = 1,
    Y = 2,
    Z = 4,
    All = 7,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum TransformMode {
    Position = 0,
    Rotation = 1,
    Scale = 2,
}

// -- ModifierBoneTarget3D ---------------------------------------------------

#[derive(Clone, Debug, Default)]
pub struct ModifierBoneTarget3DData {
    bone_name: String,
    bone: i32,
}

impl ModifierBoneTarget3DData {
    pub fn new() -> Self {
        Self {
            bone_name: String::new(),
            bone: -1,
        }
    }

    pub fn set_bone_name(&mut self, name: &str) {
        self.bone_name = name.to_string();
    }

    pub fn get_bone_name(&self) -> &str {
        &self.bone_name
    }

    pub fn set_bone(&mut self, bone: i32) {
        self.bone = bone;
    }

    pub fn get_bone(&self) -> i32 {
        self.bone
    }
}

// -- AimModifier3D ----------------------------------------------------------

#[derive(Clone, Debug)]
pub struct AimSetting {
    pub forward_axis: BoneAxis,
    pub use_euler: bool,
    pub primary_rotation_axis: i32,
    pub use_secondary_rotation: bool,
    pub relative: bool,
}

impl Default for AimSetting {
    fn default() -> Self {
        Self {
            forward_axis: BoneAxis::PlusY,
            use_euler: false,
            primary_rotation_axis: 0, // Vector3::AXIS_X
            use_secondary_rotation: true,
            relative: true,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct AimModifier3DData {
    base: BoneConstraint3DData,
    settings: Vec<AimSetting>,
}

impl AimModifier3DData {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn base(&self) -> &BoneConstraint3DData {
        &self.base
    }

    pub fn base_mut(&mut self) -> &mut BoneConstraint3DData {
        &mut self.base
    }

    /// Resizes both the base settings and the aim-specific array.
    pub fn set_setting_count(&mut self, count: usize) {
        self.base.set_setting_count(count);
        self.settings.resize(count, AimSetting::default());
    }

    pub fn get_setting_count(&self) -> usize {
        self.settings.len()
    }

    pub fn clear_settings(&mut self) {
        self.base.clear_settings();
        self.settings.clear();
    }

    pub fn set_forward_axis(&mut self, index: usize, axis: i32) {
        if let Some(setting) = self.settings.get_mut(index) {
            setting.forward_axis = match axis {
                0 => BoneAxis::PlusX,
                1 => BoneAxis::PlusY,
                2 => BoneAxis::PlusZ,
                3 => BoneAxis::MinusX,
                4 => BoneAxis::MinusY,
                5 => BoneAxis::MinusZ,
                _ => setting.forward_axis,
            };
        }
    }

    pub fn get_forward_axis(&self, index: usize) -> BoneAxis {
        self.settings
            .get(index)
            .map(|setting| setting.forward_axis)
            .unwrap_or(BoneAxis::PlusY)
    }

    pub fn set_use_euler(&mut self, index: usize, enabled: bool) {
        if let Some(setting) = self.settings.get_mut(index) {
            setting.use_euler = enabled;
        }
    }

    pub fn is_using_euler(&self, index: usize) -> bool {
        self.settings
            .get(index)
            .map(|setting| setting.use_euler)
            .unwrap_or(false)
    }

    pub fn set_primary_rotation_axis(&mut self, index: usize, axis: i32) {
        if let Some(setting) = self.settings.get_mut(index) {
            setting.primary_rotation_axis = axis;
        }
    }

    pub fn get_primary_rotation_axis(&self, index: usize) -> i32 {
        self.settings
            .get(index)
            .map(|setting| setting.primary_rotation_axis)
            .unwrap_or(0)
    }

    pub fn set_use_secondary_rotation(&mut self, index: usize, enabled: bool) {
        if let Some(setting) = self.settings.get_mut(index) {
            setting.use_secondary_rotation = enabled;
        }
    }

    pub fn is_using_secondary_rotation(&self, index: usize) -> bool {
        self.settings
            .get(index)
            .map(|setting| setting.use_secondary_rotation)
            .unwrap_or(true)
    }

    pub fn set_relative(&mut self, index: usize, relative: bool) {
        if let Some(setting) = self.settings.get_mut(index) {
            setting.relative = relative;
        }
    }

    pub fn is_relative(&self, index: usize) -> bool {
        self.settings
            .get(index)
            .map(|setting| setting.relative)
            .unwrap_or(true)
    }
}

// -- CopyTransformModifier3D ------------------------------------------------

#[derive(Clone, Debug)]
pub struct CopyTransformSetting {
    pub copy_flags: i32,
    pub axis_flags: i32,
    pub invert_flags: i32,
    pub relative: bool,
    pub additive: bool,
}

impl Default for CopyTransformSetting {
    fn default() -> Self {
        Self {
            copy_flags: TransformFlag::All as i32,
            axis_flags: AxisFlag::All as i32,
            invert_flags: 0,
            relative: true,
            additive: false,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct CopyTransformModifier3DData {
    base: BoneConstraint3DData,
    settings: Vec<CopyTransformSetting>,
}

impl CopyTransformModifier3DData {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn base(&self) -> &BoneConstraint3DData {
        &self.base
    }

    pub fn base_mut(&mut self) -> &mut BoneConstraint3DData {
        &mut self.base
    }

    pub fn set_setting_count(&mut self, count: usize) {
        self.base.set_setting_count(count);
        self.settings
            .resize(count, CopyTransformSetting::default());
    }

    pub fn get_setting_count(&self) -> usize {
        self.settings.len()
    }

    pub fn clear_settings(&mut self) {
        self.base.clear_settings();
        self.settings.clear();
    }

    pub fn set_copy_flags(&mut self, index: usize, flags: i32) {
        if let Some(setting) = self.settings.get_mut(index) {
            setting.copy_flags = flags;
        }
    }

    pub fn get_copy_flags(&self, index: usize) -> i32 {
        self.settings
            .get(index)
            .map(|setting| setting.copy_flags)
            .unwrap_or(TransformFlag::All as i32)
    }

    pub fn set_axis_flags(&mut self, index: usize, flags: i32) {
        if let Some(setting) = self.settings.get_mut(index) {
            setting.axis_flags = flags;
        }
    }

    pub fn get_axis_flags(&self, index: usize) -> i32 {
        self.settings
            .get(index)
            .map(|setting| setting.axis_flags)
            .unwrap_or(AxisFlag::All as i32)
    }

    pub fn set_invert_flags(&mut self, index: usize, flags: i32) {
        if let Some(setting) = self.settings.get_mut(index) {
            setting.invert_flags = flags;
        }
    }

    pub fn get_invert_flags(&self, index: usize) -> i32 {
        self.settings
            .get(index)
            .map(|setting| setting.invert_flags)
            .unwrap_or(0)
    }

    pub fn set_relative(&mut self, index: usize, relative: bool) {
        if let Some(setting) = self.settings.get_mut(index) {
            setting.relative = relative;
        }
    }

    pub fn is_relative(&self, index: usize) -> bool {
        self.settings
            .get(index)
            .map(|setting| setting.relative)
            .unwrap_or(true)
    }

    pub fn set_additive(&mut self, index: usize, additive: bool) {
        if let Some(setting) = self.settings.get_mut(index) {
            setting.additive = additive;
        }
    }

    pub fn is_additive(&self, index: usize) -> bool {
        self.settings
            .get(index)
            .map(|setting| setting.additive)
            .unwrap_or(false)
    }
}

// -- ConvertTransformModifier3D ---------------------------------------------

#[derive(Clone, Debug)]
pub struct ConvertTransformSetting {
    pub apply_transform_mode: TransformMode,
    pub apply_axis: i32,
    pub apply_range_min: f32,
    pub apply_range_max: f32,
    pub reference_transform_mode: TransformMode,
    pub reference_axis: i32,
    pub reference_range_min: f32,
    pub reference_range_max: f32,
    pub relative: bool,
    pub additive: bool,
}

impl Default for ConvertTransformSetting {
    fn default() -> Self {
        Self {
            apply_transform_mode: TransformMode::Position,
            apply_axis: 0,
            apply_range_min: 0.0,
            apply_range_max: 0.0,
            reference_transform_mode: TransformMode::Position,
            reference_axis: 0,
            reference_range_min: 0.0,
            reference_range_max: 0.0,
            relative: true,
            additive: false,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct ConvertTransformModifier3DData {
    base: BoneConstraint3DData,
    settings: Vec<ConvertTransformSetting>,
}

impl ConvertTransformModifier3DData {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn base(&self) -> &BoneConstraint3DData {
        &self.base
    }

    pub fn base_mut(&mut self) -> &mut BoneConstraint3DData {
        &mut self.base
    }

    pub fn set_setting_count(&mut self, count: usize) {
        self.base.set_setting_count(count);
        self.settings
            .resize(count, ConvertTransformSetting::default());
    }

    pub fn get_setting_count(&self) -> usize {
        self.settings.len()
    }

    pub fn clear_settings(&mut self) {
        self.base.clear_settings();
        self.settings.clear();
    }

    pub fn set_apply_transform_mode(&mut self, index: usize, mode: i32) {
        if let Some(setting) = self.settings.get_mut(index) {
            setting.apply_transform_mode = match mode {
                0 => TransformMode::Position,
                1 => TransformMode::Rotation,
                2 => TransformMode::Scale,
                _ => setting.apply_transform_mode,
            };
        }
    }

    pub fn get_apply_transform_mode(&self, index: usize) -> TransformMode {
        self.settings
            .get(index)
            .map(|setting| setting.apply_transform_mode)
            .unwrap_or(TransformMode::Position)
    }

    pub fn set_apply_axis(&mut self, index: usize, axis: i32) {
        if let Some(setting) = self.settings.get_mut(index) {
            setting.apply_axis = axis;
        }
    }

    pub fn get_apply_axis(&self, index: usize) -> i32 {
        self.settings
            .get(index)
            .map(|setting| setting.apply_axis)
            .unwrap_or(0)
    }

    pub fn set_apply_range_min(&mut self, index: usize, value: f32) {
        if let Some(setting) = self.settings.get_mut(index) {
            setting.apply_range_min = value;
        }
    }

    pub fn get_apply_range_min(&self, index: usize) -> f32 {
        self.settings
            .get(index)
            .map(|setting| setting.apply_range_min)
            .unwrap_or(0.0)
    }

    pub fn set_apply_range_max(&mut self, index: usize, value: f32) {
        if let Some(setting) = self.settings.get_mut(index) {
            setting.apply_range_max = value;
        }
    }

    pub fn get_apply_range_max(&self, index: usize) -> f32 {
        self.settings
            .get(index)
            .map(|setting| setting.apply_range_max)
            .unwrap_or(0.0)
    }

    pub fn set_reference_transform_mode(&mut self, index: usize, mode: i32) {
        if let Some(setting) = self.settings.get_mut(index) {
            setting.reference_transform_mode = match mode {
                0 => TransformMode::Position,
                1 => TransformMode::Rotation,
                2 => TransformMode::Scale,
                _ => setting.reference_transform_mode,
            };
        }
    }

    pub fn get_reference_transform_mode(&self, index: usize) -> TransformMode {
        self.settings
            .get(index)
            .map(|setting| setting.reference_transform_mode)
            .unwrap_or(TransformMode::Position)
    }

    pub fn set_reference_axis(&mut self, index: usize, axis: i32) {
        if let Some(setting) = self.settings.get_mut(index) {
            setting.reference_axis = axis;
        }
    }

    pub fn get_reference_axis(&self, index: usize) -> i32 {
        self.settings
            .get(index)
            .map(|setting| setting.reference_axis)
            .unwrap_or(0)
    }

    pub fn set_reference_range_min(&mut self, index: usize, value: f32) {
        if let Some(setting) = self.settings.get_mut(index) {
            setting.reference_range_min = value;
        }
    }

    pub fn get_reference_range_min(&self, index: usize) -> f32 {
        self.settings
            .get(index)
            .map(|setting| setting.reference_range_min)
            .unwrap_or(0.0)
    }

    pub fn set_reference_range_max(&mut self, index: usize, value: f32) {
        if let Some(setting) = self.settings.get_mut(index) {
            setting.reference_range_max = value;
        }
    }

    pub fn get_reference_range_max(&self, index: usize) -> f32 {
        self.settings
            .get(index)
            .map(|setting| setting.reference_range_max)
            .unwrap_or(0.0)
    }

    pub fn set_relative(&mut self, index: usize, relative: bool) {
        if let Some(setting) = self.settings.get_mut(index) {
            setting.relative = relative;
        }
    }

    pub fn is_relative(&self, index: usize) -> bool {
        self.settings
            .get(index)
            .map(|setting| setting.relative)
            .unwrap_or(true)
    }

    pub fn set_additive(&mut self, index: usize, additive: bool) {
        if let Some(setting) = self.settings.get_mut(index) {
            setting.additive = additive;
        }
    }

    pub fn is_additive(&self, index: usize) -> bool {
        self.settings
            .get(index)
            .map(|setting| setting.additive)
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bone_target_defaults() {
        let mut target = ModifierBoneTarget3DData::new();
        assert_eq!(target.get_bone_name(), "");
        assert_eq!(target.get_bone(), -1);
        target.set_bone_name("Hips");
        target.set_bone(2);
        assert_eq!(target.get_bone_name(), "Hips");
        assert_eq!(target.get_bone(), 2);
    }

    #[test]
    fn aim_settings() {
        let mut aim = AimModifier3DData::new();
        aim.set_setting_count(2);
        assert_eq!(aim.get_setting_count(), 2);
        assert_eq!(aim.base().get_setting_count(), 2);
        assert_eq!(aim.get_forward_axis(0), BoneAxis::PlusY);
        assert!(!aim.is_using_euler(0));
        assert_eq!(aim.get_primary_rotation_axis(0), 0);
        assert!(aim.is_using_secondary_rotation(0));
        assert!(aim.is_relative(0));

        aim.set_forward_axis(0, 4);
        assert_eq!(aim.get_forward_axis(0), BoneAxis::MinusY);
        aim.set_forward_axis(0, 9); // ignored
        assert_eq!(aim.get_forward_axis(0), BoneAxis::MinusY);
        aim.set_use_euler(0, true);
        aim.set_primary_rotation_axis(0, 2);
        aim.set_use_secondary_rotation(0, false);
        aim.set_relative(0, false);
        assert!(aim.is_using_euler(0));
        assert_eq!(aim.get_primary_rotation_axis(0), 2);
        assert!(!aim.is_using_secondary_rotation(0));
        assert!(!aim.is_relative(0));
        // Out of range keeps the defaults.
        assert_eq!(aim.get_forward_axis(5), BoneAxis::PlusY);
        aim.base_mut().set_amount(0, 0.5);
        assert_eq!(aim.base().get_amount(0), 0.5);
        aim.clear_settings();
        assert_eq!(aim.get_setting_count(), 0);
        assert_eq!(aim.base().get_setting_count(), 0);
        assert_eq!(BoneAxis::PlusX as i32, 0);
        assert_eq!(BoneAxis::MinusZ as i32, 5);
    }

    #[test]
    fn copy_transform_settings() {
        let mut copy = CopyTransformModifier3DData::new();
        copy.set_setting_count(1);
        assert_eq!(copy.get_copy_flags(0), TransformFlag::All as i32);
        assert_eq!(copy.get_axis_flags(0), AxisFlag::All as i32);
        assert_eq!(copy.get_invert_flags(0), 0);
        assert!(copy.is_relative(0));
        assert!(!copy.is_additive(0));

        copy.set_copy_flags(0, TransformFlag::Position as i32);
        copy.set_axis_flags(0, AxisFlag::X as i32 | AxisFlag::Z as i32);
        copy.set_invert_flags(0, AxisFlag::Y as i32);
        copy.set_relative(0, false);
        copy.set_additive(0, true);
        assert_eq!(copy.get_copy_flags(0), 1);
        assert_eq!(copy.get_axis_flags(0), 5);
        assert_eq!(copy.get_invert_flags(0), 2);
        assert!(!copy.is_relative(0));
        assert!(copy.is_additive(0));
        assert_eq!(TransformFlag::Position as i32, 1);
        assert_eq!(TransformFlag::Rotation as i32, 2);
        assert_eq!(TransformFlag::Scale as i32, 4);
        assert_eq!(AxisFlag::All as i32, 7);
        copy.clear_settings();
        assert_eq!(copy.get_setting_count(), 0);
    }

    #[test]
    fn convert_transform_settings() {
        let mut convert = ConvertTransformModifier3DData::new();
        convert.set_setting_count(1);
        assert_eq!(convert.get_apply_transform_mode(0), TransformMode::Position);
        assert_eq!(convert.get_apply_axis(0), 0);
        assert_eq!(convert.get_apply_range_min(0), 0.0);
        assert_eq!(convert.get_apply_range_max(0), 0.0);
        assert_eq!(
            convert.get_reference_transform_mode(0),
            TransformMode::Position
        );
        assert!(convert.is_relative(0));
        assert!(!convert.is_additive(0));

        convert.set_apply_transform_mode(0, 1);
        convert.set_apply_axis(0, 1);
        convert.set_apply_range_min(0, -1.5);
        convert.set_apply_range_max(0, 2.5);
        convert.set_reference_transform_mode(0, 2);
        convert.set_reference_axis(0, 2);
        convert.set_reference_range_min(0, 0.25);
        convert.set_reference_range_max(0, 0.75);
        convert.set_relative(0, false);
        convert.set_additive(0, true);

        assert_eq!(convert.get_apply_transform_mode(0), TransformMode::Rotation);
        assert_eq!(convert.get_apply_axis(0), 1);
        assert_eq!(convert.get_apply_range_min(0), -1.5);
        assert_eq!(convert.get_apply_range_max(0), 2.5);
        assert_eq!(
            convert.get_reference_transform_mode(0),
            TransformMode::Scale
        );
        assert_eq!(convert.get_reference_axis(0), 2);
        assert_eq!(convert.get_reference_range_min(0), 0.25);
        assert_eq!(convert.get_reference_range_max(0), 0.75);
        assert!(!convert.is_relative(0));
        assert!(convert.is_additive(0));
        // Invalid modes keep the previous value.
        convert.set_apply_transform_mode(0, 7);
        assert_eq!(convert.get_apply_transform_mode(0), TransformMode::Rotation);
        assert_eq!(TransformMode::Scale as i32, 2);
    }
}
