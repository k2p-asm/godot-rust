//! Port of `core/templates/hashfuncs.h` + `hashfuncs.cpp`.
//!
//! These hashes are observable: `HashMap` iteration order and `Dictionary`
//! behaviour depend on them, so they must be bit-identical to the C++ version.

pub const HASH_MURMUR3_SEED: u32 = 0x7F07C65;

#[inline]
pub fn hash_djb2(bytes: &[u8]) -> u32 {
    let mut hash: u32 = 5381;
    for &c in bytes {
        if c == 0 {
            break;
        }
        hash = hash.wrapping_shl(5).wrapping_add(hash) ^ c as u32;
    }
    hash
}

#[inline]
pub fn hash_djb2_buffer(buff: &[u8], prev: u32) -> u32 {
    let mut hash = prev;
    for &b in buff {
        hash = hash.wrapping_shl(5).wrapping_add(hash) ^ b as u32;
    }
    hash
}

#[inline]
pub fn hash_djb2_one_32(input: u32, prev: u32) -> u32 {
    (prev.wrapping_shl(5).wrapping_add(prev)) ^ input
}

/// Thomas Wang's 64 -> 32 bit hash.
#[inline]
pub fn hash_one_uint64(int: u64) -> u32 {
    let mut v = int;
    v = (!v).wrapping_add(v << 18); // v = (v << 18) - v - 1
    v ^= v >> 31;
    v = v.wrapping_mul(21);
    v ^= v >> 11;
    v = v.wrapping_add(v << 6);
    v ^= v >> 22;
    v as u32
}

#[inline]
pub fn hash64_murmur3_64(mut key: u64, seed: u64) -> u64 {
    key ^= seed;
    key ^= key >> 33;
    key = key.wrapping_mul(0xff51afd7ed558ccd);
    key ^= key >> 33;
    key = key.wrapping_mul(0xc4ceb9fe1a85ec53);
    key ^= key >> 33;
    key
}

#[inline]
pub fn hash_murmur3_one_32(mut input: u32, seed: u32) -> u32 {
    input = input.wrapping_mul(0xcc9e2d51);
    input = input.rotate_left(15);
    input = input.wrapping_mul(0x1b873593);

    let mut h = seed ^ input;
    h = h.rotate_left(13);
    h.wrapping_mul(5).wrapping_add(0xe6546b64)
}

#[inline]
pub fn hash_murmur3_one_64(input: u64, seed: u32) -> u32 {
    let s = hash_murmur3_one_32((input & 0xFFFF_FFFF) as u32, seed);
    hash_murmur3_one_32((input >> 32) as u32, s)
}

#[inline]
pub fn hash_murmur3_one_float(input: f32, seed: u32) -> u32 {
    // Normalize +/- 0.0 and NaN so they hash the same.
    let bits = if input == 0.0 {
        0.0f32.to_bits()
    } else if input.is_nan() {
        f32::NAN.to_bits()
    } else {
        input.to_bits()
    };
    hash_murmur3_one_32(bits, seed)
}

#[inline]
pub fn hash_murmur3_one_double(input: f64, seed: u32) -> u32 {
    let bits = if input == 0.0 {
        0.0f64.to_bits()
    } else if input.is_nan() {
        f64::NAN.to_bits()
    } else {
        input.to_bits()
    };
    hash_murmur3_one_64(bits, seed)
}

#[inline]
pub fn hash_rotl32(x: u32, r: i8) -> u32 {
    x.rotate_left((r as u32) & 31)
}

#[inline]
pub fn hash_fmix32(mut h: u32) -> u32 {
    h ^= h >> 16;
    h = h.wrapping_mul(0x85ebca6b);
    h ^= h >> 13;
    h = h.wrapping_mul(0xc2b2ae35);
    h ^= h >> 16;
    h
}

/// MurmurHash3 over a byte buffer. The C++ version walks the 32-bit blocks
/// backwards, which is preserved here for identical results.
pub fn hash_murmur3_buffer(key: &[u8], seed: u32) -> u32 {
    const C1: u32 = 0xcc9e2d51;
    const C2: u32 = 0x1b873593;

    let length = key.len();
    let nblocks = length / 4;
    let mut h1 = seed;

    for i in (0..nblocks).rev() {
        let off = i * 4;
        let k1 = u32::from_le_bytes([key[off], key[off + 1], key[off + 2], key[off + 3]]);
        let k1 = k1.wrapping_mul(C1).rotate_left(15).wrapping_mul(C2);
        h1 ^= k1;
        h1 = h1.rotate_left(13);
        h1 = h1.wrapping_mul(5).wrapping_add(0xe6546b64);
    }

    let tail = &key[nblocks * 4..];
    let mut k1: u32 = 0;
    match length & 3 {
        3 => {
            k1 ^= (tail[2] as u32) << 16;
            k1 ^= (tail[1] as u32) << 8;
            k1 ^= tail[0] as u32;
            k1 = k1.wrapping_mul(C1).rotate_left(15).wrapping_mul(C2);
            h1 ^= k1;
        }
        2 => {
            k1 ^= (tail[1] as u32) << 8;
            k1 ^= tail[0] as u32;
            k1 = k1.wrapping_mul(C1).rotate_left(15).wrapping_mul(C2);
            h1 ^= k1;
        }
        1 => {
            k1 ^= tail[0] as u32;
            k1 = k1.wrapping_mul(C1).rotate_left(15).wrapping_mul(C2);
            h1 ^= k1;
        }
        _ => {}
    }

    h1 ^= length as u32;
    hash_fmix32(h1)
}

#[inline]
pub fn hash_djb2_one_float(input: f64, prev: u32) -> u32 {
    let bits = if input == 0.0 {
        0.0f64.to_bits()
    } else if input.is_nan() {
        f64::NAN.to_bits()
    } else {
        input.to_bits()
    };
    prev.wrapping_shl(5).wrapping_add(prev).wrapping_add(hash_one_uint64(bits))
}

#[inline]
pub fn hash_djb2_one_float_64(input: f64, prev: u64) -> u64 {
    let bits = if input == 0.0 {
        0.0f64.to_bits()
    } else if input.is_nan() {
        f64::NAN.to_bits()
    } else {
        input.to_bits()
    };
    prev.wrapping_shl(5).wrapping_add(prev).wrapping_add(bits)
}

#[inline]
pub fn hash_djb2_one_64(input: u64, prev: u64) -> u64 {
    prev.wrapping_shl(5).wrapping_add(prev) ^ input
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn djb2_known_values() {
        // djb2a of "" is the initial value; appending changes the hash.
        assert_eq!(hash_djb2(b""), 5381);
        assert_ne!(hash_djb2(b"a"), hash_djb2(b"ab"));
    }

    #[test]
    fn fmix_is_deterministic() {
        assert_eq!(hash_fmix32(0), 0);
        assert_eq!(hash_fmix32(0x12345678), hash_fmix32(0x12345678));
    }

    #[test]
    fn zero_and_negative_zero_hash_equal() {
        assert_eq!(
            hash_murmur3_one_float(0.0, HASH_MURMUR3_SEED),
            hash_murmur3_one_float(-0.0, HASH_MURMUR3_SEED)
        );
        assert_eq!(
            hash_murmur3_one_double(0.0, HASH_MURMUR3_SEED),
            hash_murmur3_one_double(-0.0, HASH_MURMUR3_SEED)
        );
    }
}
