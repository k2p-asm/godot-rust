//! First slice of `core/variant`: the type tag and a payload-carrying value.
//!
//! The discriminants mirror `Variant::Type` in `core/variant/variant.h`
//! **exactly** (NIL = 0, BOOL = 1, ...), because the C ABI and the
//! `variant_op` table depend on that order. They are checked differentially by
//! `parity/variant_dump.cpp`, which prints the engine's own enum values.
//!
//! Only the POD payloads are implemented so far. `Object`, `Callable`,
//! `Signal`, `Dictionary`, `Array` and friends need the object system and are
//! deliberately absent.

pub mod compare;
pub mod ops;
pub mod serial;
pub mod utility;

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
use crate::string::string_name::StringName;

/// `Variant::Type`, in the engine's declaration order. Values after `SIGNAL`
/// exist in C++ but are not modelled here yet.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum VariantType {
    Nil = 0,
    Bool = 1,
    Int = 2,
    Float = 3,
    String = 4,
    Vector2 = 5,
    Vector2i = 6,
    Rect2 = 7,
    Rect2i = 8,
    Vector3 = 9,
    Vector3i = 10,
    Transform2D = 11,
    Vector4 = 12,
    Vector4i = 13,
    Plane = 14,
    Quaternion = 15,
    Aabb = 16,
    Basis = 17,
    Transform3D = 18,
    Projection = 19,
    Color = 20,
    StringName = 21,
    NodePath = 22,
    Rid = 23,
    Object = 24,
    Callable = 25,
    Signal = 26,
    /// Container tags; the engine declares Dictionary before Array.
    Dictionary = 27,
    Array = 28,
    // Typed arrays (tags only: the port has no packed-array payloads yet).
    PackedByteArray = 29,
    PackedInt32Array = 30,
    PackedInt64Array = 31,
    PackedFloat32Array = 32,
    PackedFloat64Array = 33,
    PackedStringArray = 34,
    PackedVector2Array = 35,
    PackedVector3Array = 36,
    PackedColorArray = 37,
    PackedVector4Array = 38,
}

/// A `Variant` restricted to the POD payloads the port can represent.
#[derive(Clone, Debug, PartialEq)]
pub enum Variant {
    Nil,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(GoString),
    Vector2(Vector2),
    Vector2i(Vector2i),
    Rect2(Rect2),
    Rect2i(Rect2i),
    Vector3(Vector3),
    Vector3i(Vector3i),
    Transform2D(Transform2D),
    Vector4(Vector4),
    Vector4i(Vector4i),
    Plane(Plane),
    Quaternion(Quaternion),
    Aabb(Aabb),
    Basis(Basis),
    Transform3D(Transform3D),
    Projection(Projection),
    Color(Color),
    /// Identity handle; the object itself lives in `crate::object`.
    Object(crate::object::ObjectId),
    /// Target object + method name. `ObjectId::NONE` means "unbound".
    Callable(Callable),
    /// Target object + signal name.
    Signal(Signal),
    /// Key/value container; keys are interned names, as in `Variant`.
    Dictionary(std::collections::HashMap<StringName, Variant>),
    /// Ordered container.
    Array(Vec<Variant>),
    // Typed arrays: same tag order as the engine (29..=38).
    PackedByteArray(Vec<u8>),
    PackedInt32Array(Vec<i32>),
    PackedInt64Array(Vec<i64>),
    PackedFloat32Array(Vec<f32>),
    PackedFloat64Array(Vec<f64>),
    PackedStringArray(Vec<GoString>),
    PackedVector2Array(Vec<Vector2>),
    PackedVector3Array(Vec<Vector3>),
    PackedColorArray(Vec<Color>),
    PackedVector4Array(Vec<Vector4>),
}

/// `Callable`: an object and a method name on it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Callable {
    pub object: crate::object::ObjectId,
    pub method: StringName,
}

impl Callable {
    #[inline]
    pub fn is_null(&self) -> bool {
        self.object.is_null()
    }

    #[inline]
    pub fn is_valid(&self) -> bool {
        !self.object.is_null() && self.object.is_valid()
    }
}

/// `Signal`: an object and one of its signals.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Signal {
    pub object: crate::object::ObjectId,
    pub name: StringName,
}

impl Signal {
    #[inline]
    pub fn is_null(&self) -> bool {
        self.object.is_null()
    }
}

impl Variant {
    #[inline]
    pub fn vtype(&self) -> VariantType {
        match self {
            Variant::Nil => VariantType::Nil,
            Variant::Bool(_) => VariantType::Bool,
            Variant::Int(_) => VariantType::Int,
            Variant::Float(_) => VariantType::Float,
            Variant::String(_) => VariantType::String,
            Variant::Vector2(_) => VariantType::Vector2,
            Variant::Vector2i(_) => VariantType::Vector2i,
            Variant::Rect2(_) => VariantType::Rect2,
            Variant::Rect2i(_) => VariantType::Rect2i,
            Variant::Vector3(_) => VariantType::Vector3,
            Variant::Vector3i(_) => VariantType::Vector3i,
            Variant::Transform2D(_) => VariantType::Transform2D,
            Variant::Vector4(_) => VariantType::Vector4,
            Variant::Vector4i(_) => VariantType::Vector4i,
            Variant::Plane(_) => VariantType::Plane,
            Variant::Quaternion(_) => VariantType::Quaternion,
            Variant::Aabb(_) => VariantType::Aabb,
            Variant::Basis(_) => VariantType::Basis,
            Variant::Transform3D(_) => VariantType::Transform3D,
            Variant::Projection(_) => VariantType::Projection,
            Variant::Color(_) => VariantType::Color,
            Variant::Object(_) => VariantType::Object,
            Variant::Callable(_) => VariantType::Callable,
            Variant::Signal(_) => VariantType::Signal,
            Variant::Dictionary(_) => VariantType::Dictionary,
            Variant::Array(_) => VariantType::Array,
            Variant::PackedByteArray(_) => VariantType::PackedByteArray,
            Variant::PackedInt32Array(_) => VariantType::PackedInt32Array,
            Variant::PackedInt64Array(_) => VariantType::PackedInt64Array,
            Variant::PackedFloat32Array(_) => VariantType::PackedFloat32Array,
            Variant::PackedFloat64Array(_) => VariantType::PackedFloat64Array,
            Variant::PackedStringArray(_) => VariantType::PackedStringArray,
            Variant::PackedVector2Array(_) => VariantType::PackedVector2Array,
            Variant::PackedVector3Array(_) => VariantType::PackedVector3Array,
            Variant::PackedColorArray(_) => VariantType::PackedColorArray,
            Variant::PackedVector4Array(_) => VariantType::PackedVector4Array,
        }
    }

    #[inline]
    pub fn is_nil(&self) -> bool {
        matches!(self, Variant::Nil)
    }

    /// `Variant::is_ref_counted`, restricted: none of these payloads are.
    #[inline]
    pub fn is_ref_counted(&self) -> bool {
        false
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Variant::Bool(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_int(&self) -> Option<i64> {
        match self {
            Variant::Int(i) => Some(*i),
            _ => None,
        }
    }

    pub fn as_float(&self) -> Option<f64> {
        match self {
            Variant::Float(f) => Some(*f),
            _ => None,
        }
    }

    pub fn as_vector3(&self) -> Option<Vector3> {
        match self {
            Variant::Vector3(v) => Some(*v),
            _ => None,
        }
    }

    /// Truthiness, as `Variant::booleanize()` defines it.
    pub fn booleanize(&self) -> bool {
        match self {
            Variant::Nil => false,
            Variant::Bool(b) => *b,
            Variant::Int(i) => *i != 0,
            Variant::Float(f) => *f != 0.0,
            Variant::String(s) => !s.is_empty(),
            Variant::Vector2(v) => !v.is_zero_approx(),
            Variant::Vector2i(v) => *v != Vector2i::ZERO,
            Variant::Vector3(v) => !v.is_zero_approx(),
            Variant::Vector3i(v) => *v != Vector3i::ZERO,
            Variant::Vector4(v) => !v.is_zero_approx(),
            Variant::Vector4i(v) => *v != Vector4i::ZERO,
            Variant::Rect2(v) => v.has_area(),
            Variant::Rect2i(v) => v.has_area(),
            Variant::Color(c) => !c.is_equal_approx(&Color::new(0.0, 0.0, 0.0, 0.0)),
            // Compound types are truthy when not equal to their zero value.
            Variant::Plane(p) => !p.is_finite() || p.normal != Vector3::ZERO || p.d != 0.0,
            // A null or stale handle is falsy, like a null Object pointer.
            Variant::Object(id) => !id.is_null() && id.is_valid(),
            // A callable is truthy while its target is alive.
            Variant::Callable(c) => c.is_valid(),
            Variant::Signal(s) => !s.is_null() && s.object.is_valid(),
            // Containers are falsy only when empty.
            Variant::Dictionary(d) => !d.is_empty(),
            Variant::Array(a) => !a.is_empty(),
            // Typed arrays follow the same rule: falsy only when empty.
            Variant::PackedByteArray(a) => !a.is_empty(),
            Variant::PackedInt32Array(a) => !a.is_empty(),
            Variant::PackedInt64Array(a) => !a.is_empty(),
            Variant::PackedFloat32Array(a) => !a.is_empty(),
            Variant::PackedFloat64Array(a) => !a.is_empty(),
            Variant::PackedStringArray(a) => !a.is_empty(),
            Variant::PackedVector2Array(a) => !a.is_empty(),
            Variant::PackedVector3Array(a) => !a.is_empty(),
            Variant::PackedColorArray(a) => !a.is_empty(),
            Variant::PackedVector4Array(a) => !a.is_empty(),
            _ => true,
        }
    }

    // -- container access ---------------------------------------------------

    /// Dictionary lookup; `None` for a missing key or a non-dictionary.
    pub fn dict_get(&self, key: &str) -> Option<&Variant> {
        match self {
            Variant::Dictionary(d) => d.get(&StringName::new(key)),
            _ => None,
        }
    }

    /// Dictionary insert; returns false for a non-dictionary.
    pub fn dict_set(&mut self, key: &str, value: Variant) -> bool {
        match self {
            Variant::Dictionary(d) => {
                d.insert(StringName::new(key), value);
                true
            }
            _ => false,
        }
    }

    pub fn dict_len(&self) -> usize {
        match self {
            Variant::Dictionary(d) => d.len(),
            _ => 0,
        }
    }

    /// Keys in insertion-independent (hash) order, like `Dictionary.keys()`.
    pub fn dict_keys(&self) -> Vec<StringName> {
        match self {
            Variant::Dictionary(d) => d.keys().copied().collect(),
            _ => Vec::new(),
        }
    }

    /// Keys sorted by text; useful for deterministic tests and diagnostics.
    pub fn dict_keys_sorted(&self) -> Vec<StringName> {
        let mut keys = self.dict_keys();
        keys.sort_by_key(|k| k.as_string());
        keys
    }

    #[inline]
    pub fn array_len(&self) -> usize {
        match self {
            Variant::Array(a) => a.len(),
            _ => 0,
        }
    }

    pub fn array_get(&self, idx: usize) -> Option<&Variant> {
        match self {
            Variant::Array(a) => a.get(idx),
            _ => None,
        }
    }

    /// `Array::push_back`; returns false for a non-array.
    pub fn array_push(&mut self, value: Variant) -> bool {
        match self {
            Variant::Array(a) => {
                a.push(value);
                true
            }
            _ => false,
        }
    }

    /// Length of any packed array; 0 for other payloads.
    pub fn packed_len(&self) -> usize {
        match self {
            Variant::PackedByteArray(a) => a.len(),
            Variant::PackedInt32Array(a) => a.len(),
            Variant::PackedInt64Array(a) => a.len(),
            Variant::PackedFloat32Array(a) => a.len(),
            Variant::PackedFloat64Array(a) => a.len(),
            Variant::PackedStringArray(a) => a.len(),
            Variant::PackedVector2Array(a) => a.len(),
            Variant::PackedVector3Array(a) => a.len(),
            Variant::PackedColorArray(a) => a.len(),
            Variant::PackedVector4Array(a) => a.len(),
            _ => 0,
        }
    }
    /// Numeric coercion used by `Variant::operator int()`.
    pub fn to_int(&self) -> i64 {
        match self {
            Variant::Nil => 0,
            Variant::Bool(b) => *b as i64,
            Variant::Int(i) => *i,
            Variant::Float(f) => *f as i64,
            Variant::String(s) => s.as_string().trim().parse().unwrap_or(0),
            _ => 0,
        }
    }

    /// Numeric coercion used by `Variant::operator double()`.
    pub fn to_float(&self) -> f64 {
        match self {
            Variant::Nil => 0.0,
            Variant::Bool(b) => *b as i32 as f64,
            Variant::Int(i) => *i as f64,
            Variant::Float(f) => *f,
            Variant::String(s) => s.as_string().trim().parse().unwrap_or(0.0),
            _ => 0.0,
        }
    }
}

impl Default for Variant {
    fn default() -> Self {
        Variant::Nil
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn type_tags_match_engine_order() {
        assert_eq!(VariantType::Nil as u32, 0);
        assert_eq!(VariantType::Bool as u32, 1);
        assert_eq!(VariantType::String as u32, 4);
        assert_eq!(VariantType::Vector3 as u32, 9);
        assert_eq!(VariantType::Transform3D as u32, 18);
        assert_eq!(VariantType::Color as u32, 20);
        assert_eq!(VariantType::Signal as u32, 26);
    }

    #[test]
    fn typed_accessors_reject_mismatch() {
        let v = Variant::Vector3(Vector3::new(1.0, 2.0, 3.0));
        assert_eq!(v.vtype(), VariantType::Vector3);
        assert_eq!(v.as_vector3(), Some(Vector3::new(1.0, 2.0, 3.0)));
        assert_eq!(v.as_int(), None);
        assert!(!v.is_nil());
        assert!(!v.is_ref_counted());
    }

    #[test]
    fn booleanize_and_coercion() {
        assert!(!Variant::Nil.booleanize());
        assert!(!Variant::Bool(false).booleanize());
        assert!(Variant::Int(0).booleanize() == false);
        assert!(Variant::Int(-1).booleanize());
        assert!(!Variant::String(GoString::new()).booleanize());
        assert!(Variant::String(GoString::from_str("x")).booleanize());
        assert!(!Variant::Vector3(Vector3::ZERO).booleanize());

        assert_eq!(Variant::Bool(true).to_int(), 1);
        assert_eq!(Variant::Float(-3.9).to_int(), -3);
        assert_eq!(Variant::String(GoString::from_str(" 42 ")).to_int(), 42);
        assert_eq!(Variant::String(GoString::from_str("nope")).to_int(), 0);
        assert_eq!(Variant::Int(7).to_float(), 7.0);
    }

    #[test]
    fn object_payload_carries_identity() {
        let obj = crate::object::Object::new();
        let v = Variant::Object(obj.id());
        assert_eq!(v.vtype(), VariantType::Object);
        assert!(v.booleanize()); // Live object is truthy.
        let id = obj.id();
        drop(obj);
        // Stale handle: still an Object variant, but falsy.
        let stale = Variant::Object(id);
        assert_eq!(stale.vtype(), VariantType::Object);
        assert!(!stale.booleanize());
        // A null handle is nil-ish for truthiness but keeps its type tag.
        assert!(!Variant::Object(crate::object::ObjectId::NONE).booleanize());
    }

    #[test]
    fn callable_and_signal_identity() {
        let obj = crate::object::Object::new();
        let call = Callable { object: obj.id(), method: StringName::new("take_damage") };
        let v = Variant::Callable(call.clone());
        assert_eq!(v.vtype(), VariantType::Callable);
        assert!(v.booleanize()); // Target alive.
        assert!(call.is_valid());

        let sig = Signal { object: obj.id(), name: StringName::new("died") };
        assert_eq!(Variant::Signal(sig.clone()).vtype(), VariantType::Signal);
        assert!(!sig.is_null());

        let id = obj.id();
        drop(obj);

        // Both become falsy once the target dies, but keep their type tags.
        let stale_call = Variant::Callable(Callable { object: id, method: StringName::new("x") });
        assert_eq!(stale_call.vtype(), VariantType::Callable);
        assert!(!stale_call.booleanize());

        let stale_sig = Variant::Signal(Signal { object: id, name: StringName::new("y") });
        assert_eq!(stale_sig.vtype(), VariantType::Signal);
        assert!(!stale_sig.booleanize());

        // Unbound callables are null.
        let unbound = Callable { object: crate::object::ObjectId::NONE, method: StringName::new("z") };
        assert!(unbound.is_null());
        assert!(!Variant::Callable(unbound).booleanize());
    }

    #[test]
    fn containers_hold_variants() {
        use std::collections::HashMap;

        let mut dict: HashMap<StringName, Variant> = HashMap::new();
        dict.insert(StringName::new("name"), Variant::String(GoString::from_str("player")));
        dict.insert(StringName::new("hp"), Variant::Int(100));
        let v = Variant::Dictionary(dict);

        assert_eq!(v.vtype(), VariantType::Dictionary);
        assert_eq!(v.vtype() as u32, 27); // Engine order: Dictionary before Array.
        assert!(v.booleanize());

        if let Variant::Dictionary(d) = &v {
            assert_eq!(d[&StringName::new("hp")], Variant::Int(100));
        }

        let arr = Variant::Array(vec![Variant::Int(1), Variant::Float(2.5), v.clone()]);
        assert_eq!(arr.vtype(), VariantType::Array);
        assert_eq!(arr.vtype() as u32, 28);
        assert!(arr.booleanize());

        // Empty containers are falsy, like in the engine.
        assert!(!Variant::Array(Vec::new()).booleanize());
        assert!(!Variant::Dictionary(HashMap::new()).booleanize());
    }

    #[test]
    fn packed_arrays_carry_data() {
        let bytes = Variant::PackedByteArray(vec![1, 2, 255]);
        assert_eq!(bytes.vtype(), VariantType::PackedByteArray);
        assert_eq!(bytes.vtype() as u32, 29);
        assert!(bytes.booleanize());
        assert!(Variant::PackedByteArray(Vec::new()).booleanize() == false);

        let f32s = Variant::PackedFloat32Array(vec![0.5, 1.5]);
        assert_eq!(f32s.vtype() as u32, 32);

        let strings = Variant::PackedStringArray(vec![
            GoString::from_str("a"),
            GoString::from_str("b"),
        ]);
        assert_eq!(strings.vtype() as u32, 34);

        let verts = Variant::PackedVector3Array(vec![
            Vector3::new(0.0, 1.0, 0.0),
            Vector3::new(1.0, 0.0, 0.0),
        ]);
        assert_eq!(verts.vtype() as u32, 36);
        if let Variant::PackedVector3Array(ref v) = verts {
            assert_eq!(v[1], Vector3::new(1.0, 0.0, 0.0));
        }

        let colors = Variant::PackedColorArray(vec![Color::new(1.0, 0.0, 0.0, 1.0)]);
        assert_eq!(colors.vtype() as u32, 37);

        let v4s = Variant::PackedVector4Array(vec![Vector4::new(1.0, 2.0, 3.0, 4.0)]);
        assert_eq!(v4s.vtype() as u32, 38);

        let i64s = Variant::PackedInt64Array(vec![i64::MIN, i64::MAX]);
        assert_eq!(i64s.vtype() as u32, 31);
        assert_eq!(i64s, Variant::PackedInt64Array(vec![i64::MIN, i64::MAX]));
        assert_ne!(i64s, Variant::PackedInt64Array(vec![0]));
    }

    #[test]
    fn dictionary_and_array_access() {
        use std::collections::HashMap;

        let mut v = Variant::Dictionary(HashMap::new());
        assert!(v.dict_set("hp", Variant::Int(100)));
        assert!(v.dict_set("name", Variant::String(GoString::from_str("hero"))));
        assert_eq!(v.dict_len(), 2);
        assert_eq!(v.dict_get("hp"), Some(&Variant::Int(100)));
        assert_eq!(v.dict_get("missing"), None);

        // Sorted keys make the result deterministic.
        let keys: Vec<String> = v.dict_keys_sorted().iter().map(|k| k.as_string()).collect();
        assert_eq!(keys, vec!["hp".to_string(), "name".to_string()]);

        // Nesting: a dictionary inside an array, mutated through the array.
        let mut arr = Variant::Array(vec![v.clone()]);
        assert_eq!(arr.array_len(), 1);
        let inner = arr.array_get(0).unwrap();
        assert_eq!(inner.dict_len(), 2);

        assert!(arr.array_push(Variant::Int(7)));
        assert_eq!(arr.array_len(), 2);
        assert_eq!(arr.array_get(1), Some(&Variant::Int(7)));
        assert_eq!(arr.array_get(5), None);

        // Wrong receiver types are rejected, not panicking.
        let mut not_dict = Variant::Int(1);
        assert!(!not_dict.dict_set("k", Variant::Nil));
        assert_eq!(not_dict.dict_len(), 0);
        let mut not_arr = Variant::Float(1.0);
        assert!(!not_arr.array_push(Variant::Nil));
        assert_eq!(not_arr.array_len(), 0);

        // Packed length covers the typed arrays.
        assert_eq!(Variant::PackedFloat32Array(vec![1.0, 2.0]).packed_len(), 2);
        assert_eq!(Variant::Int(3).packed_len(), 0);
    }

    #[test]
    fn default_is_nil() {
        assert!(Variant::default().is_nil());
    }
}
