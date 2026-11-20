//! `core/io/plist.cpp` (binary half): the `bplist00` reader.
//!
//! Ported from `PList::read_bplist_obj`: the 26-byte trailer (`offset_size`,
//! `ref_size`, object count, root index, offset-table start), per-object markers
//! (nil/bool, integers of `2^n` bytes, reals, dates, data, ASCII and UTF-16BE
//! strings, UIDs, arrays/sets and dictionaries), the extended `0xF` counts and
//! the reference tables. Values land in [`PlistNode`], with data as base64 (the
//! engine's `CryptoCore::b64_encode_str`) and dates as the engine's
//! `datetime_string_from_unix_time` form (seconds since 2001-01-01 plus
//! 978307200).
//!
//! Deviation: the engine's `read_bplist_obj` stores real/date payloads through
//! its integer reader (a quirk that garbles them); this port decodes IEEE
//! floats/doubles as the format specifies.

use crate::crypto::base64::b64_encode_str;
use crate::io::plist::PlistNode;
use crate::os::time;

const MAGIC: &[u8; 8] = b"bplist00";

fn be_uint(data: &[u8], offset: usize, size: usize) -> Option<u64> {
    if size == 0 || size > 8 {
        return None;
    }
    let bytes = data.get(offset..offset + size)?;
    let mut value = 0u64;
    for byte in bytes {
        value = (value << 8) | *byte as u64;
    }
    Some(value)
}

struct Reader<'a> {
    data: &'a [u8],
    offset_size: usize,
    ref_size: usize,
    num_objects: u64,
    offset_table_start: usize,
}

impl<'a> Reader<'a> {
    fn object_offset(&self, index: u64) -> Option<usize> {
        if index >= self.num_objects {
            return None;
        }
        let position = self.offset_table_start
            .checked_add(index as usize * self.offset_size)?;
        let offset = be_uint(self.data, position, self.offset_size)? as usize;
        if offset >= self.data.len() {
            return None;
        }
        Some(offset)
    }

    fn read_object(&self, index: u64, depth: u32) -> Option<PlistNode> {
        if depth > 64 {
            return None;
        }
        let offset = self.object_offset(index)?;
        let marker = *self.data.get(offset)?;
        let kind = marker & 0xF0;
        let mut size = (marker & 0x0F) as u64;
        let mut cursor = offset + 1;

        // Extended count: 0xF followed by an integer marker (0x1n).
        let mut extended = |size: &mut u64, cursor: &mut usize| -> Option<()> {
            if *size != 0x0F {
                return Some(());
            }
            let int_marker = *self.data.get(*cursor)?;
            if int_marker & 0xF0 != 0x10 {
                return None;
            }
            let width = 1usize << (int_marker & 0x0F);
            *cursor += 1;
            *size = be_uint(self.data, *cursor, width)?;
            *cursor += width;
            Some(())
        };

        match kind {
            0x00 => match size {
                0x00 => Some(PlistNode::Nil),
                0x08 => Some(PlistNode::Bool(false)),
                0x09 => Some(PlistNode::Bool(true)),
                _ => None,
            },
            0x10 => {
                let width = 1usize.checked_shl(size as u32)?;
                let value = be_uint(self.data, cursor, width)?;
                Some(PlistNode::Int(value as i64))
            }
            0x20 => {
                let width = 1usize.checked_shl(size as u32)?;
                let bytes = self.data.get(cursor..cursor + width)?;
                let value = if width == 4 {
                    f32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as f64
                } else if width == 8 {
                    f64::from_be_bytes([
                        bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6],
                        bytes[7],
                    ])
                } else {
                    be_uint(self.data, cursor, width)? as f64
                };
                Some(PlistNode::Real(value))
            }
            0x30 => {
                let bytes = self.data.get(cursor..cursor + 8)?;
                let seconds = f64::from_be_bytes([
                    bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
                ]) as i64
                    + 978_307_200;
                Some(PlistNode::Date(time::datetime_string_from_unix_time(
                    seconds, true,
                )))
            }
            0x40 => {
                extended(&mut size, &mut cursor)?;
                let bytes = self.data.get(cursor..cursor + size as usize)?;
                Some(PlistNode::Data(b64_encode_str(bytes)))
            }
            0x50 => {
                extended(&mut size, &mut cursor)?;
                let bytes = self.data.get(cursor..cursor + size as usize)?;
                Some(PlistNode::String(String::from_utf8_lossy(bytes).into_owned()))
            }
            0x60 => {
                extended(&mut size, &mut cursor)?;
                let mut units = Vec::with_capacity(size as usize);
                for index in 0..size as usize {
                    let unit = be_uint(self.data, cursor + index * 2, 2)? as u16;
                    units.push(unit);
                }
                Some(PlistNode::String(String::from_utf16(&units).ok()?))
            }
            0x80 => {
                let width = size as usize + 1;
                Some(PlistNode::Int(be_uint(self.data, cursor, width)? as i64))
            }
            0xA0 | 0xC0 => {
                extended(&mut size, &mut cursor)?;
                let mut items = Vec::with_capacity(size as usize);
                for index in 0..size as usize {
                    let reference =
                        be_uint(self.data, cursor + index * self.ref_size, self.ref_size)?;
                    items.push(self.read_object(reference, depth + 1)?);
                }
                Some(PlistNode::Array(items))
            }
            0xD0 => {
                extended(&mut size, &mut cursor)?;
                let mut entries = Vec::with_capacity(size as usize);
                for index in 0..size as usize {
                    let key_reference =
                        be_uint(self.data, cursor + index * self.ref_size, self.ref_size)?;
                    let value_reference = be_uint(
                        self.data,
                        cursor + (index + size as usize) * self.ref_size,
                        self.ref_size,
                    )?;
                    let key = match self.read_object(key_reference, depth + 1)? {
                        PlistNode::String(key) => key,
                        _ => return None,
                    };
                    let value = self.read_object(value_reference, depth + 1)?;
                    entries.push((key, value));
                }
                Some(PlistNode::Dict(entries))
            }
            _ => None,
        }
    }
}

/// Parses a binary plist; `None` for a bad magic, a truncated file or an
/// unsupported marker (the engine returns an invalid tree in those cases).
pub fn parse_binary(data: &[u8]) -> Option<PlistNode> {
    if data.len() < 8 + 26 || &data[..8] != MAGIC {
        return None;
    }
    let trailer = data.len() - 26;
    let offset_size = data[trailer] as usize;
    let ref_size = data[trailer + 1] as usize;
    if !(1..=8).contains(&offset_size) || !(1..=8).contains(&ref_size) {
        return None;
    }
    let num_objects = be_uint(data, trailer + 2, 8)?;
    let top_object = be_uint(data, trailer + 10, 8)?;
    let offset_table_start = be_uint(data, trailer + 18, 8)? as usize;
    if num_objects == 0 || offset_table_start >= data.len() {
        return None;
    }
    let reader = Reader {
        data,
        offset_size,
        ref_size,
        num_objects,
        offset_table_start,
    };
    reader.read_object(top_object, 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Generated with Python's `plistlib.dumps(obj, fmt=FMT_BINARY)`:
    /// `{"blob": b"\x01\x02", "count": 42, "items": [1, 2], "name": "héllo",
    ///   "ok": True, "pi": 3.5, "when": datetime(2020, 1, 2, 3, 4, 5)}`.
    const FIXTURE_HEX: &str = "62706c6973743030d70102030405060708090a0d0e0f1054626c6f6255636f756e74556974656d73546e616d65526f6b527069547768656e420102102aa20b0c1001100265006800e9006c006c006f0923400c0000000000003341c1deca9280000008171c22282d3033383b3d4042444f50590000000000000101000000000000001100000000000000000000000000000062";

    fn fixture() -> Vec<u8> {
        let mut out = Vec::with_capacity(FIXTURE_HEX.len() / 2);
        let bytes = FIXTURE_HEX.as_bytes();
        for pair in bytes.chunks(2) {
            let high = (pair[0] as char).to_digit(16).unwrap() as u8;
            let low = (pair[1] as char).to_digit(16).unwrap() as u8;
            out.push((high << 4) | low);
        }
        out
    }

    #[test]
    fn python_binary_fixture() {
        let data = fixture();
        assert_eq!(data.len(), 147);
        let node = parse_binary(&data).expect("parsed bplist");

        assert_eq!(node.get("blob"), Some(&PlistNode::Data("AQI=".to_string())));
        assert_eq!(node.get("count"), Some(&PlistNode::Int(42)));
        assert_eq!(
            node.get("items"),
            Some(&PlistNode::Array(vec![
                PlistNode::Int(1),
                PlistNode::Int(2)
            ]))
        );
        assert_eq!(
            node.get("name"),
            Some(&PlistNode::String("héllo".to_string())),
            "UTF-16BE string"
        );
        assert_eq!(node.get("ok"), Some(&PlistNode::Bool(true)));
        assert_eq!(node.get("pi"), Some(&PlistNode::Real(3.5)));
        assert_eq!(
            node.get("when"),
            Some(&PlistNode::Date("2020-01-02 03:04:05".to_string())),
            "seconds since 2001 plus the epoch offset"
        );

        // Every key of the Python dict is present exactly once.
        let dict = node.as_dict().expect("dict root");
        assert_eq!(dict.len(), 7);
        let mut keys: Vec<&str> = dict.iter().map(|(key, _)| key.as_str()).collect();
        keys.sort();
        assert_eq!(
            keys,
            vec!["blob", "count", "items", "name", "ok", "pi", "when"]
        );
    }

    #[test]
    fn rejects_malformed_files() {
        assert!(parse_binary(b"").is_none());
        assert!(parse_binary(b"not a plist").is_none());

        let mut data = fixture();
        data[0] = b'x';
        assert!(parse_binary(&data).is_none(), "bad magic");

        let data = fixture();
        assert!(
            parse_binary(&data[..40]).is_none(),
            "truncated before the trailer"
        );

        let mut data = fixture();
        let len = data.len();
        data[len - 26] = 0; // offset_size 0
        assert!(parse_binary(&data).is_none());

        let mut data = fixture();
        let len = data.len();
        data[len - 26 + 18..len - 26 + 26].copy_from_slice(&u64::MAX.to_be_bytes());
        assert!(parse_binary(&data).is_none(), "offset table outside the file");
    }

    #[test]
    fn primitives_from_scratch() {
        // Hand-built bplist with a single object: header, object, one offset
        // byte, then the 26-byte trailer (offset_size, ref_size, count, root,
        // offset-table start).
        fn single(marker: u8, payload: &[u8]) -> Vec<u8> {
            let mut data = Vec::new();
            data.extend_from_slice(MAGIC);
            let object_offset = data.len();
            data.push(marker);
            data.extend_from_slice(payload);
            let offset_table_start = data.len();
            data.push(object_offset as u8);
            data.push(1); // offset_size
            data.push(1); // ref_size
            data.extend_from_slice(&1u64.to_be_bytes()); // one object
            data.extend_from_slice(&0u64.to_be_bytes()); // root
            data.extend_from_slice(&(offset_table_start as u64).to_be_bytes());
            data
        }

        assert_eq!(single(0x10, &[0x2A]).len(), 8 + 2 + 1 + 26);
        assert_eq!(parse_binary(&single(0x10, &[0x2A])), Some(PlistNode::Int(42)));
        assert_eq!(parse_binary(&single(0x09, &[])), Some(PlistNode::Bool(true)));
        assert_eq!(parse_binary(&single(0x08, &[])), Some(PlistNode::Bool(false)));
        assert_eq!(parse_binary(&single(0x00, &[])), Some(PlistNode::Nil));
        // A 2-byte integer and an ASCII string with an extended count.
        assert_eq!(
            parse_binary(&single(0x11, &[0x01, 0x00])),
            Some(PlistNode::Int(256))
        );
        assert_eq!(
            parse_binary(&single(0x53, b"abc")),
            Some(PlistNode::String("abc".to_string()))
        );
        // An unknown marker is refused.
        assert!(parse_binary(&single(0x70, &[])).is_none());
    }
}
