//! `core/io/json.*` native conversions (subset).
//!
//! `JSON::from_native` turns a `Variant` into JSON-safe data and `to_native`
//! reverses it. This port covers the scalar prefixes (`i:`, `f:`, `s:`, `sn:`,
//! `np:`), arrays/dictionaries (recursively, untyped), and the math/colour
//! builtins: Vector2/2i, Rect2/2i, Vector3/3i, Transform2D, Vector4/4i, Plane,
//! Quaternion, AABB, Basis, Transform3D, Projection and Color, using the same
//! argument order as the engine.
//!
//! Not ported: typed arrays/dictionaries metadata, `Resource`/`Object`
//! serialization, real `RID`/`Callable`/`Signal` payloads (they encode as a
//! type-only dictionary and decode to `Nil`), and `p_full_objects`.

use crate::math::aabb::Aabb;
use crate::math::basis::Basis;
use crate::math::color::Color;
use crate::math::plane::Plane;
use crate::math::projection::Projection;
use crate::math::quaternion::Quaternion;
use crate::math::rect2::Rect2;
use crate::math::rect2i::Rect2i;
use crate::math::transform_2d::Transform2D;
use crate::math::transform_3d::Transform3D;
use crate::math::vector2::Vector2;
use crate::math::vector2i::Vector2i;
use crate::math::vector3::Vector3;
use crate::math::vector3i::Vector3i;
use crate::math::vector4::Vector4;
use crate::math::vector4i::Vector4i;
use crate::string::gstring::GoString;
use crate::string::num::num_float;
use crate::string::string_name::StringName;
use crate::variant::Variant;
use std::collections::HashMap;

const TYPE: &str = "type";
const ARGS: &str = "args";

fn f64v(value: f32) -> Variant {
    Variant::Float(value as f64)
}

fn args_object(type_name: &str, args: Vec<Variant>) -> Variant {
    let mut map: HashMap<StringName, Variant> = HashMap::new();
    map.insert(
        StringName::new(TYPE),
        Variant::String(GoString::from_str(type_name)),
    );
    map.insert(StringName::new(ARGS), Variant::Array(args));
    Variant::Dictionary(map)
}

fn type_only(type_name: &str) -> Variant {
    let mut map: HashMap<StringName, Variant> = HashMap::new();
    map.insert(
        StringName::new(TYPE),
        Variant::String(GoString::from_str(type_name)),
    );
    Variant::Dictionary(map)
}

fn string_of(value: &Variant) -> Option<String> {
    match value {
        Variant::String(s) => Some(s.as_string()),
        _ => None,
    }
}

fn args_of(value: &Variant) -> Option<Vec<Variant>> {
    match value {
        Variant::Dictionary(map) => match map.get(&StringName::new(ARGS)) {
            Some(Variant::Array(items)) => Some(items.clone()),
            _ => None,
        },
        _ => None,
    }
}

fn num_arg(args: &[Variant], index: usize) -> Option<f64> {
    args.get(index).map(|v| v.to_float())
}

fn int_arg(args: &[Variant], index: usize) -> Option<i32> {
    args.get(index).map(|v| v.to_int() as i32)
}

fn vec2(args: &[Variant]) -> Option<Vector2> {
    Some(Vector2::new(num_arg(args, 0)? as f32, num_arg(args, 1)? as f32))
}

fn vec2i(args: &[Variant]) -> Option<Vector2i> {
    Some(Vector2i::new(int_arg(args, 0)?, int_arg(args, 1)?))
}

fn vec3(args: &[Variant]) -> Option<Vector3> {
    Some(Vector3::new(
        num_arg(args, 0)? as f32,
        num_arg(args, 1)? as f32,
        num_arg(args, 2)? as f32,
    ))
}

fn vec3i(args: &[Variant]) -> Option<Vector3i> {
    Some(Vector3i::new(
        int_arg(args, 0)?,
        int_arg(args, 1)?,
        int_arg(args, 2)?,
    ))
}

fn vec4(args: &[Variant]) -> Option<Vector4> {
    Some(Vector4::new(
        num_arg(args, 0)? as f32,
        num_arg(args, 1)? as f32,
        num_arg(args, 2)? as f32,
        num_arg(args, 3)? as f32,
    ))
}

fn vec4i(args: &[Variant]) -> Option<Vector4i> {
    Some(Vector4i::new(
        int_arg(args, 0)?,
        int_arg(args, 1)?,
        int_arg(args, 2)?,
        int_arg(args, 3)?,
    ))
}

/// `JSON::from_native` for the supported subset.
pub fn from_native(value: &Variant) -> Variant {
    match value {
        Variant::Nil | Variant::Bool(_) => value.clone(),
        Variant::Int(i) => Variant::String(GoString::from_str(&format!("i:{i}"))),
        Variant::Float(f) => Variant::String(GoString::from_str(&format!(
            "f:{}",
            num_float(*f, -1).as_string()
        ))),
        Variant::String(s) => Variant::String(GoString::from_str(&format!("s:{}", s.as_string()))),
        Variant::Vector2(v) => args_object("Vector2", vec![f64v(v.x), f64v(v.y)]),
        Variant::Vector2i(v) => args_object(
            "Vector2i",
            vec![Variant::Int(v.x as i64), Variant::Int(v.y as i64)],
        ),
        Variant::Rect2(r) => args_object(
            "Rect2",
            vec![
                f64v(r.position.x),
                f64v(r.position.y),
                f64v(r.size.x),
                f64v(r.size.y),
            ],
        ),
        Variant::Rect2i(r) => args_object(
            "Rect2i",
            vec![
                Variant::Int(r.position.x as i64),
                Variant::Int(r.position.y as i64),
                Variant::Int(r.size.x as i64),
                Variant::Int(r.size.y as i64),
            ],
        ),
        Variant::Vector3(v) => args_object("Vector3", vec![f64v(v.x), f64v(v.y), f64v(v.z)]),
        Variant::Vector3i(v) => args_object(
            "Vector3i",
            vec![
                Variant::Int(v.x as i64),
                Variant::Int(v.y as i64),
                Variant::Int(v.z as i64),
            ],
        ),
        Variant::Transform2D(t) => args_object(
            "Transform2D",
            vec![
                f64v(t.columns[0].x),
                f64v(t.columns[0].y),
                f64v(t.columns[1].x),
                f64v(t.columns[1].y),
                f64v(t.columns[2].x),
                f64v(t.columns[2].y),
            ],
        ),
        Variant::Vector4(v) => args_object("Vector4", vec![f64v(v.x), f64v(v.y), f64v(v.z), f64v(v.w)]),
        Variant::Vector4i(v) => args_object(
            "Vector4i",
            vec![
                Variant::Int(v.x as i64),
                Variant::Int(v.y as i64),
                Variant::Int(v.z as i64),
                Variant::Int(v.w as i64),
            ],
        ),
        Variant::Plane(p) => args_object(
            "Plane",
            vec![f64v(p.normal.x), f64v(p.normal.y), f64v(p.normal.z), f64v(p.d)],
        ),
        Variant::Quaternion(q) => {
            args_object("Quaternion", vec![f64v(q.x), f64v(q.y), f64v(q.z), f64v(q.w)])
        }
        Variant::Aabb(a) => args_object(
            "AABB",
            vec![
                f64v(a.position.x),
                f64v(a.position.y),
                f64v(a.position.z),
                f64v(a.size.x),
                f64v(a.size.y),
                f64v(a.size.z),
            ],
        ),
        Variant::Basis(b) => {
            let mut args = Vec::with_capacity(9);
            for column in 0..3 {
                let c = b.get_column(column);
                args.push(f64v(c.x));
                args.push(f64v(c.y));
                args.push(f64v(c.z));
            }
            args_object("Basis", args)
        }
        Variant::Transform3D(t) => {
            let mut args = Vec::with_capacity(12);
            for column in 0..3 {
                let c = t.basis.get_column(column);
                args.push(f64v(c.x));
                args.push(f64v(c.y));
                args.push(f64v(c.z));
            }
            args.push(f64v(t.origin.x));
            args.push(f64v(t.origin.y));
            args.push(f64v(t.origin.z));
            args_object("Transform3D", args)
        }
        Variant::Projection(p) => {
            let mut args = Vec::with_capacity(16);
            for column in &p.columns {
                args.push(f64v(column.x));
                args.push(f64v(column.y));
                args.push(f64v(column.z));
                args.push(f64v(column.w));
            }
            args_object("Projection", args)
        }
        Variant::Color(c) => args_object(
            "Color",
            vec![f64v(c.r), f64v(c.g), f64v(c.b), f64v(c.a)],
        ),
        Variant::PackedByteArray(values) => args_object(
            "PackedByteArray",
            values.iter().map(|v| Variant::Int(*v as i64)).collect(),
        ),
        Variant::PackedInt32Array(values) => args_object(
            "PackedInt32Array",
            values.iter().map(|v| Variant::Int(*v as i64)).collect(),
        ),
        Variant::PackedInt64Array(values) => args_object(
            "PackedInt64Array",
            values.iter().map(|v| Variant::Int(*v)).collect(),
        ),
        Variant::PackedFloat32Array(values) => args_object(
            "PackedFloat32Array",
            values.iter().map(|v| Variant::Float(*v as f64)).collect(),
        ),
        Variant::PackedFloat64Array(values) => {
            args_object("PackedFloat64Array", values.iter().map(|v| Variant::Float(*v)).collect())
        }
        Variant::PackedStringArray(values) => args_object(
            "PackedStringArray",
            values.iter().map(|v| Variant::String(v.clone())).collect(),
        ),
        Variant::PackedVector2Array(values) => {
            let mut args = Vec::with_capacity(values.len() * 2);
            for v in values {
                args.push(f64v(v.x));
                args.push(f64v(v.y));
            }
            args_object("PackedVector2Array", args)
        }
        Variant::PackedVector3Array(values) => {
            let mut args = Vec::with_capacity(values.len() * 3);
            for v in values {
                args.push(f64v(v.x));
                args.push(f64v(v.y));
                args.push(f64v(v.z));
            }
            args_object("PackedVector3Array", args)
        }
        Variant::PackedColorArray(values) => {
            let mut args = Vec::with_capacity(values.len() * 4);
            for v in values {
                args.push(f64v(v.r));
                args.push(f64v(v.g));
                args.push(f64v(v.b));
                args.push(f64v(v.a));
            }
            args_object("PackedColorArray", args)
        }
        Variant::PackedVector4Array(values) => {
            let mut args = Vec::with_capacity(values.len() * 4);
            for v in values {
                args.push(f64v(v.x));
                args.push(f64v(v.y));
                args.push(f64v(v.z));
                args.push(f64v(v.w));
            }
            args_object("PackedVector4Array", args)
        }
        Variant::Array(items) => {
            Variant::Array(items.iter().map(from_native).collect())
        }
        Variant::Dictionary(map) => {
            let mut args = Vec::with_capacity(map.len() * 2);
            for (key, value) in map {
                args.push(Variant::String(GoString::from_str(&format!(
                    "sn:{}",
                    key.as_string()
                ))));
                args.push(from_native(value));
            }
            args_object("Dictionary", args)
        }
        // RID/Callable/Signal encode as a type-only dictionary; other payloads
        // have no JSON representation here.
        Variant::Callable(_) => type_only("Callable"),
        Variant::Signal(_) => type_only("Signal"),
        other => type_only(variant_type_name(other)),
    }
}

fn variant_type_name(value: &Variant) -> &'static str {
    match value {
        Variant::Nil => "Nil",
        Variant::Bool(_) => "bool",
        Variant::Int(_) => "int",
        Variant::Float(_) => "float",
        Variant::String(_) => "String",
        Variant::Object(_) => "Object",
        Variant::PackedByteArray(_) => "PackedByteArray",
        Variant::PackedInt32Array(_) => "PackedInt32Array",
        Variant::PackedInt64Array(_) => "PackedInt64Array",
        Variant::PackedFloat32Array(_) => "PackedFloat32Array",
        Variant::PackedFloat64Array(_) => "PackedFloat64Array",
        Variant::PackedStringArray(_) => "PackedStringArray",
        Variant::PackedVector2Array(_) => "PackedVector2Array",
        Variant::PackedVector3Array(_) => "PackedVector3Array",
        Variant::PackedColorArray(_) => "PackedColorArray",
        Variant::PackedVector4Array(_) => "PackedVector4Array",
        _ => "Variant",
    }
}

/// `JSON::to_native` for the supported subset; malformed payloads become `Nil`.
pub fn to_native(value: &Variant) -> Variant {
    match value {
        Variant::String(s) => {
            let text = s.as_string();
            if let Some(rest) = text.strip_prefix("i:") {
                rest.parse::<i64>().map(Variant::Int).unwrap_or(Variant::Nil)
            } else if let Some(rest) = text.strip_prefix("f:") {
                match rest {
                    "inf" => Variant::Float(f64::INFINITY),
                    "-inf" => Variant::Float(f64::NEG_INFINITY),
                    "nan" => Variant::Float(f64::NAN),
                    _ => rest.parse::<f64>().map(Variant::Float).unwrap_or(Variant::Nil),
                }
            } else if let Some(rest) = text.strip_prefix("s:") {
                Variant::String(GoString::from_str(rest))
            } else if let Some(rest) = text.strip_prefix("sn:") {
                Variant::String(GoString::from_str(rest))
            } else if let Some(rest) = text.strip_prefix("np:") {
                Variant::String(GoString::from_str(rest))
            } else {
                Variant::Nil
            }
        }
        Variant::Array(items) => Variant::Array(items.iter().map(to_native).collect()),
        Variant::Dictionary(map) => {
            let Some(type_name) = map.get(&StringName::new(TYPE)).and_then(string_of) else {
                return Variant::Nil;
            };
            let args = match args_of(value) {
                Some(a) => a,
                None => {
                    // Type-only dictionaries (RID/Callable/Signal) decode to Nil.
                    return Variant::Nil;
                }
            };
            decode_args(&type_name, &args)
        }
        Variant::Nil | Variant::Bool(_) | Variant::Int(_) | Variant::Float(_) => value.clone(),
        _ => Variant::Nil,
    }
}

fn decode_args(type_name: &str, args: &[Variant]) -> Variant {
    match type_name {
        "Vector2" if args.len() == 2 => vec2(args).map(Variant::Vector2).unwrap_or(Variant::Nil),
        "Vector2i" if args.len() == 2 => vec2i(args).map(Variant::Vector2i).unwrap_or(Variant::Nil),
        "Rect2" if args.len() == 4 => vec2(args)
            .and_then(|pos| vec2(&args[2..]).map(|size| Rect2::new(pos, size)))
            .map(Variant::Rect2)
            .unwrap_or(Variant::Nil),
        "Rect2i" if args.len() == 4 => vec2i(args)
            .and_then(|pos| vec2i(&args[2..]).map(|size| Rect2i::new(pos, size)))
            .map(Variant::Rect2i)
            .unwrap_or(Variant::Nil),
        "Vector3" if args.len() == 3 => vec3(args).map(Variant::Vector3).unwrap_or(Variant::Nil),
        "Vector3i" if args.len() == 3 => vec3i(args).map(Variant::Vector3i).unwrap_or(Variant::Nil),
        "Transform2D" if args.len() == 6 => vec2(args)
            .and_then(|c0| {
                vec2(&args[2..]).and_then(|c1| {
                    vec2(&args[4..]).map(|c2| Transform2D { columns: [c0, c1, c2] })
                })
            })
            .map(Variant::Transform2D)
            .unwrap_or(Variant::Nil),
        "Vector4" if args.len() == 4 => vec4(args).map(Variant::Vector4).unwrap_or(Variant::Nil),
        "Vector4i" if args.len() == 4 => vec4i(args).map(Variant::Vector4i).unwrap_or(Variant::Nil),
        "Plane" if args.len() == 4 => vec3(args)
            .map(|n| Plane::new(n.x, n.y, n.z, num_arg(args, 3).unwrap_or(0.0) as f32))
            .map(Variant::Plane)
            .unwrap_or(Variant::Nil),
        "Quaternion" if args.len() == 4 => vec4(args)
            .map(|v| Quaternion::new(v.x, v.y, v.z, v.w))
            .map(Variant::Quaternion)
            .unwrap_or(Variant::Nil),
        "AABB" if args.len() == 6 => vec3(args)
            .and_then(|pos| vec3(&args[3..]).map(|size| Aabb::new(pos, size)))
            .map(Variant::Aabb)
            .unwrap_or(Variant::Nil),
        "Basis" if args.len() == 9 => {
            let mut b = Basis::IDENTITY;
            for column in 0..3 {
                match vec3(&args[column * 3..]) {
                    Some(v) => b.set_column(column, v),
                    None => return Variant::Nil,
                }
            }
            Variant::Basis(b)
        }
        "Transform3D" if args.len() == 12 => {
            let mut basis = Basis::IDENTITY;
            for column in 0..3 {
                match vec3(&args[column * 3..]) {
                    Some(v) => basis.set_column(column, v),
                    None => return Variant::Nil,
                }
            }
            match vec3(&args[9..]) {
                Some(origin) => Variant::Transform3D(Transform3D { basis, origin }),
                None => Variant::Nil,
            }
        }
        "Projection" if args.len() == 16 => {
            let mut p = Projection::default();
            for column in 0..4 {
                match vec4(&args[column * 4..]) {
                    Some(v) => p.columns[column] = v,
                    None => return Variant::Nil,
                }
            }
            Variant::Projection(p)
        }
        "Color" if args.len() == 4 => vec4(args)
            .map(|v| Color::new(v.x, v.y, v.z, v.w))
            .map(Variant::Color)
            .unwrap_or(Variant::Nil),
        "PackedByteArray" => Variant::PackedByteArray(
            args.iter().map(|v| v.to_int() as u8).collect(),
        ),
        "PackedInt32Array" => Variant::PackedInt32Array(
            args.iter().map(|v| v.to_int() as i32).collect(),
        ),
        "PackedInt64Array" => {
            Variant::PackedInt64Array(args.iter().map(|v| v.to_int()).collect())
        }
        "PackedFloat32Array" => Variant::PackedFloat32Array(
            args.iter().map(|v| v.to_float() as f32).collect(),
        ),
        "PackedFloat64Array" => {
            Variant::PackedFloat64Array(args.iter().map(|v| v.to_float()).collect())
        }
        "PackedStringArray" => {
            let mut values = Vec::with_capacity(args.len());
            for arg in args {
                match arg {
                    Variant::String(s) => values.push(s.clone()),
                    _ => return Variant::Nil,
                }
            }
            Variant::PackedStringArray(values)
        }
        "PackedVector2Array" if args.len() % 2 == 0 => {
            let mut values = Vec::with_capacity(args.len() / 2);
            for pair in args.chunks(2) {
                match vec2(pair) {
                    Some(v) => values.push(v),
                    None => return Variant::Nil,
                }
            }
            Variant::PackedVector2Array(values)
        }
        "PackedVector3Array" if args.len() % 3 == 0 => {
            let mut values = Vec::with_capacity(args.len() / 3);
            for triple in args.chunks(3) {
                match vec3(triple) {
                    Some(v) => values.push(v),
                    None => return Variant::Nil,
                }
            }
            Variant::PackedVector3Array(values)
        }
        "PackedColorArray" if args.len() % 4 == 0 => {
            let mut values = Vec::with_capacity(args.len() / 4);
            for quad in args.chunks(4) {
                match vec4(quad) {
                    Some(v) => values.push(Color::new(v.x, v.y, v.z, v.w)),
                    None => return Variant::Nil,
                }
            }
            Variant::PackedColorArray(values)
        }
        "PackedVector4Array" if args.len() % 4 == 0 => {
            let mut values = Vec::with_capacity(args.len() / 4);
            for quad in args.chunks(4) {
                match vec4(quad) {
                    Some(v) => values.push(v),
                    None => return Variant::Nil,
                }
            }
            Variant::PackedVector4Array(values)
        }
        "Dictionary" if args.len() % 2 == 0 => {
            let mut map: HashMap<StringName, Variant> = HashMap::new();
            for pair in args.chunks(2) {
                let key = to_native(&pair[0]);
                let Variant::String(key_text) = key else {
                    return Variant::Nil;
                };
                map.insert(StringName::new(&key_text.as_string()), to_native(&pair[1]));
            }
            Variant::Dictionary(map)
        }
        _ => Variant::Nil,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scalar_prefixes_round_trip() {
        assert_eq!(from_native(&Variant::Nil), Variant::Nil);
        assert_eq!(from_native(&Variant::Bool(true)), Variant::Bool(true));
        assert_eq!(
            from_native(&Variant::Int(-7)),
            Variant::String(GoString::from_str("i:-7"))
        );
        assert_eq!(
            from_native(&Variant::Float(1.5)),
            Variant::String(GoString::from_str("f:1.5"))
        );
        assert_eq!(
            from_native(&Variant::String(GoString::from_str("hi"))),
            Variant::String(GoString::from_str("s:hi"))
        );

        for value in [
            Variant::Int(42),
            Variant::Float(-2.25),
            Variant::String(GoString::from_str("text")),
            Variant::Bool(false),
        ] {
            assert_eq!(to_native(&from_native(&value)), value);
        }
        assert_eq!(to_native(&Variant::String(GoString::from_str("junk"))), Variant::Nil);
    }

    #[test]
    fn float_specials_round_trip() {
        let inf = from_native(&Variant::Float(f64::INFINITY));
        assert_eq!(to_native(&inf), Variant::Float(f64::INFINITY));
        let ninf = from_native(&Variant::Float(f64::NEG_INFINITY));
        assert_eq!(to_native(&ninf), Variant::Float(f64::NEG_INFINITY));
        match to_native(&from_native(&Variant::Float(f64::NAN))) {
            Variant::Float(f) => assert!(f.is_nan()),
            other => panic!("expected NaN, got {other:?}"),
        }
    }

    #[test]
    fn math_types_round_trip() {
        let values = vec![
            Variant::Vector2(Vector2::new(1.5, -2.0)),
            Variant::Vector2i(Vector2i::new(3, -4)),
            Variant::Rect2(Rect2::new(Vector2::new(1.0, 2.0), Vector2::new(3.0, 4.0))),
            Variant::Rect2i(Rect2i::new_xywh(1, 2, 3, 4)),
            Variant::Vector3(Vector3::new(1.0, 2.0, 3.0)),
            Variant::Vector3i(Vector3i::new(1, 2, 3)),
            Variant::Transform2D(Transform2D {
                columns: [
                    Vector2::new(1.0, 2.0),
                    Vector2::new(3.0, 4.0),
                    Vector2::new(5.0, 6.0),
                ],
            }),
            Variant::Vector4(Vector4::new(1.0, 2.0, 3.0, 4.0)),
            Variant::Vector4i(Vector4i::new(1, 2, 3, 4)),
            Variant::Plane(Plane::new(0.0, 1.0, 0.0, -2.0)),
            Variant::Quaternion(Quaternion::new(0.1, 0.2, 0.3, 0.9)),
            Variant::Aabb(Aabb::new(Vector3::new(1.0, 2.0, 3.0), Vector3::new(4.0, 5.0, 6.0))),
            Variant::Basis(Basis::from_euler(
                Vector3::new(0.1, 0.2, 0.3),
                crate::math::basis::EulerOrder::YXZ,
            )),
            Variant::Transform3D(Transform3D {
                basis: Basis::from_axis_angle(Vector3::UP, 0.5),
                origin: Vector3::new(1.0, 2.0, 3.0),
            }),
            Variant::Color(Color::new(0.1, 0.2, 0.3, 0.4)),
        ];

        for value in values {
            let native = to_native(&from_native(&value));
            match (&value, &native) {
                (Variant::Plane(a), Variant::Plane(b)) => assert!(a.is_equal_approx(b), "{value:?}"),
                (Variant::Quaternion(a), Variant::Quaternion(b)) => {
                    assert!(a.is_equal_approx(b), "{value:?}")
                }
                (Variant::Transform2D(a), Variant::Transform2D(b)) => {
                    for i in 0..3 {
                        assert!((a.columns[i] - b.columns[i]).length() < 1e-5);
                    }
                }
                (Variant::Basis(a), Variant::Basis(b)) => assert!(a.is_equal_approx(b)),
                (Variant::Transform3D(a), Variant::Transform3D(b)) => {
                    assert!(a.basis.is_equal_approx(&b.basis));
                    assert!((a.origin - b.origin).length() < 1e-5);
                }
                _ => assert_eq!(&native, &value, "{value:?}"),
            }
        }
    }

    #[test]
    fn arrays_and_dictionaries_recurse() {
        let mut map: HashMap<StringName, Variant> = HashMap::new();
        map.insert(StringName::new("n"), Variant::Int(3));
        map.insert(
            StringName::new("v"),
            Variant::Vector2(Vector2::new(1.0, 2.0)),
        );
        let value = Variant::Array(vec![
            Variant::Dictionary(map),
            Variant::String(GoString::from_str("x")),
        ]);

        let native = to_native(&from_native(&value));
        let Variant::Array(items) = native else {
            panic!("not an array")
        };
        assert_eq!(items[1], Variant::String(GoString::from_str("x")));
        let Variant::Dictionary(dict) = &items[0] else {
            panic!("not a dictionary")
        };
        assert_eq!(dict.get(&StringName::new("n")), Some(&Variant::Int(3)));
        assert_eq!(
            dict.get(&StringName::new("v")),
            Some(&Variant::Vector2(Vector2::new(1.0, 2.0)))
        );
    }

    #[test]
    fn packed_arrays_round_trip() {
        let values = vec![
            Variant::PackedByteArray(vec![0, 1, 255]),
            Variant::PackedInt32Array(vec![-3, 0, 7]),
            Variant::PackedInt64Array(vec![-9_000_000_000, 1]),
            Variant::PackedFloat32Array(vec![1.5, -2.25]),
            Variant::PackedFloat64Array(vec![0.1, 1e300]),
            Variant::PackedStringArray(vec![
                GoString::from_str("a"),
                GoString::from_str("b"),
            ]),
            Variant::PackedVector2Array(vec![Vector2::new(1.0, 2.0), Vector2::new(-3.0, 0.5)]),
            Variant::PackedVector3Array(vec![Vector3::new(1.0, 2.0, 3.0)]),
            Variant::PackedColorArray(vec![Color::new(0.1, 0.2, 0.3, 0.4)]),
            Variant::PackedVector4Array(vec![Vector4::new(1.0, 2.0, 3.0, 4.0)]),
        ];

        for value in values {
            assert_eq!(to_native(&from_native(&value)), value);
        }
    }

    #[test]
    fn packed_array_size_mismatches_become_nil() {
        // Vector2 arrays need an even argument count.
        let bad = args_object(
            "PackedVector2Array",
            vec![Variant::Float(1.0), Variant::Float(2.0), Variant::Float(3.0)],
        );
        assert_eq!(to_native(&bad), Variant::Nil);
        // String arrays need string elements.
        let bad = args_object("PackedStringArray", vec![Variant::Int(1)]);
        assert_eq!(to_native(&bad), Variant::Nil);
    }

    #[test]
    fn malformed_payloads_become_nil() {
        // Type-only dictionary (the engine's RID/Callable/Signal shape).
        let mut rid: HashMap<StringName, Variant> = HashMap::new();
        rid.insert(
            StringName::new(TYPE),
            Variant::String(GoString::from_str("RID")),
        );
        assert_eq!(to_native(&Variant::Dictionary(rid)), Variant::Nil);

        // Wrong arg count.
        let bad = args_object("Vector2", vec![Variant::Float(1.0)]);
        assert_eq!(to_native(&bad), Variant::Nil);

        // Unknown type.
        let unknown = args_object("DoesNotExist", vec![]);
        assert_eq!(to_native(&unknown), Variant::Nil);

        // Dictionary without a type.
        let mut plain: HashMap<StringName, Variant> = HashMap::new();
        plain.insert(StringName::new("a"), Variant::Int(1));
        assert_eq!(to_native(&Variant::Dictionary(plain)), Variant::Nil);
    }
}
