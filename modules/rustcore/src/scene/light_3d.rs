//! `scene/3d/light_3d.*`: light parameters and subclass defaults.
//!
//! `Light3D` keeps the 21 engine parameters (`PARAM_ENERGY` = 0 ...
//! `PARAM_INTENSITY` = 20) with the constructor defaults: energy/indirect/fog 1,
//! specular 0.5, range 5, size 0, attenuation 1, spot angle 45, spot
//! attenuation 1, split offsets 0.1/0.2/0.5, fade start 1, pancake 20, shadow
//! opacity/blur 1, bias 0.1, normal bias 1, transmittance 0.05, intensity 1000.
//! Also white colour, shadows/negative/reverse-cull off, cull and shadow-caster
//! masks `0xffffffff`, distance fade off (40/50/10), editor-only off, dynamic
//! bake mode, temperature 6500 and correlated colour white.
//!
//! Subclasses: directional overrides max distance 100, fade start 0.8, normal
//! bias 2, intensity 100000, specular 1 and uses 4 shadow splits with sky mode
//! light+sky; omni uses cube shadows; spot overrides the shadow bias to 0.03.
//!
//! RenderingServer RIDs and the physical-units colour combination are not
//! ported.

use crate::math::color::Color;
use crate::scene::visual_instance_3d::VisualInstance3DData;
use crate::servers::rid::Rid;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum LightType {
    Directional,
    Omni,
    Spot,
    Area,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum Param {
    Energy,
    IndirectEnergy,
    VolumetricFogEnergy,
    Specular,
    Range,
    Size,
    Attenuation,
    SpotAngle,
    SpotAttenuation,
    ShadowMaxDistance,
    ShadowSplit1Offset,
    ShadowSplit2Offset,
    ShadowSplit3Offset,
    ShadowFadeStart,
    ShadowNormalBias,
    ShadowBias,
    ShadowPancakeSize,
    ShadowOpacity,
    ShadowBlur,
    TransmittanceBias,
    Intensity,
    Max,
}

pub const PARAM_MAX: usize = 21;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum BakeMode {
    Disabled,
    Static,
    Dynamic,
}

#[derive(Clone, Debug)]
pub struct Light3DData {
    pub visual: VisualInstance3DData,
    light_type: LightType,
    color: Color,
    params: [f32; PARAM_MAX],
    shadow: bool,
    negative: bool,
    reverse_cull: bool,
    cull_mask: u32,
    shadow_caster_mask: u32,
    distance_fade_enabled: bool,
    distance_fade_begin: f32,
    distance_fade_shadow: f32,
    distance_fade_length: f32,
    editor_only: bool,
    bake_mode: BakeMode,
    projector: Option<Rid>,
    correlated_color: Color,
    temperature: f32,
}

impl Light3DData {
    /// `Light3D::Light3D(type)` defaults from the engine constructor.
    pub fn new(light_type: LightType) -> Self {
        let mut params = [0.0f32; PARAM_MAX];
        params[Param::Energy as usize] = 1.0;
        params[Param::IndirectEnergy as usize] = 1.0;
        params[Param::VolumetricFogEnergy as usize] = 1.0;
        params[Param::Specular as usize] = 0.5;
        params[Param::Range as usize] = 5.0;
        params[Param::Size as usize] = 0.0;
        params[Param::Attenuation as usize] = 1.0;
        params[Param::SpotAngle as usize] = 45.0;
        params[Param::SpotAttenuation as usize] = 1.0;
        params[Param::ShadowMaxDistance as usize] = 0.0;
        params[Param::ShadowSplit1Offset as usize] = 0.1;
        params[Param::ShadowSplit2Offset as usize] = 0.2;
        params[Param::ShadowSplit3Offset as usize] = 0.5;
        // The constructor sets 0.8 and later 1.
        params[Param::ShadowFadeStart as usize] = 1.0;
        params[Param::ShadowPancakeSize as usize] = 20.0;
        params[Param::ShadowOpacity as usize] = 1.0;
        params[Param::ShadowBlur as usize] = 1.0;
        params[Param::ShadowBias as usize] = 0.1;
        params[Param::ShadowNormalBias as usize] = 1.0;
        params[Param::TransmittanceBias as usize] = 0.05;
        params[Param::Intensity as usize] = 1000.0;

        Self {
            visual: VisualInstance3DData::new(),
            light_type,
            color: Color::new(1.0, 1.0, 1.0, 1.0),
            params,
            shadow: false,
            negative: false,
            reverse_cull: false,
            cull_mask: 0xFFFFFFFF,
            shadow_caster_mask: 0xFFFFFFFF,
            distance_fade_enabled: false,
            distance_fade_begin: 40.0,
            distance_fade_shadow: 50.0,
            distance_fade_length: 10.0,
            editor_only: false,
            bake_mode: BakeMode::Dynamic,
            projector: None,
            correlated_color: Color::new(1.0, 1.0, 1.0, 1.0),
            temperature: 6500.0,
        }
    }

    pub fn get_light_type(&self) -> LightType {
        self.light_type
    }

    pub fn set_color(&mut self, color: Color) {
        self.color = color;
    }

    pub fn get_color(&self) -> Color {
        self.color
    }

    /// The engine validates the index against `PARAM_MAX`.
    pub fn set_param(&mut self, param: Param, value: f32) -> bool {
        if param == Param::Max {
            return false;
        }
        self.params[param as usize] = value;
        true
    }

    pub fn get_param(&self, param: Param) -> Option<f32> {
        if param == Param::Max {
            return None;
        }
        Some(self.params[param as usize])
    }

    pub fn set_shadow(&mut self, enable: bool) {
        self.shadow = enable;
    }

    pub fn has_shadow(&self) -> bool {
        self.shadow
    }

    pub fn set_negative(&mut self, enable: bool) {
        self.negative = enable;
    }

    pub fn is_negative(&self) -> bool {
        self.negative
    }

    pub fn set_shadow_reverse_cull_face(&mut self, enable: bool) {
        self.reverse_cull = enable;
    }

    pub fn get_shadow_reverse_cull_face(&self) -> bool {
        self.reverse_cull
    }

    pub fn set_cull_mask(&mut self, mask: u32) {
        self.cull_mask = mask;
    }

    pub fn get_cull_mask(&self) -> u32 {
        self.cull_mask
    }

    pub fn set_shadow_caster_mask(&mut self, mask: u32) {
        self.shadow_caster_mask = mask;
    }

    pub fn get_shadow_caster_mask(&self) -> u32 {
        self.shadow_caster_mask
    }

    pub fn set_enable_distance_fade(&mut self, enable: bool) {
        self.distance_fade_enabled = enable;
    }

    pub fn is_distance_fade_enabled(&self) -> bool {
        self.distance_fade_enabled
    }

    pub fn set_distance_fade_begin(&mut self, distance: f32) {
        self.distance_fade_begin = distance;
    }

    pub fn get_distance_fade_begin(&self) -> f32 {
        self.distance_fade_begin
    }

    pub fn set_distance_fade_shadow(&mut self, distance: f32) {
        self.distance_fade_shadow = distance;
    }

    pub fn get_distance_fade_shadow(&self) -> f32 {
        self.distance_fade_shadow
    }

    pub fn set_distance_fade_length(&mut self, length: f32) {
        self.distance_fade_length = length;
    }

    pub fn get_distance_fade_length(&self) -> f32 {
        self.distance_fade_length
    }

    pub fn set_editor_only(&mut self, editor_only: bool) {
        self.editor_only = editor_only;
    }

    pub fn is_editor_only(&self) -> bool {
        self.editor_only
    }

    pub fn set_bake_mode(&mut self, mode: BakeMode) {
        self.bake_mode = mode;
    }

    pub fn get_bake_mode(&self) -> BakeMode {
        self.bake_mode
    }

    pub fn set_projector(&mut self, projector: Option<Rid>) {
        self.projector = projector;
    }

    pub fn get_projector(&self) -> Option<Rid> {
        self.projector
    }

    pub fn set_temperature(&mut self, temperature: f32) {
        self.temperature = temperature;
    }

    pub fn get_temperature(&self) -> f32 {
        self.temperature
    }

    pub fn get_correlated_color(&self) -> Color {
        self.correlated_color
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum DirectionalShadowMode {
    Orthogonal,
    Parallel2Splits,
    Parallel4Splits,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum SkyMode {
    LightAndSky,
    LightOnly,
    SkyOnly,
}

#[derive(Clone, Debug)]
pub struct DirectionalLight3DData {
    pub light: Light3DData,
    shadow_mode: DirectionalShadowMode,
    blend_splits: bool,
    sky_mode: SkyMode,
}

impl Default for DirectionalLight3DData {
    fn default() -> Self {
        Self::new()
    }
}

impl DirectionalLight3DData {
    pub fn new() -> Self {
        let mut light = Light3DData::new(LightType::Directional);
        light.set_param(Param::ShadowMaxDistance, 100.0);
        light.set_param(Param::ShadowFadeStart, 0.8);
        light.set_param(Param::ShadowNormalBias, 2.0);
        light.set_param(Param::Intensity, 100000.0);
        light.set_param(Param::Specular, 1.0);
        Self {
            light,
            shadow_mode: DirectionalShadowMode::Parallel4Splits,
            blend_splits: false,
            sky_mode: SkyMode::LightAndSky,
        }
    }

    pub fn set_shadow_mode(&mut self, mode: DirectionalShadowMode) {
        self.shadow_mode = mode;
    }

    pub fn get_shadow_mode(&self) -> DirectionalShadowMode {
        self.shadow_mode
    }

    pub fn set_blend_splits(&mut self, enable: bool) {
        self.blend_splits = enable;
    }

    pub fn is_blend_splits_enabled(&self) -> bool {
        self.blend_splits
    }

    pub fn set_sky_mode(&mut self, mode: SkyMode) {
        self.sky_mode = mode;
    }

    pub fn get_sky_mode(&self) -> SkyMode {
        self.sky_mode
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum OmniShadowMode {
    DualParaboloid,
    Cube,
}

#[derive(Clone, Debug)]
pub struct OmniLight3DData {
    pub light: Light3DData,
    shadow_mode: OmniShadowMode,
}

impl Default for OmniLight3DData {
    fn default() -> Self {
        Self::new()
    }
}

impl OmniLight3DData {
    pub fn new() -> Self {
        Self {
            light: Light3DData::new(LightType::Omni),
            shadow_mode: OmniShadowMode::Cube,
        }
    }

    pub fn set_shadow_mode(&mut self, mode: OmniShadowMode) {
        self.shadow_mode = mode;
    }

    pub fn get_shadow_mode(&self) -> OmniShadowMode {
        self.shadow_mode
    }
}

#[derive(Clone, Debug)]
pub struct SpotLight3DData {
    pub light: Light3DData,
}

impl Default for SpotLight3DData {
    fn default() -> Self {
        Self::new()
    }
}

impl SpotLight3DData {
    pub fn new() -> Self {
        let mut light = Light3DData::new(LightType::Spot);
        light.set_param(Param::ShadowBias, 0.03);
        Self { light }
    }
}

#[derive(Clone, Debug)]
pub struct AreaLight3DData {
    pub light: Light3DData,
    area_size: crate::math::vector2::Vector2,
    area_texture: Option<crate::servers::rid::Rid>,
    area_normalize_energy: bool,
}

impl Default for AreaLight3DData {
    fn default() -> Self {
        Self::new()
    }
}

impl AreaLight3DData {
    pub fn new() -> Self {
        let mut light = Light3DData::new(LightType::Area);
        // The constructor overrides these three params for area lights.
        light.set_param(Param::ShadowBias, 0.1);
        light.set_param(Param::Size, 0.5);
        light.set_param(Param::Specular, 1.0);
        Self {
            light,
            area_size: crate::math::vector2::Vector2::new(1.0, 1.0),
            area_texture: None,
            area_normalize_energy: true,
        }
    }

    pub fn set_area_size(&mut self, size: crate::math::vector2::Vector2) {
        self.area_size = size;
    }

    pub fn get_area_size(&self) -> crate::math::vector2::Vector2 {
        self.area_size
    }

    pub fn set_area_texture(&mut self, texture: Option<crate::servers::rid::Rid>) {
        self.area_texture = texture;
    }

    pub fn get_area_texture(&self) -> Option<crate::servers::rid::Rid> {
        self.area_texture
    }

    pub fn set_area_normalize_energy(&mut self, enabled: bool) {
        self.area_normalize_energy = enabled;
    }

    pub fn is_area_normalizing_energy(&self) -> bool {
        self.area_normalize_energy
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn area_light_defaults() {
        let mut area = AreaLight3DData::new();
        assert_eq!(area.light.get_light_type(), LightType::Area);
        assert_eq!(area.get_area_size(), crate::math::vector2::Vector2::new(1.0, 1.0));
        assert!(area.get_area_texture().is_none());
        assert!(area.is_area_normalizing_energy());
        assert_eq!(area.light.get_param(Param::ShadowBias), Some(0.1));
        assert_eq!(area.light.get_param(Param::Size), Some(0.5));
        assert_eq!(area.light.get_param(Param::Specular), Some(1.0));

        area.set_area_size(crate::math::vector2::Vector2::new(2.0, 3.0));
        area.set_area_texture(Some(crate::servers::rid::Rid::from_u64(4)));
        area.set_area_normalize_energy(false);
        assert_eq!(area.get_area_size(), crate::math::vector2::Vector2::new(2.0, 3.0));
        assert_eq!(area.get_area_texture(), Some(crate::servers::rid::Rid::from_u64(4)));
        assert!(!area.is_area_normalizing_energy());
    }

    #[test]
    fn base_defaults() {
        let l = Light3DData::new(LightType::Omni);
        assert_eq!(l.get_light_type(), LightType::Omni);
        assert_eq!(l.get_color(), Color::new(1.0, 1.0, 1.0, 1.0));
        assert_eq!(l.get_param(Param::Energy), Some(1.0));
        assert_eq!(l.get_param(Param::IndirectEnergy), Some(1.0));
        assert_eq!(l.get_param(Param::VolumetricFogEnergy), Some(1.0));
        assert_eq!(l.get_param(Param::Specular), Some(0.5));
        assert_eq!(l.get_param(Param::Range), Some(5.0));
        assert_eq!(l.get_param(Param::Size), Some(0.0));
        assert_eq!(l.get_param(Param::Attenuation), Some(1.0));
        assert_eq!(l.get_param(Param::SpotAngle), Some(45.0));
        assert_eq!(l.get_param(Param::SpotAttenuation), Some(1.0));
        assert_eq!(l.get_param(Param::ShadowMaxDistance), Some(0.0));
        assert_eq!(l.get_param(Param::ShadowSplit1Offset), Some(0.1));
        assert_eq!(l.get_param(Param::ShadowSplit2Offset), Some(0.2));
        assert_eq!(l.get_param(Param::ShadowSplit3Offset), Some(0.5));
        assert_eq!(l.get_param(Param::ShadowFadeStart), Some(1.0));
        assert_eq!(l.get_param(Param::ShadowPancakeSize), Some(20.0));
        assert_eq!(l.get_param(Param::ShadowOpacity), Some(1.0));
        assert_eq!(l.get_param(Param::ShadowBlur), Some(1.0));
        assert_eq!(l.get_param(Param::ShadowBias), Some(0.1));
        assert_eq!(l.get_param(Param::ShadowNormalBias), Some(1.0));
        assert_eq!(l.get_param(Param::TransmittanceBias), Some(0.05));
        assert_eq!(l.get_param(Param::Intensity), Some(1000.0));
        assert_eq!(l.get_param(Param::Max), None);

        assert!(!l.has_shadow());
        assert!(!l.is_negative());
        assert!(!l.get_shadow_reverse_cull_face());
        assert_eq!(l.get_cull_mask(), 0xFFFFFFFF);
        assert_eq!(l.get_shadow_caster_mask(), 0xFFFFFFFF);
        assert!(!l.is_distance_fade_enabled());
        assert_eq!(l.get_distance_fade_begin(), 40.0);
        assert_eq!(l.get_distance_fade_shadow(), 50.0);
        assert_eq!(l.get_distance_fade_length(), 10.0);
        assert!(!l.is_editor_only());
        assert_eq!(l.get_bake_mode(), BakeMode::Dynamic);
        assert!(l.get_projector().is_none());
        assert_eq!(l.get_temperature(), 6500.0);
        assert_eq!(l.get_correlated_color(), Color::new(1.0, 1.0, 1.0, 1.0));
    }

    #[test]
    fn params_and_flags() {
        let mut l = Light3DData::new(LightType::Directional);
        assert!(l.set_param(Param::Energy, 2.5));
        assert_eq!(l.get_param(Param::Energy), Some(2.5));
        assert!(!l.set_param(Param::Max, 1.0));

        l.set_color(Color::new(0.5, 0.25, 0.125, 1.0));
        l.set_shadow(true);
        l.set_negative(true);
        l.set_shadow_reverse_cull_face(true);
        l.set_cull_mask(0xF);
        l.set_shadow_caster_mask(0x3);
        l.set_enable_distance_fade(true);
        l.set_distance_fade_begin(10.0);
        l.set_distance_fade_shadow(12.0);
        l.set_distance_fade_length(4.0);
        l.set_editor_only(true);
        l.set_bake_mode(BakeMode::Static);
        l.set_projector(Some(Rid::from_u64(9)));
        l.set_temperature(5000.0);

        assert_eq!(l.get_color(), Color::new(0.5, 0.25, 0.125, 1.0));
        assert!(l.has_shadow());
        assert!(l.is_negative());
        assert!(l.get_shadow_reverse_cull_face());
        assert_eq!(l.get_cull_mask(), 0xF);
        assert_eq!(l.get_shadow_caster_mask(), 0x3);
        assert!(l.is_distance_fade_enabled());
        assert_eq!(l.get_distance_fade_begin(), 10.0);
        assert_eq!(l.get_distance_fade_shadow(), 12.0);
        assert_eq!(l.get_distance_fade_length(), 4.0);
        assert!(l.is_editor_only());
        assert_eq!(l.get_bake_mode(), BakeMode::Static);
        assert_eq!(l.get_projector(), Some(Rid::from_u64(9)));
        assert_eq!(l.get_temperature(), 5000.0);

        assert_eq!(BakeMode::Disabled as i32, 0);
        assert_eq!(BakeMode::Static as i32, 1);
        assert_eq!(BakeMode::Dynamic as i32, 2);
    }

    #[test]
    fn subclass_defaults() {
        let d = DirectionalLight3DData::new();
        assert_eq!(d.light.get_light_type(), LightType::Directional);
        assert_eq!(d.light.get_param(Param::ShadowMaxDistance), Some(100.0));
        assert_eq!(d.light.get_param(Param::ShadowFadeStart), Some(0.8));
        assert_eq!(d.light.get_param(Param::ShadowNormalBias), Some(2.0));
        assert_eq!(d.light.get_param(Param::Intensity), Some(100000.0));
        assert_eq!(d.light.get_param(Param::Specular), Some(1.0));
        assert_eq!(d.get_shadow_mode(), DirectionalShadowMode::Parallel4Splits);
        assert!(!d.is_blend_splits_enabled());
        assert_eq!(d.get_sky_mode(), SkyMode::LightAndSky);

        let o = OmniLight3DData::new();
        assert_eq!(o.light.get_light_type(), LightType::Omni);
        assert_eq!(o.get_shadow_mode(), OmniShadowMode::Cube);

        let s = SpotLight3DData::new();
        assert_eq!(s.light.get_light_type(), LightType::Spot);
        assert_eq!(s.light.get_param(Param::ShadowBias), Some(0.03));

        assert_eq!(LightType::Directional as i32, 0);
        assert_eq!(LightType::Omni as i32, 1);
        assert_eq!(LightType::Spot as i32, 2);
        assert_eq!(LightType::Area as i32, 3);
        assert_eq!(Param::Energy as i32, 0);
        assert_eq!(Param::Intensity as i32, 20);
        assert_eq!(Param::Max as i32, 21);
        assert_eq!(DirectionalShadowMode::Orthogonal as i32, 0);
        assert_eq!(DirectionalShadowMode::Parallel2Splits as i32, 1);
        assert_eq!(DirectionalShadowMode::Parallel4Splits as i32, 2);
        assert_eq!(SkyMode::LightAndSky as i32, 0);
        assert_eq!(SkyMode::LightOnly as i32, 1);
        assert_eq!(SkyMode::SkyOnly as i32, 2);
        assert_eq!(OmniShadowMode::DualParaboloid as i32, 0);
        assert_eq!(OmniShadowMode::Cube as i32, 1);
    }
}
