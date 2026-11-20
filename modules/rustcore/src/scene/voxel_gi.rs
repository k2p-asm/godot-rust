//! `scene/3d/voxel_gi.*` (state only).
//!
//! `VoxelGI` defaults: `SUBDIV_128`, size `(20,20,20)`, no probe data and no
//! camera attributes. The probe resource parameters (dynamic range 2, energy 1,
//! bias 1.5, normal bias 0, propagation 0.5, interior off, two bounces on) are
//! kept on the node here so callers can drive them without the resource object.
//!
//! Baking, the probe upload, the mesh plot and the RenderingServer calls are not
//! ported.

use crate::math::vector3::Vector3;
use crate::servers::rid::Rid;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum Subdiv {
    Subdiv64,
    Subdiv128,
    Subdiv256,
    Subdiv512,
    Max,
}

#[derive(Clone, Copy, Debug)]
pub struct VoxelGIDataParams {
    pub dynamic_range: f32,
    pub energy: f32,
    pub bias: f32,
    pub normal_bias: f32,
    pub propagation: f32,
    pub interior: bool,
    pub use_two_bounces: bool,
}

impl Default for VoxelGIDataParams {
    fn default() -> Self {
        Self {
            dynamic_range: 2.0,
            energy: 1.0,
            bias: 1.5,
            normal_bias: 0.0,
            propagation: 0.5,
            interior: false,
            use_two_bounces: true,
        }
    }
}

#[derive(Clone, Debug)]
pub struct VoxelGIData {
    subdiv: Subdiv,
    size: Vector3,
    probe_data: Option<Rid>,
    camera_attributes: Option<Rid>,
    params: VoxelGIDataParams,
}

impl Default for VoxelGIData {
    fn default() -> Self {
        Self::new()
    }
}

impl VoxelGIData {
    pub fn new() -> Self {
        Self {
            subdiv: Subdiv::Subdiv128,
            size: Vector3::new(20.0, 20.0, 20.0),
            probe_data: None,
            camera_attributes: None,
            params: VoxelGIDataParams::default(),
        }
    }

    /// `ERR_FAIL_INDEX(subdiv, SUBDIV_MAX)`.
    pub fn set_subdiv(&mut self, subdiv: i32) -> bool {
        let parsed = match subdiv {
            0 => Subdiv::Subdiv64,
            1 => Subdiv::Subdiv128,
            2 => Subdiv::Subdiv256,
            3 => Subdiv::Subdiv512,
            _ => return false,
        };
        self.subdiv = parsed;
        true
    }

    pub fn get_subdiv(&self) -> Subdiv {
        self.subdiv
    }

    pub fn set_size(&mut self, size: Vector3) {
        self.size = size;
    }

    pub fn get_size(&self) -> Vector3 {
        self.size
    }

    pub fn set_probe_data(&mut self, data: Option<Rid>) {
        self.probe_data = data;
    }

    pub fn get_probe_data(&self) -> Option<Rid> {
        self.probe_data
    }

    pub fn set_camera_attributes(&mut self, attributes: Option<Rid>) {
        self.camera_attributes = attributes;
    }

    pub fn get_camera_attributes(&self) -> Option<Rid> {
        self.camera_attributes
    }

    pub fn params(&self) -> &VoxelGIDataParams {
        &self.params
    }

    pub fn params_mut(&mut self) -> &mut VoxelGIDataParams {
        &mut self.params
    }

    pub fn set_dynamic_range(&mut self, range: f32) {
        self.params.dynamic_range = range;
    }

    pub fn get_dynamic_range(&self) -> f32 {
        self.params.dynamic_range
    }

    pub fn set_energy(&mut self, energy: f32) {
        self.params.energy = energy;
    }

    pub fn get_energy(&self) -> f32 {
        self.params.energy
    }

    pub fn set_bias(&mut self, bias: f32) {
        self.params.bias = bias;
    }

    pub fn get_bias(&self) -> f32 {
        self.params.bias
    }

    pub fn set_normal_bias(&mut self, bias: f32) {
        self.params.normal_bias = bias;
    }

    pub fn get_normal_bias(&self) -> f32 {
        self.params.normal_bias
    }

    pub fn set_propagation(&mut self, propagation: f32) {
        self.params.propagation = propagation;
    }

    pub fn get_propagation(&self) -> f32 {
        self.params.propagation
    }

    pub fn set_interior(&mut self, interior: bool) {
        self.params.interior = interior;
    }

    pub fn is_interior(&self) -> bool {
        self.params.interior
    }

    pub fn set_use_two_bounces(&mut self, enabled: bool) {
        self.params.use_two_bounces = enabled;
    }

    pub fn is_using_two_bounces(&self) -> bool {
        self.params.use_two_bounces
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults() {
        let gi = VoxelGIData::new();
        assert_eq!(gi.get_subdiv(), Subdiv::Subdiv128);
        assert_eq!(gi.get_size(), Vector3::new(20.0, 20.0, 20.0));
        assert!(gi.get_probe_data().is_none());
        assert!(gi.get_camera_attributes().is_none());
        assert_eq!(gi.get_dynamic_range(), 2.0);
        assert_eq!(gi.get_energy(), 1.0);
        assert_eq!(gi.get_bias(), 1.5);
        assert_eq!(gi.get_normal_bias(), 0.0);
        assert_eq!(gi.get_propagation(), 0.5);
        assert!(!gi.is_interior());
        assert!(gi.is_using_two_bounces());
    }

    #[test]
    fn setters_and_guards() {
        let mut gi = VoxelGIData::new();
        assert!(gi.set_subdiv(3));
        assert!(!gi.set_subdiv(4));
        assert!(!gi.set_subdiv(-1));
        assert_eq!(gi.get_subdiv(), Subdiv::Subdiv512);
        assert_eq!(Subdiv::Subdiv64 as i32, 0);
        assert_eq!(Subdiv::Subdiv128 as i32, 1);
        assert_eq!(Subdiv::Max as i32, 4);

        gi.set_size(Vector3::new(4.0, 5.0, 6.0));
        gi.set_probe_data(Some(Rid::from_u64(2)));
        gi.set_camera_attributes(Some(Rid::from_u64(3)));
        gi.set_dynamic_range(3.0);
        gi.set_energy(2.0);
        gi.set_bias(0.5);
        gi.set_normal_bias(0.1);
        gi.set_propagation(0.75);
        gi.set_interior(true);
        gi.set_use_two_bounces(false);

        assert_eq!(gi.get_size(), Vector3::new(4.0, 5.0, 6.0));
        assert_eq!(gi.get_probe_data(), Some(Rid::from_u64(2)));
        assert_eq!(gi.get_camera_attributes(), Some(Rid::from_u64(3)));
        assert_eq!(gi.get_dynamic_range(), 3.0);
        assert_eq!(gi.get_energy(), 2.0);
        assert_eq!(gi.get_bias(), 0.5);
        assert_eq!(gi.get_normal_bias(), 0.1);
        assert_eq!(gi.get_propagation(), 0.75);
        assert!(gi.is_interior());
        assert!(!gi.is_using_two_bounces());

        // The params struct round-trips through the accessors.
        gi.params_mut().energy = 9.0;
        assert_eq!(gi.get_energy(), 9.0);
        assert_eq!(gi.params().bias, 0.5);
    }
}
