//! `scene/3d/decal.*` and `scene/3d/fog_volume.*` (state only).
//!
//! `Decal` defaults: size `(2,2,2)`, no textures, emission energy 1, albedo mix
//! 1, white modulate, cull mask `(1 << 20) - 1`, normal fade 0, upper/lower fade
//! 0.3, distance fade off with begin 40 and length 10.
//!
//! `FogVolume` defaults: size `(2,2,2)`, no material and a box shape.
//!
//! Rendering-server calls, distance-fade processing and AABB computation are not
//! ported.

use crate::math::color::Color;
use crate::math::vector3::Vector3;
use crate::servers::rid::Rid;

// -- Decal ------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum DecalTexture {
    Albedo,
    Normal,
    Orm,
    Emission,
    Max,
}

pub const DECAL_TEXTURE_MAX: usize = 4;

#[derive(Clone, Debug)]
pub struct DecalData {
    size: Vector3,
    textures: [Option<Rid>; DECAL_TEXTURE_MAX],
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
            size: Vector3::new(2.0, 2.0, 2.0),
            textures: [None; DECAL_TEXTURE_MAX],
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

    pub fn set_size(&mut self, size: Vector3) {
        self.size = size;
    }

    pub fn get_size(&self) -> Vector3 {
        self.size
    }

    /// `ERR_FAIL_INDEX(type, TEXTURE_MAX)`.
    pub fn set_texture(&mut self, kind: i32, texture: Option<Rid>) -> bool {
        if !(0..DECAL_TEXTURE_MAX as i32).contains(&kind) {
            return false;
        }
        self.textures[kind as usize] = texture;
        true
    }

    pub fn get_texture(&self, kind: i32) -> Option<Rid> {
        if !(0..DECAL_TEXTURE_MAX as i32).contains(&kind) {
            return None;
        }
        self.textures[kind as usize]
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

    pub fn set_distance_fade_enabled(&mut self, enabled: bool) {
        self.distance_fade_enabled = enabled;
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

// -- FogVolume --------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum FogVolumeShape {
    Box,
    Sphere,
    Cone,
    Cylinder,
    Ellipsoid,
    Max,
}

#[derive(Clone, Debug)]
pub struct FogVolumeData {
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
            size: Vector3::new(2.0, 2.0, 2.0),
            material: None,
            shape: FogVolumeShape::Box,
        }
    }

    pub fn set_size(&mut self, size: Vector3) {
        self.size = size;
    }

    pub fn get_size(&self) -> Vector3 {
        self.size
    }

    /// `RSE::FogVolumeShape`; `MAX` and out-of-range values are refused.
    pub fn set_shape(&mut self, shape: i32) -> bool {
        let parsed = match shape {
            0 => FogVolumeShape::Box,
            1 => FogVolumeShape::Sphere,
            2 => FogVolumeShape::Cone,
            3 => FogVolumeShape::Cylinder,
            4 => FogVolumeShape::Ellipsoid,
            _ => return false,
        };
        self.shape = parsed;
        true
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

    /// `VisualInstance3D::get_aabb` for the fog volume: centred on the origin.
    pub fn get_aabb(&self) -> crate::math::aabb::Aabb {
        crate::math::aabb::Aabb::new(self.size * -0.5, self.size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decal_defaults_and_textures() {
        let mut decal = DecalData::new();
        assert_eq!(decal.get_size(), Vector3::new(2.0, 2.0, 2.0));
        assert_eq!(decal.get_emission_energy(), 1.0);
        assert_eq!(decal.get_albedo_mix(), 1.0);
        assert_eq!(decal.get_modulate(), Color::new(1.0, 1.0, 1.0, 1.0));
        assert_eq!(decal.get_cull_mask(), (1 << 20) - 1);
        assert_eq!(decal.get_normal_fade(), 0.0);
        assert_eq!(decal.get_upper_fade(), 0.3);
        assert_eq!(decal.get_lower_fade(), 0.3);
        assert!(!decal.is_distance_fade_enabled());
        assert_eq!(decal.get_distance_fade_begin(), 40.0);
        assert_eq!(decal.get_distance_fade_length(), 10.0);
        for kind in 0..4 {
            assert!(decal.get_texture(kind).is_none());
        }

        assert!(decal.set_texture(0, Some(Rid::from_u64(1))));
        assert!(decal.set_texture(3, Some(Rid::from_u64(2))));
        assert!(!decal.set_texture(4, None), "TEXTURE_MAX is refused");
        assert!(!decal.set_texture(-1, None));
        assert_eq!(decal.get_texture(0), Some(Rid::from_u64(1)));
        assert_eq!(decal.get_texture(3), Some(Rid::from_u64(2)));
        assert_eq!(decal.get_texture(4), None);

        decal.set_size(Vector3::ONE);
        decal.set_emission_energy(2.0);
        decal.set_albedo_mix(0.5);
        decal.set_modulate(Color::new(0.5, 0.25, 0.75, 1.0));
        decal.set_cull_mask(3);
        decal.set_normal_fade(0.1);
        decal.set_upper_fade(0.2);
        decal.set_lower_fade(0.4);
        decal.set_distance_fade_enabled(true);
        decal.set_distance_fade_begin(10.0);
        decal.set_distance_fade_length(5.0);
        assert_eq!(decal.get_size(), Vector3::ONE);
        assert_eq!(decal.get_emission_energy(), 2.0);
        assert_eq!(decal.get_albedo_mix(), 0.5);
        assert_eq!(decal.get_cull_mask(), 3);
        assert_eq!(decal.get_normal_fade(), 0.1);
        assert_eq!(decal.get_upper_fade(), 0.2);
        assert_eq!(decal.get_lower_fade(), 0.4);
        assert!(decal.is_distance_fade_enabled());
        assert_eq!(decal.get_distance_fade_begin(), 10.0);
        assert_eq!(decal.get_distance_fade_length(), 5.0);
    }

    #[test]
    fn fog_volume_defaults_and_shapes() {
        let mut volume = FogVolumeData::new();
        assert_eq!(volume.get_size(), Vector3::new(2.0, 2.0, 2.0));
        assert_eq!(volume.get_shape(), FogVolumeShape::Box);
        assert!(volume.get_material().is_none());
        assert_eq!(
            volume.get_aabb(),
            crate::math::aabb::Aabb::new(Vector3::new(-1.0, -1.0, -1.0), Vector3::new(2.0, 2.0, 2.0))
        );

        assert!(volume.set_shape(4));
        assert_eq!(volume.get_shape(), FogVolumeShape::Ellipsoid);
        assert!(!volume.set_shape(5));
        assert!(!volume.set_shape(-1));
        volume.set_size(Vector3::new(4.0, 2.0, 6.0));
        assert_eq!(volume.get_aabb().size, Vector3::new(4.0, 2.0, 6.0));
        volume.set_material(Some(Rid::from_u64(7)));
        assert_eq!(volume.get_material(), Some(Rid::from_u64(7)));
        assert_eq!(FogVolumeShape::Cone as i32, 2);
        assert_eq!(FogVolumeShape::Max as i32, 5);
    }
}
