//! Port of `core/math/random_pcg.h/.cpp` + `thirdparty/misc/pcg.h/.cpp`.
//!
//! Bit-exact by construction: every operation is wrapping arithmetic, so the
//! generated sequence matches the C++ engine for the same seed. This matters
//! because `RandomNumberGenerator` is part of the scripting API and seeds are
//! stored in project files.

use crate::math::funcs;

pub const PCG_DEFAULT_INC_64: u64 = 1442695040888963407;
pub const PCG_MULTIPLIER_64: u64 = 6364136223846793005;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Pcg32 {
    pub state: u64,
    pub inc: u64,
}

impl Pcg32 {
    #[inline]
    pub fn new() -> Self {
        Self { state: 0, inc: 0 }
    }

    #[inline]
    pub fn next_u32(&mut self) -> u32 {
        let oldstate = self.state;
        self.state = oldstate
            .wrapping_mul(PCG_MULTIPLIER_64)
            .wrapping_add(self.inc | 1);
        let xorshifted = (((oldstate >> 18) ^ oldstate) >> 27) as u32;
        let rot = (oldstate >> 59) as u32;
        xorshifted.rotate_right(rot)
    }

    #[inline]
    pub fn srandom(&mut self, initstate: u64, initseq: u64) {
        self.state = 0;
        self.inc = (initseq << 1).wrapping_add(1); // | 1
        self.next_u32();
        self.state = self.state.wrapping_add(initstate);
        self.next_u32();
    }

    #[inline]
    pub fn bounded(&mut self, bound: u32) -> u32 {
        let threshold = bound.wrapping_neg() % bound;
        loop {
            let r = self.next_u32();
            if r >= threshold {
                return r % bound;
            }
        }
    }
}

impl Default for Pcg32 {
    fn default() -> Self {
        Self::new()
    }
}

/// Port of `RandomPCG`. Keeps `current_seed` the same way the engine does, so
/// `get_seed()` round-trips identically.
#[derive(Clone, Copy, Debug)]
pub struct RandomPcg {
    pcg: Pcg32,
    current_seed: u64,
    current_inc: u64,
}

impl RandomPcg {
    pub const DEFAULT_SEED: u64 = 12047754176567800795;
    pub const DEFAULT_INC: u64 = PCG_DEFAULT_INC_64;

    pub fn new() -> Self {
        let mut rng = Self {
            pcg: Pcg32::new(),
            current_seed: 0,
            current_inc: Self::DEFAULT_INC,
        };
        rng.seed(Self::DEFAULT_SEED);
        rng
    }

    pub fn with_seed(seed: u64, inc: u64) -> Self {
        let mut rng = Self {
            pcg: Pcg32::new(),
            current_seed: 0,
            current_inc: inc,
        };
        rng.seed_with_inc(seed, inc);
        rng
    }

    #[inline]
    pub fn seed(&mut self, seed: u64) {
        self.current_seed = seed;
        self.pcg.srandom(self.current_seed, self.current_inc);
    }

    #[inline]
    pub fn seed_with_inc(&mut self, seed: u64, inc: u64) {
        self.current_inc = inc;
        self.seed(seed);
    }

    #[inline]
    pub fn get_seed(&self) -> u64 {
        self.current_seed
    }

    #[inline]
    pub fn set_state(&mut self, state: u64) {
        self.pcg.state = state;
    }

    #[inline]
    pub fn get_state(&self) -> u64 {
        self.pcg.state
    }

    #[inline]
    pub fn rand(&mut self) -> u32 {
        self.pcg.next_u32()
    }

    #[inline]
    pub fn rand_bounded(&mut self, bounds: u32) -> u32 {
        self.pcg.bounded(bounds)
    }

    /// Same algorithm as the Clang/GCC/MSVC path of the C++ implementation
    /// (`CLZ32` is always available on x86-64 MSVC).
    #[inline]
    pub fn randd(&mut self) -> f64 {
        let proto_exp_offset = self.rand();
        if proto_exp_offset == 0 {
            return 0.0;
        }
        let significand =
            ((self.rand() as u64) << 32) | self.rand() as u64 | 0x8000000000000001u64;
        let e = -64i32 - proto_exp_offset.leading_zeros() as i32;
        (significand as f64) * 2.0f64.powi(e)
    }

    #[inline]
    pub fn randf(&mut self) -> f32 {
        let proto_exp_offset = self.rand();
        if proto_exp_offset == 0 {
            return 0.0;
        }
        let bits = self.rand() | 0x80000001u32;
        let e = -32i32 - proto_exp_offset.leading_zeros() as i32;
        (bits as f32) * 2.0f32.powi(e)
    }

    #[inline]
    pub fn randfn_f64(&mut self, mean: f64, deviation: f64) -> f64 {
        let mut temp = self.randd();
        if temp < funcs::CMP_EPSILON {
            temp += funcs::CMP_EPSILON;
        }
        mean + deviation * ((funcs::TAU * self.randd()).cos() * (-2.0 * temp.ln()).sqrt())
    }

    #[inline]
    pub fn randfn_f32(&mut self, mean: f32, deviation: f32) -> f32 {
        let mut temp = self.randf();
        let eps = funcs::CMP_EPSILON as f32;
        if temp < eps {
            temp += eps;
        }
        mean + deviation * ((funcs::TAU as f32 * self.randf()).cos() * (-2.0f32 * temp.ln()).sqrt())
    }

    pub fn random_f64(&mut self, from: f64, to: f64) -> f64 {
        self.randd() * (to - from) + from
    }

    pub fn random_f32(&mut self, from: f32, to: f32) -> f32 {
        self.randf() * (to - from) + from
    }

    pub fn random_int(&mut self, from: i32, to: i32) -> i32 {
        if from == to {
            return from;
        }
        let min = from.min(to) as i64;
        let max = from.max(to) as i64;
        let diff = (max - min) as u32;
        if diff == u32::MAX {
            return self.rand() as i64 as i32 + min as i32;
        }
        (self.rand_bounded(diff + 1) as i64 + min) as i32
    }

    /// `Math::rand_from_seed`: advances a temporary RNG and writes the seed back.
    pub fn rand_from_seed(seed: &mut u64) -> u32 {
        let mut rng = RandomPcg::with_seed(*seed, PCG_DEFAULT_INC_64);
        let r = rng.rand();
        *seed = rng.get_seed();
        r
    }
}

impl Default for RandomPcg {
    fn default() -> Self {
        Self::new()
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bounded_is_in_range() {
        let mut rng = RandomPcg::new();
        for _ in 0..1000 {
            let v = rng.rand_bounded(7);
            assert!(v < 7);
        }
    }

    #[test]
    fn randf_is_unit_interval() {
        let mut rng = RandomPcg::with_seed(1234, PCG_DEFAULT_INC_64);
        for _ in 0..10000 {
            let v = rng.randf();
            assert!((0.0..=1.0).contains(&v), "{v}");
        }
    }

    #[test]
    fn seed_roundtrip() {
        let mut seed = 987654321u64;
        let a = RandomPcg::rand_from_seed(&mut seed);
        let b = RandomPcg::rand_from_seed(&mut seed);
        // `get_seed()` returns the seed the generator started from, so the
        // seed is not advanced and the result is deterministic.
        assert_eq!(a, b);
        assert_eq!(seed, 987654321u64);
    }
}
