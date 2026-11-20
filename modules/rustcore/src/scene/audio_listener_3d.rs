//! `scene/3d/audio_listener_3d.*`: listener state.
//!
//! Tracks the `current` flag, the Doppler mode (disabled by default) and an
//! internal velocity tracker. The viewport registration, scenario RID and
//! listener transform updates are not ported; the tracked velocity is queried
//! through the tracker with explicit frames/ticks.

use crate::math::vector3::Vector3;
use crate::scene::velocity_tracker_3d::VelocityTracker3DData;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum DopplerTracking {
    Disabled,
    IdleStep,
    PhysicsStep,
}

#[derive(Clone, Debug)]
pub struct AudioListener3DData {
    current: bool,
    doppler_tracking: DopplerTracking,
    velocity_tracker: VelocityTracker3DData,
}

impl Default for AudioListener3DData {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioListener3DData {
    pub fn new() -> Self {
        Self {
            current: false,
            doppler_tracking: DopplerTracking::Disabled,
            velocity_tracker: VelocityTracker3DData::new(),
        }
    }

    pub fn make_current(&mut self) {
        self.current = true;
    }

    pub fn clear_current(&mut self) {
        self.current = false;
    }

    pub fn is_current(&self) -> bool {
        self.current
    }

    pub fn set_doppler_tracking(&mut self, tracking: DopplerTracking) {
        self.doppler_tracking = tracking;
        self.velocity_tracker
            .set_track_physics_step(tracking == DopplerTracking::PhysicsStep);
    }

    pub fn get_doppler_tracking(&self) -> DopplerTracking {
        self.doppler_tracking
    }

    /// Tracker hook used when the listener moves.
    pub fn update_position(&mut self, position: Vector3, frame: u64) {
        self.velocity_tracker.update_position(position, frame);
    }

    pub fn reset_tracking(&mut self, position: Vector3, frame: u64) {
        self.velocity_tracker.reset(position, frame);
    }

    /// Zero while Doppler tracking is disabled, like the engine.
    pub fn get_doppler_tracked_velocity(
        &self,
        now: u64,
        physics_ticks_per_second: f64,
    ) -> Vector3 {
        if self.doppler_tracking == DopplerTracking::Disabled {
            return Vector3::ZERO;
        }
        self.velocity_tracker
            .get_tracked_linear_velocity(now, physics_ticks_per_second)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults() {
        let l = AudioListener3DData::new();
        assert!(!l.is_current());
        assert_eq!(l.get_doppler_tracking(), DopplerTracking::Disabled);
        assert_eq!(l.get_doppler_tracked_velocity(0, 60.0), Vector3::ZERO);
    }

    #[test]
    fn current_flag_and_modes() {
        let mut l = AudioListener3DData::new();
        l.make_current();
        assert!(l.is_current());
        l.make_current();
        assert!(l.is_current());
        l.clear_current();
        assert!(!l.is_current());

        l.set_doppler_tracking(DopplerTracking::PhysicsStep);
        assert_eq!(l.get_doppler_tracking(), DopplerTracking::PhysicsStep);
        assert_eq!(DopplerTracking::Disabled as i32, 0);
        assert_eq!(DopplerTracking::IdleStep as i32, 1);
        assert_eq!(DopplerTracking::PhysicsStep as i32, 2);
    }

    #[test]
    fn doppler_tracking_uses_the_tracker() {
        let mut l = AudioListener3DData::new();
        l.set_doppler_tracking(DopplerTracking::PhysicsStep);
        l.reset_tracking(Vector3::ZERO, 0);
        l.update_position(Vector3::new(0.5, 0.0, 0.0), 1);
        let v = l.get_doppler_tracked_velocity(1, 60.0);
        assert!((v.x - 30.0).abs() < 1e-4, "{v:?}");

        // Disabling returns zero even with history.
        l.set_doppler_tracking(DopplerTracking::Disabled);
        assert_eq!(l.get_doppler_tracked_velocity(1, 60.0), Vector3::ZERO);

        // Idle mode uses microseconds.
        l.set_doppler_tracking(DopplerTracking::IdleStep);
        l.reset_tracking(Vector3::ZERO, 1_000_000);
        l.update_position(Vector3::new(2.0, 0.0, 0.0), 1_100_000);
        let v = l.get_doppler_tracked_velocity(1_100_000, 60.0);
        assert!((v.x - 20.0).abs() < 1e-4, "{v:?}");
    }
}
