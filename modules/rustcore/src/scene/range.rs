//! `scene/gui/range.*`: value/min/max/page/step state.
//!
//! Ported exactly: `_calc_value` (step snapping offset by `min`, optional
//! rounding, the `max - page` and `min` clamps with `allow_greater` /
//! `allow_lesser`), the `set_min`/`set_max`/`set_page` revalidation order, and
//! the linear and exponential `as_ratio` conversions.
//!
//! The engine's `_snapped_r128` rescales before snapping to keep precision for
//! fine steps; this port uses `Math::snapped` (`round(value / step) * step`),
//! which is identical for ordinary ranges and only differs past the precision
//! the 128-bit path preserves. Range sharing between nodes and the
//! `value_changed`/`changed` signals are not ported.

#[derive(Clone, Copy, Debug)]
pub struct RangeData {
    value: f64,
    min: f64,
    max: f64,
    step: f64,
    page: f64,
    exp_ratio: bool,
    allow_greater: bool,
    allow_lesser: bool,
    rounded_values: bool,
}

impl Default for RangeData {
    fn default() -> Self {
        Self::new()
    }
}

fn snapped(value: f64, step: f64) -> f64 {
    if step == 0.0 {
        return value;
    }
    (value / step).round() * step
}

impl RangeData {
    pub fn new() -> Self {
        Self {
            value: 0.0,
            min: 0.0,
            max: 100.0,
            step: 1.0,
            page: 0.0,
            exp_ratio: false,
            allow_greater: false,
            allow_lesser: false,
            rounded_values: false,
        }
    }

    /// `_calc_value`: snap, round, then clamp against `max - page` / `min`.
    fn calc_value(&self, value: f64) -> f64 {
        if value.is_nan() {
            return value;
        }

        let mut value = value;
        if self.step > 0.0 {
            if self.min.abs() > self.step * 1e14 {
                value = snapped(value, self.step);
            } else {
                // Subtract min so min = 0.1, step = 0.2 snaps to 0.1, 0.3, ...
                value = snapped(value - self.min, self.step) + self.min;
            }
        }

        if self.rounded_values {
            value = value.round();
        }

        if !self.allow_greater && value > self.max - self.page {
            value = self.max - self.page;
        }
        if !self.allow_lesser && value < self.min {
            value = self.min;
        }
        value
    }

    /// Returns whether the value actually changed (the engine emits a signal).
    pub fn set_value(&mut self, value: f64) -> bool {
        let previous = self.value;
        self.value = self.calc_value(value);
        self.value != previous && !(self.value.is_nan() && previous.is_nan())
    }

    /// `set_value_no_signal`: recalculates without reporting the change.
    pub fn set_value_no_signal(&mut self, value: f64) {
        self.value = self.calc_value(value);
    }

    pub fn get_value(&self) -> f64 {
        self.value
    }

    pub fn set_min(&mut self, min: f64) {
        if self.min == min {
            return;
        }
        self.min = min;
        self.max = self.max.max(self.min);
        self.page = self.page.clamp(0.0, self.max - self.min);
        self.set_value(self.value);
    }

    pub fn get_min(&self) -> f64 {
        self.min
    }

    pub fn set_max(&mut self, max: f64) {
        let validated = max.max(self.min);
        if self.max == validated {
            return;
        }
        self.max = validated;
        self.page = self.page.clamp(0.0, self.max - self.min);
        self.set_value(self.value);
    }

    pub fn get_max(&self) -> f64 {
        self.max
    }

    pub fn set_step(&mut self, step: f64) {
        self.step = step;
    }

    pub fn get_step(&self) -> f64 {
        self.step
    }

    pub fn set_page(&mut self, page: f64) {
        let validated = page.clamp(0.0, self.max - self.min);
        if self.page == validated {
            return;
        }
        self.page = validated;
        self.set_value(self.value);
    }

    pub fn get_page(&self) -> f64 {
        self.page
    }

    pub fn set_as_ratio(&mut self, ratio: f64) {
        let value = if self.exp_ratio && self.min >= 0.0 {
            let exp_min = if self.min == 0.0 {
                0.0
            } else {
                self.min.log2()
            };
            let exp_max = self.max.log2();
            2.0f64.powf(exp_min + (exp_max - exp_min) * ratio)
        } else {
            let percent = (self.max - self.min) * ratio;
            if self.step > 0.0 {
                (percent / self.step).round() * self.step + self.min
            } else {
                percent + self.min
            }
        };
        let value = value.clamp(self.min, self.max);
        self.set_value(value);
    }

    pub fn get_as_ratio(&self) -> f64 {
        if (self.max - self.min).abs() < 1e-5 * self.max.abs().max(self.min.abs()).max(1.0) {
            // Approximate equality, avoiding the division by zero.
            return 1.0;
        }

        if self.exp_ratio && self.min >= 0.0 {
            let exp_min = if self.min == 0.0 {
                0.0
            } else {
                self.min.log2()
            };
            let exp_max = self.max.log2();
            let value = self.value.clamp(self.min, self.max);
            let v = value.log2();
            ((v - exp_min) / (exp_max - exp_min)).clamp(0.0, 1.0)
        } else {
            let value = self.value.clamp(self.min, self.max);
            ((value - self.min) / (self.max - self.min)).clamp(0.0, 1.0)
        }
    }

    pub fn set_use_rounded_values(&mut self, enable: bool) {
        self.rounded_values = enable;
    }

    pub fn is_using_rounded_values(&self) -> bool {
        self.rounded_values
    }

    pub fn set_exp_ratio(&mut self, enable: bool) {
        self.exp_ratio = enable;
    }

    pub fn is_ratio_exp(&self) -> bool {
        self.exp_ratio
    }

    pub fn set_allow_greater(&mut self, allow: bool) {
        self.allow_greater = allow;
    }

    pub fn is_greater_allowed(&self) -> bool {
        self.allow_greater
    }

    pub fn set_allow_lesser(&mut self, allow: bool) {
        self.allow_lesser = allow;
    }

    pub fn is_lesser_allowed(&self) -> bool {
        self.allow_lesser
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults() {
        let r = RangeData::new();
        assert_eq!(r.get_value(), 0.0);
        assert_eq!(r.get_min(), 0.0);
        assert_eq!(r.get_max(), 100.0);
        assert_eq!(r.get_step(), 1.0);
        assert_eq!(r.get_page(), 0.0);
        assert!(!r.is_ratio_exp());
        assert!(!r.is_greater_allowed());
        assert!(!r.is_lesser_allowed());
        assert!(!r.is_using_rounded_values());
    }

    #[test]
    fn values_snap_to_the_step_and_clamp() {
        let mut r = RangeData::new();
        assert!(r.set_value(2.4));
        assert_eq!(r.get_value(), 2.0);
        assert!(r.set_value(2.6));
        assert_eq!(r.get_value(), 3.0);

        // Above max clamps to max.
        r.set_value(1000.0);
        assert_eq!(r.get_value(), 100.0);
        // Below min clamps to min.
        r.set_value(-100.0);
        assert_eq!(r.get_value(), 0.0);

        // Setting the same snapped value reports no change.
        r.set_value(1000.0);
        assert!(!r.set_value(1000.0));
    }

    #[test]
    fn snapping_is_offset_by_the_minimum() {
        let mut r = RangeData::new();
        r.set_min(0.1);
        r.set_max(1.0);
        r.set_step(0.2);
        r.set_value(0.4);
        // (0.4 - 0.1) / 0.2 = 1.5 -> round away from zero -> 2 -> 0.1 + 0.4.
        assert!((r.get_value() - 0.5).abs() < 1e-9, "{}", r.get_value());
        r.set_value(0.31);
        // (0.31 - 0.1) / 0.2 = 1.05 -> 1 -> 0.3.
        assert!((r.get_value() - 0.3).abs() < 1e-9, "{}", r.get_value());
    }

    #[test]
    fn rounded_values_and_page_clamp() {
        let mut r = RangeData::new();
        r.set_step(0.0);
        r.set_use_rounded_values(true);
        r.set_value(2.6);
        assert_eq!(r.get_value(), 3.0);

        // Page reduces the reachable maximum.
        r.set_use_rounded_values(false);
        r.set_step(1.0);
        r.set_page(10.0);
        r.set_value(95.0);
        assert_eq!(r.get_value(), 90.0);

        // allow_greater lifts the max-page clamp; allow_lesser lifts the min one.
        r.set_allow_greater(true);
        r.set_value(95.0);
        assert_eq!(r.get_value(), 95.0);
        r.set_allow_lesser(true);
        r.set_value(-10.0);
        assert_eq!(r.get_value(), -10.0);
    }

    #[test]
    fn min_max_and_page_revalidate() {
        let mut r = RangeData::new();
        r.set_value(50.0);
        r.set_min(80.0);
        // max stays at 100 (it is only raised when it would fall below min),
        // and the value is re-clamped up to the new minimum.
        assert_eq!(r.get_max(), 100.0);
        assert_eq!(r.get_value(), 80.0);
        assert_eq!(r.get_page(), 0.0);

        r.set_max(40.0);
        // max cannot go below min.
        assert_eq!(r.get_max(), 80.0);

        r.set_max(120.0);
        r.set_page(30.0);
        assert_eq!(r.get_page(), 30.0);
        r.set_page(500.0);
        assert_eq!(r.get_page(), 40.0);
        r.set_page(-5.0);
        assert_eq!(r.get_page(), 0.0);

        r.set_value(120.0);
        assert_eq!(r.get_value(), 120.0);

        // Shrinking max pulls page and value back in.
        r.set_page(40.0);
        r.set_max(100.0);
        assert_eq!(r.get_page(), 20.0);
        // max - page is now 80, so the value is pulled down with it.
        assert_eq!(r.get_value(), 80.0);
    }

    #[test]
    fn ratio_round_trips_linearly() {
        let mut r = RangeData::new();
        r.set_as_ratio(0.25);
        assert_eq!(r.get_value(), 25.0);
        assert!((r.get_as_ratio() - 0.25).abs() < 1e-9);

        r.set_min(20.0);
        r.set_max(40.0);
        r.set_as_ratio(0.5);
        assert_eq!(r.get_value(), 30.0);
        assert!((r.get_as_ratio() - 0.5).abs() < 1e-9);

        // Equal min/max reports 1.0 instead of dividing by zero.
        r.set_min(10.0);
        r.set_max(10.0);
        assert_eq!(r.get_as_ratio(), 1.0);
    }

    #[test]
    fn exponential_ratio() {
        let mut r = RangeData::new();
        r.set_min(1.0);
        r.set_max(8.0);
        r.set_exp_ratio(true);
        assert!(r.is_ratio_exp());
        // Disable snapping so the exponential value is observable exactly.
        r.set_step(0.0);

        r.set_as_ratio(0.5);
        // 2^(0 + 3*0.5) = 2^1.5 = 2.828427...
        assert!((r.get_value() - 2.8284271247461903).abs() < 1e-9, "{}", r.get_value());
        assert!((r.get_as_ratio() - 0.5).abs() < 1e-9);

        r.set_as_ratio(0.0);
        assert!((r.get_value() - 1.0).abs() < 1e-9);
        r.set_as_ratio(1.0);
        assert!((r.get_value() - 8.0).abs() < 1e-9);
    }
}
