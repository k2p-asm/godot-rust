//! Arithmetic on `Variant` for the POD payloads (`variant_op.cpp` subset).
//!
//! Implemented: `+`, `-`, `*`, `/` for Int, Float, Vector2, Vector3 and the
//! Int/Float scalar mixes. Not implemented: strings, arrays, dictionaries and
//! every conversion rule beyond Int <-> Float.
//!
//! Non-differential: `variant_op.cpp` includes the object system, so the parity
//! harness cannot link it. These tests pin the arithmetic the port defines.

use crate::math::vector2::Vector2;
use crate::math::color::Color;
use crate::math::vector3::Vector3;
use crate::math::vector4::Vector4;
use crate::variant::Variant;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Op {
    Add,
    Sub,
    Mul,
    Div,
}

/// Applies `op` to two variants; `None` when the combination is unsupported
/// (the engine reports an "invalid operands" error in that case).
pub fn apply(lhs: &Variant, rhs: &Variant, op: Op) -> Option<Variant> {
    use Op::*;

    match (lhs, rhs) {
        // Integers stay integers.
        (Variant::Int(a), Variant::Int(b)) => match op {
            Add => Some(Variant::Int(a.wrapping_add(*b))),
            Sub => Some(Variant::Int(a.wrapping_sub(*b))),
            Mul => Some(Variant::Int(a.wrapping_mul(*b))),
            Div => {
                if *b == 0 {
                    None
                } else {
                    Some(Variant::Int(a.wrapping_div(*b)))
                }
            }
        },

        // Any float operand promotes to float.
        (Variant::Float(a), Variant::Float(b)) => Some(Variant::Float(float_op(*a, *b, op))),
        (Variant::Int(a), Variant::Float(b)) => Some(Variant::Float(float_op(*a as f64, *b, op))),
        (Variant::Float(a), Variant::Int(b)) => Some(Variant::Float(float_op(*a, *b as f64, op))),

        // Vectors: component-wise with vectors, uniform with scalars.
        (Variant::Vector2(a), Variant::Vector2(b)) => Some(Variant::Vector2(match op {
            Add => Vector2::new(a.x + b.x, a.y + b.y),
            Sub => Vector2::new(a.x - b.x, a.y - b.y),
            Mul => Vector2::new(a.x * b.x, a.y * b.y),
            Div => Vector2::new(a.x / b.x, a.y / b.y),
        })),
        (Variant::Vector3(a), Variant::Vector3(b)) => Some(Variant::Vector3(match op {
            Add => Vector3::new(a.x + b.x, a.y + b.y, a.z + b.z),
            Sub => Vector3::new(a.x - b.x, a.y - b.y, a.z - b.z),
            Mul => Vector3::new(a.x * b.x, a.y * b.y, a.z * b.z),
            Div => Vector3::new(a.x / b.x, a.y / b.y, a.z / b.z),
        })),
        (Variant::Vector2(a), Variant::Float(s)) => Some(Variant::Vector2(scale2(*a, *s as f32, op))),
        (Variant::Vector2(a), Variant::Int(s)) => Some(Variant::Vector2(scale2(*a, *s as f32, op))),
        (Variant::Vector3(a), Variant::Float(s)) => Some(Variant::Vector3(scale3(*a, *s as f32, op))),
        (Variant::Vector3(a), Variant::Int(s)) => Some(Variant::Vector3(scale3(*a, *s as f32, op))),

        // 4D vectors and colors: component-wise, like the engine's variant ops.
        (Variant::Vector4(a), Variant::Vector4(b)) => Some(Variant::Vector4(match op {
            Add => Vector4::new(a.x + b.x, a.y + b.y, a.z + b.z, a.w + b.w),
            Sub => Vector4::new(a.x - b.x, a.y - b.y, a.z - b.z, a.w - b.w),
            Mul => Vector4::new(a.x * b.x, a.y * b.y, a.z * b.z, a.w * b.w),
            Div => Vector4::new(a.x / b.x, a.y / b.y, a.z / b.z, a.w / b.w),
        })),
        (Variant::Vector4(a), Variant::Float(s)) => {
            let s = *s as f32;
            Some(Variant::Vector4(match op {
                Add => Vector4::new(a.x + s, a.y + s, a.z + s, a.w + s),
                Sub => Vector4::new(a.x - s, a.y - s, a.z - s, a.w - s),
                Mul => Vector4::new(a.x * s, a.y * s, a.z * s, a.w * s),
                Div => Vector4::new(a.x / s, a.y / s, a.z / s, a.w / s),
            }))
        }
        (Variant::Color(a), Variant::Color(b)) => Some(Variant::Color(match op {
            Add => Color::new(a.r + b.r, a.g + b.g, a.b + b.b, a.a + b.a),
            Sub => Color::new(a.r - b.r, a.g - b.g, a.b - b.b, a.a - b.a),
            Mul => Color::new(a.r * b.r, a.g * b.g, a.b * b.b, a.a * b.a),
            Div => Color::new(a.r / b.r, a.g / b.g, a.b / b.b, a.a / b.a),
        })),
        (Variant::Color(a), Variant::Float(s)) => {
            let s = *s as f32;
            Some(Variant::Color(match op {
                Add => Color::new(a.r + s, a.g + s, a.b + s, a.a + s),
                Sub => Color::new(a.r - s, a.g - s, a.b - s, a.a - s),
                Mul => Color::new(a.r * s, a.g * s, a.b * s, a.a * s),
                Div => Color::new(a.r / s, a.g / s, a.b / s, a.a / s),
            }))
        }

        // Strings concatenate; every other string operation is invalid.
        (Variant::String(a), Variant::String(b)) => match op {
            Add => Some(Variant::String(a.concatenated(b))),
            _ => None,
        },

        _ => None,
    }
}

fn float_op(a: f64, b: f64, op: Op) -> f64 {
    match op {
        Op::Add => a + b,
        Op::Sub => a - b,
        Op::Mul => a * b,
        Op::Div => a / b,
    }
}

fn scale2(v: Vector2, s: f32, op: Op) -> Vector2 {
    match op {
        Op::Add => Vector2::new(v.x + s, v.y + s),
        Op::Sub => Vector2::new(v.x - s, v.y - s),
        Op::Mul => Vector2::new(v.x * s, v.y * s),
        Op::Div => Vector2::new(v.x / s, v.y / s),
    }
}

fn scale3(v: Vector3, s: f32, op: Op) -> Vector3 {
    match op {
        Op::Add => Vector3::new(v.x + s, v.y + s, v.z + s),
        Op::Sub => Vector3::new(v.x - s, v.y - s, v.z - s),
        Op::Mul => Vector3::new(v.x * s, v.y * s, v.z * s),
        Op::Div => Vector3::new(v.x / s, v.y / s, v.z / s),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn integers_stay_integers() {
        assert_eq!(apply(&Variant::Int(7), &Variant::Int(3), Op::Add), Some(Variant::Int(10)));
        assert_eq!(apply(&Variant::Int(7), &Variant::Int(3), Op::Sub), Some(Variant::Int(4)));
        assert_eq!(apply(&Variant::Int(7), &Variant::Int(3), Op::Mul), Some(Variant::Int(21)));
        // Integer division truncates, like the engine.
        assert_eq!(apply(&Variant::Int(7), &Variant::Int(3), Op::Div), Some(Variant::Int(2)));
        // Division by zero is an error, not infinity.
        assert_eq!(apply(&Variant::Int(1), &Variant::Int(0), Op::Div), None);
    }

    #[test]
    fn int_and_float_promote() {
        assert_eq!(apply(&Variant::Int(2), &Variant::Float(0.5), Op::Mul), Some(Variant::Float(1.0)));
        assert_eq!(apply(&Variant::Float(1.5), &Variant::Int(2), Op::Add), Some(Variant::Float(3.5)));
    }

    #[test]
    fn vectors_are_component_wise_with_vectors() {
        let a = Variant::Vector3(Vector3::new(1.0, 2.0, 3.0));
        let b = Variant::Vector3(Vector3::new(4.0, 5.0, 6.0));
        assert_eq!(
            apply(&a, &b, Op::Add),
            Some(Variant::Vector3(Vector3::new(5.0, 7.0, 9.0)))
        );
        assert_eq!(
            apply(&a, &b, Op::Mul),
            Some(Variant::Vector3(Vector3::new(4.0, 10.0, 18.0)))
        );
    }

    #[test]
    fn vectors_scale_uniformly_with_scalars() {
        let a = Variant::Vector2(Vector2::new(1.0, -2.0));
        assert_eq!(
            apply(&a, &Variant::Float(2.0), Op::Mul),
            Some(Variant::Vector2(Vector2::new(2.0, -4.0)))
        );
        assert_eq!(
            apply(&a, &Variant::Int(2), Op::Div),
            Some(Variant::Vector2(Vector2::new(0.5, -1.0)))
        );
    }

    #[test]
    fn vector4_and_color_ops() {
        use crate::math::color::Color;
        use crate::math::vector4::Vector4;

        assert_eq!(
            apply(
                &Variant::Vector4(Vector4::new(1.0, 2.0, 3.0, 4.0)),
                &Variant::Vector4(Vector4::new(1.0, 1.0, 1.0, 1.0)),
                Op::Sub
            ),
            Some(Variant::Vector4(Vector4::new(0.0, 1.0, 2.0, 3.0)))
        );

        let c = Variant::Color(Color::new(1.0, 0.5, 0.0, 1.0));
        assert_eq!(
            apply(&c, &Variant::Float(2.0), Op::Mul),
            Some(Variant::Color(Color::new(2.0, 1.0, 0.0, 2.0)))
        );
        // Color + Int is not supported (only Float).
        assert_eq!(apply(&c, &Variant::Int(2), Op::Mul), None);
    }

    #[test]
    fn strings_concatenate_but_do_not_subtract() {
        use crate::string::gstring::GoString;

        let a = Variant::String(GoString::from_str("foo"));
        let b = Variant::String(GoString::from_str("bar"));
        assert_eq!(
            apply(&a, &b, Op::Add),
            Some(Variant::String(GoString::from_str("foobar")))
        );
        assert_eq!(apply(&a, &b, Op::Sub), None);
        assert_eq!(apply(&a, &b, Op::Mul), None);
        // String + Int is not supported either.
        assert_eq!(apply(&a, &Variant::Int(1), Op::Add), None);
    }

    #[test]
    fn unsupported_combinations_return_none() {
        assert_eq!(apply(&Variant::Vector3(Vector3::ONE), &Variant::Vector2(Vector2::ONE), Op::Add), None);
        assert_eq!(apply(&Variant::Bool(true), &Variant::Bool(false), Op::Add), None);
        assert_eq!(apply(&Variant::Nil, &Variant::Int(1), Op::Add), None);
    }
}
