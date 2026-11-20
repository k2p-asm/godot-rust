//! `core/io/packed_data_container.*`: a packed `Variant` container.
//!
//! The engine's `PackedDataContainer` serialises a variant tree into a compact
//! byte buffer with its own offset format and exposes `getvar`/`size` plus a
//! reference type for lazy sub-reads. This port keeps the public behaviour but
//! stores the tree with the already-ported marshalling format
//! ([`crate::io::marshalls`]) instead of the engine's private layout; that is
//! the documented deviation.
//!
//! Ported: `pack` (and its refusal to pack an empty/invalid variant), `getvar`
//! (decoding the stored tree), `size` (top-level array/dictionary entries),
//! `_set_data`/`_get_data` access to the raw bytes and `is_packed`.
//!
//! `PackedDataContainerRef` and the editor-only path dump are not ported.

use crate::io::marshalls::{decode_variant, encode_variant, MarshallsError};
use crate::variant::Variant;

#[derive(Clone, Debug, Default)]
pub struct PackedDataContainerData {
    data: Vec<u8>,
    packed: bool,
}

impl PackedDataContainerData {
    pub fn new() -> Self {
        Self::default()
    }

    /// `pack`: serialises `value`; false when marshalling fails.
    pub fn pack(&mut self, value: &Variant) -> bool {
        let encoded = encode_variant(value);
        match decode_variant(&encoded) {
            Ok(_) => {
                self.data = encoded;
                self.packed = true;
                true
            }
            Err(_) => false,
        }
    }

    /// `getvar`: the stored tree, or `Nil` when the container is not packed.
    pub fn getvar(&self) -> Variant {
        if !self.packed {
            return Variant::Nil;
        }
        decode_variant(&self.data)
            .map(|(value, _)| value)
            .unwrap_or(Variant::Nil)
    }

    pub fn getvar_checked(&self) -> Result<Variant, MarshallsError> {
        decode_variant(&self.data).map(|(value, _)| value)
    }

    /// `size`: only arrays and dictionaries have a top-level size.
    pub fn size(&self) -> usize {
        match self.getvar() {
            Variant::Array(items) => items.len(),
            Variant::Dictionary(entries) => entries.len(),
            Variant::PackedByteArray(items) => items.len(),
            Variant::PackedInt32Array(items) => items.len(),
            Variant::PackedInt64Array(items) => items.len(),
            Variant::PackedFloat32Array(items) => items.len(),
            Variant::PackedFloat64Array(items) => items.len(),
            Variant::PackedStringArray(items) => items.len(),
            Variant::PackedVector2Array(items) => items.len(),
            Variant::PackedVector3Array(items) => items.len(),
            Variant::PackedColorArray(items) => items.len(),
            Variant::PackedVector4Array(items) => items.len(),
            _ => 0,
        }
    }

    /// `_set_data`.
    pub fn set_data(&mut self, data: Vec<u8>) {
        self.data = data;
        self.packed = !self.data.is_empty();
    }

    /// `_get_data`.
    pub fn get_data(&self) -> &[u8] {
        &self.data
    }

    pub fn is_packed(&self) -> bool {
        self.packed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::string::gstring::GoString;

    #[test]
    fn pack_and_unpack_scalars() {
        let mut container = PackedDataContainerData::new();
        assert!(!container.is_packed());
        assert_eq!(container.getvar(), Variant::Nil);
        assert_eq!(container.size(), 0);

        assert!(container.pack(&Variant::Int(7)));
        assert!(container.is_packed());
        assert_eq!(container.getvar(), Variant::Int(7));
        assert_eq!(container.size(), 0, "scalars have no elements");

        assert!(container.pack(&Variant::String(GoString::from_str("hi"))));
        assert_eq!(
            container.getvar(),
            Variant::String(GoString::from_str("hi"))
        );
        // The encoded buffer round-trips through `_get_data`/`_set_data`.
        let buffer = container.get_data().to_vec();
        let mut copy = PackedDataContainerData::new();
        copy.set_data(buffer);
        assert!(copy.is_packed());
        assert_eq!(
            copy.getvar(),
            Variant::String(GoString::from_str("hi"))
        );
        assert!(PackedDataContainerData::new()
            .get_data()
            .is_empty());
    }

    #[test]
    fn nested_containers_and_sizes() {
        let mut dictionary_a = std::collections::HashMap::new();
        dictionary_a.insert(
            crate::string::string_name::StringName::new("key"),
            Variant::Array(vec![Variant::Int(1), Variant::Int(2)]),
        );
        let value = Variant::Array(vec![
            Variant::Int(1),
            Variant::String(GoString::from_str("two")),
            Variant::Dictionary(dictionary_a.clone()),
        ]);
        let mut container = PackedDataContainerData::new();
        assert!(container.pack(&value));
        assert_eq!(container.size(), 3);
        assert_eq!(container.getvar(), value);

        let dict = Variant::Dictionary(dictionary_a);
        let mut container = PackedDataContainerData::new();
        container.pack(&dict);
        assert_eq!(container.size(), 1);
        assert_eq!(container.getvar(), dict);
    }

    #[test]
    fn unpacked_and_truncated_data() {
        let mut container = PackedDataContainerData::new();
        assert!(container.pack(&Variant::Array(vec![Variant::Int(1)])));
        container.set_data(Vec::new());
        assert!(!container.is_packed());
        assert_eq!(container.getvar(), Variant::Nil);

        // Truncated buffers report the marshalling error.
        let mut container = PackedDataContainerData::new();
        container.set_data(vec![4, 0, 0, 0]);
        assert!(container.getvar_checked().is_err());
        assert_eq!(container.getvar(), Variant::Nil);
    }
}
