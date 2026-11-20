//! `core/math/random_number_generator.*`: the scripting-facing RNG.
//!
//! A thin wrapper over [`RandomPcg`]: `set_seed`/`get_seed`, `set_state`/
//! `get_state`, `randomize`, and the six draw functions. The constructor
//! randomizes exactly like the engine (`RandomPCG` seeded from the clock), so
//! tests must seed first for determinism.
//!
//! `randf_range` uses the 32-bit path (`real_t` in this build is `float`) and
//! `randi_range` the signed-int path with the engine's swap/clamp rules.

use crate::math::pcg::RandomPcg;

#[derive(Clone, Copy, Debug)]
pub struct RandomNumberGenerator {
    randbase: RandomPcg,
}

impl Default for RandomNumberGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl RandomNumberGenerator {
    /// `RandomNumberGenerator()` calls `randbase.randomize()`.
    pub fn new() -> Self {
        let mut randbase = RandomPcg::new();
        randbase.randomize();
        Self { randbase }
    }

    /// Deterministic constructor for tests and callers that seed immediately.
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

    pub fn rand_weighted(&mut self, weights: &[f32]) -> i64 {
        self.randbase.rand_weighted(weights)
    }

    /// Direct access for callers that need the underlying generator.
    pub fn randbase(&self) -> &RandomPcg {
        &self.randbase
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seeding_is_deterministic() {
        let mut a = RandomNumberGenerator::with_seed(12345);
        let mut b = RandomNumberGenerator::with_seed(12345);
        for _ in 0..16 {
            assert_eq!(a.randi(), b.randi());
        }

        let mut c = RandomNumberGenerator::with_seed(54321);
        assert_ne!(a.randi(), c.randi());
        assert_eq!(c.get_seed(), 54321);
    }

    #[test]
    fn state_round_trips() {
        let mut a = RandomNumberGenerator::with_seed(1);
        a.randi();
        let state = a.get_state();
        let next = a.randi();

        let mut b = RandomNumberGenerator::with_seed(1);
        b.set_state(state);
        assert_eq!(b.get_state(), state);
        assert_eq!(b.randi(), next);
    }

    #[test]
    fn ranges_are_inclusive_and_ordered() {
        let mut rng = RandomNumberGenerator::with_seed(7);
        for _ in 0..64 {
            let float = rng.randf_range(2.0, 3.0);
            assert!((2.0..=3.0).contains(&float), "{float}");

            let int = rng.randi_range(-3, 3);
            assert!((-3..=3).contains(&int), "{int}");

            // Reversed bounds are swapped by the engine.
            let swapped = rng.randi_range(5, 1);
            assert!((1..=5).contains(&swapped), "{swapped}");
        }

        // Equal bounds return that value without consuming a number.
        let state = rng.get_state();
        assert_eq!(rng.randi_range(4, 4), 4);
        assert_eq!(rng.get_state(), state);
    }

    #[test]
    fn randf_is_in_the_unit_interval() {
        let mut rng = RandomNumberGenerator::with_seed(99);
        for _ in 0..128 {
            let value = rng.randf();
            assert!((0.0..1.0).contains(&value), "{value}");
        }
    }

    #[test]
    fn weighted_picks_are_valid_or_negative() {
        let mut rng = RandomNumberGenerator::with_seed(3);
        assert_eq!(rng.rand_weighted(&[]), -1);

        for _ in 0..64 {
            let index = rng.rand_weighted(&[1.0, 0.0, 2.0]);
            assert!(index == 0 || index == 2, "{index}");
        }

        // All-zero weights select nothing.
        assert_eq!(rng.rand_weighted(&[0.0, 0.0]), -1);
    }

    #[test]
    fn gaussian_draws_stay_near_the_mean() {
        let mut rng = RandomNumberGenerator::with_seed(11);
        let mut sum = 0.0f64;
        const N: usize = 2000;
        for _ in 0..N {
            sum += rng.randfn(5.0, 2.0) as f64;
        }
        let mean = sum / N as f64;
        assert!((mean - 5.0).abs() < 0.25, "{mean}");
    }

    #[test]
    fn randomize_changes_the_seed() {
        let mut rng = RandomNumberGenerator::with_seed(42);
        let before = rng.get_seed();
        rng.randomize();
        assert_ne!(rng.get_seed(), before);
    }
}
