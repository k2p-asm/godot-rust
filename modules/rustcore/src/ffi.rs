//! C ABI boundary for the ported math.
//!
//! Only POD crosses this boundary (plain `f32` and `#[repr(C)]` structs), never
//! `Variant`/`String`/`Object*`. These are the first symbols the engine will
//! call once the module is linked; they exist so the link can be verified end to
//! end before any C++ call site is switched over.

use crate::math::basis::Basis;
use crate::math::transform_3d::Transform3D;
use crate::math::vector3::Vector3;

/// ABI version of this boundary. Bump on any signature change.
#[no_mangle]
pub extern "C" fn gcr_abi_version() -> u32 {
    1
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CGcrVector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl From<Vector3> for CGcrVector3 {
    fn from(v: Vector3) -> Self {
        Self { x: v.x, y: v.y, z: v.z }
    }
}

impl From<CGcrVector3> for Vector3 {
    fn from(v: CGcrVector3) -> Self {
        Vector3::new(v.x, v.y, v.z)
    }
}

/// `Vector3::dot` across the boundary.
#[no_mangle]
pub extern "C" fn gcr_vector3_dot(a: CGcrVector3, b: CGcrVector3) -> f32 {
    Vector3::from(a).dot(Vector3::from(b))
}

/// `Vector3::normalized` across the boundary.
#[no_mangle]
pub extern "C" fn gcr_vector3_normalized(v: CGcrVector3) -> CGcrVector3 {
    Vector3::from(v).normalized().into()
}

/// `Transform3D::xform`, taking the basis as 9 floats in row-major order.
#[no_mangle]
pub extern "C" fn gcr_transform3d_xform(
    rows: *const f32,
    origin: CGcrVector3,
    v: CGcrVector3,
) -> CGcrVector3 {
    if rows.is_null() {
        return v;
    }
    // SAFETY: the caller guarantees a 9-element row-major matrix.
    let r = unsafe { std::slice::from_raw_parts(rows, 9) };
    let basis = Basis::from_rows(
        Vector3::new(r[0], r[1], r[2]),
        Vector3::new(r[3], r[4], r[5]),
        Vector3::new(r[6], r[7], r[8]),
    );
    let t = Transform3D::new(basis, Vector3::from(origin));
    t.xform(Vector3::from(v)).into()
}

/// Flat, POD-only `Variant` for the C ABI.
///
/// `tag` is a `Variant::Type` value; `bits` carries bool/int/float/string-name
/// payloads and `data` the vector-like ones. Nothing here allocates or holds a
/// pointer, so the struct is safe to pass by value across the boundary.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CGcrVariant {
    pub tag: u32,
    pub bits: u64,
    pub data: [f32; 4],
}

use crate::variant::{Variant, VariantType};

#[no_mangle]
pub extern "C" fn gcr_variant_nil() -> CGcrVariant {
    CGcrVariant::default()
}

#[no_mangle]
pub extern "C" fn gcr_variant_bool(v: i32) -> CGcrVariant {
    CGcrVariant { tag: VariantType::Bool as u32, bits: if v != 0 { 1 } else { 0 }, data: [0.0; 4] }
}

#[no_mangle]
pub extern "C" fn gcr_variant_int(v: i64) -> CGcrVariant {
    CGcrVariant { tag: VariantType::Int as u32, bits: v as u64, data: [0.0; 4] }
}

#[no_mangle]
pub extern "C" fn gcr_variant_float(v: f64) -> CGcrVariant {
    CGcrVariant { tag: VariantType::Float as u32, bits: v.to_bits(), data: [0.0; 4] }
}

#[no_mangle]
pub extern "C" fn gcr_variant_vector3(v: CGcrVector3) -> CGcrVariant {
    CGcrVariant {
        tag: VariantType::Vector3 as u32,
        bits: 0,
        data: [v.x, v.y, v.z, 0.0],
    }
}

#[no_mangle]
pub extern "C" fn gcr_variant_type(v: CGcrVariant) -> u32 {
    to_variant(v).vtype() as u32
}

#[no_mangle]
pub extern "C" fn gcr_variant_as_int(v: CGcrVariant) -> i64 {
    to_variant(v).to_int()
}

#[no_mangle]
pub extern "C" fn gcr_variant_as_float(v: CGcrVariant) -> f64 {
    to_variant(v).to_float()
}

#[no_mangle]
pub extern "C" fn gcr_variant_as_vector3(v: CGcrVariant) -> CGcrVector3 {
    match to_variant(v) {
        Variant::Vector3(p) => p.into(),
        _ => CGcrVector3::default(),
    }
}

#[no_mangle]
pub extern "C" fn gcr_variant_booleanize(v: CGcrVariant) -> i32 {
    to_variant(v).booleanize() as i32
}

fn to_variant(v: CGcrVariant) -> Variant {
    match v.tag {
        1 => Variant::Bool(v.bits != 0),
        2 => Variant::Int(v.bits as i64),
        3 => Variant::Float(f64::from_bits(v.bits)),
        9 => Variant::Vector3(Vector3::new(v.data[0], v.data[1], v.data[2])),
        20 => Variant::Color(crate::math::color::Color::new(
            v.data[0], v.data[1], v.data[2], v.data[3],
        )),
        _ => Variant::Nil,
    }
}
/// Sum of a packed byte array, taking the same `(ptr, len)` shape the engine
/// uses for `PackedByteArray`. Returns 0 for a null pointer.
#[no_mangle]
pub extern "C" fn gcr_packed_byte_sum(data: *const u8, len: u32) -> u64 {
    if data.is_null() || len == 0 {
        return 0;
    }
    // SAFETY: the caller guarantees `len` readable bytes at `data`.
    let slice = unsafe { std::slice::from_raw_parts(data, len as usize) };
    let packed = Variant::PackedByteArray(slice.to_vec());
    match &packed {
        Variant::PackedByteArray(bytes) => bytes.iter().map(|b| *b as u64).sum(),
        _ => 0,
    }
}

/// Length of a packed f32 array, to prove the typed payloads also cross.
#[no_mangle]
pub extern "C" fn gcr_packed_f32_len(data: *const f32, len: u32) -> u32 {
    if data.is_null() {
        return 0;
    }
    // SAFETY: the caller guarantees `len` readable floats at `data`.
    let slice = unsafe { std::slice::from_raw_parts(data, len as usize) };
    Variant::PackedFloat32Array(slice.to_vec()).packed_len() as u32
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn variant_bridge_roundtrip() {
        let i = gcr_variant_int(-42);
        assert_eq!(gcr_variant_type(i), 2);
        assert_eq!(gcr_variant_as_int(i), -42);
        assert_eq!(gcr_variant_booleanize(i), 1);

        let f = gcr_variant_float(2.5);
        assert_eq!(gcr_variant_as_float(f), 2.5);

        let b = gcr_variant_bool(0);
        assert_eq!(gcr_variant_booleanize(b), 0);

        let v = gcr_variant_vector3(CGcrVector3 { x: 1.0, y: 2.0, z: 3.0 });
        assert_eq!(gcr_variant_type(v), 9);
        assert_eq!(gcr_variant_as_vector3(v), CGcrVector3 { x: 1.0, y: 2.0, z: 3.0 });
        assert_eq!(gcr_variant_booleanize(v), 1);

        assert_eq!(gcr_variant_type(gcr_variant_nil()), 0);
        assert_eq!(gcr_variant_booleanize(gcr_variant_nil()), 0);
    }

    #[test]
    fn packed_array_bridge() {
        let bytes: [u8; 4] = [1, 2, 3, 250];
        assert_eq!(gcr_packed_byte_sum(bytes.as_ptr(), 4), 256);
        assert_eq!(gcr_packed_byte_sum(std::ptr::null(), 4), 0);
        assert_eq!(gcr_packed_byte_sum(bytes.as_ptr(), 0), 0);

        let floats: [f32; 3] = [0.5, 1.5, 2.5];
        assert_eq!(gcr_packed_f32_len(floats.as_ptr(), 3), 3);
        assert_eq!(gcr_packed_f32_len(std::ptr::null(), 3), 0);
    }

    #[test]
    fn ffi_roundtrip() {
        let a = CGcrVector3 { x: 1.0, y: 2.0, z: 3.0 };
        let b = CGcrVector3 { x: 4.0, y: 5.0, z: 6.0 };
        assert_eq!(gcr_vector3_dot(a, b), 32.0);

        let n = gcr_vector3_normalized(CGcrVector3 { x: 0.0, y: 0.0, z: 0.0 });
        assert_eq!(n, CGcrVector3::default());
    }

    #[test]
    fn ffi_identity_transform() {
        let rows = [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0];
        let out = gcr_transform3d_xform(
            rows.as_ptr(),
            CGcrVector3 { x: 1.0, y: 2.0, z: 3.0 },
            CGcrVector3 { x: 0.5, y: 0.5, z: 0.5 },
        );
        assert_eq!(out, CGcrVector3 { x: 1.5, y: 2.5, z: 3.5 });
    }
}
