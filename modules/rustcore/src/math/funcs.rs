//! Port of `core/math/math_funcs.h` + `core/math/math_funcs.cpp`.
//!
//! The C++ code uses overloads for `float`/`double`. Rust has no overloading, so
//! the same API is expressed through the [`Real`] trait, implemented for `f32`
//! and `f64`. Primitives are computed in `f64` and narrowed on return, which is
//! correctly rounded for `f32` and keeps results portable across platforms.

pub const CMP_EPSILON: f64 = 0.00001;
pub const CMP_EPSILON2: f64 = CMP_EPSILON * CMP_EPSILON;
pub const CMP_NORMALIZE_TOLERANCE: f64 = 0.000001;
pub const CMP_POINT_IN_PLANE_EPSILON: f64 = 0.00001;

pub const PI: f64 = 3.1415926535897932384626433833;
pub const TAU: f64 = 6.2831853071795864769252867666;
pub const SQRT2: f64 = 1.4142135623730950488016887242;
pub const SQRT3: f64 = 1.7320508075688772935274463415059;
pub const SQRT12: f64 = 0.7071067811865475244008443621048490;
pub const SQRT13: f64 = 0.57735026918962576450914878050196;
pub const LN2: f64 = 0.6931471805599453094172321215;
pub const E: f64 = 2.7182818284590452353602874714;

pub trait Real:
    Copy
    + PartialOrd
    + PartialEq
    + std::ops::Add<Output = Self>
    + std::ops::Sub<Output = Self>
    + std::ops::Mul<Output = Self>
    + std::ops::Div<Output = Self>
    + std::ops::Neg<Output = Self>
{
    const CMP_EPSILON: Self;
    const PI: Self;
    const TAU: Self;
    const INF: Self;
    const NAN: Self;

    fn zero() -> Self;
    fn one() -> Self;
    fn from_f64(v: f64) -> Self;
    fn to_f64(self) -> f64;

    fn abs(self) -> Self {
        Self::from_f64(self.to_f64().abs())
    }
    fn floor(self) -> Self {
        Self::from_f64(self.to_f64().floor())
    }
    fn ceil(self) -> Self {
        Self::from_f64(self.to_f64().ceil())
    }
    fn round(self) -> Self {
        Self::from_f64(self.to_f64().round())
    }
    fn trunc(self) -> Self {
        Self::from_f64(self.to_f64().trunc())
    }
    fn sqrt(self) -> Self {
        Self::from_f64(self.to_f64().sqrt())
    }
    fn sin(self) -> Self {
        Self::from_f64(self.to_f64().sin())
    }
    fn cos(self) -> Self {
        Self::from_f64(self.to_f64().cos())
    }
    fn tan(self) -> Self {
        Self::from_f64(self.to_f64().tan())
    }
    fn asin(self) -> Self {
        Self::from_f64(self.to_f64().asin())
    }
    fn acos(self) -> Self {
        Self::from_f64(self.to_f64().acos())
    }
    fn atan(self) -> Self {
        Self::from_f64(self.to_f64().atan())
    }
    fn atan2(self, other: Self) -> Self {
        Self::from_f64(self.to_f64().atan2(other.to_f64()))
    }
    fn sinh(self) -> Self {
        Self::from_f64(self.to_f64().sinh())
    }
    fn cosh(self) -> Self {
        Self::from_f64(self.to_f64().cosh())
    }
    fn tanh(self) -> Self {
        Self::from_f64(self.to_f64().tanh())
    }
    fn asinh(self) -> Self {
        Self::from_f64(self.to_f64().asinh())
    }
    fn acosh(self) -> Self {
        Self::from_f64(self.to_f64().acosh())
    }
    fn atanh(self) -> Self {
        Self::from_f64(self.to_f64().atanh())
    }
    fn exp(self) -> Self {
        Self::from_f64(self.to_f64().exp())
    }
    fn ln(self) -> Self {
        Self::from_f64(self.to_f64().ln())
    }
    fn log2(self) -> Self {
        Self::from_f64(self.to_f64().log2())
    }
    fn log10(self) -> Self {
        Self::from_f64(self.to_f64().log10())
    }
    fn powf(self, e: Self) -> Self {
        Self::from_f64(self.to_f64().powf(e.to_f64()))
    }
    fn fmod(self, y: Self) -> Self {
        Self::from_f64(self.to_f64() % y.to_f64())
    }
    fn fract(self) -> Self {
        Self::from_f64(self.to_f64().fract())
    }
    fn is_nan(self) -> bool {
        self.to_f64().is_nan()
    }
    fn is_inf(self) -> bool {
        self.to_f64().is_infinite()
    }
    fn is_finite(self) -> bool {
        self.to_f64().is_finite()
    }
    fn clamp(self, lo: Self, hi: Self) -> Self {
        if self < lo {
            lo
        } else if self > hi {
            hi
        } else {
            self
        }
    }
    fn sign(self) -> Self {
        // C++ `SIGN(m_x)` is `((m_x < 0) ? -1 : 1)`, i.e. zero counts as positive.
        if self < Self::zero() {
            Self::from_f64(-1.0)
        } else {
            Self::one()
        }
    }
}

impl Real for f32 {
    const CMP_EPSILON: Self = CMP_EPSILON as f32;
    const PI: Self = PI as f32;
    const TAU: Self = TAU as f32;
    const INF: Self = f32::INFINITY;
    const NAN: Self = f32::NAN;

    fn zero() -> Self {
        0.0
    }
    fn one() -> Self {
        1.0
    }
    fn from_f64(v: f64) -> Self {
        v as f32
    }
    fn to_f64(self) -> f64 {
        self as f64
    }
}

impl Real for f64 {
    const CMP_EPSILON: Self = CMP_EPSILON;
    const PI: Self = PI;
    const TAU: Self = TAU;
    const INF: Self = f64::INFINITY;
    const NAN: Self = f64::NAN;

    fn zero() -> Self {
        0.0
    }
    fn one() -> Self {
        1.0
    }
    fn from_f64(v: f64) -> Self {
        v
    }
    fn to_f64(self) -> f64 {
        self
    }
}

// -- trigonometric helpers -------------------------------------------------

pub fn sinc<T: Real>(x: T) -> T {
    if x == T::zero() {
        T::one()
    } else {
        x.sin() / x
    }
}

pub fn tan<T: Real>(x: T) -> T {
    x.tan()
}

pub fn atan2<T: Real>(y: T, x: T) -> T {
    y.atan2(x)
}

pub fn sincn<T: Real>(x: T) -> T {
    sinc(T::PI * x)
}

/// Always clamps, so always safe to use.
pub fn asin<T: Real>(x: T) -> T {
    let half_pi = T::PI / T::from_f64(2.0);
    if x < -T::one() {
        -half_pi
    } else if x > T::one() {
        half_pi
    } else {
        x.asin()
    }
}

/// Always clamps, so always safe to use.
pub fn acos<T: Real>(x: T) -> T {
    if x < -T::one() {
        T::PI
    } else if x > T::one() {
        T::zero()
    } else {
        x.acos()
    }
}

/// Always clamps, so always safe to use.
pub fn acosh<T: Real>(x: T) -> T {
    if x < T::one() {
        T::zero()
    } else {
        x.acosh()
    }
}

/// Always clamps, so always safe to use.
pub fn atanh<T: Real>(x: T) -> T {
    if x <= -T::one() {
        -T::INF
    } else if x >= T::one() {
        T::INF
    } else {
        x.atanh()
    }
}

// -- integer helpers -------------------------------------------------------

/// `SIGN(m_x)` from the C++ macros: `((m_x < 0) ? -1 : 1)`, so `-0.0` and `0.0`
/// both yield `1`.
#[inline]
pub fn sign_f32(v: f32) -> f32 {
    if v < 0.0 {
        -1.0
    } else if v > 0.0 {
        1.0
    } else {
        0.0
    }
}

#[inline]
pub fn sign_f64(v: f64) -> f64 {
    if v < 0.0 {
        -1.0
    } else {
        1.0
    }
}

/// `CLAMP(m_a, m_min, m_max)` without the panic that `f32::clamp` raises when
/// `min > max`.
#[inline]
pub fn clamp_f32(v: f32, min: f32, max: f32) -> f32 {
    if v < min {
        min
    } else if v > max {
        max
    } else {
        v
    }
}

#[inline]
pub fn clamp_f64(v: f64, min: f64, max: f64) -> f64 {
    if v < min {
        min
    } else if v > max {
        max
    } else {
        v
    }
}


/// Assumes `(num + den)` doesn't overflow.
pub fn division_round_up_i32(num: i32, den: i32) -> i32 {
    let offset = if num < 0 && den < 0 { 1 } else { -1 };
    (num + den + offset) / den
}

pub fn division_round_up_u32(num: u32, den: u32) -> u32 {
    (num + den - 1) / den
}

pub fn division_round_up_i64(num: i64, den: i64) -> i64 {
    let offset: i32 = if num < 0 && den < 0 { 1 } else { -1 };
    (num + den + offset as i64) / den
}

pub fn division_round_up_u64(num: u64, den: u64) -> u64 {
    (num + den - 1) / den
}

// -- modular arithmetic ----------------------------------------------------

pub fn fposmod<T: Real>(x: T, y: T) -> T {
    let mut value = x.fmod(y);
    if (value < T::zero() && y > T::zero()) || (value > T::zero() && y < T::zero()) {
        value = value + y;
    }
    // `value += 0.0` in the original normalizes -0.0 into 0.0.
    value + T::zero()
}

pub fn fposmodp<T: Real>(x: T, y: T) -> T {
    let mut value = x.fmod(y);
    if value < T::zero() {
        value = value + y;
    }
    value + T::zero()
}

/// Returns 0 when `y == 0` (the C++ version errors and returns 0 as fallback).
pub fn posmod(x: i64, y: i64) -> i64 {
    if y == 0 {
        return 0;
    }
    let mut value = x % y;
    if (value < 0 && y > 0) || (value > 0 && y < 0) {
        value += y;
    }
    value
}

pub fn deg_to_rad<T: Real>(deg: T) -> T {
    deg * (T::PI / T::from_f64(180.0))
}

pub fn rad_to_deg<T: Real>(rad: T) -> T {
    rad * (T::from_f64(180.0) / T::PI)
}

// -- interpolation ---------------------------------------------------------

pub fn lerp<T: Real>(from: T, to: T, weight: T) -> T {
    from + (to - from) * weight
}

pub fn cubic_interpolate<T: Real>(from: T, to: T, pre: T, post: T, weight: T) -> T {
    let w2 = weight * weight;
    let w3 = w2 * weight;
    T::from_f64(0.5)
        * ((from * T::from_f64(2.0))
            + (-pre + to) * weight
            + (T::from_f64(2.0) * pre - T::from_f64(5.0) * from + T::from_f64(4.0) * to - post) * w2
            + (-pre + T::from_f64(3.0) * from - T::from_f64(3.0) * to + post) * w3)
}

pub fn cubic_interpolate_angle<T: Real>(from: T, to: T, pre: T, post: T, weight: T) -> T {
    let two = T::from_f64(2.0);
    let from_rot = from.fmod(T::TAU);
    let pre_diff = (pre - from_rot).fmod(T::TAU);
    let pre_rot = from_rot + (two * pre_diff).fmod(T::TAU) - pre_diff;
    let to_diff = (to - from_rot).fmod(T::TAU);
    let to_rot = from_rot + (two * to_diff).fmod(T::TAU) - to_diff;
    let post_diff = (post - to_rot).fmod(T::TAU);
    let post_rot = to_rot + (two * post_diff).fmod(T::TAU) - post_diff;
    cubic_interpolate(from_rot, to_rot, pre_rot, post_rot, weight)
}

pub fn cubic_interpolate_in_time<T: Real>(
    from: T,
    to: T,
    pre: T,
    post: T,
    weight: T,
    to_t: T,
    pre_t: T,
    post_t: T,
) -> T {
    // Barry-Goldman method.
    let t = lerp(T::zero(), to_t, weight);
    let a1 = lerp(
        pre,
        from,
        if pre_t == T::zero() {
            T::zero()
        } else {
            (t - pre_t) / -pre_t
        },
    );
    let a2 = lerp(
        from,
        to,
        if to_t == T::zero() {
            T::from_f64(0.5)
        } else {
            t / to_t
        },
    );
    let a3 = lerp(
        to,
        post,
        if post_t - to_t == T::zero() {
            T::one()
        } else {
            (t - to_t) / (post_t - to_t)
        },
    );
    let b1 = lerp(
        a1,
        a2,
        if to_t - pre_t == T::zero() {
            T::zero()
        } else {
            (t - pre_t) / (to_t - pre_t)
        },
    );
    let b2 = lerp(
        a2,
        a3,
        if post_t == T::zero() {
            T::one()
        } else {
            t / post_t
        },
    );
    lerp(
        b1,
        b2,
        if to_t == T::zero() {
            T::from_f64(0.5)
        } else {
            t / to_t
        },
    )
}

pub fn cubic_interpolate_angle_in_time<T: Real>(
    from: T,
    to: T,
    pre: T,
    post: T,
    weight: T,
    to_t: T,
    pre_t: T,
    post_t: T,
) -> T {
    let two = T::from_f64(2.0);
    let from_rot = from.fmod(T::TAU);
    let pre_diff = (pre - from_rot).fmod(T::TAU);
    let pre_rot = from_rot + (two * pre_diff).fmod(T::TAU) - pre_diff;
    let to_diff = (to - from_rot).fmod(T::TAU);
    let to_rot = from_rot + (two * to_diff).fmod(T::TAU) - to_diff;
    let post_diff = (post - to_rot).fmod(T::TAU);
    let post_rot = to_rot + (two * post_diff).fmod(T::TAU) - post_diff;
    cubic_interpolate_in_time(from_rot, to_rot, pre_rot, post_rot, weight, to_t, pre_t, post_t)
}

pub fn bezier_interpolate<T: Real>(start: T, control_1: T, control_2: T, end: T, t: T) -> T {
    let three = T::from_f64(3.0);
    let omt = T::one() - t;
    let omt2 = omt * omt;
    let omt3 = omt2 * omt;
    let t2 = t * t;
    let t3 = t2 * t;
    start * omt3 + control_1 * omt2 * t * three + control_2 * omt * t2 * three + end * t3
}

pub fn bezier_derivative<T: Real>(start: T, control_1: T, control_2: T, end: T, t: T) -> T {
    let three = T::from_f64(3.0);
    let six = T::from_f64(6.0);
    let omt = T::one() - t;
    let omt2 = omt * omt;
    let t2 = t * t;
    (control_1 - start) * three * omt2
        + (control_2 - control_1) * six * omt * t
        + (end - control_2) * three * t2
}

pub fn angle_difference<T: Real>(from: T, to: T) -> T {
    let two = T::from_f64(2.0);
    let difference = (to - from).fmod(T::TAU);
    (two * difference).fmod(T::TAU) - difference
}

pub fn lerp_angle<T: Real>(from: T, to: T, weight: T) -> T {
    from + angle_difference(from, to) * weight
}

pub fn inverse_lerp<T: Real>(from: T, to: T, value: T) -> T {
    (value - from) / (to - from)
}

pub fn remap<T: Real>(value: T, istart: T, istop: T, ostart: T, ostop: T) -> T {
    lerp(ostart, ostop, inverse_lerp(istart, istop, value))
}

// -- comparisons -----------------------------------------------------------

pub fn is_equal_approx_tol<T: Real>(left: T, right: T, tolerance: T) -> bool {
    if left == right {
        return true;
    }
    (left - right).abs() < tolerance
}

pub fn is_equal_approx<T: Real>(left: T, right: T) -> bool {
    if left == right {
        return true;
    }
    let mut tolerance = T::CMP_EPSILON * left.abs();
    if tolerance < T::CMP_EPSILON {
        tolerance = T::CMP_EPSILON;
    }
    (left - right).abs() < tolerance
}

pub fn is_zero_approx<T: Real>(value: T) -> bool {
    value.abs() < T::CMP_EPSILON
}

pub fn is_same<T: Real>(left: T, right: T) -> bool {
    (left == right) || (left.is_nan() && right.is_nan())
}

pub fn smoothstep<T: Real>(from: T, to: T, s: T) -> T {
    let three = T::from_f64(3.0);
    let two = T::from_f64(2.0);
    if is_equal_approx(from, to) {
        return if from <= to {
            if s <= from {
                T::zero()
            } else {
                T::one()
            }
        } else if s <= to {
            T::one()
        } else {
            T::zero()
        };
    }
    let s = ((s - from) / (to - from)).clamp(T::zero(), T::one());
    s * s * (three - two * s)
}

pub fn move_toward<T: Real>(from: T, to: T, delta: T) -> T {
    if (to - from).abs() <= delta {
        to
    } else {
        from + (to - from).sign() * delta
    }
}

pub fn rotate_toward<T: Real>(from: T, to: T, delta: T) -> T {
    let difference = angle_difference(from, to);
    let abs_difference = difference.abs();
    // When `delta < 0` move no further than to PI radians away from `to`.
    from + delta
        .clamp(abs_difference - T::PI, abs_difference)
        * if difference >= T::zero() {
            T::one()
        } else {
            -T::one()
        }
}

// -- audio helpers ---------------------------------------------------------

pub fn linear_to_db<T: Real>(linear: T) -> T {
    linear.ln() * T::from_f64(8.6858896380650365530225783783321)
}

pub fn db_to_linear<T: Real>(db: T) -> T {
    (db * T::from_f64(0.11512925464970228420089957273422)).exp()
}

// -- wrapping --------------------------------------------------------------

pub fn wrapf<T: Real>(value: T, min: T, max: T) -> T {
    let range = max - min;
    if is_zero_approx(range) {
        return min;
    }
    let result = value - (range * ((value - min) / range).floor());
    if is_equal_approx(result, max) {
        return min;
    }
    result
}

pub fn wrapi(value: i64, min: i64, max: i64) -> i64 {
    let range = max - min;
    if range == 0 {
        min
    } else {
        min + (((value - min) % range) + range) % range
    }
}

pub fn pingpong<T: Real>(value: T, length: T) -> T {
    let two = T::from_f64(2.0);
    if length != T::zero() {
        ((value - length) / (length * two)).fract() * length * two - length
    } else {
        T::zero()
    }
    .abs()
}

// -- double-only helpers (editor, not performance critical) ----------------

pub fn ease(x: f64, c: f64) -> f64 {
    let x = x.clamp(0.0, 1.0);
    if c > 0.0 {
        if c < 1.0 {
            1.0 - (1.0 - x).powf(1.0 / c)
        } else {
            x.powf(c)
        }
    } else if c < 0.0 {
        if x < 0.5 {
            (x * 2.0).powf(-c) * 0.5
        } else {
            (1.0 - (1.0 - (x - 0.5) * 2.0).powf(-c)) * 0.5 + 0.5
        }
    } else {
        0.0
    }
}

pub fn step_decimals(step: f64) -> i32 {
    const SD: [f64; 10] = [
        0.9999,
        0.09999,
        0.009999,
        0.0009999,
        0.00009999,
        0.000009999,
        0.0000009999,
        0.00000009999,
        0.000000009999,
        0.0000000009999,
    ];
    let abs = step.abs();
    let decs = abs - (abs as i32) as f64; // strip integer part
    for (i, &sd) in SD.iter().enumerate() {
        if decs >= sd {
            return i as i32;
        }
    }
    0
}

pub fn range_step_decimals(step: f64) -> i32 {
    if step < 0.0000000000001 {
        return 16; // Max value hardcoded in String::num.
    }
    step_decimals(step)
}

pub fn snapped(value: f64, step: f64) -> f64 {
    if step != 0.0 {
        (value / step + 0.5).floor() * step
    } else {
        value
    }
}

pub fn snap_scalar(offset: f32, step: f32, target: f32) -> f32 {
    if step != 0.0 {
        snapped((target - offset) as f64, step as f64) as f32 + offset
    } else {
        target
    }
}

pub fn snap_scalar_separation(offset: f32, step: f32, target: f32, separation: f32) -> f32 {
    if step != 0.0 {
        let a = snapped((target - offset) as f64, (step + separation) as f64) as f32 + offset;
        let mut b = a;
        if target >= 0.0 {
            b -= separation;
        } else {
            b += step;
        }
        if (target - a).abs() < (target - b).abs() {
            a
        } else {
            b
        }
    } else {
        target
    }
}

pub fn larger_prime(val: u32) -> u32 {
    const PRIMES: [u32; 29] = [
        5, 13, 23, 47, 97, 193, 389, 769, 1543, 3079, 6151, 12289, 24593, 49157, 98317, 196613,
        393241, 786433, 1572869, 3145739, 6291469, 12582917, 25165843, 50331653, 100663319,
        201326611, 402653189, 805306457, 1610612741,
    ];
    for &p in PRIMES.iter() {
        if p > val {
            return p;
        }
    }
    0
}

// -- float packing ---------------------------------------------------------

pub fn fast_ftoi(value: f32) -> i32 {
    // std::rint respects the current rounding mode; round-half-even is the default.
    value.round_ties_even() as i32
}

pub fn halfbits_to_floatbits(half: u16) -> u32 {
    let h_exp = half & 0x7c00u16;
    let f_sgn = ((half as u32) & 0x8000u32) << 16;
    match h_exp {
        0x0000 => {
            // 0 or subnormal
            let mut h_sig = half & 0x03ffu16;
            if h_sig == 0 {
                return f_sgn; // signed zero
            }
            h_sig <<= 1;
            // The C++ code increments h_exp inside the loop, so the number of
            // normalizing shifts is part of the result. Do not simplify.
            let mut shifts = 0u32;
            while (h_sig & 0x0400u16) == 0 {
                h_sig <<= 1;
                shifts += 1;
            }
            let f_exp = ((127 - 15 - shifts) << 23) as u32;
            let f_sig = ((h_sig & 0x03ffu16) as u32) << 13;
            f_sgn + f_exp + f_sig
        }
        0x7c00 => {
            // inf or NaN
            f_sgn + 0x7f800000u32 + (((half & 0x03ffu16) as u32) << 13)
        }
        _ => {
            // normalized
            f_sgn + ((((half & 0x7fffu16) as u32) + 0x1c000u32) << 13)
        }
    }
}

pub fn half_to_float(half: u16) -> f32 {
    f32::from_bits(halfbits_to_floatbits(half))
}

pub fn make_half_float(value: f32) -> u16 {
    let x = value.to_bits();
    let sign = (x >> 31) & 1;
    let mut mantissa = x & ((1 << 23) - 1);
    let exponent = x & (0xFF << 23);
    if exponent >= 0x47800000 {
        // inf / NaN
        if mantissa != 0 && exponent == (0xFF << 23) {
            mantissa = (1 << 23) - 1;
        } else {
            mantissa = 0;
        }
        ((sign as u16) << 15) | ((0x1Fu16) << 10) | ((mantissa >> 13) as u16)
    } else if exponent <= 0x38000000 {
        0 // denormals do not work for 3D, convert to zero
    } else {
        ((sign as u16) << 15)
            | ((((exponent - 0x38000000) >> 13) & 0xFFFF) as u16)
            | ((mantissa >> 13) as u16)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constants_match_cpp() {
        assert_eq!(CMP_EPSILON, 0.00001);
        assert!((PI - std::f64::consts::PI).abs() < 1e-15);
        assert!((TAU - 2.0 * std::f64::consts::PI).abs() < 1e-15);
    }

    #[test]
    fn mod_helpers() {
        assert_eq!(posmod(-7, 3), 2);
        assert_eq!(posmod(7, -3), -2);
        assert_eq!(posmod(5, 0), 0);
        assert_eq!(wrapi(-1, 0, 4), 3);
        assert_eq!(wrapi(5, 0, 4), 1);
        assert_eq!(wrapf(-0.5f32, 0.0, 1.0), 0.5);
        assert!((fposmod(-1.5f64, 1.0) - 0.5).abs() < 1e-12);
    }

    #[test]
    fn approx_helpers() {
        assert!(is_equal_approx(1.0f32, 1.0));
        assert!(is_equal_approx(1.0f32, 1.000001));
        assert!(!is_equal_approx(1.0f32, 1.001));
        assert!(is_zero_approx(0.000001f32));
        assert!(!is_zero_approx(0.001f32));
        assert!(is_same(f32::NAN, f32::NAN));
        assert!(is_same(0.0f32, -0.0f32)); // 0.0 == -0.0
    }

    #[test]
    fn half_float_roundtrip() {
        for v in [0.0f32, 1.0, -1.0, 0.5, 2.0, 100.0, -0.25, 65504.0] {
            let h = make_half_float(v);
            let back = half_to_float(h);
            assert!((back - v).abs() <= v.abs() * 1e-3 + 1e-6, "{v} -> {back}");
        }
        assert_eq!(make_half_float(1e6), 0x7c00); // -> inf
        assert_eq!(half_to_float(0x7c00), f32::INFINITY);
    }

    #[test]
    fn ease_and_snap() {
        assert!((ease(0.5, 1.0) - 0.5).abs() < 1e-12);
        assert_eq!(snapped(1.26, 0.5), 1.5);
        assert_eq!(step_decimals(0.01), 2);
        assert_eq!(step_decimals(0.001), 3);
        assert_eq!(larger_prime(4), 5);
        assert_eq!(larger_prime(200), 389);
    }
}
