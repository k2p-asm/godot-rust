//! Port of `core/math/audio_frame.h`: a stereo sample pair.
//!
//! The C++ type is a union over `{left, right}`, `{l, r}` and `levels[2]`. The
//! Rust version keeps the same `#[repr(C)]` layout with public fields, so it can
//! cross the FFI as two floats.
//!
//! `undenormalize` is the interesting part: it zeroes samples whose exponent is
//! tiny, which prevents denormal slowdowns in the audio mixer.

pub const AUDIO_PEAK_OFFSET: f32 = 0.0000000001;
/// `linear_to_db(AUDIO_PEAK_OFFSET)`.
pub const AUDIO_MIN_PEAK_DB: f32 = -200.0;

/// Sets denormal floats to zero (Tim Blechmann's version).
#[inline]
pub fn undenormalize(f: f32) -> f32 {
    let bits = f.to_bits();
    if (bits & 0x7f80_0000) < 0x0800_0000 {
        0.0
    } else {
        f
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct AudioFrame {
    pub left: f32,
    pub right: f32,
}

impl AudioFrame {
    #[inline]
    pub const fn new(left: f32, right: f32) -> Self {
        Self { left, right }
    }

    #[inline]
    pub const fn splat(v: f32) -> Self {
        Self { left: v, right: v }
    }

    #[inline]
    pub fn get(&self, idx: usize) -> f32 {
        match idx {
            0 => self.left,
            _ => self.right,
        }
    }

    #[inline]
    pub fn set(&mut self, idx: usize, v: f32) {
        match idx {
            0 => self.left = v,
            _ => self.right = v,
        }
    }

    /// `AudioFrame::undenormalize`.
    #[inline]
    pub fn undenormalize(&mut self) {
        self.left = undenormalize(self.left);
        self.right = undenormalize(self.right);
    }

    /// `AudioFrame::lerp`: `a + t * (b - a)` per channel.
    #[inline]
    pub fn lerp(&self, b: &AudioFrame, t: f32) -> Self {
        Self {
            left: self.left + t * (b.left - self.left),
            right: self.right + t * (b.right - self.right),
        }
    }

    /// Peak magnitude, as used by `AudioFrame` meters.
    #[inline]
    pub fn peak(&self) -> f32 {
        self.left.abs().max(self.right.abs())
    }
}

macro_rules! frame_binop {
    ($trait:ident, $method:ident, $assign:ident, $assign_method:ident, $op:tt) => {
        impl std::ops::$trait for AudioFrame {
            type Output = AudioFrame;
            #[inline]
            fn $method(self, o: AudioFrame) -> AudioFrame {
                AudioFrame::new(self.left $op o.left, self.right $op o.right)
            }
        }
        impl std::ops::$trait<f32> for AudioFrame {
            type Output = AudioFrame;
            #[inline]
            fn $method(self, s: f32) -> AudioFrame {
                AudioFrame::new(self.left $op s, self.right $op s)
            }
        }
        impl std::ops::$assign for AudioFrame {
            #[inline]
            fn $assign_method(&mut self, o: AudioFrame) {
                self.left = self.left $op o.left;
                self.right = self.right $op o.right;
            }
        }
        impl std::ops::$assign<f32> for AudioFrame {
            #[inline]
            fn $assign_method(&mut self, s: f32) {
                self.left = self.left $op s;
                self.right = self.right $op s;
            }
        }
    };
}

frame_binop!(Add, add, AddAssign, add_assign, +);
frame_binop!(Sub, sub, SubAssign, sub_assign, -);
frame_binop!(Mul, mul, MulAssign, mul_assign, *);
frame_binop!(Div, div, DivAssign, div_assign, /);

impl std::ops::Neg for AudioFrame {
    type Output = AudioFrame;
    #[inline]
    fn neg(self) -> AudioFrame {
        AudioFrame::new(-self.left, -self.right)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arithmetic_per_channel() {
        let a = AudioFrame::new(0.5, -0.25);
        let b = AudioFrame::new(0.25, 0.75);
        assert_eq!(a + b, AudioFrame::new(0.75, 0.5));
        assert_eq!(a - b, AudioFrame::new(0.25, -1.0));
        assert_eq!(a * b, AudioFrame::new(0.125, -0.1875));
        assert_eq!(a * 2.0, AudioFrame::new(1.0, -0.5));
        assert_eq!(-a, AudioFrame::new(-0.5, 0.25));
    }

    #[test]
    fn compound_assignment() {
        let mut a = AudioFrame::new(1.0, 1.0);
        a += AudioFrame::new(0.5, -0.5);
        assert_eq!(a, AudioFrame::new(1.5, 0.5));
        a *= 2.0;
        assert_eq!(a, AudioFrame::new(3.0, 1.0));
    }

    #[test]
    fn lerp_endpoints_and_middle() {
        let a = AudioFrame::new(0.0, 1.0);
        let b = AudioFrame::new(2.0, -1.0);
        assert_eq!(a.lerp(&b, 0.0), a);
        assert_eq!(a.lerp(&b, 1.0), b);
        assert_eq!(a.lerp(&b, 0.5), AudioFrame::new(1.0, 0.0));
    }

    #[test]
    fn denormals_become_zero() {
        // A denormal (exponent 0, tiny mantissa) is flushed.
        let tiny = f32::from_bits(0x0000_0001);
        assert_eq!(undenormalize(tiny), 0.0);
        assert_eq!(undenormalize(-tiny), 0.0);

        // The threshold is `exponent field < 0x08000000`, i.e. every value
        // whose exponent is below 16 -- that includes small *normal* floats,
        // not only denormals. This is the engine's actual behaviour.
        let small_normal = f32::from_bits(0x0080_0000); // exponent 1
        assert_eq!(undenormalize(small_normal), 0.0);
        let above_threshold = f32::from_bits(0x0800_0000); // exponent 16
        assert_eq!(undenormalize(above_threshold), above_threshold);

        // Ordinary values are untouched.
        assert_eq!(undenormalize(0.5), 0.5);
        assert_eq!(undenormalize(-1.25), -1.25);

        let mut frame = AudioFrame::new(tiny, 0.5);
        frame.undenormalize();
        assert_eq!(frame, AudioFrame::new(0.0, 0.5));
    }

    #[test]
    fn indexing_and_peak() {
        let mut f = AudioFrame::new(-0.75, 0.25);
        assert_eq!(f.get(0), -0.75);
        assert_eq!(f.get(1), 0.25);
        f.set(1, 1.5);
        assert_eq!(f.right, 1.5);
        assert_eq!(f.peak(), 1.5);
    }
}
