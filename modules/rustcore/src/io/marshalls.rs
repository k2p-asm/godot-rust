//! `core/io/marshalls.*`: the binary `Variant` wire format (subset).
//!
//! Every value is a little-endian `uint32` header followed by its payload. The
//! low byte is the `Variant` type tag; bit 16 is the "64-bit payload" flag used
//! by `int`, `float` and the real-valued math types; for arrays and
//! dictionaries bits 16..19 hold the typed-container metadata (which this port
//! rejects, because its containers are untyped).
//!
//! Ported types: nil, bool, int (32/64), float (32/64), string (length-prefixed
//! UTF-8 padded to four bytes), `Vector2`, `Vector2i`, `Rect2`, `Rect2i`,
//! `Vector3`, `Vector3i`, `Transform2D`, `Vector4`, `Vector4i`, `Plane`,
//! `Quaternion`, `Aabb`, `Basis`, `Transform3D`, `Projection`, `Color` (always
//! single precision), untyped `Array`/`Dictionary`, and the packed byte, int32,
//! int64, float32, float64, string, vector2, vector3, vector4 and color arrays.
//!
//! Objects, callables, signals, node paths, RIDs and string names are not
//! representable in the port's `Variant` and decode to
//! [`MarshallsError::Unsupported`].

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
use crate::variant::{Variant, VariantType};

const HEADER_TYPE_MASK: u32 = 0xFF;
const HEADER_DATA_FLAG_64: u32 = 1 << 16;
const TYPED_ARRAY_MASK: u32 = 0b11 << 16;
const TYPED_DICTIONARY_KEY_MASK: u32 = 0b11 << 16;
const TYPED_DICTIONARY_VALUE_MASK: u32 = 0b11 << 18;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MarshallsError {
    /// `ERR_INVALID_DATA` / `ERR_FILE_EOF`.
    InvalidData,
    /// A type that the port cannot represent.
    Unsupported(&'static str),
}

fn push_u32(out: &mut Vec<u8>, value: u32) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn push_f32(out: &mut Vec<u8>, value: f32) {
    out.extend_from_slice(&value.to_bits().to_le_bytes());
}

fn push_i64(out: &mut Vec<u8>, value: i64) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn push_f64(out: &mut Vec<u8>, value: f64) {
    out.extend_from_slice(&value.to_bits().to_le_bytes());
}

/// `_encode_string`: length, UTF-8 bytes, zero padding to a multiple of four.
fn push_string(out: &mut Vec<u8>, text: &str) {
    let bytes = text.as_bytes();
    push_u32(out, bytes.len() as u32);
    out.extend_from_slice(bytes);
    while out.len() % 4 != 0 {
        out.push(0);
    }
}

fn push_real(out: &mut Vec<u8>, value: f32) {
    push_f32(out, value);
}

/// `encode_variant` for the supported subset.
pub fn encode_variant(value: &Variant) -> Vec<u8> {
    let mut out = Vec::new();
    encode_into(value, &mut out);
    out
}

fn encode_into(value: &Variant, out: &mut Vec<u8>) {
    let tag = variant_tag(value);
    match value {
        Variant::Nil => push_u32(out, tag),
        Variant::Bool(value) => {
            push_u32(out, tag);
            push_u32(out, u32::from(*value));
        }
        Variant::Int(value) => {
            let mut header = tag;
            if *value > i32::MAX as i64 || *value < i32::MIN as i64 {
                header |= HEADER_DATA_FLAG_64;
                push_u32(out, header);
                push_i64(out, *value);
            } else {
                push_u32(out, header);
                push_u32(out, *value as i32 as u32);
            }
        }
        Variant::Float(value) => {
            let as_f32 = *value as f32;
            let mut header = tag;
            if as_f32 as f64 != *value {
                header |= HEADER_DATA_FLAG_64;
                push_u32(out, header);
                push_f64(out, *value);
            } else {
                push_u32(out, header);
                push_f32(out, as_f32);
            }
        }
        Variant::String(text) => {
            push_u32(out, tag);
            push_string(out, &text.as_string());
        }
        Variant::Vector2(value) => {
            push_u32(out, tag);
            push_real(out, value.x);
            push_real(out, value.y);
        }
        Variant::Vector2i(value) => {
            push_u32(out, tag);
            push_u32(out, value.x as u32);
            push_u32(out, value.y as u32);
        }
        Variant::Rect2(value) => {
            push_u32(out, tag);
            push_real(out, value.position.x);
            push_real(out, value.position.y);
            push_real(out, value.size.x);
            push_real(out, value.size.y);
        }
        Variant::Rect2i(value) => {
            push_u32(out, tag);
            push_u32(out, value.position.x as u32);
            push_u32(out, value.position.y as u32);
            push_u32(out, value.size.x as u32);
            push_u32(out, value.size.y as u32);
        }
        Variant::Vector3(value) => {
            push_u32(out, tag);
            push_real(out, value.x);
            push_real(out, value.y);
            push_real(out, value.z);
        }
        Variant::Vector3i(value) => {
            push_u32(out, tag);
            push_u32(out, value.x as u32);
            push_u32(out, value.y as u32);
            push_u32(out, value.z as u32);
        }
        Variant::Transform2D(value) => {
            push_u32(out, tag);
            for column in value.columns {
                push_real(out, column.x);
                push_real(out, column.y);
            }
        }
        Variant::Vector4(value) => {
            push_u32(out, tag);
            push_real(out, value.x);
            push_real(out, value.y);
            push_real(out, value.z);
            push_real(out, value.w);
        }
        Variant::Vector4i(value) => {
            push_u32(out, tag);
            push_u32(out, value.x as u32);
            push_u32(out, value.y as u32);
            push_u32(out, value.z as u32);
            push_u32(out, value.w as u32);
        }
        Variant::Plane(value) => {
            push_u32(out, tag);
            push_real(out, value.normal.x);
            push_real(out, value.normal.y);
            push_real(out, value.normal.z);
            push_real(out, value.d);
        }
        Variant::Quaternion(value) => {
            push_u32(out, tag);
            push_real(out, value.x);
            push_real(out, value.y);
            push_real(out, value.z);
            push_real(out, value.w);
        }
        Variant::Aabb(value) => {
            push_u32(out, tag);
            push_real(out, value.position.x);
            push_real(out, value.position.y);
            push_real(out, value.position.z);
            push_real(out, value.size.x);
            push_real(out, value.size.y);
            push_real(out, value.size.z);
        }
        Variant::Basis(value) => {
            push_u32(out, tag);
            for column in value.rows {
                push_real(out, column.x);
                push_real(out, column.y);
                push_real(out, column.z);
            }
        }
        Variant::Transform3D(value) => {
            push_u32(out, tag);
            for column in value.basis.rows {
                push_real(out, column.x);
                push_real(out, column.y);
                push_real(out, column.z);
            }
            push_real(out, value.origin.x);
            push_real(out, value.origin.y);
            push_real(out, value.origin.z);
        }
        Variant::Projection(value) => {
            push_u32(out, tag);
            for column in &value.columns {
                push_real(out, column.x);
                push_real(out, column.y);
                push_real(out, column.z);
                push_real(out, column.w);
            }
        }
        Variant::Color(value) => {
            push_u32(out, tag);
            push_f32(out, value.r);
            push_f32(out, value.g);
            push_f32(out, value.b);
            push_f32(out, value.a);
        }
        Variant::Array(items) => {
            push_u32(out, tag);
            push_u32(out, items.len() as u32);
            for item in items {
                encode_into(item, out);
            }
        }
        Variant::Dictionary(entries) => {
            push_u32(out, tag);
            push_u32(out, entries.len() as u32);
            for (key, item) in entries {
                encode_into(
                    &Variant::String(GoString::from_str(&key.as_string())),
                    out,
                );
                encode_into(item, out);
            }
        }
        Variant::PackedByteArray(bytes) => {
            push_u32(out, tag);
            push_u32(out, bytes.len() as u32);
            out.extend_from_slice(bytes);
            while out.len() % 4 != 0 {
                out.push(0);
            }
        }
        Variant::PackedInt32Array(values) => {
            push_u32(out, tag);
            push_u32(out, values.len() as u32);
            for value in values {
                push_u32(out, *value as u32);
            }
        }
        Variant::PackedInt64Array(values) => {
            push_u32(out, tag);
            push_u32(out, values.len() as u32);
            for value in values {
                push_i64(out, *value);
            }
        }
        Variant::PackedFloat32Array(values) => {
            push_u32(out, tag);
            push_u32(out, values.len() as u32);
            for value in values {
                push_f32(out, *value);
            }
        }
        Variant::PackedFloat64Array(values) => {
            push_u32(out, tag);
            push_u32(out, values.len() as u32);
            for value in values {
                push_f64(out, *value);
            }
        }
        Variant::PackedStringArray(values) => {
            push_u32(out, tag);
            push_u32(out, values.len() as u32);
            for value in values {
                push_string(out, &value.as_string());
            }
        }
        Variant::PackedVector2Array(values) => {
            push_u32(out, tag);
            push_u32(out, values.len() as u32);
            for value in values {
                push_real(out, value.x);
                push_real(out, value.y);
            }
        }
        Variant::PackedVector3Array(values) => {
            push_u32(out, tag);
            push_u32(out, values.len() as u32);
            for value in values {
                push_real(out, value.x);
                push_real(out, value.y);
                push_real(out, value.z);
            }
        }
        Variant::PackedColorArray(values) => {
            push_u32(out, tag);
            push_u32(out, values.len() as u32);
            for value in values {
                push_f32(out, value.r);
                push_f32(out, value.g);
                push_f32(out, value.b);
                push_f32(out, value.a);
            }
        }
        Variant::PackedVector4Array(values) => {
            push_u32(out, tag);
            push_u32(out, values.len() as u32);
            for value in values {
                push_real(out, value.x);
                push_real(out, value.y);
                push_real(out, value.z);
                push_real(out, value.w);
            }
        }
        Variant::Object(_) | Variant::Callable(_) | Variant::Signal(_) => {
            // The port cannot round-trip object handles; emit the type tag so the
            // stream stays well-formed (decode reports it as unsupported).
            push_u32(out, tag);
        }
    }
}

fn variant_tag(value: &Variant) -> u32 {
    let tag = match value {
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
    };
    tag as u32
}

struct Reader<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    fn remaining(&self) -> usize {
        self.data.len().saturating_sub(self.pos)
    }

    fn take(&mut self, count: usize) -> Result<&'a [u8], MarshallsError> {
        if self.remaining() < count {
            return Err(MarshallsError::InvalidData);
        }
        let slice = &self.data[self.pos..self.pos + count];
        self.pos += count;
        Ok(slice)
    }

    fn u32(&mut self) -> Result<u32, MarshallsError> {
        let bytes = self.take(4)?;
        Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    fn i32(&mut self) -> Result<i32, MarshallsError> {
        Ok(self.u32()? as i32)
    }

    fn u64(&mut self) -> Result<u64, MarshallsError> {
        let bytes = self.take(8)?;
        Ok(u64::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        ]))
    }

    fn i64(&mut self) -> Result<i64, MarshallsError> {
        Ok(self.u64()? as i64)
    }

    fn f32(&mut self) -> Result<f32, MarshallsError> {
        Ok(f32::from_bits(self.u32()?))
    }

    fn f64(&mut self) -> Result<f64, MarshallsError> {
        Ok(f64::from_bits(self.u64()?))
    }

    fn real(&mut self, is_64: bool) -> Result<f32, MarshallsError> {
        if is_64 {
            Ok(self.f64()? as f32)
        } else {
            self.f32()
        }
    }

    /// `_decode_string`.
    fn string(&mut self) -> Result<String, MarshallsError> {
        let length = self.i32()?;
        if length < 0 {
            return Err(MarshallsError::InvalidData);
        }
        let bytes = self.take(length as usize)?.to_vec();
        let padding = (4 - length as usize % 4) % 4;
        self.take(padding)?;
        String::from_utf8(bytes).map_err(|_| MarshallsError::InvalidData)
    }
}

/// `decode_variant` for the supported subset; `Ok((value, consumed))`.
pub fn decode_variant(bytes: &[u8]) -> Result<(Variant, usize), MarshallsError> {
    let mut reader = Reader::new(bytes);
    let value = decode_from(&mut reader, 0)?;
    Ok((value, reader.pos))
}

fn decode_from(reader: &mut Reader, depth: i32) -> Result<Variant, MarshallsError> {
    if depth > 32 {
        return Err(MarshallsError::InvalidData);
    }
    let header = reader.u32()?;
    let tag = header & HEADER_TYPE_MASK;
    let is_64 = header & HEADER_DATA_FLAG_64 != 0;

    let value = match tag {
        0 => Variant::Nil,
        1 => Variant::Bool(reader.u32()? != 0),
        2 => {
            if is_64 {
                Variant::Int(reader.i64()?)
            } else {
                Variant::Int(reader.i32()? as i64)
            }
        }
        3 => {
            if is_64 {
                Variant::Float(reader.f64()?)
            } else {
                Variant::Float(reader.f32()? as f64)
            }
        }
        4 => Variant::String(GoString::from_str(&reader.string()?)),
        5 => Variant::Vector2(Vector2::new(reader.real(is_64)?, reader.real(is_64)?)),
        6 => Variant::Vector2i(Vector2i::new(reader.i32()?, reader.i32()?)),
        7 => Variant::Rect2(Rect2::new(
            Vector2::new(reader.real(is_64)?, reader.real(is_64)?),
            Vector2::new(reader.real(is_64)?, reader.real(is_64)?),
        )),
        8 => Variant::Rect2i(Rect2i::new(
            Vector2i::new(reader.i32()?, reader.i32()?),
            Vector2i::new(reader.i32()?, reader.i32()?),
        )),
        9 => Variant::Vector3(Vector3::new(
            reader.real(is_64)?,
            reader.real(is_64)?,
            reader.real(is_64)?,
        )),
        10 => Variant::Vector3i(Vector3i::new(
            reader.i32()?,
            reader.i32()?,
            reader.i32()?,
        )),
        11 => {
            let mut columns = [Vector2::ZERO; 3];
            for column in columns.iter_mut() {
                *column = Vector2::new(reader.real(is_64)?, reader.real(is_64)?);
            }
            Variant::Transform2D(Transform2D { columns })
        }
        12 => Variant::Vector4(Vector4::new(
            reader.real(is_64)?,
            reader.real(is_64)?,
            reader.real(is_64)?,
            reader.real(is_64)?,
        )),
        13 => Variant::Vector4i(Vector4i::new(
            reader.i32()?,
            reader.i32()?,
            reader.i32()?,
            reader.i32()?,
        )),
        14 => {
            let normal = Vector3::new(
                reader.real(is_64)?,
                reader.real(is_64)?,
                reader.real(is_64)?,
            );
            Variant::Plane(Plane::new(normal.x, normal.y, normal.z, reader.real(is_64)?))
        }
        15 => Variant::Quaternion(Quaternion::new(
            reader.real(is_64)?,
            reader.real(is_64)?,
            reader.real(is_64)?,
            reader.real(is_64)?,
        )),
        16 => {
            let position = Vector3::new(
                reader.real(is_64)?,
                reader.real(is_64)?,
                reader.real(is_64)?,
            );
            let size = Vector3::new(
                reader.real(is_64)?,
                reader.real(is_64)?,
                reader.real(is_64)?,
            );
            Variant::Aabb(Aabb::new(position, size))
        }
        17 => {
            let mut rows = [Vector3::ZERO; 3];
            for row in rows.iter_mut() {
                *row = Vector3::new(
                    reader.real(is_64)?,
                    reader.real(is_64)?,
                    reader.real(is_64)?,
                );
            }
            Variant::Basis(Basis { rows })
        }
        18 => {
            let mut rows = [Vector3::ZERO; 3];
            for row in rows.iter_mut() {
                *row = Vector3::new(
                    reader.real(is_64)?,
                    reader.real(is_64)?,
                    reader.real(is_64)?,
                );
            }
            let origin = Vector3::new(
                reader.real(is_64)?,
                reader.real(is_64)?,
                reader.real(is_64)?,
            );
            Variant::Transform3D(Transform3D {
                basis: Basis { rows },
                origin,
            })
        }
        19 => {
            let mut columns = [Vector4::ZERO; 4];
            for column in columns.iter_mut() {
                column.x = reader.real(is_64)?;
                column.y = reader.real(is_64)?;
                column.z = reader.real(is_64)?;
                column.w = reader.real(is_64)?;
            }
            Variant::Projection(Projection { columns })
        }
        20 => Variant::Color(Color::new(
            reader.f32()?,
            reader.f32()?,
            reader.f32()?,
            reader.f32()?,
        )),
        24 => return Err(MarshallsError::Unsupported("Object")),
        25 => return Err(MarshallsError::Unsupported("Callable")),
        26 => return Err(MarshallsError::Unsupported("Signal")),
        27 => {
            if header & TYPED_DICTIONARY_KEY_MASK != 0
                || header & TYPED_DICTIONARY_VALUE_MASK != 0
            {
                return Err(MarshallsError::Unsupported("typed Dictionary"));
            }
            let count = reader.u32()?;
            let mut entries = std::collections::HashMap::new();
            for _ in 0..count {
                let key = decode_from(reader, depth + 1)?;
                let key = match key {
                    Variant::String(text) => StringName::new(&text.as_string()),
                    Variant::Int(number) => StringName::new(&number.to_string()),
                    _ => return Err(MarshallsError::Unsupported("Dictionary key type")),
                };
                let value = decode_from(reader, depth + 1)?;
                entries.insert(key, value);
            }
            Variant::Dictionary(entries)
        }
        28 => {
            if header & TYPED_ARRAY_MASK != 0 {
                return Err(MarshallsError::Unsupported("typed Array"));
            }
            let count = reader.u32()?;
            let mut items = Vec::with_capacity(count.min(1024) as usize);
            for _ in 0..count {
                items.push(decode_from(reader, depth + 1)?);
            }
            Variant::Array(items)
        }
        29 => {
            let count = reader.u32()? as usize;
            let bytes = reader.take(count)?.to_vec();
            let padding = (4 - count % 4) % 4;
            reader.take(padding)?;
            Variant::PackedByteArray(bytes)
        }
        30 => {
            let count = reader.u32()?;
            let mut values = Vec::with_capacity(count.min(4096) as usize);
            for _ in 0..count {
                values.push(reader.i32()?);
            }
            Variant::PackedInt32Array(values)
        }
        31 => {
            let count = reader.u32()?;
            let mut values = Vec::with_capacity(count.min(4096) as usize);
            for _ in 0..count {
                values.push(reader.i64()?);
            }
            Variant::PackedInt64Array(values)
        }
        32 => {
            let count = reader.u32()?;
            let mut values = Vec::with_capacity(count.min(4096) as usize);
            for _ in 0..count {
                values.push(reader.f32()?);
            }
            Variant::PackedFloat32Array(values)
        }
        33 => {
            let count = reader.u32()?;
            let mut values = Vec::with_capacity(count.min(4096) as usize);
            for _ in 0..count {
                values.push(reader.f64()?);
            }
            Variant::PackedFloat64Array(values)
        }
        34 => {
            let count = reader.u32()?;
            let mut values = Vec::with_capacity(count.min(4096) as usize);
            for _ in 0..count {
                values.push(GoString::from_str(&reader.string()?));
            }
            Variant::PackedStringArray(values)
        }
        35 => {
            let count = reader.u32()?;
            let mut values = Vec::with_capacity(count.min(4096) as usize);
            for _ in 0..count {
                values.push(Vector2::new(reader.real(is_64)?, reader.real(is_64)?));
            }
            Variant::PackedVector2Array(values)
        }
        36 => {
            let count = reader.u32()?;
            let mut values = Vec::with_capacity(count.min(4096) as usize);
            for _ in 0..count {
                values.push(Vector3::new(
                    reader.real(is_64)?,
                    reader.real(is_64)?,
                    reader.real(is_64)?,
                ));
            }
            Variant::PackedVector3Array(values)
        }
        37 => {
            let count = reader.u32()?;
            let mut values = Vec::with_capacity(count.min(4096) as usize);
            for _ in 0..count {
                values.push(Color::new(
                    reader.f32()?,
                    reader.f32()?,
                    reader.f32()?,
                    reader.f32()?,
                ));
            }
            Variant::PackedColorArray(values)
        }
        38 => {
            let count = reader.u32()?;
            let mut values = Vec::with_capacity(count.min(4096) as usize);
            for _ in 0..count {
                values.push(Vector4::new(
                    reader.real(is_64)?,
                    reader.real(is_64)?,
                    reader.real(is_64)?,
                    reader.real(is_64)?,
                ));
            }
            Variant::PackedVector4Array(values)
        }
        _ => return Err(MarshallsError::Unsupported("Variant type")),
    };

    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::vector4i::Vector4i as V4i;

    #[test]
    fn scalar_layouts() {
        assert_eq!(encode_variant(&Variant::Nil), vec![0, 0, 0, 0]);
        assert_eq!(
            encode_variant(&Variant::Bool(true)),
            vec![1, 0, 0, 0, 1, 0, 0, 0]
        );
        assert_eq!(
            encode_variant(&Variant::Int(5)),
            vec![2, 0, 0, 0, 5, 0, 0, 0]
        );

        let wide = encode_variant(&Variant::Int(1 << 40));
        assert_eq!(&wide[0..4], &[2, 0, 1, 0], "64-bit flag");
        assert_eq!(wide.len(), 12);
        assert_eq!(
            decode_variant(&wide).unwrap(),
            (Variant::Int(1 << 40), 12)
        );

        let float = encode_variant(&Variant::Float(1.5));
        assert_eq!(float, vec![3, 0, 0, 0, 0x00, 0x00, 0xC0, 0x3F]);

        // 1.1 is not representable as f32, so the 64-bit form is used.
        let double = encode_variant(&Variant::Float(1.1));
        assert_eq!(double.len(), 12);
        assert_eq!(double[2], 1, "64-bit flag");
        assert_eq!(decode_variant(&double).unwrap().0, Variant::Float(1.1));
    }

    #[test]
    fn string_layout_and_padding() {
        let encoded = encode_variant(&Variant::String(GoString::from_str("ab")));
        assert_eq!(
            encoded,
            vec![4, 0, 0, 0, 2, 0, 0, 0, b'a', b'b', 0, 0]
        );
        let encoded = encode_variant(&Variant::String(GoString::from_str("abcd")));
        assert_eq!(encoded.len(), 12, "already four-byte aligned");

        let (decoded, consumed) = decode_variant(&encoded).unwrap();
        assert_eq!(consumed, 12);
        assert_eq!(
            decoded,
            Variant::String(GoString::from_str("abcd"))
        );
    }

    #[test]
    fn math_type_sizes_and_round_trips() {
        let values = vec![
            Variant::Vector2(Vector2::new(1.0, 2.0)),
            Variant::Vector2i(Vector2i::new(-1, 2)),
            Variant::Rect2(Rect2::new(Vector2::new(1.0, 2.0), Vector2::new(3.0, 4.0))),
            Variant::Rect2i(Rect2i::new(Vector2i::new(1, 2), Vector2i::new(3, 4))),
            Variant::Vector3(Vector3::new(1.0, 2.0, 3.0)),
            Variant::Vector3i(Vector3i::new(1, 2, 3)),
            Variant::Transform2D(Transform2D::IDENTITY),
            Variant::Vector4(Vector4::new(1.0, 2.0, 3.0, 4.0)),
            Variant::Vector4i(V4i::new(1, 2, 3, 4)),
            Variant::Plane(Plane::new(0.0, 1.0, 0.0, 5.0)),
            Variant::Quaternion(Quaternion::new(0.0, 0.0, 0.0, 1.0)),
            Variant::Aabb(Aabb::new(Vector3::ZERO, Vector3::ONE)),
            Variant::Basis(Basis::IDENTITY),
            Variant::Transform3D(Transform3D::IDENTITY),
            Variant::Projection(Projection::IDENTITY),
            Variant::Color(Color::new(0.25, 0.5, 0.75, 1.0)),
        ];
        for value in values {
            let encoded = encode_variant(&value);
            assert_eq!(encoded.len() % 4, 0);
            let (decoded, consumed) = decode_variant(&encoded).unwrap();
            assert_eq!(consumed, encoded.len());
            assert_eq!(decoded, value);
        }

        // Explicit sizes for the fixed-width types.
        assert_eq!(
            encode_variant(&Variant::Vector2(Vector2::new(1.0, 2.0))).len(),
            4 + 8
        );
        assert_eq!(
            encode_variant(&Variant::Vector2i(Vector2i::new(1, 2))).len(),
            4 + 8
        );
        assert_eq!(encode_variant(&Variant::Color(Color::new(1.0, 1.0, 1.0, 1.0))).len(), 4 + 16);
        assert_eq!(encode_variant(&Variant::Projection(Projection::IDENTITY)).len(), 4 + 64);
    }

    #[test]
    fn containers_and_packed_arrays() {
        let array = Variant::Array(vec![Variant::Int(1), Variant::String(GoString::from_str("x"))]);
        let encoded = encode_variant(&array);
        assert_eq!(&encoded[0..4], &[28, 0, 0, 0], "Array tag");
        assert_eq!(&encoded[4..8], &[2, 0, 0, 0], "element count");
        assert_eq!(decode_variant(&encoded).unwrap(), (array, encoded.len()));

        let mut dict = std::collections::HashMap::new();
        dict.insert(StringName::new("key"), Variant::Int(7));
        let encoded = encode_variant(&Variant::Dictionary(dict.clone()));
        assert_eq!(&encoded[0..4], &[27, 0, 0, 0], "Dictionary tag");
        assert_eq!(
            decode_variant(&encoded).unwrap(),
            (Variant::Dictionary(dict), encoded.len())
        );

        let packed_i32 = Variant::PackedInt32Array(vec![1, -2, 3]);
        let encoded = encode_variant(&packed_i32);
        assert_eq!(&encoded[0..4], &[30, 0, 0, 0]);
        assert_eq!(&encoded[4..8], &[3, 0, 0, 0]);
        assert_eq!(decode_variant(&encoded).unwrap(), (packed_i32, encoded.len()));

        let packed_bytes = Variant::PackedByteArray(vec![1, 2, 3]);
        let encoded = encode_variant(&packed_bytes);
        assert_eq!(encoded.len(), 4 + 4 + 4, "one padding byte");
        assert_eq!(decode_variant(&encoded).unwrap(), (packed_bytes, encoded.len()));

        let packed_strings =
            Variant::PackedStringArray(vec![GoString::from_str("aa"), GoString::from_str("bb")]);
        let encoded = encode_variant(&packed_strings);
        assert_eq!(
            decode_variant(&encoded).unwrap(),
            (packed_strings, encoded.len())
        );

        let packed_vec3 = Variant::PackedVector3Array(vec![Vector3::ONE, Vector3::ZERO]);
        let encoded = encode_variant(&packed_vec3);
        assert_eq!(encoded.len(), 4 + 4 + 2 * 12);
        assert_eq!(decode_variant(&encoded).unwrap().0, packed_vec3);
    }

    #[test]
    fn errors() {
        assert_eq!(decode_variant(&[]).unwrap_err(), MarshallsError::InvalidData);
        assert_eq!(
            decode_variant(&[3, 0, 0, 0]).unwrap_err(),
            MarshallsError::InvalidData,
            "float header without payload"
        );
        assert_eq!(
            decode_variant(&[4, 0, 0, 0, 9, 0, 0, 0, b'a']).unwrap_err(),
            MarshallsError::InvalidData,
            "truncated string"
        );
        assert_eq!(
            decode_variant(&[24, 0, 0, 0]).unwrap_err(),
            MarshallsError::Unsupported("Object")
        );
        // A typed array header is refused.
        assert_eq!(
            decode_variant(&[28, 0, 1, 0]).unwrap_err(),
            MarshallsError::Unsupported("typed Array")
        );
    }
}
