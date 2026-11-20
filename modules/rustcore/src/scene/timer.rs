//! `scene/main/timer.*`: the timing state machine.
//!
//! Ported: `wait_time` (rejects values `<= 0`), `one_shot`, `autostart`,
//! `paused`, `time_left`, and the process step: decrement while running, emit
//! once at zero, then stop (`one_shot`) or restart.
//!
//! Not ported: the scene-tree requirement of `start()`, the `timeout` signal,
//! and the process callback registration (that lives in `Node`).

#[derive(Clone, Copy, Debug)]
pub struct TimerData {
    wait_time: f64,
    one_shot: bool,
    autostart: bool,
    paused: bool,
    time_left: f64,
}

impl Default for TimerData {
    fn default() -> Self {
        Self::new()
    }
}

impl TimerData {
    pub fn new() -> Self {
        Self {
            wait_time: 1.0,
            one_shot: false,
            autostart: false,
            paused: false,
            time_left: -1.0,
        }
    }

    /// `set_wait_time`: zero or negative is rejected and the old value kept.
    pub fn set_wait_time(&mut self, time: f64) -> bool {
        if time <= 0.0 {
            return false;
        }
        self.wait_time = time;
        true
    }

    pub fn get_wait_time(&self) -> f64 {
        self.wait_time
    }

    pub fn set_one_shot(&mut self, one_shot: bool) {
        self.one_shot = one_shot;
    }

    pub fn is_one_shot(&self) -> bool {
        self.one_shot
    }

    pub fn set_autostart(&mut self, autostart: bool) {
        self.autostart = autostart;
    }

    pub fn has_autostart(&self) -> bool {
        self.autostart
    }

    /// `start()`: a positive argument replaces `wait_time` first.
    pub fn start(&mut self, time: Option<f64>) -> bool {
        if let Some(t) = time {
            if t > 0.0 && !self.set_wait_time(t) {
                return false;
            }
        }
        self.time_left = self.wait_time;
        true
    }

    /// `stop()`: `time_left = -1` and autostart cleared, like the engine.
    pub fn stop(&mut self) {
        self.time_left = -1.0;
        self.autostart = false;
    }

    pub fn set_paused(&mut self, paused: bool) {
        self.paused = paused;
    }

    pub fn is_paused(&self) -> bool {
        self.paused
    }

    pub fn get_time_left(&self) -> f64 {
        self.time_left
    }

    pub fn is_stopped(&self) -> bool {
        self.time_left < 0.0
    }

    /// One process step. Returns true when the timer fired this step; repeat
    /// timers restart with `wait_time`, one-shots stop.
    pub fn advance(&mut self, delta: f64) -> bool {
        if self.paused || self.time_left < 0.0 {
            return false;
        }

        if self.time_left > delta {
            self.time_left -= delta;
            return false;
        }

        // Fired. Carry the remainder is not part of the engine's behaviour:
        // it restarts from the full wait time.
        if self.one_shot {
            self.time_left = -1.0;
        } else {
            self.time_left = self.wait_time;
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults() {
        let t = TimerData::new();
        assert_eq!(t.get_wait_time(), 1.0);
        assert!(!t.is_one_shot());
        assert!(!t.has_autostart());
        assert!(!t.is_paused());
        assert!(t.is_stopped());
        assert_eq!(t.get_time_left(), -1.0);
    }

    #[test]
    fn wait_time_must_be_positive() {
        let mut t = TimerData::new();
        assert!(!t.set_wait_time(0.0));
        assert!(!t.set_wait_time(-3.0));
        assert_eq!(t.get_wait_time(), 1.0);
        assert!(t.set_wait_time(0.5));
        assert_eq!(t.get_wait_time(), 0.5);
    }

    #[test]
    fn start_and_countdown() {
        let mut t = TimerData::new();
        t.set_wait_time(2.0);
        assert!(t.start(None));
        assert_eq!(t.get_time_left(), 2.0);
        assert!(!t.advance(0.5));
        assert_eq!(t.get_time_left(), 1.5);
        assert!(!t.advance(1.0));
        assert_eq!(t.get_time_left(), 0.5);
    }

    #[test]
    fn one_shot_stops_after_firing() {
        let mut t = TimerData::new();
        t.set_one_shot(true);
        t.set_wait_time(1.0);
        t.start(None);
        assert!(t.advance(1.0));
        assert!(t.is_stopped());
        // Further steps do nothing.
        assert!(!t.advance(1.0));
    }

    #[test]
    fn repeating_timer_restarts() {
        let mut t = TimerData::new();
        t.set_wait_time(1.0);
        t.start(None);
        assert!(t.advance(1.0));
        assert_eq!(t.get_time_left(), 1.0, "restarted with wait_time");
        assert!(!t.is_stopped());
        assert!(t.advance(1.1));
    }

    #[test]
    fn paused_and_stopped_timers_do_not_advance() {
        let mut t = TimerData::new();
        t.set_paused(true);
        t.start(None);
        assert!(!t.advance(5.0));
        assert_eq!(t.get_time_left(), 1.0);

        t.set_paused(false);
        t.stop();
        assert!(!t.advance(5.0));
        assert!(t.is_stopped());
        // stop() also clears autostart.
        assert!(!t.has_autostart());
    }

    #[test]
    fn start_with_a_time_overrides_wait_time() {
        let mut t = TimerData::new();
        assert!(t.start(Some(3.0)));
        assert_eq!(t.get_wait_time(), 3.0);
        assert_eq!(t.get_time_left(), 3.0);
        // A non-positive override is ignored, keeping the current wait time.
        assert!(t.start(Some(0.0)));
        assert_eq!(t.get_wait_time(), 3.0);
        assert_eq!(t.get_time_left(), 3.0);
    }
}
