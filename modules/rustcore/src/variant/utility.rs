//! `core/variant/variant_utility.*` (functions subset).
//!
//! The engine exposes the `@GlobalScope` functions. This port covers the numeric
//! half — enough for gameplay and animation code:
//!
//! - scalar math (`sin`…`atanh`, `sqrt`, `pow`, `log`, `exp`, `fmod`, `fposmod`,
//!   `deg_to_rad`, `rad_to_deg`, `linear_to_db`, `db_to_linear`, the `is_*`
//!   predicates, `ease`, `step_decimals`);
//! - interpolation (`lerp`/`lerpf`, `lerp_angle`, `angle_difference`,
//!   `inverse_lerp`, `remap`, `smoothstep`, `move_toward`, `rotate_toward`,
//!   `pingpong`, `wrapf`, `cubic_interpolate`(+`_angle`+`_in_time`),
//!   `bezier_interpolate`/`bezier_derivative`);
//! - `Variant`-dispatching forms with the engine's `CallError` contract for
//!   `floor`, `ceil`, `round`, `abs`, `sign`, `snapped`, `lerp`, `clamp`,
//!   `wrap`, `wrapi`, `min` and `max` (int/float/Vector2/Vector3/Vector4/Color
//!   where the engine supports them).
//!
//! Not ported (documented as pending): array/dictionary/string helpers,
//! `type_convert`, `str`/`var_to_str`, `print*`, object/instance helpers and the
//! `RandomNumberGenerator` bound methods.

use crate::math::color::Color;
use crate::math::funcs;
use crate::math::vector2::Vector2;
use crate::math::vector2i::Vector2i;
use crate::math::vector3::Vector3;
use crate::math::vector3i::Vector3i;
use crate::math::vector4::Vector4;
use crate::math::vector4i::Vector4i;
use crate::variant::{Variant, VariantType};

/// `Callable::CallError` for the utility functions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CallError {
    Ok,
    InvalidArgument { argument: i32, expected: VariantType },
}

pub type UtilityResult = Result<Variant, CallError>;

fn invalid(argument: i32, expected: VariantType) -> CallError {
    CallError::InvalidArgument { argument, expected }
}

// -- scalar math -------------------------------------------------------------

pub fn sin(arg: f64) -> f64 {
    arg.sin()
}
pub fn cos(arg: f64) -> f64 {
    arg.cos()
}
pub fn tan(arg: f64) -> f64 {
    arg.tan()
}
pub fn sinh(arg: f64) -> f64 {
    arg.sinh()
}
pub fn cosh(arg: f64) -> f64 {
    arg.cosh()
}
pub fn tanh(arg: f64) -> f64 {
    arg.tanh()
}
pub fn asin(arg: f64) -> f64 {
    arg.asin()
}
pub fn acos(arg: f64) -> f64 {
    arg.acos()
}
pub fn atan(arg: f64) -> f64 {
    arg.atan()
}
pub fn asinh(arg: f64) -> f64 {
    arg.asinh()
}
pub fn acosh(arg: f64) -> f64 {
    arg.acosh()
}
pub fn atanh(arg: f64) -> f64 {
    arg.atanh()
}
pub fn sqrt(x: f64) -> f64 {
    x.sqrt()
}
pub fn pow(x: f64, y: f64) -> f64 {
    x.powf(y)
}
pub fn log(x: f64) -> f64 {
    x.ln()
}
pub fn exp(x: f64) -> f64 {
    x.exp()
}
pub fn fmod(b: f64, r: f64) -> f64 {
    b % r
}
/// `Math::fposmod`: the result is always in `[0, r)` (or `(-r, 0]` for a
/// negative divisor).
pub fn fposmod(b: f64, r: f64) -> f64 {
    let value = b % r;
    (value + r) % r
}
pub fn deg_to_rad(angle_deg: f64) -> f64 {
    angle_deg * (std::f64::consts::PI / 180.0)
}
pub fn rad_to_deg(angle_rad: f64) -> f64 {
    angle_rad * (180.0 / std::f64::consts::PI)
}
pub fn linear_to_db(linear: f64) -> f64 {
    linear.ln() * 8.685_889_638_065_036
}
pub fn db_to_linear(db: f64) -> f64 {
    (db * 0.115_129_254_649_702_28).exp()
}
pub fn is_nan(x: f64) -> bool {
    x.is_nan()
}
pub fn is_inf(x: f64) -> bool {
    x.is_infinite()
}
pub fn is_finite(x: f64) -> bool {
    x.is_finite()
}
pub fn is_equal_approx(x: f64, y: f64) -> bool {
    funcs::is_equal_approx(x, y)
}
pub fn is_zero_approx(x: f64) -> bool {
    funcs::is_zero_approx(x)
}
pub fn is_same(x: f64, y: f64) -> bool {
    funcs::is_same(x, y)
}
pub fn ease(x: f64, curve: f64) -> f64 {
    funcs::ease(x, curve)
}
pub fn step_decimals(step: f64) -> i32 {
    funcs::step_decimals(step)
}

// -- interpolation -----------------------------------------------------------

pub fn lerpf(from: f64, to: f64, weight: f64) -> f64 {
    from + (to - from) * weight
}
pub fn inverse_lerp(from: f64, to: f64, value: f64) -> f64 {
    (value - from) / (to - from)
}
pub fn remap(value: f64, istart: f64, istop: f64, ostart: f64, ostop: f64) -> f64 {
    lerpf(ostart, ostop, inverse_lerp(istart, istop, value))
}
pub fn smoothstep(from: f64, to: f64, value: f64) -> f64 {
    if funcs::is_equal_approx(from, to) {
        return if from <= to {
            if value <= from {
                0.0
            } else {
                1.0
            }
        } else if value <= to {
            1.0
        } else {
            0.0
        };
    }
    let s = ((value - from) / (to - from)).clamp(0.0, 1.0);
    s * s * (3.0 - 2.0 * s)
}
pub fn move_toward(from: f64, to: f64, delta: f64) -> f64 {
    if (to - from).abs() <= delta {
        to
    } else {
        from + (to - from).signum() * delta
    }
}
pub fn angle_difference(from: f64, to: f64) -> f64 {
    let mut difference = fposmod(to - from, std::f64::consts::TAU);
    if difference > std::f64::consts::PI {
        difference -= std::f64::consts::TAU;
    }
    difference
}
pub fn lerp_angle(from: f64, to: f64, weight: f64) -> f64 {
    from + angle_difference(from, to) * weight
}
pub fn rotate_toward(from: f64, to: f64, delta: f64) -> f64 {
    let difference = angle_difference(from, to);
    let abs_difference = difference.abs();
    from + delta
        .clamp(abs_difference - std::f64::consts::PI, abs_difference)
        * if difference >= 0.0 { 1.0 } else { -1.0 }
}
pub fn pingpong(value: f64, length: f64) -> f64 {
    if length == 0.0 {
        return 0.0;
    }
    length - (fposmod(value, length * 2.0) - length).abs()
}
pub fn wrapf(value: f64, min: f64, max: f64) -> f64 {
    let range = max - min;
    if range == 0.0 {
        return min;
    }
    value - range * ((value - min) / range).floor()
}
pub fn cubic_interpolate(from: f64, to: f64, pre: f64, post: f64, weight: f64) -> f64 {
    0.5 * ((from * 2.0)
        + (-pre + to) * weight
        + (2.0 * pre - 5.0 * from + 4.0 * to - post) * (weight * weight)
        + (-pre + 3.0 * from - 3.0 * to + post) * (weight * weight * weight))
}
pub fn cubic_interpolate_angle(from: f64, to: f64, pre: f64, post: f64, weight: f64) -> f64 {
    let from_rot = from % std::f64::consts::TAU;
    let pre_diff = (pre - from_rot) % std::f64::consts::TAU;
    let pre_rot = from_rot + (2.0 * pre_diff) % std::f64::consts::TAU - pre_diff;
    let to_diff = (to - from_rot) % std::f64::consts::TAU;
    let to_rot = from_rot + (2.0 * to_diff) % std::f64::consts::TAU - to_diff;
    let post_diff = (post - to_rot) % std::f64::consts::TAU;
    let post_rot = to_rot + (2.0 * post_diff) % std::f64::consts::TAU - post_diff;
    cubic_interpolate(from_rot, to_rot, pre_rot, post_rot, weight)
}
#[allow(clippy::too_many_arguments)]
pub fn cubic_interpolate_in_time(
    from: f64,
    to: f64,
    pre: f64,
    post: f64,
    weight: f64,
    to_t: f64,
    pre_t: f64,
    post_t: f64,
) -> f64 {
    let t = lerpf(0.0, to_t, weight);
    let a1 = lerpf(
        pre,
        from,
        if pre_t == 0.0 {
            0.0
        } else {
            (t - pre_t) / -pre_t
        },
    );
    let a2 = lerpf(from, to, if to_t == 0.0 { 0.5 } else { t / to_t });
    let a3 = lerpf(
        to,
        post,
        if post_t - to_t == 0.0 {
            1.0
        } else {
            (t - to_t) / (post_t - to_t)
        },
    );
    let b1 = lerpf(
        a1,
        a2,
        if to_t - pre_t == 0.0 {
            0.0
        } else {
            (t - pre_t) / (to_t - pre_t)
        },
    );
    let b2 = lerpf(a2, a3, if post_t == 0.0 { 1.0 } else { t / post_t });
    lerpf(b1, b2, if to_t == 0.0 { 0.5 } else { t / to_t })
}
pub fn bezier_interpolate(start: f64, control_1: f64, control_2: f64, end: f64, t: f64) -> f64 {
    let omt = 1.0 - t;
    let omt2 = omt * omt;
    let omt3 = omt2 * omt;
    let t2 = t * t;
    let t3 = t2 * t;
    start * omt3 + control_1 * omt2 * t * 3.0 + control_2 * omt * t2 * 3.0 + end * t3
}
pub fn bezier_derivative(start: f64, control_1: f64, control_2: f64, end: f64, t: f64) -> f64 {
    let omt = 1.0 - t;
    let omt2 = omt * omt;
    let t2 = t * t;
    3.0 * omt2 * (control_1 - start)
        + 6.0 * omt * t * (control_2 - control_1)
        + 3.0 * t2 * (end - control_2)
}

// -- Variant dispatch --------------------------------------------------------

fn as_f64(value: &Variant) -> Option<f64> {
    match value {
        Variant::Int(number) => Some(*number as f64),
        Variant::Float(number) => Some(*number),
        _ => None,
    }
}

fn vector2_parts(value: &Variant) -> Option<(f64, f64)> {
    match value {
        Variant::Vector2(v) => Some((v.x as f64, v.y as f64)),
        Variant::Vector2i(v) => Some((v.x as f64, v.y as f64)),
        _ => None,
    }
}

/// `VariantUtilityFunctions::floor`.
pub fn floor(x: &Variant) -> UtilityResult {
    match x {
        Variant::Int(value) => Ok(Variant::Int(*value)),
        Variant::Float(value) => Ok(Variant::Float(value.floor())),
        Variant::Vector2(v) => Ok(Variant::Vector2(v.floor())),
        Variant::Vector2i(v) => Ok(Variant::Vector2i(*v)),
        Variant::Vector3(v) => Ok(Variant::Vector3(v.floor())),
        Variant::Vector3i(v) => Ok(Variant::Vector3i(*v)),
        Variant::Vector4(v) => Ok(Variant::Vector4(v.floor())),
        Variant::Vector4i(v) => Ok(Variant::Vector4i(*v)),
        _ => Err(invalid(0, VariantType::Float)),
    }
}

/// `VariantUtilityFunctions::ceil`.
pub fn ceil(x: &Variant) -> UtilityResult {
    match x {
        Variant::Int(value) => Ok(Variant::Int(*value)),
        Variant::Float(value) => Ok(Variant::Float(value.ceil())),
        Variant::Vector2(v) => Ok(Variant::Vector2(v.ceil())),
        Variant::Vector2i(v) => Ok(Variant::Vector2i(*v)),
        Variant::Vector3(v) => Ok(Variant::Vector3(v.ceil())),
        Variant::Vector3i(v) => Ok(Variant::Vector3i(*v)),
        Variant::Vector4(v) => Ok(Variant::Vector4(v.ceil())),
        Variant::Vector4i(v) => Ok(Variant::Vector4i(*v)),
        _ => Err(invalid(0, VariantType::Float)),
    }
}

/// `VariantUtilityFunctions::round`.
pub fn round(x: &Variant) -> UtilityResult {
    match x {
        Variant::Int(value) => Ok(Variant::Int(*value)),
        Variant::Float(value) => Ok(Variant::Float(value.round())),
        Variant::Vector2(v) => Ok(Variant::Vector2(v.round())),
        Variant::Vector2i(v) => Ok(Variant::Vector2i(*v)),
        Variant::Vector3(v) => Ok(Variant::Vector3(v.round())),
        Variant::Vector3i(v) => Ok(Variant::Vector3i(*v)),
        Variant::Vector4(v) => Ok(Variant::Vector4(v.round())),
        Variant::Vector4i(v) => Ok(Variant::Vector4i(*v)),
        _ => Err(invalid(0, VariantType::Float)),
    }
}

/// `VariantUtilityFunctions::abs`.
pub fn abs(x: &Variant) -> UtilityResult {
    match x {
        Variant::Int(value) => Ok(Variant::Int(value.wrapping_abs())),
        Variant::Float(value) => Ok(Variant::Float(value.abs())),
        Variant::Vector2(v) => Ok(Variant::Vector2(v.abs())),
        Variant::Vector2i(v) => Ok(Variant::Vector2i(v.abs())),
        Variant::Vector3(v) => Ok(Variant::Vector3(v.abs())),
        Variant::Vector3i(v) => Ok(Variant::Vector3i(v.abs())),
        Variant::Vector4(v) => Ok(Variant::Vector4(v.abs())),
        Variant::Vector4i(v) => Ok(Variant::Vector4i(v.abs())),
        _ => Err(invalid(0, VariantType::Float)),
    }
}

/// `VariantUtilityFunctions::sign`.
pub fn sign(x: &Variant) -> UtilityResult {
    match x {
        Variant::Int(value) => Ok(Variant::Int(value.signum())),
        Variant::Float(value) => Ok(Variant::Float(if *value < 0.0 {
            -1.0
        } else if *value > 0.0 {
            1.0
        } else {
            0.0
        })),
        Variant::Vector2(v) => Ok(Variant::Vector2(v.sign())),
        Variant::Vector2i(v) => Ok(Variant::Vector2i(v.sign())),
        Variant::Vector3(v) => Ok(Variant::Vector3(v.sign())),
        Variant::Vector3i(v) => Ok(Variant::Vector3i(v.sign())),
        Variant::Vector4(v) => Ok(Variant::Vector4(v.sign())),
        Variant::Vector4i(v) => Ok(Variant::Vector4i(v.sign())),
        _ => Err(invalid(0, VariantType::Float)),
    }
}

/// `VariantUtilityFunctions::snapped`: int/float snap by the matching step type.
pub fn snapped(x: &Variant, step: &Variant) -> UtilityResult {
    match x {
        Variant::Int(value) => {
            let step_value = match step {
                Variant::Int(step) => *step,
                Variant::Float(step) => *step as i64,
                _ => return Err(invalid(1, VariantType::Int)),
            };
            if step_value == 0 {
                return Ok(Variant::Int(0));
            }
            Ok(Variant::Int(
                (*value as f64 / step_value as f64).round() as i64 * step_value,
            ))
        }
        Variant::Float(value) => {
            let step_value = as_f64(step).ok_or(invalid(1, VariantType::Float))?;
            Ok(Variant::Float(functions_snappedf(*value, step_value)))
        }
        Variant::Vector2(v) => {
            let s = vector2_parts(step).ok_or(invalid(1, VariantType::Vector2))?;
            Ok(Variant::Vector2(Vector2::new(
                functions_snappedf(v.x as f64, s.0) as f32,
                functions_snappedf(v.y as f64, s.1) as f32,
            )))
        }
        Variant::Vector2i(v) => match step {
            Variant::Vector2i(s) => Ok(Variant::Vector2i(Vector2i::new(
                snap_i(v.x, s.x),
                snap_i(v.y, s.y),
            ))),
            _ => Err(invalid(1, VariantType::Vector2i)),
        },
        Variant::Vector3(v) => {
            let step = match step {
                Variant::Vector3(s) => (*s),
                _ => return Err(invalid(1, VariantType::Vector3)),
            };
            Ok(Variant::Vector3(Vector3::new(
                functions_snappedf(v.x as f64, step.x as f64) as f32,
                functions_snappedf(v.y as f64, step.y as f64) as f32,
                functions_snappedf(v.z as f64, step.z as f64) as f32,
            )))
        }
        Variant::Vector3i(v) => match step {
            Variant::Vector3i(s) => Ok(Variant::Vector3i(Vector3i::new(
                snap_i(v.x, s.x),
                snap_i(v.y, s.y),
                snap_i(v.z, s.z),
            ))),
            _ => Err(invalid(1, VariantType::Vector3i)),
        },
        Variant::Vector4(v) => {
            let step = match step {
                Variant::Vector4(s) => (*s),
                _ => return Err(invalid(1, VariantType::Vector4)),
            };
            Ok(Variant::Vector4(Vector4::new(
                functions_snappedf(v.x as f64, step.x as f64) as f32,
                functions_snappedf(v.y as f64, step.y as f64) as f32,
                functions_snappedf(v.z as f64, step.z as f64) as f32,
                functions_snappedf(v.w as f64, step.w as f64) as f32,
            )))
        }
        Variant::Vector4i(v) => match step {
            Variant::Vector4i(s) => Ok(Variant::Vector4i(Vector4i::new(
                snap_i(v.x, s.x),
                snap_i(v.y, s.y),
                snap_i(v.z, s.z),
                snap_i(v.w, s.w),
            ))),
            _ => Err(invalid(1, VariantType::Vector4i)),
        },
        _ => Err(invalid(0, VariantType::Float)),
    }
}

fn snap_i(value: i32, step: i32) -> i32 {
    if step == 0 {
        return 0;
    }
    ((value as f64 / step as f64).round() as i32) * step
}

/// `Math::snappedf`.
pub fn functions_snappedf(x: f64, step: f64) -> f64 {
    if step != 0.0 {
        (x / step).floor() * step + ((x % step) / step).round() * step
    } else {
        0.0
    }
}

/// `VariantUtilityFunctions::lerp`: ints/floats lerp to a float, matching the
/// engine's `lerpf` dispatch.
pub fn lerp(from: &Variant, to: &Variant, weight: f64) -> UtilityResult {
    match from {
        Variant::Int(value) => Ok(Variant::Float(lerpf(*value as f64, to_f64(to)?, weight))),
        Variant::Float(value) => Ok(Variant::Float(lerpf(*value, to_f64(to)?, weight))),
        Variant::Vector2(a) => {
            let b = as_vector2(to).ok_or(invalid(1, VariantType::Vector2))?;
            Ok(Variant::Vector2(Vector2::new(
                lerpf(a.x as f64, b.x as f64, weight) as f32,
                lerpf(a.y as f64, b.y as f64, weight) as f32,
            )))
        }
        Variant::Vector3(a) => {
            let b = match to {
                Variant::Vector3(b) => *b,
                _ => return Err(invalid(1, VariantType::Vector3)),
            };
            Ok(Variant::Vector3(Vector3::new(
                lerpf(a.x as f64, b.x as f64, weight) as f32,
                lerpf(a.y as f64, b.y as f64, weight) as f32,
                lerpf(a.z as f64, b.z as f64, weight) as f32,
            )))
        }
        Variant::Vector4(a) => {
            let b = match to {
                Variant::Vector4(b) => *b,
                _ => return Err(invalid(1, VariantType::Vector4)),
            };
            Ok(Variant::Vector4(Vector4::new(
                lerpf(a.x as f64, b.x as f64, weight) as f32,
                lerpf(a.y as f64, b.y as f64, weight) as f32,
                lerpf(a.z as f64, b.z as f64, weight) as f32,
                lerpf(a.w as f64, b.w as f64, weight) as f32,
            )))
        }
        Variant::Color(a) => {
            let b = match to {
                Variant::Color(b) => *b,
                _ => return Err(invalid(1, VariantType::Color)),
            };
            Ok(Variant::Color(Color::new(
                lerpf(a.r as f64, b.r as f64, weight) as f32,
                lerpf(a.g as f64, b.g as f64, weight) as f32,
                lerpf(a.b as f64, b.b as f64, weight) as f32,
                lerpf(a.a as f64, b.a as f64, weight) as f32,
            )))
        }
        _ => Err(invalid(0, VariantType::Float)),
    }
}

fn to_f64(value: &Variant) -> Result<f64, CallError> {
    as_f64(value).ok_or(invalid(1, VariantType::Float))
}

fn as_vector2(value: &Variant) -> Option<Vector2> {
    match value {
        Variant::Vector2(v) => Some(*v),
        _ => None,
    }
}

/// `VariantUtilityFunctions::clamp` for the numeric types.
pub fn clamp(x: &Variant, min: &Variant, max: &Variant) -> UtilityResult {
    match x {
        Variant::Int(value) => {
            let min = as_f64(min).ok_or(invalid(1, VariantType::Int))?;
            let max = as_f64(max).ok_or(invalid(2, VariantType::Int))?;
            let numeric = *value as f64;
            if numeric < min {
                Ok(Variant::Int(min as i64))
            } else if numeric > max {
                Ok(Variant::Int(max as i64))
            } else {
                Ok(Variant::Int(*value))
            }
        }
        Variant::Float(value) => {
            let min = as_f64(min).ok_or(invalid(1, VariantType::Float))?;
            let max = as_f64(max).ok_or(invalid(2, VariantType::Float))?;
            Ok(Variant::Float(value.clamp(min, max)))
        }
        Variant::Vector2(v) => {
            let min = as_vector2(min).ok_or(invalid(1, VariantType::Vector2))?;
            let max = as_vector2(max).ok_or(invalid(2, VariantType::Vector2))?;
            Ok(Variant::Vector2(Vector2::new(
                v.x.clamp(min.x, max.x),
                v.y.clamp(min.y, max.y),
            )))
        }
        _ => Err(invalid(0, VariantType::Float)),
    }
}

/// `VariantUtilityFunctions::wrap` for ints and floats.
pub fn wrap(x: &Variant, min: &Variant, max: &Variant) -> UtilityResult {
    match x {
        Variant::Int(value) => {
            let min = as_f64(min).ok_or(invalid(1, VariantType::Int))? as i64;
            let max = as_f64(max).ok_or(invalid(2, VariantType::Int))? as i64;
            Ok(Variant::Int(wrapi(*value, min, max)))
        }
        Variant::Float(value) => {
            let min = as_f64(min).ok_or(invalid(1, VariantType::Float))?;
            let max = as_f64(max).ok_or(invalid(2, VariantType::Float))?;
            Ok(Variant::Float(wrapf(*value, min, max)))
        }
        _ => Err(invalid(0, VariantType::Float)),
    }
}

/// `VariantUtilityFunctions::wrapi`.
pub fn wrapi(value: i64, min: i64, max: i64) -> i64 {
    let range = max - min;
    if range == 0 {
        return min;
    }
    min + (value - min).rem_euclid(range)
}

/// `VariantUtilityFunctions::min` for two values of the same type.
pub fn min(a: &Variant, b: &Variant) -> UtilityResult {
    match (a, b) {
        (Variant::Int(x), Variant::Int(y)) => Ok(Variant::Int(*x.min(y))),
        (Variant::Float(x), Variant::Float(y)) => Ok(Variant::Float(x.min(*y))),
        (Variant::Int(x), Variant::Float(y)) => Ok(Variant::Float((*x as f64).min(*y))),
        (Variant::Float(x), Variant::Int(y)) => Ok(Variant::Float(x.min(*y as f64))),
        (Variant::Vector2(x), Variant::Vector2(y)) => Ok(Variant::Vector2(Vector2::new(
            x.x.min(y.x),
            x.y.min(y.y),
        ))),
        (Variant::Vector3(x), Variant::Vector3(y)) => Ok(Variant::Vector3(Vector3::new(
            x.x.min(y.x),
            x.y.min(y.y),
            x.z.min(y.z),
        ))),
        (Variant::Vector2i(x), Variant::Vector2i(y)) => Ok(Variant::Vector2i(Vector2i::new(
            x.x.min(y.x),
            x.y.min(y.y),
        ))),
        (Variant::Vector3i(x), Variant::Vector3i(y)) => Ok(Variant::Vector3i(Vector3i::new(
            x.x.min(y.x),
            x.y.min(y.y),
            x.z.min(y.z),
        ))),
        _ => Err(invalid(1, a.vtype())),
    }
}

/// `VariantUtilityFunctions::max` for two values of the same type.
pub fn max(a: &Variant, b: &Variant) -> UtilityResult {
    match (a, b) {
        (Variant::Int(x), Variant::Int(y)) => Ok(Variant::Int(*x.max(y))),
        (Variant::Float(x), Variant::Float(y)) => Ok(Variant::Float(x.max(*y))),
        (Variant::Int(x), Variant::Float(y)) => Ok(Variant::Float((*x as f64).max(*y))),
        (Variant::Float(x), Variant::Int(y)) => Ok(Variant::Float(x.max(*y as f64))),
        (Variant::Vector2(x), Variant::Vector2(y)) => Ok(Variant::Vector2(Vector2::new(
            x.x.max(y.x),
            x.y.max(y.y),
        ))),
        (Variant::Vector3(x), Variant::Vector3(y)) => Ok(Variant::Vector3(Vector3::new(
            x.x.max(y.x),
            x.y.max(y.y),
            x.z.max(y.z),
        ))),
        (Variant::Vector2i(x), Variant::Vector2i(y)) => Ok(Variant::Vector2i(Vector2i::new(
            x.x.max(y.x),
            x.y.max(y.y),
        ))),
        (Variant::Vector3i(x), Variant::Vector3i(y)) => Ok(Variant::Vector3i(Vector3i::new(
            x.x.max(y.x),
            x.y.max(y.y),
            x.z.max(y.z),
        ))),
        _ => Err(invalid(1, a.vtype())),
    }
}

// -- string conversions ------------------------------------------------------

/// `str_to_var`: parses a variant expression (`[1, "two"]`, `{"k": 3}`, ...)
/// through the ported variant parser; `Nil` on a parse error (the engine
/// returns a nil variant too).
pub fn str_to_var(text: &str) -> Variant {
    crate::io::variant_parser::parse(text).unwrap_or(Variant::Nil)
}

/// `str_to_var` with the error preserved (port-side helper).
pub fn str_to_var_checked(
    text: &str,
) -> Result<Variant, crate::io::variant_parser::ParseError> {
    crate::io::variant_parser::parse(text)
}

/// `var_to_str`: the canonical text form produced by the ported writer.
pub fn var_to_str(value: &Variant) -> String {
    crate::io::variant_parser::write_variant(value)
}

/// `type_convert`: the numeric/string core of the engine's conversion table.
/// Unsupported targets report the expected type.
pub fn type_convert(value: &Variant, target: VariantType) -> UtilityResult {
    match target {
        VariantType::Nil => Ok(Variant::Nil),
        VariantType::Bool => Ok(Variant::Bool(value.booleanize())),
        VariantType::Int => Ok(Variant::Int(value.to_int())),
        VariantType::Float => Ok(Variant::Float(value.to_float())),
        VariantType::String => Ok(Variant::String(crate::string::gstring::GoString::from_str(
            &var_to_str(value),
        ))),
        VariantType::StringName => Ok(Variant::String(crate::string::gstring::GoString::from_str(
            &var_to_str(value),
        ))),
        other => Err(invalid(1, other)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::string::gstring::GoString;

    fn approx(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    #[test]
    fn scalar_wrappers() {
        assert!(approx(sin(std::f64::consts::FRAC_PI_2), 1.0));
        assert!(approx(cos(0.0), 1.0));
        assert!(approx(tan(0.0), 0.0));
        assert!(approx(sqrt(9.0), 3.0));
        assert!(approx(pow(2.0, 10.0), 1024.0));
        assert!(approx(log(std::f64::consts::E), 1.0));
        assert!(approx(exp(0.0), 1.0));
        assert!(approx(fmod(7.0, 3.0), 1.0));
        assert!(approx(fmod(-1.0, 3.0), -1.0));

        // fposmod keeps the sign of the divisor.
        assert!(approx(fposmod(-1.0, 3.0), 2.0));
        assert!(approx(fposmod(7.0, 3.0), 1.0));
        assert!(approx(fposmod(1.0, -3.0), -2.0));

        assert!(approx(deg_to_rad(180.0), std::f64::consts::PI));
        assert!(approx(rad_to_deg(std::f64::consts::PI), 180.0));
        assert!(approx(linear_to_db(1.0), 0.0));
        assert!(approx(db_to_linear(0.0), 1.0));
        assert!(approx(db_to_linear(linear_to_db(0.5)), 0.5));

        assert!(is_nan(f64::NAN));
        assert!(is_inf(f64::INFINITY));
        assert!(!is_finite(f64::INFINITY));
        assert!(is_equal_approx(1.0, 1.0 + 1e-9));
        assert!(is_zero_approx(1e-9));
        assert!(is_same(f64::NAN, f64::NAN));
        assert!(!is_same(1.0, 2.0));

        assert!(approx(ease(0.0, 2.0), 0.0));
        assert!(approx(ease(1.0, 2.0), 1.0));
        assert_eq!(step_decimals(0.01), 2);
        assert_eq!(step_decimals(1.0), 0);
    }

    #[test]
    fn interpolation_helpers() {
        assert!(approx(lerpf(0.0, 10.0, 0.25), 2.5));
        assert!(approx(inverse_lerp(0.0, 10.0, 2.5), 0.25));
        assert!(approx(remap(5.0, 0.0, 10.0, 100.0, 200.0), 150.0));
        assert!(approx(smoothstep(0.0, 1.0, 0.5), 0.5));
        assert!(approx(smoothstep(0.0, 1.0, -1.0), 0.0));
        assert!(approx(smoothstep(0.0, 1.0, 2.0), 1.0));
        // Degenerate ranges: 0 below, 1 above.
        assert!(approx(smoothstep(1.0, 1.0, 0.5), 0.0));
        assert!(approx(smoothstep(1.0, 1.0, 1.5), 1.0));

        assert!(approx(move_toward(0.0, 1.0, 0.25), 0.25));
        assert!(approx(move_toward(0.0, 1.0, 2.0), 1.0));
        assert!(approx(move_toward(0.0, -1.0, 0.5), -0.5));

        // Angle helpers wrap across PI.
        assert!(approx(angle_difference(0.0, 1.0), 1.0));
        let half_turn = angle_difference(0.0, std::f64::consts::PI + 0.5);
        assert!(approx(half_turn, -std::f64::consts::PI + 0.5));
        assert!(approx(lerp_angle(0.0, std::f64::consts::TAU, 0.5), 0.0));
        assert!(approx(rotate_toward(0.0, 1.0, 0.5), 0.5));
        assert!(approx(rotate_toward(0.0, 1.0, 2.0), 1.0));
        assert!(approx(rotate_toward(0.0, 1.0, -10.0), -std::f64::consts::PI + 1.0));

        assert!(approx(pingpong(0.0, 1.0), 0.0));
        assert!(approx(pingpong(0.5, 1.0), 0.5));
        assert!(approx(pingpong(1.5, 1.0), 0.5));
        assert!(approx(pingpong(2.0, 1.0), 0.0));
        assert!(approx(pingpong(3.0, 0.0), 0.0));

        assert!(approx(wrapf(1.5, 0.0, 1.0), 0.5));
        assert!(approx(wrapf(-0.25, 0.0, 1.0), 0.75));
        assert!(approx(wrapf(5.0, 1.0, 1.0), 1.0));

        // Cubic and Bezier forms hit their endpoints.
        assert!(approx(cubic_interpolate(0.0, 10.0, 5.0, 5.0, 0.0), 0.0));
        assert!(approx(cubic_interpolate(0.0, 10.0, 5.0, 5.0, 1.0), 10.0));
        assert!(approx(
            cubic_interpolate_angle(0.0, 10.0, 5.0, 5.0, 0.0),
            0.0
        ));
        assert!(approx(cubic_interpolate_in_time(0.0, 10.0, 5.0, 5.0, 0.0, 1.0, 0.5, 1.5), 0.0));
        assert!(approx(cubic_interpolate_in_time(0.0, 10.0, 5.0, 5.0, 1.0, 1.0, 0.5, 1.5), 10.0));
        assert!(approx(bezier_interpolate(0.0, 0.0, 10.0, 10.0, 0.5), 5.0));
        // 6 * 0.5 * 0.5 * (10 - 0) = 15\n        assert!(approx(bezier_derivative(0.0, 0.0, 10.0, 10.0, 0.5), 15.0));
    }

    #[test]
    fn variant_dispatch_numeric() {
        assert_eq!(floor(&Variant::Int(3)).unwrap(), Variant::Int(3));
        assert_eq!(floor(&Variant::Float(3.7)).unwrap(), Variant::Float(3.0));
        assert_eq!(ceil(&Variant::Float(3.2)).unwrap(), Variant::Float(4.0));
        assert_eq!(round(&Variant::Float(3.5)).unwrap(), Variant::Float(4.0));
        assert_eq!(round(&Variant::Float(-2.5)).unwrap(), Variant::Float(-3.0));
        assert_eq!(abs(&Variant::Int(-4)).unwrap(), Variant::Int(4));
        assert_eq!(abs(&Variant::Float(-4.5)).unwrap(), Variant::Float(4.5));
        assert_eq!(sign(&Variant::Int(-9)).unwrap(), Variant::Int(-1));
        assert_eq!(sign(&Variant::Float(0.0)).unwrap(), Variant::Float(0.0));

        // Vectors are component-wise.
        assert_eq!(
            floor(&Variant::Vector2(Vector2::new(1.9, -0.1))).unwrap(),
            Variant::Vector2(Vector2::new(1.0, -1.0))
        );
        assert_eq!(
            ceil(&Variant::Vector3(Vector3::new(1.1, 1.0, -0.5))).unwrap(),
            Variant::Vector3(Vector3::new(2.0, 1.0, 0.0))
        );
        assert_eq!(
            abs(&Variant::Vector2(Vector2::new(-1.0, 2.0))).unwrap(),
            Variant::Vector2(Vector2::new(1.0, 2.0))
        );
        assert_eq!(
            sign(&Variant::Vector3(Vector3::new(-1.0, 0.0, 3.0))).unwrap(),
            Variant::Vector3(Vector3::new(-1.0, 0.0, 1.0))
        );
        // Integer vectors come back unchanged for rounding.
        assert_eq!(
            round(&Variant::Vector2i(Vector2i::new(1, -2))).unwrap(),
            Variant::Vector2i(Vector2i::new(1, -2))
        );

        // Unsupported types report the argument and expected type.
        match floor(&Variant::String(GoString::from_str("x"))) {
            Err(CallError::InvalidArgument { argument, expected }) => {
                assert_eq!(argument, 0);
                assert_eq!(expected, VariantType::Float);
            }
            other => panic!("expected an invalid-argument error, got {other:?}"),
        }
    }

    #[test]
    fn snapping_lerping_and_ranges() {
        assert_eq!(snapped(&Variant::Int(7), &Variant::Int(5)).unwrap(), Variant::Int(5));
        assert_eq!(
            snapped(&Variant::Float(1.26), &Variant::Float(0.25)).unwrap(),
            Variant::Float(1.25)
        );
        assert_eq!(
            snapped(&Variant::Float(1.26), &Variant::Float(0.0)).unwrap(),
            Variant::Float(0.0)
        );
        assert_eq!(
            snapped(
                &Variant::Vector2(Vector2::new(1.26, 2.74)),
                &Variant::Vector2(Vector2::new(0.25, 0.5))
            )
            .unwrap(),
            Variant::Vector2(Vector2::new(1.25, 2.5))
        );
        assert!(snapped(&Variant::Vector2(Vector2::ZERO), &Variant::Int(1)).is_err());

        // Lerp produces floats for numeric inputs.
        assert_eq!(lerp(&Variant::Int(0), &Variant::Int(10), 0.5).unwrap(), Variant::Float(5.0));
        assert_eq!(
            lerp(&Variant::Float(2.0), &Variant::Float(4.0), 0.25).unwrap(),
            Variant::Float(2.5)
        );
        assert_eq!(
            lerp(
                &Variant::Vector2(Vector2::ZERO),
                &Variant::Vector2(Vector2::new(10.0, 20.0)),
                0.5
            )
            .unwrap(),
            Variant::Vector2(Vector2::new(5.0, 10.0))
        );
        assert!(lerp(&Variant::Int(1), &Variant::String(GoString::from_str("x")), 0.5).is_err());

        assert_eq!(
            clamp(&Variant::Int(5), &Variant::Int(0), &Variant::Int(3)).unwrap(),
            Variant::Int(3)
        );
        assert_eq!(
            clamp(&Variant::Float(-1.0), &Variant::Float(0.0), &Variant::Float(2.0)).unwrap(),
            Variant::Float(0.0)
        );
        assert_eq!(
            clamp(
                &Variant::Vector2(Vector2::new(-1.0, 5.0)),
                &Variant::Vector2(Vector2::ZERO),
                &Variant::Vector2(Vector2::ONE)
            )
            .unwrap(),
            Variant::Vector2(Vector2::new(0.0, 1.0))
        );

        assert_eq!(
            wrap(&Variant::Int(5), &Variant::Int(0), &Variant::Int(3)).unwrap(),
            Variant::Int(2)
        );
        assert_eq!(
            wrap(&Variant::Float(-0.25), &Variant::Float(0.0), &Variant::Float(1.0)).unwrap(),
            Variant::Float(0.75)
        );
        assert_eq!(wrapi(7, 0, 3), 1);
        assert_eq!(wrapi(-1, 0, 3), 2);
        assert_eq!(wrapi(5, 2, 2), 2);

        assert_eq!(min(&Variant::Int(3), &Variant::Int(5)).unwrap(), Variant::Int(3));
        assert_eq!(max(&Variant::Float(3.0), &Variant::Float(5.0)).unwrap(), Variant::Float(5.0));
        assert_eq!(
            min(&Variant::Vector2(Vector2::new(1.0, 4.0)), &Variant::Vector2(Vector2::new(3.0, 2.0)))
                .unwrap(),
            Variant::Vector2(Vector2::new(1.0, 2.0))
        );
        assert_eq!(
            max(&Variant::Vector3i(Vector3i::new(1, 5, 3)), &Variant::Vector3i(Vector3i::new(2, 4, 9)))
                .unwrap(),
            Variant::Vector3i(Vector3i::new(2, 5, 9))
        );
        // Mixed numeric types promote to float (as in the engine's Variant comparison).
        assert_eq!(min(&Variant::Int(2), &Variant::Float(1.5)).unwrap(), Variant::Float(1.5));
        assert!(min(&Variant::Int(2), &Variant::String(GoString::from_str("x"))).is_err());
    }

    #[test]
    fn string_conversions() {
        // Parsing goes through the ported variant parser; failures are Nil.
        assert_eq!(str_to_var("42"), Variant::Int(42));
        assert_eq!(str_to_var("1.5"), Variant::Float(1.5));
        assert_eq!(str_to_var("\"text\""), Variant::String(GoString::from_str("text")));
        assert_eq!(
            str_to_var("[1, 2]"),
            Variant::Array(vec![Variant::Int(1), Variant::Int(2)])
        );
        assert_eq!(str_to_var("not a variant at all"), Variant::Nil);
        assert!(str_to_var_checked("not a variant at all").is_err());

        // Writing is the canonical text form; parsing it back round-trips.
        let value = Variant::Array(vec![
            Variant::Int(1),
            Variant::String(GoString::from_str("two")),
        ]);
        let text = var_to_str(&value);
        assert_eq!(str_to_var(&text), value);
        assert_eq!(var_to_str(&Variant::Int(7)), "7");
        assert_eq!(var_to_str(&Variant::Float(1.5)), "1.5");

        // type_convert covers the scalar targets.
        assert_eq!(
            type_convert(&Variant::Float(2.9), VariantType::Int).unwrap(),
            Variant::Int(2)
        );
        assert_eq!(
            type_convert(&Variant::Int(3), VariantType::Float).unwrap(),
            Variant::Float(3.0)
        );
        assert_eq!(
            type_convert(&Variant::Int(0), VariantType::Bool).unwrap(),
            Variant::Bool(false)
        );
        assert_eq!(
            type_convert(&Variant::Int(1), VariantType::Bool).unwrap(),
            Variant::Bool(true)
        );
        assert_eq!(
            type_convert(&Variant::Bool(true), VariantType::String).unwrap(),
            Variant::String(GoString::from_str("true"))
        );
        assert_eq!(
            type_convert(&Variant::Nil, VariantType::Nil).unwrap(),
            Variant::Nil
        );
        // Unsupported targets report the requested type.
        match type_convert(&Variant::Int(1), VariantType::Vector2) {
            Err(CallError::InvalidArgument { expected, .. }) => {
                assert_eq!(expected, VariantType::Vector2);
            }
            other => panic!("expected an invalid-argument error, got {other:?}"),
        }
    }
}
