//! Small 3D resources: `JointLimitation3D`/`JointLimitationCone3D`,
//! `WorldBoundaryShape3D` and `FogMaterial`.
//!
//! Ported state:
//!
//! - `JointLimitation3D` has no data (the solve/space math is not ported);
//! - `JointLimitationCone3D` stores the cone angle (`TAU / 4` by default);
//! - `WorldBoundaryShape3D` stores its plane and reports an enclosing radius of
//!   0 (like the header, an "infinite" boundary cannot be enclosed);
//! - `FogMaterial` stores density 1.0, white albedo, transparent black emission,
//!   zero height falloff, 0.1 edge fade and the density texture handle.

use crate::math::color::Color;
use crate::math::plane::Plane;
use crate::servers::rid::Rid;

#[derive(Clone, Debug, Default)]
pub struct JointLimitationData;

impl JointLimitationData {
    pub fn new() -> Self {
        Self
    }
}

#[derive(Clone, Debug)]
pub struct JointLimitationConeData {
    angle: f32,
}

impl Default for JointLimitationConeData {
    fn default() -> Self {
        Self::new()
    }
}

impl JointLimitationConeData {
    pub fn new() -> Self {
        Self {
            angle: std::f32::consts::TAU * 0.25,
        }
    }

    pub fn set_angle(&mut self, angle: f32) {
        self.angle = angle;
    }

    pub fn get_angle(&self) -> f32 {
        self.angle
    }
}

#[derive(Clone, Debug, Default)]
pub struct WorldBoundaryShape3DData {
    plane: Plane,
}

impl WorldBoundaryShape3DData {
    pub fn new() -> Self {
        Self {
            plane: Plane::default(),
        }
    }

    pub fn set_plane(&mut self, plane: Plane) {
        self.plane = plane;
    }

    pub fn get_plane(&self) -> Plane {
        self.plane
    }

    /// `get_enclosing_radius` returns 0 for the infinite boundary.
    pub fn get_enclosing_radius(&self) -> f32 {
        0.0
    }
}

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
            emission: Color::new(0.0, 0.0, 0.0, 0.0),
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

    pub fn set_edge_fade(&mut self, edge_fade: f32) {
        self.edge_fade = edge_fade;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cone_limitation() {
        let mut cone = JointLimitationConeData::new();
        assert!((cone.get_angle() - std::f32::consts::TAU * 0.25).abs() < 1e-6);
        cone.set_angle(0.5);
        assert_eq!(cone.get_angle(), 0.5);
        let _base = JointLimitationData::new();
    }

    #[test]
    fn world_boundary_shape() {
        let mut shape = WorldBoundaryShape3DData::new();
        assert_eq!(shape.get_plane(), Plane::default());
        let plane = Plane::new(0.0, 1.0, 0.0, 2.0);
        shape.set_plane(plane);
        assert_eq!(shape.get_plane(), plane);
        assert_eq!(shape.get_enclosing_radius(), 0.0);
    }

    #[test]
    fn fog_material_defaults_and_setters() {
        let mut material = FogMaterialData::new();
        assert_eq!(material.get_density(), 1.0);
        assert_eq!(material.get_albedo(), Color::new(1.0, 1.0, 1.0, 1.0));
        assert_eq!(material.get_emission(), Color::new(0.0, 0.0, 0.0, 0.0));
        assert_eq!(material.get_height_falloff(), 0.0);
        assert_eq!(material.get_edge_fade(), 0.1);
        assert!(material.get_density_texture().is_none());

        material.set_density(0.25);
        material.set_albedo(Color::new(0.5, 0.6, 0.7, 1.0));
        material.set_emission(Color::new(0.1, 0.0, 0.0, 1.0));
        material.set_height_falloff(1.5);
        material.set_edge_fade(0.3);
        material.set_density_texture(Some(Rid::from_u64(6)));
        assert_eq!(material.get_density(), 0.25);
        assert_eq!(material.get_albedo(), Color::new(0.5, 0.6, 0.7, 1.0));
        assert_eq!(material.get_emission(), Color::new(0.1, 0.0, 0.0, 1.0));
        assert_eq!(material.get_height_falloff(), 1.5);
        assert_eq!(material.get_edge_fade(), 0.3);
        assert_eq!(material.get_density_texture(), Some(Rid::from_u64(6)));
    }
}
