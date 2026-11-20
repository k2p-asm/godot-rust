//! Small resource states: `SkeletonProfile`, `BoneMap`, `CanvasItemMaterial`
//! and `LabelSettings`.
//!
//! - `SkeletonProfile`: root/scale-base bone names, the group list (name +
//!   texture handle), the bone list (name, parent, tail direction/tail,
//!   reference pose, handle offset, group, required) and the lookups
//!   (`find_bone`, `has_bone`, `get_bone_names`), with `is_read_only` for the
//!   humanoid subclass.
//! - `BoneMap`: the profile handle and the profile-bone → skeleton-bone map with
//!   the lookups and reverse lookup.
//! - `CanvasItemMaterial`: blend mode (mix), light mode (normal), the particles
//!   animation flag and its H/V frame counts and loop flag.
//! - `LabelSettings`: line/paragraph spacing (3/0), font size 16, white font and
//!   outline colours, zero outline size, the 1 px shadow with its transparent
//!   colour and (1, 1) offset, plus the stacked outline/shadow lists.
//!
//! Shader/material compilation and font handling are not ported.

use crate::math::color::Color;
use crate::math::transform_3d::Transform3D;
use crate::math::vector2::Vector2;
use crate::servers::rid::Rid;

// -- SkeletonProfile --------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum TailDirection {
    AverageChildren = 0,
    SpecificChild = 1,
    End = 2,
}

#[derive(Clone, Debug)]
pub struct SkeletonProfileGroup {
    pub group_name: String,
    pub texture: Option<Rid>,
}

impl Default for SkeletonProfileGroup {
    fn default() -> Self {
        Self {
            group_name: String::new(),
            texture: None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct SkeletonProfileBone {
    pub bone_name: String,
    pub bone_parent: String,
    pub tail_direction: TailDirection,
    pub bone_tail: String,
    pub reference_pose: Transform3D,
    pub handle_offset: Vector2,
    pub group: String,
    pub required: bool,
}

impl Default for SkeletonProfileBone {
    fn default() -> Self {
        Self {
            bone_name: String::new(),
            bone_parent: String::new(),
            tail_direction: TailDirection::AverageChildren,
            bone_tail: String::new(),
            reference_pose: Transform3D::default(),
            handle_offset: Vector2::ZERO,
            group: String::new(),
            required: false,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct SkeletonProfileData {
    is_read_only: bool,
    root_bone: String,
    scale_base_bone: String,
    groups: Vec<SkeletonProfileGroup>,
    bones: Vec<SkeletonProfileBone>,
}

impl SkeletonProfileData {
    pub fn new() -> Self {
        Self::default()
    }

    /// `SkeletonProfileHumanoid` is read-only.
    pub fn new_read_only() -> Self {
        Self {
            is_read_only: true,
            ..Self::default()
        }
    }

    pub fn is_read_only(&self) -> bool {
        self.is_read_only
    }

    pub fn get_root_bone(&self) -> &str {
        &self.root_bone
    }

    pub fn set_root_bone(&mut self, name: &str) {
        self.root_bone = name.to_string();
    }

    pub fn get_scale_base_bone(&self) -> &str {
        &self.scale_base_bone
    }

    pub fn set_scale_base_bone(&mut self, name: &str) {
        self.scale_base_bone = name.to_string();
    }

    pub fn get_group_size(&self) -> usize {
        self.groups.len()
    }

    /// `set_group_size`: grows with default groups, shrinks by truncation.
    pub fn set_group_size(&mut self, size: usize) {
        self.groups.resize_with(size, SkeletonProfileGroup::default);
    }

    pub fn get_group_name(&self, index: usize) -> &str {
        self.groups
            .get(index)
            .map(|group| group.group_name.as_str())
            .unwrap_or("")
    }

    pub fn set_group_name(&mut self, index: usize, name: &str) {
        if let Some(group) = self.groups.get_mut(index) {
            group.group_name = name.to_string();
        }
    }

    pub fn get_texture(&self, index: usize) -> Option<Rid> {
        self.groups.get(index).and_then(|group| group.texture)
    }

    pub fn set_texture(&mut self, index: usize, texture: Option<Rid>) {
        if let Some(group) = self.groups.get_mut(index) {
            group.texture = texture;
        }
    }

    pub fn get_bone_size(&self) -> usize {
        self.bones.len()
    }

    pub fn set_bone_size(&mut self, size: usize) {
        self.bones.resize_with(size, SkeletonProfileBone::default);
    }

    /// `find_bone`: index by name, `None` when absent.
    pub fn find_bone(&self, name: &str) -> Option<usize> {
        self.bones.iter().position(|bone| bone.bone_name == name)
    }

    /// `has_bone`.
    pub fn has_bone(&self, name: &str) -> bool {
        self.find_bone(name).is_some()
    }

    /// `get_bone_names`: the names in order, empty entries included.
    pub fn get_bone_names(&self) -> Vec<String> {
        self.bones.iter().map(|bone| bone.bone_name.clone()).collect()
    }

    pub fn get_bone_name(&self, index: usize) -> &str {
        self.bones
            .get(index)
            .map(|bone| bone.bone_name.as_str())
            .unwrap_or("")
    }

    pub fn set_bone_name(&mut self, index: usize, name: &str) {
        if let Some(bone) = self.bones.get_mut(index) {
            bone.bone_name = name.to_string();
        }
    }

    pub fn get_bone_parent(&self, index: usize) -> &str {
        self.bones
            .get(index)
            .map(|bone| bone.bone_parent.as_str())
            .unwrap_or("")
    }

    pub fn set_bone_parent(&mut self, index: usize, parent: &str) {
        if let Some(bone) = self.bones.get_mut(index) {
            bone.bone_parent = parent.to_string();
        }
    }

    pub fn get_tail_direction(&self, index: usize) -> TailDirection {
        self.bones
            .get(index)
            .map(|bone| bone.tail_direction)
            .unwrap_or(TailDirection::AverageChildren)
    }

    pub fn set_tail_direction(&mut self, index: usize, direction: TailDirection) {
        if let Some(bone) = self.bones.get_mut(index) {
            bone.tail_direction = direction;
        }
    }

    pub fn get_bone_tail(&self, index: usize) -> &str {
        self.bones
            .get(index)
            .map(|bone| bone.bone_tail.as_str())
            .unwrap_or("")
    }

    pub fn set_bone_tail(&mut self, index: usize, tail: &str) {
        if let Some(bone) = self.bones.get_mut(index) {
            bone.bone_tail = tail.to_string();
        }
    }

    pub fn get_reference_pose(&self, index: usize) -> Transform3D {
        self.bones
            .get(index)
            .map(|bone| bone.reference_pose)
            .unwrap_or_default()
    }

    pub fn set_reference_pose(&mut self, index: usize, pose: Transform3D) {
        if let Some(bone) = self.bones.get_mut(index) {
            bone.reference_pose = pose;
        }
    }

    pub fn get_handle_offset(&self, index: usize) -> Vector2 {
        self.bones
            .get(index)
            .map(|bone| bone.handle_offset)
            .unwrap_or(Vector2::ZERO)
    }

    pub fn set_handle_offset(&mut self, index: usize, offset: Vector2) {
        if let Some(bone) = self.bones.get_mut(index) {
            bone.handle_offset = offset;
        }
    }

    pub fn get_group(&self, index: usize) -> &str {
        self.bones
            .get(index)
            .map(|bone| bone.group.as_str())
            .unwrap_or("")
    }

    pub fn set_group(&mut self, index: usize, group: &str) {
        if let Some(bone) = self.bones.get_mut(index) {
            bone.group = group.to_string();
        }
    }

    pub fn is_required(&self, index: usize) -> bool {
        self.bones
            .get(index)
            .map(|bone| bone.required)
            .unwrap_or(false)
    }

    pub fn set_required(&mut self, index: usize, required: bool) {
        if let Some(bone) = self.bones.get_mut(index) {
            bone.required = required;
        }
    }
}

// -- BoneMap ----------------------------------------------------------------

#[derive(Clone, Debug, Default)]
pub struct BoneMapData {
    profile: Option<Rid>,
    bone_map: Vec<(String, String)>,
}

impl BoneMapData {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_profile(&self) -> Option<Rid> {
        self.profile
    }

    pub fn set_profile(&mut self, profile: Option<Rid>) {
        self.profile = profile;
    }

    /// `get_skeleton_bone_name`: the mapped name, or empty.
    pub fn get_skeleton_bone_name(&self, profile_bone: &str) -> &str {
        self.bone_map
            .iter()
            .find(|(profile, _)| profile == profile_bone)
            .map(|(_, skeleton)| skeleton.as_str())
            .unwrap_or("")
    }

    /// `set_skeleton_bone_name`: last write wins; empty removes the entry.
    pub fn set_skeleton_bone_name(&mut self, profile_bone: &str, skeleton_bone: &str) {
        if skeleton_bone.is_empty() {
            self.bone_map
                .retain(|(profile, _)| profile != profile_bone);
            return;
        }
        match self
            .bone_map
            .iter_mut()
            .find(|(profile, _)| profile == profile_bone)
        {
            Some(entry) => entry.1 = skeleton_bone.to_string(),
            None => self
                .bone_map
                .push((profile_bone.to_string(), skeleton_bone.to_string())),
        }
    }

    /// `get_skeleton_bone_name_count`.
    pub fn get_skeleton_bone_name_count(&self, skeleton_bone: &str) -> usize {
        self.bone_map
            .iter()
            .filter(|(_, skeleton)| skeleton == skeleton_bone)
            .count()
    }

    /// `find_profile_bone_name`: the first profile bone mapped to the skeleton
    /// bone, or empty.
    pub fn find_profile_bone_name(&self, skeleton_bone: &str) -> &str {
        self.bone_map
            .iter()
            .find(|(_, skeleton)| skeleton == skeleton_bone)
            .map(|(profile, _)| profile.as_str())
            .unwrap_or("")
    }

    pub fn get_map(&self) -> &[(String, String)] {
        &self.bone_map
    }
}

// -- CanvasItemMaterial -----------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum BlendMode {
    Mix = 0,
    Add = 1,
    Sub = 2,
    Mul = 3,
    PremultAlpha = 4,
    Disabled = 5,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum LightMode {
    Normal = 0,
    Unshaded = 1,
    LightOnly = 2,
}

#[derive(Clone, Debug)]
pub struct CanvasItemMaterialData {
    blend_mode: BlendMode,
    light_mode: LightMode,
    particles_animation: bool,
    particles_anim_h_frames: i32,
    particles_anim_v_frames: i32,
    particles_anim_loop: bool,
}

impl Default for CanvasItemMaterialData {
    fn default() -> Self {
        Self::new()
    }
}

impl CanvasItemMaterialData {
    pub fn new() -> Self {
        Self {
            blend_mode: BlendMode::Mix,
            light_mode: LightMode::Normal,
            particles_animation: false,
            particles_anim_h_frames: 1,
            particles_anim_v_frames: 1,
            particles_anim_loop: true,
        }
    }

    pub fn set_blend_mode(&mut self, mode: i32) -> bool {
        let parsed = match mode {
            0 => BlendMode::Mix,
            1 => BlendMode::Add,
            2 => BlendMode::Sub,
            3 => BlendMode::Mul,
            4 => BlendMode::PremultAlpha,
            5 => BlendMode::Disabled,
            _ => return false,
        };
        self.blend_mode = parsed;
        true
    }

    pub fn get_blend_mode(&self) -> BlendMode {
        self.blend_mode
    }

    pub fn set_light_mode(&mut self, mode: i32) -> bool {
        let parsed = match mode {
            0 => LightMode::Normal,
            1 => LightMode::Unshaded,
            2 => LightMode::LightOnly,
            _ => return false,
        };
        self.light_mode = parsed;
        true
    }

    pub fn get_light_mode(&self) -> LightMode {
        self.light_mode
    }

    pub fn set_particles_animation(&mut self, enabled: bool) {
        self.particles_animation = enabled;
    }

    pub fn get_particles_animation(&self) -> bool {
        self.particles_animation
    }

    pub fn set_particles_anim_h_frames(&mut self, frames: i32) {
        self.particles_anim_h_frames = frames;
    }

    pub fn get_particles_anim_h_frames(&self) -> i32 {
        self.particles_anim_h_frames
    }

    pub fn set_particles_anim_v_frames(&mut self, frames: i32) {
        self.particles_anim_v_frames = frames;
    }

    pub fn get_particles_anim_v_frames(&self) -> i32 {
        self.particles_anim_v_frames
    }

    pub fn set_particles_anim_loop(&mut self, looping: bool) {
        self.particles_anim_loop = looping;
    }

    pub fn get_particles_anim_loop(&self) -> bool {
        self.particles_anim_loop
    }
}

// -- LabelSettings ----------------------------------------------------------

#[derive(Clone, Debug)]
pub struct StackedOutlineData {
    pub size: i32,
    pub color: Color,
}

impl Default for StackedOutlineData {
    fn default() -> Self {
        Self {
            size: 0,
            color: Color::new(1.0, 1.0, 1.0, 1.0),
        }
    }
}

#[derive(Clone, Debug)]
pub struct StackedShadowData {
    pub offset: Vector2,
    pub color: Color,
    pub outline_size: i32,
}

impl Default for StackedShadowData {
    fn default() -> Self {
        Self {
            offset: Vector2::new(1.0, 1.0),
            color: Color::new(0.0, 0.0, 0.0, 0.0),
            outline_size: 0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct LabelSettingsData {
    line_spacing: f32,
    paragraph_spacing: f32,
    font: Option<Rid>,
    font_size: i32,
    font_color: Color,
    outline_size: i32,
    outline_color: Color,
    shadow_size: i32,
    shadow_color: Color,
    shadow_offset: Vector2,
    stacked_outline_data: Vec<StackedOutlineData>,
    stacked_shadow_data: Vec<StackedShadowData>,
}

impl Default for LabelSettingsData {
    fn default() -> Self {
        Self::new()
    }
}

impl LabelSettingsData {
    pub fn new() -> Self {
        Self {
            line_spacing: 3.0,
            paragraph_spacing: 0.0,
            font: None,
            // `Font::DEFAULT_FONT_SIZE`.
            font_size: 16,
            font_color: Color::new(1.0, 1.0, 1.0, 1.0),
            outline_size: 0,
            outline_color: Color::new(1.0, 1.0, 1.0, 1.0),
            shadow_size: 1,
            shadow_color: Color::new(0.0, 0.0, 0.0, 0.0),
            shadow_offset: Vector2::new(1.0, 1.0),
            stacked_outline_data: Vec::new(),
            stacked_shadow_data: Vec::new(),
        }
    }

    pub fn set_line_spacing(&mut self, spacing: f32) {
        self.line_spacing = spacing;
    }

    pub fn get_line_spacing(&self) -> f32 {
        self.line_spacing
    }

    pub fn set_paragraph_spacing(&mut self, spacing: f32) {
        self.paragraph_spacing = spacing;
    }

    pub fn get_paragraph_spacing(&self) -> f32 {
        self.paragraph_spacing
    }

    pub fn set_font(&mut self, font: Option<Rid>) {
        self.font = font;
    }

    pub fn get_font(&self) -> Option<Rid> {
        self.font
    }

    pub fn set_font_size(&mut self, size: i32) {
        self.font_size = size;
    }

    pub fn get_font_size(&self) -> i32 {
        self.font_size
    }

    pub fn set_font_color(&mut self, color: Color) {
        self.font_color = color;
    }

    pub fn get_font_color(&self) -> Color {
        self.font_color
    }

    pub fn set_outline_size(&mut self, size: i32) {
        self.outline_size = size;
    }

    pub fn get_outline_size(&self) -> i32 {
        self.outline_size
    }

    pub fn set_outline_color(&mut self, color: Color) {
        self.outline_color = color;
    }

    pub fn get_outline_color(&self) -> Color {
        self.outline_color
    }

    pub fn set_shadow_size(&mut self, size: i32) {
        self.shadow_size = size;
    }

    pub fn get_shadow_size(&self) -> i32 {
        self.shadow_size
    }

    pub fn set_shadow_color(&mut self, color: Color) {
        self.shadow_color = color;
    }

    pub fn get_shadow_color(&self) -> Color {
        self.shadow_color
    }

    pub fn set_shadow_offset(&mut self, offset: Vector2) {
        self.shadow_offset = offset;
    }

    pub fn get_shadow_offset(&self) -> Vector2 {
        self.shadow_offset
    }

    pub fn set_stacked_outline_count(&mut self, count: usize) {
        self.stacked_outline_data
            .resize_with(count, StackedOutlineData::default);
    }

    pub fn get_stacked_outline_count(&self) -> usize {
        self.stacked_outline_data.len()
    }

    pub fn get_stacked_outline(&self, index: usize) -> Option<&StackedOutlineData> {
        self.stacked_outline_data.get(index)
    }

    pub fn set_stacked_outline_size(&mut self, index: usize, size: i32) {
        if let Some(outline) = self.stacked_outline_data.get_mut(index) {
            outline.size = size;
        }
    }

    pub fn set_stacked_outline_color(&mut self, index: usize, color: Color) {
        if let Some(outline) = self.stacked_outline_data.get_mut(index) {
            outline.color = color;
        }
    }

    pub fn set_stacked_shadow_count(&mut self, count: usize) {
        self.stacked_shadow_data
            .resize_with(count, StackedShadowData::default);
    }

    pub fn get_stacked_shadow_count(&self) -> usize {
        self.stacked_shadow_data.len()
    }

    pub fn get_stacked_shadow(&self, index: usize) -> Option<&StackedShadowData> {
        self.stacked_shadow_data.get(index)
    }

    pub fn set_stacked_shadow_offset(&mut self, index: usize, offset: Vector2) {
        if let Some(shadow) = self.stacked_shadow_data.get_mut(index) {
            shadow.offset = offset;
        }
    }

    pub fn set_stacked_shadow_color(&mut self, index: usize, color: Color) {
        if let Some(shadow) = self.stacked_shadow_data.get_mut(index) {
            shadow.color = color;
        }
    }

    pub fn set_stacked_shadow_outline_size(&mut self, index: usize, size: i32) {
        if let Some(shadow) = self.stacked_shadow_data.get_mut(index) {
            shadow.outline_size = size;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skeleton_profile_state() {
        let mut profile = SkeletonProfileData::new();
        assert!(!profile.is_read_only());
        assert_eq!(profile.get_root_bone(), "");
        assert_eq!(profile.get_scale_base_bone(), "");

        profile.set_root_bone("Root");
        profile.set_scale_base_bone("Hips");
        assert_eq!(profile.get_root_bone(), "Root");
        assert_eq!(profile.get_scale_base_bone(), "Hips");

        profile.set_group_size(2);
        assert_eq!(profile.get_group_size(), 2);
        profile.set_group_name(0, "Body");
        profile.set_texture(0, Some(Rid::from_u64(3)));
        assert_eq!(profile.get_group_name(0), "Body");
        assert_eq!(profile.get_texture(0), Some(Rid::from_u64(3)));
        assert_eq!(profile.get_group_name(5), "");

        profile.set_bone_size(2);
        assert_eq!(profile.get_bone_size(), 2);
        profile.set_bone_name(0, "Hips");
        profile.set_bone_parent(0, "Root");
        profile.set_tail_direction(0, TailDirection::End);
        profile.set_bone_tail(0, "HipsTail");
        profile.set_handle_offset(0, Vector2::new(4.0, 5.0));
        profile.set_group(0, "Body");
        profile.set_required(0, true);
        assert_eq!(profile.get_bone_name(0), "Hips");
        assert_eq!(profile.get_bone_parent(0), "Root");
        assert_eq!(profile.get_tail_direction(0), TailDirection::End);
        assert_eq!(profile.get_bone_tail(0), "HipsTail");
        assert_eq!(profile.get_handle_offset(0), Vector2::new(4.0, 5.0));
        assert_eq!(profile.get_group(0), "Body");
        assert!(profile.is_required(0));
        assert_eq!(profile.find_bone("Hips"), Some(0));
        assert!(profile.has_bone("Hips"));
        assert!(!profile.has_bone("Head"));
        assert_eq!(profile.get_bone_names(), vec!["Hips", ""]);
        assert_eq!(TailDirection::AverageChildren as i32, 0);
        assert_eq!(TailDirection::SpecificChild as i32, 1);

        let humanoid = SkeletonProfileData::new_read_only();
        assert!(humanoid.is_read_only());
    }

    #[test]
    fn bone_map_state() {
        let mut map = BoneMapData::new();
        assert!(map.get_profile().is_none());
        assert_eq!(map.get_skeleton_bone_name("Hips"), "");
        assert_eq!(map.find_profile_bone_name("A"), "");

        map.set_profile(Some(Rid::from_u64(9)));
        map.set_skeleton_bone_name("Hips", "pelvis");
        map.set_skeleton_bone_name("Head", "head");
        map.set_skeleton_bone_name("Hips", "pelvis_2");
        assert_eq!(map.get_profile(), Some(Rid::from_u64(9)));
        assert_eq!(map.get_skeleton_bone_name("Hips"), "pelvis_2");
        assert_eq!(map.find_profile_bone_name("head"), "Head");
        assert_eq!(map.get_map().len(), 2);

        // Duplicate skeleton names are counted.
        map.set_skeleton_bone_name("Extra", "head");
        assert_eq!(map.get_skeleton_bone_name_count("head"), 2);
        assert_eq!(map.find_profile_bone_name("head"), "Head");

        // An empty skeleton name removes the mapping.
        map.set_skeleton_bone_name("Head", "");
        assert_eq!(map.get_skeleton_bone_name("Head"), "");
        assert_eq!(map.get_skeleton_bone_name_count("head"), 1);
    }

    #[test]
    fn canvas_item_material_state() {
        let mut material = CanvasItemMaterialData::new();
        assert_eq!(material.get_blend_mode(), BlendMode::Mix);
        assert_eq!(material.get_light_mode(), LightMode::Normal);
        assert!(!material.get_particles_animation());
        assert_eq!(material.get_particles_anim_h_frames(), 1);
        assert_eq!(material.get_particles_anim_v_frames(), 1);
        assert!(material.get_particles_anim_loop());

        assert!(material.set_blend_mode(4));
        assert!(!material.set_blend_mode(9));
        assert!(material.set_light_mode(2));
        assert!(!material.set_light_mode(3));
        material.set_particles_animation(true);
        material.set_particles_anim_h_frames(4);
        material.set_particles_anim_v_frames(2);
        material.set_particles_anim_loop(false);
        assert_eq!(material.get_blend_mode(), BlendMode::PremultAlpha);
        assert_eq!(material.get_light_mode(), LightMode::LightOnly);
        assert!(material.get_particles_animation());
        assert_eq!(material.get_particles_anim_h_frames(), 4);
        assert_eq!(material.get_particles_anim_v_frames(), 2);
        assert!(!material.get_particles_anim_loop());
        assert_eq!(BlendMode::Disabled as i32, 5);
        assert_eq!(LightMode::Unshaded as i32, 1);
    }

    #[test]
    fn label_settings_state() {
        let mut settings = LabelSettingsData::new();
        assert_eq!(settings.get_line_spacing(), 3.0);
        assert_eq!(settings.get_paragraph_spacing(), 0.0);
        assert!(settings.get_font().is_none());
        assert_eq!(settings.get_font_size(), 16);
        assert_eq!(settings.get_font_color(), Color::new(1.0, 1.0, 1.0, 1.0));
        assert_eq!(settings.get_outline_size(), 0);
        assert_eq!(settings.get_outline_color(), Color::new(1.0, 1.0, 1.0, 1.0));
        assert_eq!(settings.get_shadow_size(), 1);
        assert_eq!(settings.get_shadow_color(), Color::new(0.0, 0.0, 0.0, 0.0));
        assert_eq!(settings.get_shadow_offset(), Vector2::new(1.0, 1.0));
        assert_eq!(settings.get_stacked_outline_count(), 0);
        assert_eq!(settings.get_stacked_shadow_count(), 0);

        settings.set_line_spacing(1.5);
        settings.set_paragraph_spacing(2.0);
        settings.set_font(Some(Rid::from_u64(2)));
        settings.set_font_size(24);
        settings.set_font_color(Color::new(0.1, 0.2, 0.3, 1.0));
        settings.set_outline_size(2);
        settings.set_outline_color(Color::new(0.0, 0.0, 0.0, 1.0));
        settings.set_shadow_size(3);
        settings.set_shadow_color(Color::new(0.0, 0.0, 0.0, 0.5));
        settings.set_shadow_offset(Vector2::new(2.0, 3.0));
        assert_eq!(settings.get_line_spacing(), 1.5);
        assert_eq!(settings.get_paragraph_spacing(), 2.0);
        assert_eq!(settings.get_font(), Some(Rid::from_u64(2)));
        assert_eq!(settings.get_font_size(), 24);
        assert_eq!(settings.get_font_color(), Color::new(0.1, 0.2, 0.3, 1.0));
        assert_eq!(settings.get_outline_size(), 2);
        assert_eq!(settings.get_outline_color(), Color::new(0.0, 0.0, 0.0, 1.0));
        assert_eq!(settings.get_shadow_size(), 3);
        assert_eq!(settings.get_shadow_color(), Color::new(0.0, 0.0, 0.0, 0.5));
        assert_eq!(settings.get_shadow_offset(), Vector2::new(2.0, 3.0));

        settings.set_stacked_outline_count(2);
        settings.set_stacked_outline_size(1, 4);
        settings.set_stacked_outline_color(1, Color::new(1.0, 0.0, 0.0, 1.0));
        assert_eq!(settings.get_stacked_outline_count(), 2);
        let outline = settings.get_stacked_outline(1).expect("outline");
        assert_eq!(outline.size, 4);
        assert_eq!(outline.color, Color::new(1.0, 0.0, 0.0, 1.0));

        settings.set_stacked_shadow_count(1);
        settings.set_stacked_shadow_offset(0, Vector2::new(4.0, 4.0));
        settings.set_stacked_shadow_color(0, Color::new(0.0, 0.5, 0.0, 1.0));
        settings.set_stacked_shadow_outline_size(0, 2);
        let shadow = settings.get_stacked_shadow(0).expect("shadow");
        assert_eq!(shadow.offset, Vector2::new(4.0, 4.0));
        assert_eq!(shadow.color, Color::new(0.0, 0.5, 0.0, 1.0));
        assert_eq!(shadow.outline_size, 2);
    }
}
