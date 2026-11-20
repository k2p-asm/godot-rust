//! `scene/3d/velocity_tracker_3d.*`: finite-difference velocity.
//!
//! The engine keeps a four-entry history of `(frame, position)` and averages the
//! displacement over at most 0.2 s, dropping older samples once that window is
//! exceeded. Frames are either physics frames or microsecond ticks (idle), and
//! the time conversion uses the physics rate or `1e6` respectively.
//!
//! Because the port has no `Engine` singleton, the caller passes the current
//! frame/ticks and the physics rate to `update_position`/`get_tracked_linear_velocity`.

use crate::math::vector3::Vector3;

const HISTORY_SIZE: usize = 4;
const MAX_TIME: f64 = 1.0 / 5.0;

#[derive(Clone, Copy, Debug, Default)]
struct PositionHistory {
    frame: u64,
    position: Vector3,
}

#[derive(Clone, Debug)]
pub struct VelocityTracker3DData {
    physics_step: bool,
    position_history: [PositionHistory; HISTORY_SIZE],
    position_history_len: usize,
}

impl Default for VelocityTracker3DData {
    fn default() -> Self {
        Self::new()
    }
}

impl VelocityTracker3DData {
    pub fn new() -> Self {
        Self {
            physics_step: false,
            position_history: [PositionHistory::default(); HISTORY_SIZE],
            position_history_len: 0,
        }
    }

    pub fn set_track_physics_step(&mut self, track_physics_step: bool) {
        self.physics_step = track_physics_step;
    }

    pub fn is_tracking_physics_step(&self) -> bool {
        self.physics_step
    }

    /// `update_position`: same-frame updates overwrite the newest sample.
    pub fn update_position(&mut self, position: Vector3, frame: u64) {
        if self.position_history_len == 0 || self.position_history[0].frame != frame {
            self.position_history_len = HISTORY_SIZE.min(self.position_history_len + 1);
            for i in (1..self.position_history_len).rev() {
                self.position_history[i] = self.position_history[i - 1];
            }
        }
        self.position_history[0] = PositionHistory { frame, position };
    }

    /// `reset` seeds the history with one sample at `frame`.
    pub fn reset(&mut self, position: Vector3, frame: u64) {
        self.position_history[0] = PositionHistory { frame, position };
        self.position_history_len = 1;
    }

    /// Average velocity over the history window ending at `now`.
    pub fn get_tracked_linear_velocity(&self, now: u64, physics_ticks_per_second: f64) -> Vector3 {
        if self.position_history_len == 0 {
            return Vector3::ZERO;
        }

        let to_seconds = |frames: u64| -> f64 {
            if self.physics_step {
                frames as f64 / physics_ticks_per_second
            } else {
                frames as f64 / 1_000_000.0
            }
        };

        let base_time = to_seconds(now.saturating_sub(self.position_history[0].frame));

        let mut distance_accum = Vector3::ZERO;
        let mut time_accum = 0.0;
        for i in 0..self.position_history_len.saturating_sub(1) {
            let current = self.position_history[i];
            let previous = self.position_history[i + 1];
            let diff = current.frame.saturating_sub(previous.frame);
            let distance = current.position - previous.position;
            let delta = to_seconds(diff);

            if base_time + time_accum + delta > MAX_TIME {
                break;
            }

            distance_accum += distance;
            time_accum += delta;
        }

        if time_accum != 0.0 {
            distance_accum / time_accum as f32
        } else {
            Vector3::ZERO
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_and_empty_history() {
        let mut t = VelocityTracker3DData::new();
        assert!(!t.is_tracking_physics_step());
        assert_eq!(t.get_tracked_linear_velocity(0, 60.0), Vector3::ZERO);
        t.set_track_physics_step(true);
        assert!(t.is_tracking_physics_step());
    }

    #[test]
    fn physics_velocity_from_two_frames() {
        let mut t = VelocityTracker3DData::new();
        t.set_track_physics_step(true);
        t.reset(Vector3::ZERO, 10);
        t.update_position(Vector3::new(0.5, 0.0, 0.0), 11);

        // 0.5 units over 1 frame at 60 Hz.
        let v = t.get_tracked_linear_velocity(11, 60.0);
        assert!((v.x - 30.0).abs() < 1e-4, "{v:?}");
        assert_eq!(v.y, 0.0);
    }

    #[test]
    fn same_frame_updates_use_the_latest_position() {
        let mut t = VelocityTracker3DData::new();
        t.set_track_physics_step(true);
        t.reset(Vector3::ZERO, 0);
        t.update_position(Vector3::new(1.0, 0.0, 0.0), 1);
        t.update_position(Vector3::new(2.0, 0.0, 0.0), 1);

        let v = t.get_tracked_linear_velocity(1, 60.0);
        assert!((v.x - 120.0).abs() < 1e-4, "{v:?}");
    }

    #[test]
    fn idle_uses_microsecond_ticks() {
        let mut t = VelocityTracker3DData::new();
        t.reset(Vector3::ZERO, 1_000_000);
        t.update_position(Vector3::new(2.0, 0.0, 0.0), 1_100_000);

        // 2 units over 0.1 s (the 0.2 s window caps larger gaps).
        let v = t.get_tracked_linear_velocity(1_100_000, 60.0);
        assert!((v.x - 20.0).abs() < 1e-4, "{v:?}");

        // A gap longer than the window produces no velocity.
        let stale = t.get_tracked_linear_velocity(1_500_000, 60.0);
        assert_eq!(stale, Vector3::ZERO);
    }

    #[test]
    fn history_is_capped_and_window_limited() {
        let mut t = VelocityTracker3DData::new();
        t.set_track_physics_step(true);
        t.reset(Vector3::ZERO, 0);
        // Four more samples: the history only keeps four entries.
        for frame in 1..=5 {
            t.update_position(Vector3::new(frame as f32, 0.0, 0.0), frame);
        }
        assert_eq!(t.position_history_len, 4);
        // Newest four are frames 2..=5, so the average covers 3 frames = 0.05 s
        // and 3 units.
        let v = t.get_tracked_linear_velocity(5, 60.0);
        assert!((v.x - 60.0).abs() < 1e-3, "{v:?}");

        // Waiting more than 0.2 s after the newest sample drops everything.
        let stale = t.get_tracked_linear_velocity(1000, 60.0);
        assert_eq!(stale, Vector3::ZERO);
    }

    #[test]
    fn reset_replaces_history() {
        let mut t = VelocityTracker3DData::new();
        t.set_track_physics_step(true);
        t.update_position(Vector3::new(5.0, 0.0, 0.0), 1);
        t.reset(Vector3::new(-1.0, 0.0, 0.0), 2);
        assert_eq!(t.position_history_len, 1);
        // Only one sample: no interval to average, so zero velocity.
        assert_eq!(t.get_tracked_linear_velocity(2, 60.0), Vector3::ZERO);
    }
}
