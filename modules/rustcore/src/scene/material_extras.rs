//! `scene/resources/3d/fog_material.*` and `scene/resources/blit_material.*`.
//!
//! `FogMaterial` constructor defaults: density 1, white albedo, **opaque black**
//! emission (`(0,0,0,1)` in the constructor even though the header initialises
//! `(0,0,0,0)`), height falloff 0 and edge fade 0.1, plus an optional 3D density
//! texture.
//!
//! `BlitMaterial` only carries its blend mode (mix by default, `DISABLED` being
//! the last of five values).
//!
//! Shader selection/compilation and server parameters are not ported.

use crate::math::color::Color;
use crate::servers::rid::Rid;

#[derive(Clone, Debug)]
pub struct FogMaterialData {
    density: f32,
    albedo: Color,
    emission: Color,
    height_falloff: f32,
    edge_fade: f32,
    density_texture: Option<Rid>,
}

impl Default for FogMaterialData {
    fn default() -> Self {
        Self::new()
    }
}

impl FogMaterialData {
    pub fn new() -> Self {
        Self {
            density: 1.0,
            albedo: Color::new(1.0, 1.0, 1.0, 1.0),
            emission: Color::new(0.0, 0.0, 0.0, 1.0),
            height_falloff: 0.0,
            edge_fade: 0.1,
            density_texture: None,
        }
    }

    pub fn set_density(&mut self, density: f32) {
        self.density = density;
    }

    pub fn get_density(&self) -> f32 {
        self.density
    }

    pub fn set_albedo(&mut self, albedo: Color) {
        self.albedo = albedo;
    }

    pub fn get_albedo(&self) -> Color {
        self.albedo
    }

    pub fn set_emission(&mut self, emission: Color) {
        self.emission = emission;
    }

    pub fn get_emission(&self) -> Color {
        self.emission
    }

    pub fn set_height_falloff(&mut self, falloff: f32) {
        self.height_falloff = falloff;
    }

    pub fn get_height_falloff(&self) -> f32 {
        self.height_falloff
    }

    pub fn set_edge_fade(&mut self, fade: f32) {
        self.edge_fade = fade;
    }

    pub fn get_edge_fade(&self) -> f32 {
        self.edge_fade
    }

    pub fn set_density_texture(&mut self, texture: Option<Rid>) {
        self.density_texture = texture;
    }

    pub fn get_density_texture(&self) -> Option<Rid> {
        self.density_texture
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum BlendMode {
    Mix,
    Add,
    Sub,
    Mul,
    Disabled,
}

#[derive(Clone, Copy, Debug)]
pub struct BlitMaterialData {
    blend_mode: BlendMode,
}

impl Default for BlitMaterialData {
    fn default() -> Self {
        Self::new()
    }
}

impl BlitMaterialData {
    pub fn new() -> Self {
        Self {
            blend_mode: BlendMode::Mix,
        }
    }

    pub fn set_blend_mode(&mut self, mode: BlendMode) {
        self.blend_mode = mode;
    }

    pub fn get_blend_mode(&self) -> BlendMode {
        self.blend_mode
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fog_material_defaults() {
        let f = FogMaterialData::new();
        assert_eq!(f.get_density(), 1.0);
        assert_eq!(f.get_albedo(), Color::new(1.0, 1.0, 1.0, 1.0));
        // The constructor passes (0,0,0,1), which wins over the header's (0,0,0,0).
        assert_eq!(f.get_emission(), Color::new(0.0, 0.0, 0.0, 1.0));
        assert_eq!(f.get_height_falloff(), 0.0);
        assert_eq!(f.get_edge_fade(), 0.1);
        assert!(f.get_density_texture().is_none());
    }

    #[test]
    fn fog_material_setters() {
        let mut f = FogMaterialData::new();
        f.set_density(2.0);
        f.set_albedo(Color::new(0.5, 0.5, 0.5, 1.0));
        f.set_emission(Color::new(0.1, 0.0, 0.0, 0.5));
        f.set_height_falloff(0.75);
        f.set_edge_fade(0.25);
        f.set_density_texture(Some(Rid::from_u64(6)));

        assert_eq!(f.get_density(), 2.0);
        assert_eq!(f.get_albedo(), Color::new(0.5, 0.5, 0.5, 1.0));
        assert_eq!(f.get_emission(), Color::new(0.1, 0.0, 0.0, 0.5));
        assert_eq!(f.get_height_falloff(), 0.75);
        assert_eq!(f.get_edge_fade(), 0.25);
        assert_eq!(f.get_density_texture(), Some(Rid::from_u64(6)));

        f.set_density_texture(None);
        assert!(f.get_density_texture().is_none());
    }

    #[test]
    fn blit_material_modes() {
        let mut b = BlitMaterialData::new();
        assert_eq!(b.get_blend_mode(), BlendMode::Mix);
        b.set_blend_mode(BlendMode::Mul);
        assert_eq!(b.get_blend_mode(), BlendMode::Mul);

        assert_eq!(BlendMode::Mix as i32, 0);
        assert_eq!(BlendMode::Add as i32, 1);
        assert_eq!(BlendMode::Sub as i32, 2);
        assert_eq!(BlendMode::Mul as i32, 3);
        assert_eq!(BlendMode::Disabled as i32, 4);
    }
}
