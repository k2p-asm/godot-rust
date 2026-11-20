//! `servers/audio/effects/audio_effect_eq.*`: band gains.
//!
//! `AudioEffectEQ` holds one gain per band; the three subclasses fix the band
//! count (6, 10, 21). Gains start at 0 dB, `set_band_gain_db` rejects an
//! out-of-range band (`ERR_FAIL_INDEX`) and `get_band_gain_db` returns 0 for it,
//! with no clamping of the dB value. The EQ filters are not ported.

pub const EQ6_BANDS: usize = 6;
pub const EQ10_BANDS: usize = 10;
pub const EQ21_BANDS: usize = 21;

#[derive(Clone, Debug)]
pub struct AudioEffectEq {
    gain: Vec<f32>,
}

impl Default for AudioEffectEq {
    fn default() -> Self {
        Self::eq6()
    }
}

impl AudioEffectEq {
    /// Only the engine's presets are valid band counts.
    pub fn new(bands: usize) -> Self {
        assert!(
            matches!(bands, EQ6_BANDS | EQ10_BANDS | EQ21_BANDS),
            "band count must be 6, 10 or 21"
        );
        Self {
            gain: vec![0.0; bands],
        }
    }

    pub fn eq6() -> Self {
        Self::new(EQ6_BANDS)
    }

    pub fn eq10() -> Self {
        Self::new(EQ10_BANDS)
    }

    pub fn eq21() -> Self {
        Self::new(EQ21_BANDS)
    }

    pub fn get_band_count(&self) -> usize {
        self.gain.len()
    }

    /// Rejected for out-of-range bands; the value is stored as given.
    pub fn set_band_gain_db(&mut self, band: usize, volume_db: f32) -> bool {
        match self.gain.get_mut(band) {
            Some(g) => {
                *g = volume_db;
                true
            }
            None => false,
        }
    }

    /// Out-of-range bands read as 0 dB, like the engine's fallback.
    pub fn get_band_gain_db(&self, band: usize) -> f32 {
        self.gain.get(band).copied().unwrap_or(0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn band_counts_match_the_presets() {
        assert_eq!(AudioEffectEq::eq6().get_band_count(), 6);
        assert_eq!(AudioEffectEq::eq10().get_band_count(), 10);
        assert_eq!(AudioEffectEq::eq21().get_band_count(), 21);
        assert_eq!(AudioEffectEq::default().get_band_count(), 6);
    }

    #[test]
    fn gains_start_at_zero_and_round_trip() {
        let mut eq = AudioEffectEq::eq10();
        for band in 0..10 {
            assert_eq!(eq.get_band_gain_db(band), 0.0);
        }
        assert!(eq.set_band_gain_db(3, -6.5));
        assert_eq!(eq.get_band_gain_db(3), -6.5);

        // No clamping on the value.
        assert!(eq.set_band_gain_db(4, 40.0));
        assert_eq!(eq.get_band_gain_db(4), 40.0);
    }

    #[test]
    fn out_of_range_bands_are_handled_like_the_engine() {
        let mut eq = AudioEffectEq::eq6();
        assert!(!eq.set_band_gain_db(6, 3.0));
        assert_eq!(eq.get_band_gain_db(6), 0.0);
        assert!(!eq.set_band_gain_db(100, 1.0));
        assert_eq!(eq.get_band_gain_db(100), 0.0);
    }
}
