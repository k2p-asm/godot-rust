//! `core/math/random_number_generator.h` (the `RandomNumberGenerator` class).
//!
//! Every method is `_FORCE_INLINE_` in the header and forwards to `RandomPCG`,
//! so this is a thin wrapper over the already bit-exact PCG port. The only
//! non-deterministic entry point is `new`, which mirrors the engine's
//! `randbase.randomize()`.
//!
//! Non-differential: the class is a `RefCounted` with `ClassDB` bindings, which
//! the parity harness cannot link. `RandomPCG` itself is differentially
//! verified elsewhere (`pcg.rs`, stream compared word by word).

use crate::math::pcg::RandomPcg;

pub struct RandomNumberGenerator {
    randbase: RandomPcg,
}

impl Default for RandomNumberGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl RandomNumberGenerator {
    /// Default constructor: seeds from the clock, like the engine.
    pub fn new() -> Self {
        let mut randbase = RandomPcg::new();
        randbase.randomize();
        Self { randbase }
    }

    /// Deterministic constructor, for tests and for `set_seed`.
    pub fn with_seed(seed: u64) -> Self {
        let mut randbase = RandomPcg::new();
        randbase.seed(seed);
        Self { randbase }
    }

    pub fn set_seed(&mut self, seed: u64) {
        self.randbase.seed(seed);
    }

    pub fn get_seed(&self) -> u64 {
        self.randbase.get_seed()
    }

    pub fn set_state(&mut self, state: u64) {
        self.randbase.set_state(state);
    }

    pub fn get_state(&self) -> u64 {
        self.randbase.get_state()
    }

    pub fn randomize(&mut self) {
        self.randbase.randomize();
    }

    pub fn randi(&mut self) -> u32 {
        self.randbase.rand()
    }

    pub fn randf(&mut self) -> f32 {
        self.randbase.randf()
    }

    pub fn randf_range(&mut self, from: f32, to: f32) -> f32 {
        self.randbase.random_f32(from, to)
    }

    pub fn randfn(&mut self, mean: f32, deviation: f32) -> f32 {
        self.randbase.randfn_f32(mean, deviation)
    }

    pub fn randi_range(&mut self, from: i32, to: i32) -> i32 {
        self.randbase.random_int(from, to)
    }

    /// -1 when the array is empty or has a negative weight.
    pub fn rand_weighted(&mut self, weights: &[f32]) -> i64 {
        self.randbase.rand_weighted(weights)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_same_stream() {
        let mut a = RandomNumberGenerator::with_seed(1234567);
        let mut b = RandomNumberGenerator::with_seed(1234567);
        for _ in 0..64 {
            assert_eq!(a.randi(), b.randi());
        }
        assert_eq!(a.get_state(), b.get_state());
    }

    #[test]
    fn state_round_trip_continues_the_stream() {
        let mut a = RandomNumberGenerator::with_seed(99);
        for _ in 0..10 {
            a.randi();
        }
        let state = a.get_state();
        let expected: Vec<u32> = (0..8).map(|_| a.randi()).collect();

        let mut b = RandomNumberGenerator::with_seed(0);
        b.set_state(state);
        let got: Vec<u32> = (0..8).map(|_| b.randi()).collect();
        assert_eq!(expected, got);
    }

    #[test]
    fn ranges_respect_their_bounds() {
        let mut r = RandomNumberGenerator::with_seed(7);
        for _ in 0..5000 {
            let f = r.randf();
            assert!((0.0..1.0).contains(&f), "randf out of range: {f}");
            let i = r.randi_range(-3, 5);
            assert!((-3..=5).contains(&i), "randi_range out of range: {i}");
            let v = r.randf_range(2.0, 4.0);
            assert!((2.0..4.0).contains(&v), "randf_range out of range: {v}");
        }
    }

    #[test]
    fn randfn_is_centred() {
        let mut r = RandomNumberGenerator::with_seed(42);
        let n = 20_000;
        let mut sum = 0.0f64;
        let mut sum_sq = 0.0f64;
        for _ in 0..n {
            let v = r.randfn(3.0, 2.0) as f64;
            sum += v;
            sum_sq += v * v;
        }
        let mean = sum / n as f64;
        let var = sum_sq / n as f64 - mean * mean;
        assert!((mean - 3.0).abs() < 0.05, "mean {mean}");
        assert!((var - 4.0).abs() < 0.15, "variance {var}");
    }

    #[test]
    fn weighted_picks_follow_the_weights() {
        let mut r = RandomNumberGenerator::with_seed(2024);
        let weights = [0.0, 1.0, 3.0];
        let mut counts = [0usize; 3];
        for _ in 0..10_000 {
            let i = r.rand_weighted(&weights);
            assert!(i >= 0 && i < 3, "index {i}");
            counts[i as usize] += 1;
        }
        assert_eq!(counts[0], 0, "zero weight must never be picked");
        let ratio = counts[2] as f64 / counts[1] as f64;
        assert!((ratio - 3.0).abs() < 0.3, "ratio {ratio}");

        assert_eq!(r.rand_weighted(&[]), -1);
        assert_eq!(r.rand_weighted(&[1.0, -1.0]), -1);
    }
}
