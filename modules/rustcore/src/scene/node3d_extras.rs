//! `scene/3d/remote_transform_3d.*` (state) and `velocity_tracker_3d.*`.
//!
//! - `RemoteTransform3D`: the remote node path, the global-coordinates flag and
//!   the per-transform update flags (all on by default). The remote-node cache
//!   and the transform copy are not ported.
//! - `VelocityTracker3D`: the four-entry position history, same-frame
//!   replacement, the physics/process tick choice, the 1/5 s averaging window
//!   and `reset`. The engine frame counters are passed in by the caller.

use crate::math::vector3::Vector3;
use crate::string::node_path::NodePath;

#[derive(Clone, Debug)]
pub struct RemoteTransform3DData {
    remote_node: NodePath,
    use_global_coordinates: bool,
    update_remote_position: bool,
    update_remote_rotation: bool,
    update_remote_scale: bool,
}

impl Default for RemoteTransform3DData {
    fn default() -> Self {
        Self::new()
    }
}

impl RemoteTransform3DData {
    pub fn new() -> Self {
        Self {
            remote_node: NodePath::new(),
            use_global_coordinates: true,
            update_remote_position: true,
            update_remote_rotation: true,
            update_remote_scale: true,
        }
    }

    pub fn set_remote_node(&mut self, path: NodePath) {
        self.remote_node = path;
    }

    pub fn get_remote_node(&self) -> &NodePath {
        &self.remote_node
    }

    pub fn set_use_global_coordinates(&mut self, enabled: bool) {
        self.use_global_coordinates = enabled;
    }

    pub fn get_use_global_coordinates(&self) -> bool {
        self.use_global_coordinates
    }

    pub fn set_update_position(&mut self, enabled: bool) {
        self.update_remote_position = enabled;
    }

    pub fn get_update_position(&self) -> bool {
        self.update_remote_position
    }

    pub fn set_update_rotation(&mut self, enabled: bool) {
        self.update_remote_rotation = enabled;
    }

    pub fn get_update_rotation(&self) -> bool {
        self.update_remote_rotation
    }

    pub fn set_update_scale(&mut self, enabled: bool) {
        self.update_remote_scale = enabled;
    }

    pub fn get_update_scale(&self) -> bool {
        self.update_remote_scale
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PositionHistory {
    pub frame: u64,
    pub position: Vector3,
}

#[derive(Clone, Debug)]
pub struct VelocityTracker3D {
    physics_step: bool,
    /// The engine's `position_history.resize(4)`.
    position_history: Vec<PositionHistory>,
    position_history_len: usize,
}

impl Default for VelocityTracker3D {
    fn default() -> Self {
        Self::new()
    }
}

impl VelocityTracker3D {
    pub fn new() -> Self {
        Self {
            physics_step: false,
            position_history: vec![PositionHistory::default(); 4],
            position_history_len: 0,
        }
    }

    pub fn set_track_physics_step(&mut self, enabled: bool) {
        self.physics_step = enabled;
    }

    pub fn is_tracking_physics_step(&self) -> bool {
        self.physics_step
    }

    pub fn get_history_len(&self) -> usize {
        self.position_history_len
    }

    pub fn get_history_entry(&self, index: usize) -> Option<PositionHistory> {
        self.position_history.get(index).copied()
    }

    /// `update_position`: `frame` is the engine's frame tick (microseconds) or
    /// physics frame depending on the tracking mode.
    pub fn update_position(&mut self, position: Vector3, frame: u64) {
        let newest = PositionHistory { frame, position };
        if self.position_history_len == 0 || self.position_history[0].frame != frame {
            self.position_history_len =
                self.position_history.len().min(self.position_history_len + 1);
            for index in (1..self.position_history_len).rev() {
                self.position_history[index] = self.position_history[index - 1];
            }
        }
        self.position_history[0] = newest;
    }

    /// `reset`: a single history entry at `p_new_pos`.
    pub fn reset(&mut self, position: Vector3, frame: u64) {
        self.position_history[0] = PositionHistory { frame, position };
        self.position_history_len = 1;
    }

    /// `get_tracked_linear_velocity`: `ticks_per_second` is the physics tick
    /// rate (60 by default in the engine); it is only used in physics mode,
    /// where frames are counted in ticks rather than microseconds.
    pub fn get_tracked_linear_velocity(
        &self,
        current_frame: u64,
        ticks_per_second: f64,
    ) -> Vector3 {
        let mut linear_velocity = Vector3::ZERO;

        const MAX_TIME: f64 = 1.0 / 5.0;
        let mut distance_accum = Vector3::ZERO;
        let mut time_accum = 0.0f64;
        let mut base_time = 0.0f64;

        let seconds_per_unit = if self.physics_step {
            ticks_per_second
        } else {
            1_000_000.0
        };

        if self.position_history_len > 0 {
            base_time = current_frame.saturating_sub(self.position_history[0].frame) as f64
                / seconds_per_unit;
        }

        if self.position_history_len > 0 {
            for index in 0..self.position_history_len - 1 {
                let diff = self.position_history[index]
                    .frame
                    .saturating_sub(self.position_history[index + 1].frame);
                let distance = self.position_history[index].position
                    - self.position_history[index + 1].position;
                let delta = diff as f64 / seconds_per_unit;

                if base_time + time_accum + delta > MAX_TIME {
                    break;
                }

                distance_accum += distance;
                time_accum += delta;
            }
        }

        if time_accum != 0.0 {
            linear_velocity = distance_accum / time_accum as f32;
        }
        linear_velocity
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn remote_transform_defaults_and_setters() {
        let mut transform = RemoteTransform3DData::new();
        assert!(transform.get_remote_node().is_empty());
        assert!(transform.get_use_global_coordinates());
        assert!(transform.get_update_position());
        assert!(transform.get_update_rotation());
        assert!(transform.get_update_scale());

        let path = NodePath::parse("Target").expect("path");
        transform.set_remote_node(path.clone());
        transform.set_use_global_coordinates(false);
        transform.set_update_position(false);
        transform.set_update_rotation(false);
        transform.set_update_scale(false);
        assert_eq!(transform.get_remote_node(), &path);
        assert!(!transform.get_use_global_coordinates());
        assert!(!transform.get_update_position());
        assert!(!transform.get_update_rotation());
        assert!(!transform.get_update_scale());
    }

    #[test]
    fn velocity_history_starts_empty() {
        let tracker = VelocityTracker3D::new();
        assert!(!tracker.is_tracking_physics_step());
        assert_eq!(tracker.get_history_len(), 0);
        assert_eq!(
            tracker.get_tracked_linear_velocity(0, 60.0),
            Vector3::ZERO
        );
    }

    #[test]
    fn constant_velocity_from_process_ticks() {
        let mut tracker = VelocityTracker3D::new();
        // Microsecond frame ticks: 0.1 s apart, one unit per step.
        tracker.update_position(Vector3::ZERO, 0);
        tracker.update_position(Vector3::new(1.0, 0.0, 0.0), 100_000);
        tracker.update_position(Vector3::new(2.0, 0.0, 0.0), 200_000);
        assert_eq!(tracker.get_history_len(), 3);

        let velocity = tracker.get_tracked_linear_velocity(200_000, 60.0);
        assert!(
            (velocity - Vector3::new(10.0, 0.0, 0.0)).length() < 1e-4,
            "velocity {velocity:?}"
        );
        // A diagonal motion accumulates per-axis.
        let mut diagonal = VelocityTracker3D::new();
        diagonal.update_position(Vector3::ZERO, 0);
        diagonal.update_position(Vector3::new(0.5, 0.5, 0.0), 50_000);
        let velocity = diagonal.get_tracked_linear_velocity(50_000, 60.0);
        assert!((velocity.x - 10.0).abs() < 1e-4);
        assert!((velocity.y - 10.0).abs() < 1e-4);
    }

    #[test]
    fn physics_mode_uses_ticks() {
        let mut tracker = VelocityTracker3D::new();
        tracker.set_track_physics_step(true);
        assert!(tracker.is_tracking_physics_step());
        // Physics frames: one unit per tick at 60 ticks/s.
        tracker.update_position(Vector3::ZERO, 0);
        tracker.update_position(Vector3::new(1.0, 0.0, 0.0), 1);
        tracker.update_position(Vector3::new(2.0, 0.0, 0.0), 2);
        let velocity = tracker.get_tracked_linear_velocity(2, 60.0);
        assert!((velocity.x - 60.0).abs() < 1e-4, "velocity {velocity:?}");
    }

    #[test]
    fn same_frame_updates_replace_the_latest() {
        let mut tracker = VelocityTracker3D::new();
        tracker.update_position(Vector3::ZERO, 0);
        tracker.update_position(Vector3::new(9.0, 9.0, 9.0), 0);
        assert_eq!(tracker.get_history_len(), 1);
        assert_eq!(
            tracker.get_history_entry(0).unwrap().position,
            Vector3::new(9.0, 9.0, 9.0)
        );

        // The history caps at four entries.
        for frame in 1..10u64 {
            tracker.update_position(Vector3::new(frame as f32, 0.0, 0.0), frame);
        }
        assert_eq!(tracker.get_history_len(), 4);
        assert_eq!(tracker.get_history_entry(0).unwrap().frame, 9);
        assert_eq!(tracker.get_history_entry(3).unwrap().frame, 6);

        tracker.reset(Vector3::new(5.0, 5.0, 5.0), 100);
        assert_eq!(tracker.get_history_len(), 1);
        assert_eq!(tracker.get_history_entry(0).unwrap().frame, 100);
    }

    #[test]
    fn window_stops_after_a_fifth_of_a_second() {
        let mut tracker = VelocityTracker3D::new();
        // One entry per 0.1 s: three steps = 0.3 s, so the loop stops early.
        tracker.update_position(Vector3::ZERO, 0);
        tracker.update_position(Vector3::new(1.0, 0.0, 0.0), 100_000);
        tracker.update_position(Vector3::new(2.0, 0.0, 0.0), 200_000);
        tracker.update_position(Vector3::new(3.0, 0.0, 0.0), 300_000);
        let velocity = tracker.get_tracked_linear_velocity(300_000, 60.0);
        // Only the first two deltas fit into the 0.2 s window.
        assert!((velocity.x - 10.0).abs() < 1e-4, "velocity {velocity:?}");
    }
}
