//! `scene/3d/fog_volume.*` and `decal.*`: volume state.
//!
//! FogVolume: size `(2,2,2)` clamped to `>= 0`, material handle and shape
//! (box by default; the shape enum is ellipsoid 0, cone 1, cylinder 2, box 3,
//! world 4, max 5).
//!
//! Decal: size `(2,2,2)` clamped to `>= 0.001`, four texture handles
//! (albedo/normal/ORM/emission), emission energy and albedo mix 1, white
//! modulate, cull mask `(1 << 20) - 1`, normal/upper/lower fade 0/0.3/0.3 and
//! distance fade off with begin 40 and length 10.
//!
//! RenderingServer RIDs and the fade curve upload are not ported.

use crate::math::color::Color;
use crate::math::vector3::Vector3;
use crate::scene::visual_instance_3d::VisualInstance3DData;
use crate::servers::rid::Rid;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum FogVolumeShape {
    Ellipsoid,
    Cone,
    Cylinder,
    Box,
    World,
    Max,
}

#[derive(Clone, Debug)]
pub struct FogVolumeData {
    pub visual: VisualInstance3DData,
    size: Vector3,
    material: Option<Rid>,
    shape: FogVolumeShape,
}

impl Default for FogVolumeData {
    fn default() -> Self {
        Self::new()
    }
}

impl FogVolumeData {
    pub fn new() -> Self {
        Self {
            visual: VisualInstance3DData::new(),
            size: Vector3::new(2.0, 2.0, 2.0),
            material: None,
            shape: FogVolumeShape::Box,
        }
    }

    /// Components clamped to `>= 0` (`maxf(0)`).
    pub fn set_size(&mut self, size: Vector3) {
        self.size = Vector3::new(size.x.max(0.0), size.y.max(0.0), size.z.max(0.0));
    }

    pub fn get_size(&self) -> Vector3 {
        self.size
    }

    pub fn set_shape(&mut self, shape: FogVolumeShape) {
        self.shape = shape;
    }

    pub fn get_shape(&self) -> FogVolumeShape {
        self.shape
    }

    pub fn set_material(&mut self, material: Option<Rid>) {
        self.material = material;
    }

    pub fn get_material(&self) -> Option<Rid> {
        self.material
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum DecalTexture {
    Albedo,
    Normal,
    Orm,
    Emission,
    Max,
}

#[derive(Clone, Debug)]
pub struct DecalData {
    pub visual: VisualInstance3DData,
    size: Vector3,
    textures: [Option<Rid>; 4],
    emission_energy: f32,
    albedo_mix: f32,
    modulate: Color,
    cull_mask: u32,
    normal_fade: f32,
    upper_fade: f32,
    lower_fade: f32,
    distance_fade_enabled: bool,
    distance_fade_begin: f32,
    distance_fade_length: f32,
}

impl Default for DecalData {
    fn default() -> Self {
        Self::new()
    }
}

impl DecalData {
    pub fn new() -> Self {
        Self {
            visual: VisualInstance3DData::new(),
            size: Vector3::new(2.0, 2.0, 2.0),
            textures: [None; 4],
            emission_energy: 1.0,
            albedo_mix: 1.0,
            modulate: Color::new(1.0, 1.0, 1.0, 1.0),
            cull_mask: (1 << 20) - 1,
            normal_fade: 0.0,
            upper_fade: 0.3,
            lower_fade: 0.3,
            distance_fade_enabled: false,
            distance_fade_begin: 40.0,
            distance_fade_length: 10.0,
        }
    }

    /// Components clamped to `>= 0.001` (`maxf(0.001)`).
    pub fn set_size(&mut self, size: Vector3) {
        self.size = Vector3::new(
            size.x.max(0.001),
            size.y.max(0.001),
            size.z.max(0.001),
        );
    }

    pub fn get_size(&self) -> Vector3 {
        self.size
    }

    pub fn set_texture(&mut self, slot: DecalTexture, texture: Option<Rid>) -> bool {
        if slot == DecalTexture::Max {
            return false;
        }
        self.textures[slot as usize] = texture;
        true
    }

    pub fn get_texture(&self, slot: DecalTexture) -> Option<Rid> {
        if slot == DecalTexture::Max {
            return None;
        }
        self.textures[slot as usize]
    }

    pub fn set_emission_energy(&mut self, energy: f32) {
        self.emission_energy = energy;
    }

    pub fn get_emission_energy(&self) -> f32 {
        self.emission_energy
    }

    pub fn set_albedo_mix(&mut self, mix: f32) {
        self.albedo_mix = mix;
    }

    pub fn get_albedo_mix(&self) -> f32 {
        self.albedo_mix
    }

    pub fn set_modulate(&mut self, modulate: Color) {
        self.modulate = modulate;
    }

    pub fn get_modulate(&self) -> Color {
        self.modulate
    }

    pub fn set_cull_mask(&mut self, mask: u32) {
        self.cull_mask = mask;
    }

    pub fn get_cull_mask(&self) -> u32 {
        self.cull_mask
    }

    pub fn set_normal_fade(&mut self, fade: f32) {
        self.normal_fade = fade;
    }

    pub fn get_normal_fade(&self) -> f32 {
        self.normal_fade
    }

    pub fn set_upper_fade(&mut self, fade: f32) {
        self.upper_fade = fade;
    }

    pub fn get_upper_fade(&self) -> f32 {
        self.upper_fade
    }

    pub fn set_lower_fade(&mut self, fade: f32) {
        self.lower_fade = fade;
    }

    pub fn get_lower_fade(&self) -> f32 {
        self.lower_fade
    }

    pub fn set_enable_distance_fade(&mut self, enable: bool) {
        self.distance_fade_enabled = enable;
    }

    pub fn is_distance_fade_enabled(&self) -> bool {
        self.distance_fade_enabled
    }

    pub fn set_distance_fade_begin(&mut self, begin: f32) {
        self.distance_fade_begin = begin;
    }

    pub fn get_distance_fade_begin(&self) -> f32 {
        self.distance_fade_begin
    }

    pub fn set_distance_fade_length(&mut self, length: f32) {
        self.distance_fade_length = length;
    }

    pub fn get_distance_fade_length(&self) -> f32 {
        self.distance_fade_length
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fog_volume_defaults_and_clamps() {
        let mut f = FogVolumeData::new();
        assert_eq!(f.get_size(), Vector3::new(2.0, 2.0, 2.0));
        assert_eq!(f.get_shape(), FogVolumeShape::Box);
        assert!(f.get_material().is_none());

        f.set_size(Vector3::new(-1.0, 3.0, -2.0));
        assert_eq!(f.get_size(), Vector3::new(0.0, 3.0, 0.0));

        f.set_shape(FogVolumeShape::World);
        f.set_material(Some(Rid::from_u64(2)));
        assert_eq!(f.get_shape(), FogVolumeShape::World);
        assert_eq!(f.get_material(), Some(Rid::from_u64(2)));

        assert_eq!(FogVolumeShape::Ellipsoid as i32, 0);
        assert_eq!(FogVolumeShape::Cone as i32, 1);
        assert_eq!(FogVolumeShape::Cylinder as i32, 2);
        assert_eq!(FogVolumeShape::Box as i32, 3);
        assert_eq!(FogVolumeShape::World as i32, 4);
        assert_eq!(FogVolumeShape::Max as i32, 5);
    }

    #[test]
    fn decal_defaults_and_size() {
        let mut d = DecalData::new();
        assert_eq!(d.get_size(), Vector3::new(2.0, 2.0, 2.0));
        assert_eq!(d.get_emission_energy(), 1.0);
        assert_eq!(d.get_albedo_mix(), 1.0);
        assert_eq!(d.get_modulate(), Color::new(1.0, 1.0, 1.0, 1.0));
        assert_eq!(d.get_cull_mask(), (1 << 20) - 1);
        assert_eq!(d.get_normal_fade(), 0.0);
        assert_eq!(d.get_upper_fade(), 0.3);
        assert_eq!(d.get_lower_fade(), 0.3);
        assert!(!d.is_distance_fade_enabled());
        assert_eq!(d.get_distance_fade_begin(), 40.0);
        assert_eq!(d.get_distance_fade_length(), 10.0);
        for slot in [
            DecalTexture::Albedo,
            DecalTexture::Normal,
            DecalTexture::Orm,
            DecalTexture::Emission,
        ] {
            assert!(d.get_texture(slot).is_none());
        }

        // Size clamps each component to at least 0.001.
        d.set_size(Vector3::new(0.0, -1.0, 5.0));
        assert_eq!(d.get_size(), Vector3::new(0.001, 0.001, 5.0));
    }

    #[test]
    fn decal_textures_and_fades() {
        let mut d = DecalData::new();
        assert!(d.set_texture(DecalTexture::Albedo, Some(Rid::from_u64(10))));
        assert!(d.set_texture(DecalTexture::Emission, Some(Rid::from_u64(11))));
        assert_eq!(d.get_texture(DecalTexture::Albedo), Some(Rid::from_u64(10)));
        assert_eq!(d.get_texture(DecalTexture::Emission), Some(Rid::from_u64(11)));
        assert_eq!(d.get_texture(DecalTexture::Max), None);
        assert!(!d.set_texture(DecalTexture::Max, None));

        d.set_emission_energy(2.5);
        d.set_albedo_mix(0.5);
        d.set_modulate(Color::new(1.0, 0.5, 0.25, 1.0));
        d.set_cull_mask(3);
        d.set_normal_fade(0.4);
        d.set_upper_fade(0.6);
        d.set_lower_fade(0.2);
        d.set_enable_distance_fade(true);
        d.set_distance_fade_begin(30.0);
        d.set_distance_fade_length(5.0);

        assert_eq!(d.get_emission_energy(), 2.5);
        assert_eq!(d.get_albedo_mix(), 0.5);
        assert_eq!(d.get_modulate(), Color::new(1.0, 0.5, 0.25, 1.0));
        assert_eq!(d.get_cull_mask(), 3);
        assert_eq!(d.get_normal_fade(), 0.4);
        assert_eq!(d.get_upper_fade(), 0.6);
        assert_eq!(d.get_lower_fade(), 0.2);
        assert!(d.is_distance_fade_enabled());
        assert_eq!(d.get_distance_fade_begin(), 30.0);
        assert_eq!(d.get_distance_fade_length(), 5.0);

        assert_eq!(DecalTexture::Albedo as i32, 0);
        assert_eq!(DecalTexture::Normal as i32, 1);
        assert_eq!(DecalTexture::Orm as i32, 2);
        assert_eq!(DecalTexture::Emission as i32, 3);
        assert_eq!(DecalTexture::Max as i32, 4);
    }
}
