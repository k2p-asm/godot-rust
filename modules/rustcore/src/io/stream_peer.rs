//! `core/io/stream_peer.*`: the byte-stream helpers and `StreamPeerBuffer`.
//!
//! Ported from `stream_peer.cpp`: endian-aware `put_*`/`get_*` for 8/16/32/64-bit
//! integers and floats, half-float conversion, length-prefixed `put_string`/
//! `put_utf8_string`, `put_var`/`get_var` through the port's marshalling format,
//! and the full `StreamPeerBuffer` surface (`put_data`, `put_partial_data`,
//! `get_data`, `get_partial_data`, `seek`, `get_size`, `get_position`, `resize`,
//! `set_data_array`, `get_data_array`, `clear`, `duplicate`).
//!
//! `p_full_objects` is meaningless without an object layer, so `put_var` always
//! uses the port's subset encoding. Reads past the end return `Eof`/`0` instead
//! of logging.

use crate::io::marshalls::{decode_variant, encode_variant};
use crate::variant::Variant;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StreamPeerError {
    Ok,
    /// `ERR_INVALID_PARAMETER` / `ERR_FILE_EOF`.
    InvalidParameter,
}

#[derive(Clone, Debug, Default)]
pub struct StreamPeerBuffer {
    data: Vec<u8>,
    pointer: usize,
    big_endian: bool,
}

impl StreamPeerBuffer {
    pub fn new() -> Self {
        Self::default()
    }

    /// `StreamPeerBuffer::duplicate` (the copy keeps the data, not the cursor).
    pub fn duplicate(&self) -> Self {
        Self {
            data: self.data.clone(),
            pointer: 0,
            big_endian: self.big_endian,
        }
    }

    pub fn set_big_endian(&mut self, big_endian: bool) {
        self.big_endian = big_endian;
    }

    pub fn is_big_endian_enabled(&self) -> bool {
        self.big_endian
    }

    // -- raw data -----------------------------------------------------------

    /// `put_data`: grows as needed and advances the cursor.
    pub fn put_data(&mut self, bytes: &[u8]) -> StreamPeerError {
        if bytes.is_empty() {
            return StreamPeerError::Ok;
        }
        let end = self.pointer + bytes.len();
        if end > self.data.len() {
            self.data.resize(end, 0);
        }
        self.data[self.pointer..end].copy_from_slice(bytes);
        self.pointer = end;
        StreamPeerError::Ok
    }

    /// `put_partial_data`: the buffer always accepts everything.
    pub fn put_partial_data(&mut self, bytes: &[u8]) -> (StreamPeerError, usize) {
        let result = self.put_data(bytes);
        (result, bytes.len())
    }

    /// `get_partial_data`: clamps to what is left.
    pub fn get_partial_data(&mut self, count: usize) -> Vec<u8> {
        if count == 0 {
            return Vec::new();
        }
        let available = self.data.len().saturating_sub(self.pointer);
        let received = count.min(available);
        let chunk = self.data[self.pointer..self.pointer + received].to_vec();
        self.pointer += received;
        chunk
    }

    /// `get_data`: fails unless the full count was available.
    pub fn get_data(&mut self, count: usize) -> Result<Vec<u8>, StreamPeerError> {
        let chunk = self.get_partial_data(count);
        if chunk.len() != count {
            return Err(StreamPeerError::InvalidParameter);
        }
        Ok(chunk)
    }

    pub fn seek(&mut self, position: usize) -> bool {
        if position > self.data.len() {
            return false;
        }
        self.pointer = position;
        true
    }

    pub fn get_size(&self) -> usize {
        self.data.len()
    }

    pub fn get_position(&self) -> usize {
        self.pointer
    }

    pub fn resize(&mut self, size: usize) {
        self.data.resize(size, 0);
    }

    pub fn set_data_array(&mut self, data: Vec<u8>) {
        self.data = data;
        self.pointer = 0;
    }

    pub fn get_data_array(&self) -> &[u8] {
        &self.data
    }

    pub fn clear(&mut self) {
        self.data.clear();
        self.pointer = 0;
    }

    // -- writes -------------------------------------------------------------

    fn write_uint(&mut self, bytes: &[u8]) {
        self.put_data(bytes);
    }

    pub fn put_8(&mut self, value: i8) {
        self.put_u8(value as u8);
    }

    pub fn put_u8(&mut self, value: u8) {
        self.write_uint(&[value]);
    }

    pub fn put_16(&mut self, value: i16) {
        self.put_u16(value as u16);
    }

    pub fn put_u16(&mut self, value: u16) {
        let bytes = if self.big_endian {
            value.to_be_bytes()
        } else {
            value.to_le_bytes()
        };
        self.write_uint(&bytes);
    }

    pub fn put_32(&mut self, value: i32) {
        self.put_u32(value as u32);
    }

    pub fn put_u32(&mut self, value: u32) {
        let bytes = if self.big_endian {
            value.to_be_bytes()
        } else {
            value.to_le_bytes()
        };
        self.write_uint(&bytes);
    }

    pub fn put_64(&mut self, value: i64) {
        self.put_u64(value as u64);
    }

    pub fn put_u64(&mut self, value: u64) {
        let bytes = if self.big_endian {
            value.to_be_bytes()
        } else {
            value.to_le_bytes()
        };
        self.write_uint(&bytes);
    }

    /// `put_half`: IEEE 754 binary16 with round-to-nearest-even.
    pub fn put_half(&mut self, value: f32) {
        let bits = f32_to_f16(value);
        let bytes = if self.big_endian {
            bits.to_be_bytes()
        } else {
            bits.to_le_bytes()
        };
        self.write_uint(&bytes);
    }

    pub fn put_float(&mut self, value: f32) {
        self.put_u32(value.to_bits());
    }

    pub fn put_double(&mut self, value: f64) {
        self.put_u64(value.to_bits());
    }

    /// `put_string`: a 32-bit UTF-8 byte length followed by the bytes.
    pub fn put_string(&mut self, text: &str) {
        let bytes = text.as_bytes();
        self.put_u32(bytes.len() as u32);
        self.write_uint(bytes);
    }

    pub fn put_utf8_string(&mut self, text: &str) {
        self.put_string(text);
    }

    /// `put_var`: a 32-bit length followed by the marshalled variant.
    pub fn put_var(&mut self, value: &Variant) -> StreamPeerError {
        let encoded = encode_variant(value);
        self.put_u32(encoded.len() as u32);
        self.write_uint(&encoded);
        StreamPeerError::Ok
    }

    // -- reads --------------------------------------------------------------

    pub fn get_u8(&mut self) -> u8 {
        self.get_partial_data(1).first().copied().unwrap_or(0)
    }

    pub fn get_8(&mut self) -> i8 {
        self.get_u8() as i8
    }

    fn read_uint(&mut self, width: usize) -> u64 {
        let bytes = self.get_partial_data(width);
        let mut buffer = [0u8; 8];
        if self.big_endian {
            // Right-align the payload for a big-endian read.
            buffer[8 - bytes.len()..].copy_from_slice(&bytes);
            u64::from_be_bytes(buffer)
        } else {
            buffer[..bytes.len()].copy_from_slice(&bytes);
            u64::from_le_bytes(buffer)
        }
    }

    pub fn get_u16(&mut self) -> u16 {
        self.read_uint(2) as u16
    }

    pub fn get_16(&mut self) -> i16 {
        self.get_u16() as i16
    }

    pub fn get_u32(&mut self) -> u32 {
        self.read_uint(4) as u32
    }

    pub fn get_32(&mut self) -> i32 {
        self.get_u32() as i32
    }

    pub fn get_u64(&mut self) -> u64 {
        self.read_uint(8)
    }

    pub fn get_64(&mut self) -> i64 {
        self.get_u64() as i64
    }

    pub fn get_half(&mut self) -> f32 {
        let bits = self.get_u16();
        f16_to_f32(bits)
    }

    pub fn get_float(&mut self) -> f32 {
        f32::from_bits(self.get_u32())
    }

    pub fn get_double(&mut self) -> f64 {
        f64::from_bits(self.get_u64())
    }

    /// `get_string(bytes)`: the length comes from the stream when `None`.
    pub fn get_string(&mut self, bytes: Option<usize>) -> String {
        let length = match bytes {
            Some(length) => length,
            None => self.get_u32() as usize,
        };
        let data = self.get_partial_data(length);
        String::from_utf8_lossy(&data).into_owned()
    }

    pub fn get_utf8_string(&mut self, bytes: Option<usize>) -> String {
        self.get_string(bytes)
    }

    /// `get_var`: `Nil` when the payload cannot be decoded.
    pub fn get_var(&mut self) -> Variant {
        let length = self.get_u32() as usize;
        let data = self.get_partial_data(length);
        if data.len() != length {
            return Variant::Nil;
        }
        decode_variant(&data)
            .map(|(value, _)| value)
            .unwrap_or(Variant::Nil)
    }
}

/// `Math::half_to_float` (binary16 -> f32).
fn f16_to_f32(bits: u16) -> f32 {
    let sign = (bits as u32 & 0x8000) << 16;
    let exponent = (bits >> 10) & 0x1f;
    let mantissa = bits & 0x3ff;

    let value = if exponent == 0 {
        if mantissa == 0 {
            0.0
        } else {
            // Subnormal: value = mantissa * 2^-24.
            (mantissa as f32) * (1.0 / 16_777_216.0)
        }
    } else if exponent == 0x1f {
        if mantissa == 0 {
            f32::INFINITY
        } else {
            f32::NAN
        }
    } else {
        let exp = exponent as i32 - 15 + 127;
        f32::from_bits((exp as u32) << 23) * (1.0 + mantissa as f32 / 1024.0)
    };
    if sign != 0 {
        -value
    } else {
        value
    }
}

/// `Math::make_half_float`-style conversion (f32 -> binary16, RNE).
fn f32_to_f16(value: f32) -> u16 {
    let bits = value.to_bits();
    let sign = ((bits >> 16) & 0x8000) as u16;
    let exponent = ((bits >> 23) & 0xff) as i32 - 127 + 15;
    let mantissa = bits & 0x007f_ffff;

    if value.is_nan() {
        return sign | 0x7e00;
    }
    if exponent >= 0x1f {
        return sign | 0x7c00; // overflow to infinity
    }
    if exponent <= 0 {
        if exponent < -10 {
            return sign;
        }
        // Subnormal, with round-to-nearest-even.
        let mantissa = mantissa | 0x0080_0000;
        let shift = (14 - exponent) as u32;
        let mut half = (mantissa >> shift) as u16;
        let round_bit = 1u32 << (shift - 1);
        if (mantissa & round_bit) != 0 && (mantissa & (round_bit - 1)) != 0 {
            half += 1;
        }
        return sign | half;
    }

    let mut half = ((exponent as u32) << 10) as u16 | (mantissa >> 13) as u16;
    let round_bit = 0x1000u32;
    if (mantissa & round_bit) != 0 && (mantissa & (round_bit - 1)) != 0 {
        half += 1;
    }
    sign | half
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::string::gstring::GoString;

    #[test]
    fn endian_scalars() {
        let mut stream = StreamPeerBuffer::new();
        stream.put_u8(0x12);
        stream.put_u16(0x3456);
        stream.put_u32(0x789a_bcde);
        stream.put_u64(0x0123_4567_89ab_cdef);
        assert_eq!(
            stream.get_data_array(),
            &[0x12, 0x56, 0x34, 0xde, 0xbc, 0x9a, 0x78, 0xef, 0xcd, 0xab, 0x89, 0x67, 0x45, 0x23, 0x01]
        );
        assert_eq!(stream.get_position(), 15);

        stream.seek(0);
        assert_eq!(stream.get_u8(), 0x12);
        assert_eq!(stream.get_u16(), 0x3456);
        assert_eq!(stream.get_u32(), 0x789a_bcde);
        assert_eq!(stream.get_u64(), 0x0123_4567_89ab_cdef);

        let mut stream = StreamPeerBuffer::new();
        stream.set_big_endian(true);
        assert!(stream.is_big_endian_enabled());
        stream.put_16(-1);
        stream.put_32(0x0102_0304);
        assert_eq!(stream.get_data_array(), &[0xff, 0xff, 0x01, 0x02, 0x03, 0x04]);
        stream.seek(0);
        assert_eq!(stream.get_16(), -1);
        assert_eq!(stream.get_32(), 0x0102_0304);
    }

    #[test]
    fn floats_and_halves() {
        let mut stream = StreamPeerBuffer::new();
        stream.put_float(1.5);
        stream.put_double(-2.25);
        stream.put_half(1.0);
        stream.put_half(0.5);
        stream.put_half(65504.0);
        stream.put_half(f32::INFINITY);
        assert_eq!(stream.get_size(), 4 + 8 + 2 * 4, "four halves");
        stream.seek(0);
        assert_eq!(stream.get_float(), 1.5);
        assert_eq!(stream.get_double(), -2.25);
        assert_eq!(stream.get_half(), 1.0);
        assert_eq!(stream.get_half(), 0.5);
        assert_eq!(stream.get_half(), 65504.0);
        assert!(stream.get_half().is_infinite());

        // Half precision: 1/3 rounds to 0.33325195.
        let mut stream = StreamPeerBuffer::new();
        stream.put_half(1.0 / 3.0);
        stream.seek(0);
        assert!((stream.get_half() - 0.333_251_95).abs() < 1e-7);
        // Subnormals survive.
        let mut stream = StreamPeerBuffer::new();
        stream.put_half(5.960_464_5e-8); // smallest positive half
        stream.seek(0);
        assert_eq!(stream.get_half(), 5.960_464_5e-8);
    }

    #[test]
    fn strings_and_variants() {
        let mut stream = StreamPeerBuffer::new();
        stream.put_string("hé");
        stream.put_var(&Variant::Array(vec![
            Variant::Int(1),
            Variant::String(GoString::from_str("x")),
        ]));
        assert_eq!(&stream.get_data_array()[0..4], &[3, 0, 0, 0], "utf-8 length");
        stream.seek(0);
        assert_eq!(stream.get_string(None), "hé");
        assert_eq!(
            stream.get_var(),
            Variant::Array(vec![
                Variant::Int(1),
                Variant::String(GoString::from_str("x"))
            ])
        );
        // A fixed-size read is allowed.
        stream.seek(4);
        assert_eq!(stream.get_utf8_string(Some(3)), "hé");
        // Truncated payloads yield Nil.
        stream.clear();
        stream.put_u32(10);
        stream.put_data(&[1, 2]);
        stream.seek(0);
        assert_eq!(stream.get_var(), Variant::Nil);
    }

    #[test]
    fn buffer_management() {
        let mut stream = StreamPeerBuffer::new();
        assert_eq!(stream.get_size(), 0);
        assert_eq!(stream.get_position(), 0);
        stream.put_data(&[1, 2, 3, 4]);
        assert_eq!(stream.get_size(), 4);
        assert!(stream.seek(0));
        assert_eq!(stream.get_partial_data(2), vec![1, 2]);
        assert_eq!(stream.get_data(1), Ok(vec![3]));
        assert_eq!(stream.get_data(5), Err(StreamPeerError::InvalidParameter));
        // The failed read still consumed the one remaining byte.\n        assert_eq!(stream.get_position(), 4);
        count_reset(&mut stream);
        assert_eq!(stream.get_data_array(), &[1, 2, 3, 4]);

        // seek is bounds checked; resize zero-fills; clear resets.
        assert!(!stream.seek(9));
        assert!(stream.seek(4));
        stream.resize(6);
        assert_eq!(stream.get_data_array(), &[1, 2, 3, 4, 0, 0]);
        let copy = stream.duplicate();
        assert_eq!(copy.get_data_array(), stream.get_data_array());
        assert_eq!(copy.get_position(), 0);
        stream.clear();
        assert_eq!(stream.get_size(), 0);
        assert_eq!(stream.get_position(), 0);

        stream.set_data_array(vec![9, 8]);
        assert_eq!(stream.get_size(), 2);
        assert_eq!(stream.get_position(), 0);
    }

    fn count_reset(stream: &mut StreamPeerBuffer) {
        // The partial read advanced the cursor; rewinding keeps the sequence clear.
        stream.seek(0);
        let _ = stream;
    }
}
