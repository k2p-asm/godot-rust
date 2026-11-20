//! `core/crypto/crypto_core.cpp` base64 helpers (`CryptoCore::b64_*`).
//!
//! The engine delegates to mbedtls; this port implements the standard alphabet
//! directly: `b64_encode_str` (with `=` padding), `b64_encode` into a byte
//! buffer and `b64_decode`. Decoding rejects invalid characters, impossible
//! lengths and bad padding (mbedtls' behaviour for strict inputs); it accepts
//! both padded and, like mbedtls, the unpadded tail forms that still map to whole
//! bytes.

const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

fn value_of(byte: u8) -> Option<u8> {
    match byte {
        b'A'..=b'Z' => Some(byte - b'A'),
        b'a'..=b'z' => Some(byte - b'a' + 26),
        b'0'..=b'9' => Some(byte - b'0' + 52),
        b'+' => Some(62),
        b'/' => Some(63),
        _ => None,
    }
}

/// `CryptoCore::b64_encode_str`.
pub fn b64_encode_str(source: &[u8]) -> String {
    let encoded = b64_encode(source);
    String::from_utf8_lossy(&encoded).into_owned()
}

/// `CryptoCore::b64_encode`: the padded encoding as bytes.
pub fn b64_encode(source: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity((source.len() + 2) / 3 * 4);
    for chunk in source.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = chunk.get(1).copied().unwrap_or(0) as u32;
        let b2 = chunk.get(2).copied().unwrap_or(0) as u32;
        let triple = (b0 << 16) | (b1 << 8) | b2;
        out.push(ALPHABET[(triple >> 18) as usize & 0x3f]);
        out.push(ALPHABET[(triple >> 12) as usize & 0x3f]);
        if chunk.len() > 1 {
            out.push(ALPHABET[(triple >> 6) as usize & 0x3f]);
        } else {
            out.push(b'=');
        }
        if chunk.len() > 2 {
            out.push(ALPHABET[triple as usize & 0x3f]);
        } else {
            out.push(b'=');
        }
    }
    out
}

/// `CryptoCore::b64_decode`: `None` mirrors the engine's `FAILED`.
pub fn b64_decode(source: &[u8]) -> Option<Vec<u8>> {
    // Trailing '=' padding determines how many tail characters are dropped.
    let mut data_len = source.len();
    while data_len > 0 && source[data_len - 1] == b'=' {
        data_len -= 1;
    }
    let padding = source.len() - data_len;
    if padding > 2 {
        return None;
    }
    let remainder = data_len % 4;
    if remainder == 1 {
        return None;
    }
    // Padding must complete the final quantum exactly.
    if padding > 0 && (data_len + padding) % 4 != 0 {
        return None;
    }

    let mut out = Vec::with_capacity(data_len * 3 / 4);
    let mut accumulator = 0u32;
    let mut bits = 0u32;
    for (index, byte) in source[..data_len].iter().enumerate() {
        let value = value_of(*byte)? as u32;
        accumulator = (accumulator << 6) | value;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((accumulator >> bits) as u8);
            accumulator &= (1 << bits) - 1;
        }
        // Bits are only ignored in the final, incomplete quantum.
        let _ = index;
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rfc_4648_vectors() {
        assert_eq!(b64_encode_str(b""), "");
        assert_eq!(b64_encode_str(b"f"), "Zg==");
        assert_eq!(b64_encode_str(b"fo"), "Zm8=");
        assert_eq!(b64_encode_str(b"foo"), "Zm9v");
        assert_eq!(b64_encode_str(b"foob"), "Zm9vYg==");
        assert_eq!(b64_encode_str(b"fooba"), "Zm9vYmE=");
        assert_eq!(b64_encode_str(b"foobar"), "Zm9vYmFy");

        assert_eq!(b64_decode(b"Zg==").unwrap(), b"f".to_vec());
        assert_eq!(b64_decode(b"Zm8=").unwrap(), b"fo".to_vec());
        assert_eq!(b64_decode(b"Zm9v").unwrap(), b"foo".to_vec());
        assert_eq!(b64_decode(b"Zm9vYg==").unwrap(), b"foob".to_vec());
        assert_eq!(b64_decode(b"Zm9vYmE=").unwrap(), b"fooba".to_vec());
        assert_eq!(b64_decode(b"Zm9vYmFy").unwrap(), b"foobar".to_vec());
        assert_eq!(b64_decode(b"").unwrap(), Vec::<u8>::new());
    }

    #[test]
    fn binary_round_trip_and_alphabet_edges() {
        let data: Vec<u8> = (0..=255u8).collect();
        let encoded = b64_encode(&data);
        assert_eq!(encoded.len(), (data.len() + 2) / 3 * 4);
        assert_eq!(b64_decode(&encoded).unwrap(), data);

        // Bytes that exercise the last two alphabet characters.
        assert_eq!(b64_encode_str(&[0xfb]), "+w==");
        assert_eq!(b64_encode_str(&[0xff]), "/w==");
        assert_eq!(b64_decode(b"+w==").unwrap(), vec![0xfb]);
        assert_eq!(b64_decode(b"/w==").unwrap(), vec![0xff]);
    }

    #[test]
    fn invalid_inputs() {
        assert!(b64_decode(b"Zg=").is_none(), "pad count must complete a quantum");
        assert!(b64_decode(b"Zg===").is_none(), "too much padding");
        assert!(b64_decode(b"Z").is_none(), "single tail character");
        assert!(b64_decode(b"Zm9v!").is_none(), "invalid character");
        assert!(b64_decode(b"Zm 9v").is_none(), "whitespace is rejected");
        assert!(b64_decode(b"Zg==Zg==").is_none(), "padding inside a quantum");
        // Unpadded tails that map to whole bytes are accepted.
        assert_eq!(b64_decode(b"Zm9vYg").unwrap(), b"foob".to_vec());
        assert_eq!(b64_decode(b"Zm8").unwrap(), b"fo".to_vec());
    }
}
