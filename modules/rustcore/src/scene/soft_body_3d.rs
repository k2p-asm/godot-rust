//! `scene/3d/physics/soft_body_3d.*` (state half).
//!
//! The node keeps collision layer/mask (both 1, with the engine's `1..=32` bit
//! helpers), the disable mode (`REMOVE`), ray-pickable (on) and
//! capture-input-on-drag (off), plus the pinned-point set and the simulation
//! flags.
//!
//! The soft-body parameters (`simulation_precision`, `total_mass`,
//! `linear_stiffness`, `pressure_coefficient`, `damping_coefficient`,
//! `drag_coefficient`) live in `PhysicsServer3D` in the engine; this port stores
//! them locally with the server's defaults (5, 1.0 and 0.0) so callers can drive
//! them. The server, mesh upload and pinned-point attachments are not ported.

use crate::scene::collision_object::DisableMode;

#[derive(Clone, Debug)]
pub struct SoftBody3DData {
    collision_layer: u32,
    collision_mask: u32,
    disable_mode: DisableMode,
    ray_pickable: bool,
    capture_input_on_drag: bool,
    simulation_started: bool,
    simulation_precision: i32,
    total_mass: f32,
    linear_stiffness: f32,
    pressure_coefficient: f32,
    damping_coefficient: f32,
    drag_coefficient: f32,
    pinned_points: Vec<i32>,
}

impl Default for SoftBody3DData {
    fn default() -> Self {
        Self::new()
    }
}

impl SoftBody3DData {
    pub fn new() -> Self {
        Self {
            collision_layer: 1,
            collision_mask: 1,
            disable_mode: DisableMode::Remove,
            ray_pickable: true,
            capture_input_on_drag: false,
            simulation_started: false,
            simulation_precision: 5,
            total_mass: 1.0,
            linear_stiffness: 0.0,
            pressure_coefficient: 0.0,
            damping_coefficient: 0.0,
            drag_coefficient: 0.0,
            pinned_points: Vec::new(),
        }
    }

    pub fn set_collision_layer(&mut self, layer: u32) {
        self.collision_layer = layer;
    }

    pub fn get_collision_layer(&self) -> u32 {
        self.collision_layer
    }

    pub fn set_collision_mask(&mut self, mask: u32) {
        self.collision_mask = mask;
    }

    pub fn get_collision_mask(&self) -> u32 {
        self.collision_mask
    }

    /// `ERR_FAIL_COND_MSG(layer_number < 1 || layer_number > 32, ...)`.
    pub fn set_collision_layer_value(&mut self, layer: i32, value: bool) -> bool {
        if !(1..=32).contains(&layer) {
            return false;
        }
        let bit = 1u32 << (layer - 1);
        if value {
            self.collision_layer |= bit;
        } else {
            self.collision_layer &= !bit;
        }
        true
    }

    pub fn get_collision_layer_value(&self, layer: i32) -> bool {
        if !(1..=32).contains(&layer) {
            return false;
        }
        self.collision_layer & (1u32 << (layer - 1)) != 0
    }

    pub fn set_collision_mask_value(&mut self, layer: i32, value: bool) -> bool {
        if !(1..=32).contains(&layer) {
            return false;
        }
        let bit = 1u32 << (layer - 1);
        if value {
            self.collision_mask |= bit;
        } else {
            self.collision_mask &= !bit;
        }
        true
    }

    pub fn get_collision_mask_value(&self, layer: i32) -> bool {
        if !(1..=32).contains(&layer) {
            return false;
        }
        self.collision_mask & (1u32 << (layer - 1)) != 0
    }

    pub fn set_disable_mode(&mut self, mode: DisableMode) {
        self.disable_mode = mode;
    }

    pub fn get_disable_mode(&self) -> DisableMode {
        self.disable_mode
    }

    pub fn set_ray_pickable(&mut self, pickable: bool) {
        self.ray_pickable = pickable;
    }

    pub fn is_ray_pickable(&self) -> bool {
        self.ray_pickable
    }

    pub fn set_capture_input_on_drag(&mut self, capture: bool) {
        self.capture_input_on_drag = capture;
    }

    pub fn get_capture_input_on_drag(&self) -> bool {
        self.capture_input_on_drag
    }

    pub fn set_simulation_started(&mut self, started: bool) {
        self.simulation_started = started;
    }

    pub fn is_simulation_started(&self) -> bool {
        self.simulation_started
    }

    pub fn set_simulation_precision(&mut self, precision: i32) {
        self.simulation_precision = precision;
    }

    pub fn get_simulation_precision(&self) -> i32 {
        self.simulation_precision
    }

    pub fn set_total_mass(&mut self, mass: f32) {
        self.total_mass = mass;
    }

    pub fn get_total_mass(&self) -> f32 {
        self.total_mass
    }

    pub fn set_linear_stiffness(&mut self, stiffness: f32) {
        self.linear_stiffness = stiffness;
    }

    pub fn get_linear_stiffness(&self) -> f32 {
        self.linear_stiffness
    }

    pub fn set_pressure_coefficient(&mut self, coefficient: f32) {
        self.pressure_coefficient = coefficient;
    }

    pub fn get_pressure_coefficient(&self) -> f32 {
        self.pressure_coefficient
    }

    pub fn set_damping_coefficient(&mut self, coefficient: f32) {
        self.damping_coefficient = coefficient;
    }

    pub fn get_damping_coefficient(&self) -> f32 {
        self.damping_coefficient
    }

    pub fn set_drag_coefficient(&mut self, coefficient: f32) {
        self.drag_coefficient = coefficient;
    }

    pub fn get_drag_coefficient(&self) -> f32 {
        self.drag_coefficient
    }

    /// `pin_point`: adds/removes the vertex from the pinned set.
    pub fn pin_point(&mut self, point_index: i32, pin: bool) -> bool {
        if point_index < 0 {
            return false;
        }
        if pin {
            if !self.pinned_points.contains(&point_index) {
                self.pinned_points.push(point_index);
            }
        } else {
            self.pinned_points.retain(|index| *index != point_index);
        }
        true
    }

    pub fn is_point_pinned(&self, point_index: i32) -> bool {
        self.pinned_points.contains(&point_index)
    }

    pub fn get_pinned_point_count(&self) -> usize {
        self.pinned_points.len()
    }

    pub fn get_pinned_points(&self) -> &[i32] {
        &self.pinned_points
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults() {
        let body = SoftBody3DData::new();
        assert_eq!(body.get_collision_layer(), 1);
        assert_eq!(body.get_collision_mask(), 1);
        assert_eq!(body.get_disable_mode(), DisableMode::Remove);
        assert!(body.is_ray_pickable());
        assert!(!body.get_capture_input_on_drag());
        assert!(!body.is_simulation_started());
        assert_eq!(body.get_simulation_precision(), 5);
        assert_eq!(body.get_total_mass(), 1.0);
        assert_eq!(body.get_linear_stiffness(), 0.0);
        assert_eq!(body.get_pressure_coefficient(), 0.0);
        assert_eq!(body.get_damping_coefficient(), 0.0);
        assert_eq!(body.get_drag_coefficient(), 0.0);
        assert_eq!(body.get_pinned_point_count(), 0);
    }

    #[test]
    fn collision_bits() {
        let mut body = SoftBody3DData::new();
        body.set_collision_layer(0);
        assert!(body.set_collision_layer_value(1, true));
        assert!(body.set_collision_layer_value(32, true));
        assert_eq!(body.get_collision_layer(), 1 | (1 << 31));
        assert!(body.get_collision_layer_value(32));
        assert!(!body.set_collision_layer_value(0, true));
        assert!(!body.set_collision_layer_value(33, true));
        assert!(!body.get_collision_layer_value(33));

        body.set_collision_mask(0);
        assert!(body.set_collision_mask_value(4, true));
        assert_eq!(body.get_collision_mask(), 8);
        assert!(body.get_collision_mask_value(4));
        assert!(body.set_collision_mask_value(4, false));
        assert!(!body.get_collision_mask_value(4));
    }

    #[test]
    fn parameters_and_pins() {
        let mut body = SoftBody3DData::new();
        body.set_simulation_precision(8);
        body.set_total_mass(2.0);
        body.set_linear_stiffness(0.5);
        body.set_pressure_coefficient(0.25);
        body.set_damping_coefficient(0.75);
        body.set_drag_coefficient(0.1);
        body.set_ray_pickable(false);
        body.set_capture_input_on_drag(true);
        body.set_disable_mode(DisableMode::KeepActive);
        body.set_simulation_started(true);

        assert_eq!(body.get_simulation_precision(), 8);
        assert_eq!(body.get_total_mass(), 2.0);
        assert_eq!(body.get_linear_stiffness(), 0.5);
        assert_eq!(body.get_pressure_coefficient(), 0.25);
        assert_eq!(body.get_damping_coefficient(), 0.75);
        assert_eq!(body.get_drag_coefficient(), 0.1);
        assert!(!body.is_ray_pickable());
        assert!(body.get_capture_input_on_drag());
        assert_eq!(body.get_disable_mode(), DisableMode::KeepActive);
        assert!(body.is_simulation_started());

        assert!(body.pin_point(3, true));
        assert!(body.pin_point(3, true), "already pinned is still accepted");
        assert!(body.pin_point(5, true));
        assert_eq!(body.get_pinned_point_count(), 2);
        assert!(body.is_point_pinned(5));
        assert!(!body.pin_point(-1, true));
        assert!(body.pin_point(3, false));
        assert_eq!(body.get_pinned_points(), &[5]);
    }
}
