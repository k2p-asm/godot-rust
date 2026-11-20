//! `scene/3d/lightmap_gi.*`: baked lightmap settings (no baking).
//!
//! Defaults from the header: medium bake quality, denoiser on with strength 0.1
//! and range 10, three bounces with indirect energy 1, bias 0.0005, texel scale
//! 1, 16384 max texture size, supersampling off with factor 2, exterior mode,
//! environment mode `SCENE`, white custom colour at energy 1, directional off,
//! bounces read from the texture, no shadowmask, eight probe subdivisions and the
//! camera-attributes/light-data handles.
//!
//! Baking, the lightmap atlas/probe generation and the light-data resource are
//! not ported; every knob is stored and exposed.

use crate::math::color::Color;
use crate::servers::rid::Rid;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum BakeQuality {
    Low,
    Medium,
    High,
    Ultra,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum GenerateProbes {
    Disabled,
    Subdiv4,
    Subdiv8,
    Subdiv16,
    Subdiv32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum EnvironmentMode {
    Disabled,
    Scene,
    CustomSky,
    CustomColor,
}

/// `LightmapGIData::ShadowmaskMode`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum ShadowmaskMode {
    None,
    Replace,
    Overlay,
    Multiply,
}

#[derive(Clone, Debug)]
pub struct LightmapGIData {
    bake_quality: BakeQuality,
    use_denoiser: bool,
    denoiser_strength: f32,
    denoiser_range: i32,
    bounces: i32,
    bounce_indirect_energy: f32,
    bias: f32,
    texel_scale: f32,
    max_texture_size: i32,
    supersampling_enabled: bool,
    supersampling_factor: f32,
    interior: bool,
    environment_mode: EnvironmentMode,
    environment_custom_sky: Option<Rid>,
    environment_custom_color: Color,
    environment_custom_energy: f32,
    directional: bool,
    use_texture_for_bounces: bool,
    shadowmask_mode: ShadowmaskMode,
    gen_probes: GenerateProbes,
    camera_attributes: Option<Rid>,
    light_data: Option<Rid>,
}

impl Default for LightmapGIData {
    fn default() -> Self {
        Self::new()
    }
}

impl LightmapGIData {
    pub fn new() -> Self {
        Self {
            bake_quality: BakeQuality::Medium,
            use_denoiser: true,
            denoiser_strength: 0.1,
            denoiser_range: 10,
            bounces: 3,
            bounce_indirect_energy: 1.0,
            bias: 0.0005,
            texel_scale: 1.0,
            max_texture_size: 16384,
            supersampling_enabled: false,
            supersampling_factor: 2.0,
            interior: false,
            environment_mode: EnvironmentMode::Scene,
            environment_custom_sky: None,
            environment_custom_color: Color::new(1.0, 1.0, 1.0, 1.0),
            environment_custom_energy: 1.0,
            directional: false,
            use_texture_for_bounces: true,
            shadowmask_mode: ShadowmaskMode::None,
            gen_probes: GenerateProbes::Subdiv8,
            camera_attributes: None,
            light_data: None,
        }
    }

    pub fn set_bake_quality(&mut self, quality: BakeQuality) {
        self.bake_quality = quality;
    }

    pub fn get_bake_quality(&self) -> BakeQuality {
        self.bake_quality
    }

    pub fn set_use_denoiser(&mut self, enabled: bool) {
        self.use_denoiser = enabled;
    }

    pub fn is_using_denoiser(&self) -> bool {
        self.use_denoiser
    }

    pub fn set_denoiser_strength(&mut self, strength: f32) {
        self.denoiser_strength = strength;
    }

    pub fn get_denoiser_strength(&self) -> f32 {
        self.denoiser_strength
    }

    pub fn set_denoiser_range(&mut self, range: i32) {
        self.denoiser_range = range;
    }

    pub fn get_denoiser_range(&self) -> i32 {
        self.denoiser_range
    }

    pub fn set_bounces(&mut self, bounces: i32) {
        self.bounces = bounces;
    }

    pub fn get_bounces(&self) -> i32 {
        self.bounces
    }

    pub fn set_bounce_indirect_energy(&mut self, energy: f32) {
        self.bounce_indirect_energy = energy;
    }

    pub fn get_bounce_indirect_energy(&self) -> f32 {
        self.bounce_indirect_energy
    }

    pub fn set_bias(&mut self, bias: f32) {
        self.bias = bias;
    }

    pub fn get_bias(&self) -> f32 {
        self.bias
    }

    pub fn set_texel_scale(&mut self, scale: f32) {
        self.texel_scale = scale;
    }

    pub fn get_texel_scale(&self) -> f32 {
        self.texel_scale
    }

    pub fn set_max_texture_size(&mut self, size: i32) {
        self.max_texture_size = size;
    }

    pub fn get_max_texture_size(&self) -> i32 {
        self.max_texture_size
    }

    pub fn set_supersampling_enabled(&mut self, enabled: bool) {
        self.supersampling_enabled = enabled;
    }

    pub fn is_supersampling_enabled(&self) -> bool {
        self.supersampling_enabled
    }

    pub fn set_supersampling_factor(&mut self, factor: f32) {
        self.supersampling_factor = factor;
    }

    pub fn get_supersampling_factor(&self) -> f32 {
        self.supersampling_factor
    }

    pub fn set_interior(&mut self, interior: bool) {
        self.interior = interior;
    }

    pub fn is_interior(&self) -> bool {
        self.interior
    }

    pub fn set_environment_mode(&mut self, mode: EnvironmentMode) {
        self.environment_mode = mode;
    }

    pub fn get_environment_mode(&self) -> EnvironmentMode {
        self.environment_mode
    }

    pub fn set_environment_custom_sky(&mut self, sky: Option<Rid>) {
        self.environment_custom_sky = sky;
    }

    pub fn get_environment_custom_sky(&self) -> Option<Rid> {
        self.environment_custom_sky
    }

    pub fn set_environment_custom_color(&mut self, color: Color) {
        self.environment_custom_color = color;
    }

    pub fn get_environment_custom_color(&self) -> Color {
        self.environment_custom_color
    }

    pub fn set_environment_custom_energy(&mut self, energy: f32) {
        self.environment_custom_energy = energy;
    }

    pub fn get_environment_custom_energy(&self) -> f32 {
        self.environment_custom_energy
    }

    pub fn set_directional(&mut self, directional: bool) {
        self.directional = directional;
    }

    pub fn is_directional(&self) -> bool {
        self.directional
    }

    pub fn set_use_texture_for_bounces(&mut self, enabled: bool) {
        self.use_texture_for_bounces = enabled;
    }

    pub fn is_using_texture_for_bounces(&self) -> bool {
        self.use_texture_for_bounces
    }

    pub fn set_shadowmask_mode(&mut self, mode: ShadowmaskMode) {
        self.shadowmask_mode = mode;
    }

    pub fn get_shadowmask_mode(&self) -> ShadowmaskMode {
        self.shadowmask_mode
    }

    pub fn set_generate_probes(&mut self, probes: GenerateProbes) {
        self.gen_probes = probes;
    }

    pub fn get_generate_probes(&self) -> GenerateProbes {
        self.gen_probes
    }

    pub fn set_camera_attributes(&mut self, attributes: Option<Rid>) {
        self.camera_attributes = attributes;
    }

    pub fn get_camera_attributes(&self) -> Option<Rid> {
        self.camera_attributes
    }

    /// The baked light data resource handle.
    pub fn set_light_data(&mut self, data: Option<Rid>) {
        self.light_data = data;
    }

    pub fn get_light_data(&self) -> Option<Rid> {
        self.light_data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults() {
        let l = LightmapGIData::new();
        assert_eq!(l.get_bake_quality(), BakeQuality::Medium);
        assert!(l.is_using_denoiser());
        assert_eq!(l.get_denoiser_strength(), 0.1);
        assert_eq!(l.get_denoiser_range(), 10);
        assert_eq!(l.get_bounces(), 3);
        assert_eq!(l.get_bounce_indirect_energy(), 1.0);
        assert_eq!(l.get_bias(), 0.0005);
        assert_eq!(l.get_texel_scale(), 1.0);
        assert_eq!(l.get_max_texture_size(), 16384);
        assert!(!l.is_supersampling_enabled());
        assert_eq!(l.get_supersampling_factor(), 2.0);
        assert!(!l.is_interior());
        assert_eq!(l.get_environment_mode(), EnvironmentMode::Scene);
        assert!(l.get_environment_custom_sky().is_none());
        assert_eq!(l.get_environment_custom_color(), Color::new(1.0, 1.0, 1.0, 1.0));
        assert_eq!(l.get_environment_custom_energy(), 1.0);
        assert!(!l.is_directional());
        assert!(l.is_using_texture_for_bounces());
        assert_eq!(l.get_shadowmask_mode(), ShadowmaskMode::None);
        assert_eq!(l.get_generate_probes(), GenerateProbes::Subdiv8);
        assert!(l.get_camera_attributes().is_none());
        assert!(l.get_light_data().is_none());

        assert_eq!(BakeQuality::Low as i32, 0);
        assert_eq!(BakeQuality::Ultra as i32, 3);
        assert_eq!(GenerateProbes::Disabled as i32, 0);
        assert_eq!(GenerateProbes::Subdiv32 as i32, 4);
        assert_eq!(EnvironmentMode::Disabled as i32, 0);
        assert_eq!(EnvironmentMode::CustomColor as i32, 3);
        assert_eq!(ShadowmaskMode::None as i32, 0);
        assert_eq!(ShadowmaskMode::Multiply as i32, 3);
    }

    #[test]
    fn setters_round_trip() {
        let mut l = LightmapGIData::new();
        l.set_bake_quality(BakeQuality::Ultra);
        l.set_use_denoiser(false);
        l.set_denoiser_strength(0.5);
        l.set_denoiser_range(4);
        l.set_bounces(8);
        l.set_bounce_indirect_energy(2.0);
        l.set_bias(0.001);
        l.set_texel_scale(0.5);
        l.set_max_texture_size(4096);
        l.set_supersampling_enabled(true);
        l.set_supersampling_factor(4.0);
        l.set_interior(true);
        l.set_environment_mode(EnvironmentMode::CustomSky);
        l.set_environment_custom_sky(Some(Rid::from_u64(1)));
        l.set_environment_custom_color(Color::new(0.5, 0.5, 0.5, 1.0));
        l.set_environment_custom_energy(3.0);
        l.set_directional(true);
        l.set_use_texture_for_bounces(false);
        l.set_shadowmask_mode(ShadowmaskMode::Overlay);
        l.set_generate_probes(GenerateProbes::Subdiv32);
        l.set_camera_attributes(Some(Rid::from_u64(2)));
        l.set_light_data(Some(Rid::from_u64(3)));

        assert_eq!(l.get_bake_quality(), BakeQuality::Ultra);
        assert!(!l.is_using_denoiser());
        assert_eq!(l.get_denoiser_strength(), 0.5);
        assert_eq!(l.get_denoiser_range(), 4);
        assert_eq!(l.get_bounces(), 8);
        assert_eq!(l.get_bounce_indirect_energy(), 2.0);
        assert_eq!(l.get_bias(), 0.001);
        assert_eq!(l.get_texel_scale(), 0.5);
        assert_eq!(l.get_max_texture_size(), 4096);
        assert!(l.is_supersampling_enabled());
        assert_eq!(l.get_supersampling_factor(), 4.0);
        assert!(l.is_interior());
        assert_eq!(l.get_environment_mode(), EnvironmentMode::CustomSky);
        assert_eq!(l.get_environment_custom_sky(), Some(Rid::from_u64(1)));
        assert_eq!(l.get_environment_custom_color(), Color::new(0.5, 0.5, 0.5, 1.0));
        assert_eq!(l.get_environment_custom_energy(), 3.0);
        assert!(l.is_directional());
        assert!(!l.is_using_texture_for_bounces());
        assert_eq!(l.get_shadowmask_mode(), ShadowmaskMode::Overlay);
        assert_eq!(l.get_generate_probes(), GenerateProbes::Subdiv32);
        assert_eq!(l.get_camera_attributes(), Some(Rid::from_u64(2)));
        assert_eq!(l.get_light_data(), Some(Rid::from_u64(3)));
    }
}
