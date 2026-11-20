//! `core/variant/variant_op.cpp` comparisons and conversions (subset).
//!
//! Implemented: numeric cross-type comparison (Int/Float), Bool, String,
//! Vector2/Vector3 lexicographic ordering, equality for the POD payloads,
//! `can_convert`/`convert` for the numeric/bool/string subset, and
//! `stringify`/`get_index` (Array by int, Dictionary by any key).
//!
//! Not implemented: Object/Callable/Signal comparisons, PackedArray element
//! conversions, and the full engine conversion matrix.
//!
//! Non-differential: `variant_op.cpp` includes the object system, which the
//! parity harness cannot link.

use crate::string::gstring::GoString;
use crate::variant::{Variant, VariantType};
use std::cmp::Ordering;

/// Strict ordering between two variants, or `None` when the engine would report
/// an invalid operand pair.
pub fn compare(lhs: &Variant, rhs: &Variant) -> Option<Ordering> {
    use Variant::*;
    match (lhs, rhs) {
        (Int(a), Int(b)) => Some(a.cmp(b)),
        (Float(a), Float(b)) => a.partial_cmp(b),
        (Int(a), Float(b)) => (*a as f64).partial_cmp(b),
        (Float(a), Int(b)) => a.partial_cmp(&(*b as f64)),
        (Bool(a), Bool(b)) => Some(a.cmp(b)),
        (String(a), String(b)) => Some(cmp_gstring(a, b)),
        (Vector2(a), Vector2(b)) => Some(
            a.x.partial_cmp(&b.x)
                .unwrap_or(Ordering::Equal)
                .then(a.y.partial_cmp(&b.y).unwrap_or(Ordering::Equal)),
        ),
        (Vector3(a), Vector3(b)) => Some(
            a.x.partial_cmp(&b.x)
                .unwrap_or(Ordering::Equal)
                .then(a.y.partial_cmp(&b.y).unwrap_or(Ordering::Equal))
                .then(a.z.partial_cmp(&b.z).unwrap_or(Ordering::Equal)),
        ),
        _ => None,
    }
}

/// Lexicographic comparison over the UTF-32 units, then by length.
fn cmp_gstring(a: &GoString, b: &GoString) -> Ordering {
    let au = a.as_string();
    let bu = b.as_string();
    match au.as_bytes().cmp(bu.as_bytes()) {
        Ordering::Equal => au.len().cmp(&bu.len()),
        other => other,
    }
}

/// `Variant::operator==` for the supported payloads. `None` means the engine
/// would reject the comparison (different types).
pub fn equals(lhs: &Variant, rhs: &Variant) -> Option<bool> {
    use Variant::*;
    match (lhs, rhs) {
        (Nil, Nil) => Some(true),
        (Nil, _) | (_, Nil) => Some(false),
        (Int(a), Int(b)) => Some(a == b),
        (Float(a), Float(b)) => Some(a == b),
        (Int(a), Float(b)) => Some(*a as f64 == *b),
        (Float(a), Int(b)) => Some(*a == *b as f64),
        (Bool(a), Bool(b)) => Some(a == b),
        (String(a), String(b)) => Some(a.as_string() == b.as_string()),
        (Vector2(a), Vector2(b)) => Some(a == b),
        (Vector3(a), Vector3(b)) => Some(a == b),
        (Vector4(a), Vector4(b)) => Some(a == b),
        (Color(a), Color(b)) => Some(a == b),
        (Array(a), Array(b)) => Some(
            a.len() == b.len()
                && a.iter()
                    .zip(b.iter())
                    .all(|(x, y)| equals(x, y) == Some(true)),
        ),
        _ => None,
    }
}

/// The subset of `Variant::can_convert` that this port implements.
pub fn can_convert(from: VariantType, to: VariantType) -> bool {
    use VariantType::*;
    if from == to {
        return true;
    }
    matches!(
        (from, to),
        (Int, Float)
            | (Float, Int)
            | (Bool, Int)
            | (Bool, Float)
            | (Int, Bool)
            | (Float, Bool)
            | (Int, String)
            | (Float, String)
            | (Bool, String)
            | (String, Int)
            | (String, Float)
            | (String, Bool)
            | (Nil, String)
    )
}

/// Conversion for the pairs accepted by `can_convert`.
pub fn convert(value: &Variant, to: VariantType) -> Option<Variant> {
    use VariantType::*;
    if !can_convert(value.vtype(), to) {
        return None;
    }
    Some(match to {
        Int => Variant::Int(value.to_int()),
        Float => Variant::Float(value.to_float()),
        Bool => Variant::Bool(value.booleanize()),
        String => Variant::String(stringify(value)),
        _ => return None,
    })
}

/// `Variant::stringify` for the scalar payloads.
pub fn stringify(value: &Variant) -> GoString {
    match value {
        Variant::Nil => GoString::from_str("null"),
        Variant::Bool(b) => GoString::from_str(if *b { "true" } else { "false" }),
        Variant::Int(i) => GoString::from_str(&i.to_string()),
        Variant::Float(f) => crate::string::num::num_float(*f, -1),
        Variant::String(s) => s.clone(),
        Variant::Vector2(v) => GoString::from_str(&format!(
            "({}, {})",
            crate::string::num::num_float(v.x as f64, -1).as_string(),
            crate::string::num::num_float(v.y as f64, -1).as_string()
        )),
        Variant::Vector2i(v) => GoString::from_str(&format!("({}, {})", v.x, v.y)),
        Variant::Vector3(v) => GoString::from_str(&format!(
            "({}, {}, {})",
            crate::string::num::num_float(v.x as f64, -1).as_string(),
            crate::string::num::num_float(v.y as f64, -1).as_string(),
            crate::string::num::num_float(v.z as f64, -1).as_string()
        )),
        Variant::Vector3i(v) => GoString::from_str(&format!("({}, {}, {})", v.x, v.y, v.z)),
        Variant::Color(c) => GoString::from_str(&format!(
            "({}, {}, {}, {})",
            crate::string::num::num_float(c.r as f64, -1).as_string(),
            crate::string::num::num_float(c.g as f64, -1).as_string(),
            crate::string::num::num_float(c.b as f64, -1).as_string(),
            crate::string::num::num_float(c.a as f64, -1).as_string()
        )),
        _ => GoString::new(),
    }
}

/// `Variant::get`: Array by integer index, Dictionary by key.
pub fn get_index(container: &Variant, key: &Variant) -> Option<Variant> {
    match container {
        Variant::Array(items) => {
            let i = key.to_int();
            if i < 0 || i as usize >= items.len() {
                // The engine returns a null Variant for out-of-bounds reads in
                // the expression evaluator's indexing path.
                return Some(Variant::Nil);
            }
            Some(items[i as usize].clone())
        }
        Variant::Dictionary(entries) => {
            let name = match key {
                Variant::String(s) => crate::string::string_name::StringName::new(&s.as_string()),
                _ => return Some(Variant::Nil),
            };
            entries.get(&name).cloned().or(Some(Variant::Nil))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::vector2::Vector2;
    use crate::math::vector3::Vector3;

    #[test]
    fn numeric_cross_type_comparison() {
        assert_eq!(compare(&Variant::Int(2), &Variant::Float(2.5)), Some(Ordering::Less));
        assert_eq!(
            compare(&Variant::Float(3.5), &Variant::Int(3)),
            Some(Ordering::Greater)
        );
        assert_eq!(compare(&Variant::Int(4), &Variant::Int(4)), Some(Ordering::Equal));
        // NaN does not compare.
        assert_eq!(compare(&Variant::Float(f64::NAN), &Variant::Float(0.0)), None);
    }

    #[test]
    fn strings_compare_lexicographically() {
        let a = Variant::String(GoString::from_str("abc"));
        let b = Variant::String(GoString::from_str("abd"));
        let c = Variant::String(GoString::from_str("ab"));
        assert_eq!(compare(&a, &b), Some(Ordering::Less));
        assert_eq!(compare(&a, &c), Some(Ordering::Greater));
        assert_eq!(compare(&c, &c), Some(Ordering::Equal));
    }

    #[test]
    fn vectors_compare_component_wise() {
        let a = Variant::Vector2(Vector2::new(1.0, 5.0));
        let b = Variant::Vector2(Vector2::new(2.0, 0.0));
        assert_eq!(compare(&a, &b), Some(Ordering::Less));

        let c = Variant::Vector3(Vector3::new(1.0, 1.0, 9.0));
        let d = Variant::Vector3(Vector3::new(1.0, 1.0, 2.0));
        assert_eq!(compare(&c, &d), Some(Ordering::Greater));
    }

    #[test]
    fn invalid_comparisons() {
        assert_eq!(compare(&Variant::Int(1), &Variant::Bool(true)), None);
        assert_eq!(equals(&Variant::Int(1), &Variant::Bool(true)), None);
        // Nil equals only nil.
        assert_eq!(equals(&Variant::Nil, &Variant::Nil), Some(true));
        assert_eq!(equals(&Variant::Nil, &Variant::Int(0)), Some(false));
    }

    #[test]
    fn equality_and_mixed_numeric_equality() {
        assert_eq!(equals(&Variant::Int(2), &Variant::Float(2.0)), Some(true));
        assert_eq!(equals(&Variant::Int(2), &Variant::Float(2.5)), Some(false));
        assert_eq!(
            equals(
                &Variant::String(GoString::from_str("x")),
                &Variant::String(GoString::from_str("x"))
            ),
            Some(true)
        );
    }

    #[test]
    fn conversion_matrix_and_values() {
        assert!(can_convert(VariantType::Int, VariantType::Float));
        assert!(can_convert(VariantType::String, VariantType::Int));
        assert!(!can_convert(VariantType::Nil, VariantType::Int));
        assert!(!can_convert(VariantType::Vector3, VariantType::String));

        assert_eq!(
            convert(&Variant::Int(3), VariantType::Float),
            Some(Variant::Float(3.0))
        );
        assert_eq!(
            convert(&Variant::Float(3.9), VariantType::Int),
            Some(Variant::Int(3))
        );
        assert_eq!(
            convert(&Variant::String(GoString::from_str("42")), VariantType::Int),
            Some(Variant::Int(42))
        );
        assert_eq!(
            convert(&Variant::Int(0), VariantType::Bool),
            Some(Variant::Bool(false))
        );
        assert_eq!(
            convert(&Variant::Bool(true), VariantType::String),
            Some(Variant::String(GoString::from_str("true")))
        );
        assert_eq!(convert(&Variant::Vector3(Vector3::ONE), VariantType::Int), None);
    }

    #[test]
    fn stringify_scalars() {
        assert_eq!(stringify(&Variant::Nil).as_string(), "null");
        assert_eq!(stringify(&Variant::Bool(false)).as_string(), "false");
        assert_eq!(stringify(&Variant::Int(-12)).as_string(), "-12");
        // Trailing zeros trimmed, like `String::num_real`.
        assert_eq!(stringify(&Variant::Float(1.5)).as_string(), "1.5");
        assert_eq!(stringify(&Variant::Float(2.0)).as_string(), "2");
    }

    #[test]
    fn indexing_arrays_and_dictionaries() {
        let arr = Variant::Array(vec![
            Variant::Int(10),
            Variant::Int(20),
            Variant::Int(30),
        ]);
        assert_eq!(get_index(&arr, &Variant::Int(1)), Some(Variant::Int(20)));
        // Out of bounds reads yield nil rather than failing to evaluate.
        assert_eq!(get_index(&arr, &Variant::Int(5)), Some(Variant::Nil));
        assert_eq!(get_index(&arr, &Variant::Int(-1)), Some(Variant::Nil));

        let mut map = std::collections::HashMap::new();
        map.insert(crate::string::string_name::StringName::new("a"), Variant::Int(1));
        map.insert(crate::string::string_name::StringName::new("b"), Variant::Int(2));
        let dict = Variant::Dictionary(map);
        assert_eq!(
            get_index(&dict, &Variant::String(GoString::from_str("b"))),
            Some(Variant::Int(2))
        );
        assert_eq!(
            get_index(&dict, &Variant::String(GoString::from_str("z"))),
            Some(Variant::Nil)
        );

        assert_eq!(get_index(&Variant::Int(1), &Variant::Int(0)), None);
    }
}
