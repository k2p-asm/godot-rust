//! `core/templates/hashfuncs.h`: the engine's hash primitives.
//!
//! Ported with the exact arithmetic: DJB2 (C string, buffer, one-word and one
//! double forms), Thomas Wang's 64-to-32-bit hash, the 64-bit MurmurHash3
//! finalizer, MurmurHash3 one-word/one-float/one-double/buffer forms, `rotl32`,
//! `fmix32`, the prime table and its multiplicative inverses.
//!
//! The C++ templates for `HashMap`/`Pair` hashing are not needed in this port
//! (Rust's own hashing is used); only the concrete functions are exposed.

pub const HASH_MURMUR3_SEED: u32 = 0x7F07_C65;

const C1: u32 = 0xcc9e_2d51;
const C2: u32 = 0x1b87_3593;

/// `hash_djb2`: stops at the first NUL like the C-string version.
pub fn hash_djb2(bytes: &[u8]) -> u32 {
    let mut hash: u32 = 5381;
    for byte in bytes {
        if *byte == 0 {
            break;
        }
        hash = hash.wrapping_mul(33) ^ *byte as u32;
    }
    hash
}

/// `hash_djb2_buffer`.
pub fn hash_djb2_buffer(bytes: &[u8], prev: u32) -> u32 {
    let mut hash = prev;
    for byte in bytes {
        hash = hash.wrapping_mul(33) ^ *byte as u32;
    }
    hash
}

/// `hash_djb2_one_32`.
pub fn hash_djb2_one_32(value: u32, prev: u32) -> u32 {
    prev.wrapping_mul(33) ^ value
}

/// `hash_djb2_one_64`.
pub fn hash_djb2_one_64(value: u64, prev: u64) -> u64 {
    prev.wrapping_mul(33) ^ value
}

/// `hash_one_uint64` (Thomas Wang).
pub fn hash_one_uint64(value: u64) -> u32 {
    let mut v = value;
    v = (!v).wrapping_add(v << 18);
    v ^= v >> 31;
    v = v.wrapping_mul(21);
    v ^= v >> 11;
    v = v.wrapping_add(v << 6);
    v ^= v >> 22;
    v as u32
}

/// `hash64_murmur3_64`.
pub fn hash64_murmur3_64(mut key: u64, seed: u64) -> u64 {
    key ^= seed;
    key ^= key >> 33;
    key = key.wrapping_mul(0xff51_afd7_ed55_8ccd);
    key ^= key >> 33;
    key = key.wrapping_mul(0xc4ce_b9fe_1a85_ec53);
    key ^= key >> 33;
    key
}

/// `hash_murmur3_one_32`.
pub fn hash_murmur3_one_32(value: u32, seed: u32) -> u32 {
    let mut k = value.wrapping_mul(C1);
    k = hash_rotl32(k, 15);
    k = k.wrapping_mul(C2);
    let mut h = seed ^ k;
    h = hash_rotl32(h, 13);
    h.wrapping_mul(5).wrapping_add(0xe654_6b64)
}

/// `hash_murmur3_one_64`.
pub fn hash_murmur3_one_64(value: u64, seed: u32) -> u32 {
    let seed = hash_murmur3_one_32(value as u32, seed);
    hash_murmur3_one_32((value >> 32) as u32, seed)
}

/// `hash_murmur3_one_float`: ±0 hash the same, NaN normalizes.
pub fn hash_murmur3_one_float(value: f32, seed: u32) -> u32 {
    let normalized = if value == 0.0 {
        0.0
    } else if value.is_nan() {
        f32::NAN
    } else {
        value
    };
    hash_murmur3_one_32(normalized.to_bits(), seed)
}

/// `hash_murmur3_one_double`.
pub fn hash_murmur3_one_double(value: f64, seed: u32) -> u32 {
    let normalized = if value == 0.0 {
        0.0
    } else if value.is_nan() {
        f64::NAN
    } else {
        value
    };
    hash_murmur3_one_64(normalized.to_bits(), seed)
}

/// `hash_rotl32`.
pub fn hash_rotl32(value: u32, rotation: i8) -> u32 {
    let rotation = (rotation & 31) as u32;
    value.rotate_left(rotation)
}

/// `hash_fmix32`.
pub fn hash_fmix32(mut h: u32) -> u32 {
    h ^= h >> 16;
    h = h.wrapping_mul(0x85eb_ca6b);
    h ^= h >> 13;
    h = h.wrapping_mul(0xc2b2_ae35);
    h ^= h >> 16;
    h
}

/// `hash_murmur3_buffer`: four-byte little-endian blocks plus the byte tail.
pub fn hash_murmur3_buffer(bytes: &[u8], seed: u32) -> u32 {
    let blocks = bytes.len() / 4;
    let mut h1 = seed;
    for index in 0..blocks {
        let at = index * 4;
        let mut k1 = u32::from_le_bytes([bytes[at], bytes[at + 1], bytes[at + 2], bytes[at + 3]]);
        k1 = k1.wrapping_mul(C1);
        k1 = hash_rotl32(k1, 15);
        k1 = k1.wrapping_mul(C2);
        h1 ^= k1;
        h1 = hash_rotl32(h1, 13);
        h1 = h1.wrapping_mul(5).wrapping_add(0xe654_6b64);
    }
    let tail = &bytes[blocks * 4..];
    let mut k1 = 0u32;
    if tail.len() == 3 {
        k1 ^= (tail[2] as u32) << 16;
    }
    if tail.len() >= 2 {
        k1 ^= (tail[1] as u32) << 8;
    }
    if !tail.is_empty() {
        k1 ^= tail[0] as u32;
        k1 = k1.wrapping_mul(C1);
        k1 = hash_rotl32(k1, 15);
        k1 = k1.wrapping_mul(C2);
        h1 ^= k1;
    }
    h1 ^= bytes.len() as u32;
    hash_fmix32(h1)
}

/// `hash_djb2_one_float`.
pub fn hash_djb2_one_float(value: f64, prev: u32) -> u32 {
    let normalized = if value == 0.0 {
        0.0
    } else if value.is_nan() {
        f64::NAN
    } else {
        value
    };
    prev.wrapping_mul(33)
        .wrapping_add(hash_one_uint64(normalized.to_bits()))
}

/// `hash_djb2_one_float_64`.
pub fn hash_djb2_one_float_64(value: f64, prev: u64) -> u64 {
    let normalized = if value == 0.0 {
        0.0
    } else if value.is_nan() {
        f64::NAN
    } else {
        value
    };
    prev.wrapping_mul(33).wrapping_add(normalized.to_bits())
}

pub const HASH_TABLE_SIZE_MAX: usize = 29;

pub const HASH_TABLE_SIZE_PRIMES: [u32; HASH_TABLE_SIZE_MAX] = [
    5,
    13,
    23,
    47,
    97,
    193,
    389,
    769,
    1543,
    3079,
    6151,
    12289,
    24593,
    49157,
    98317,
    196613,
    393241,
    786433,
    1572869,
    3145739,
    6291469,
    12582917,
    25165843,
    50331653,
    100663319,
    201326611,
    402653189,
    805306457,
    1610612741,
];

/// `hash_table_size_primes_inv[i] = u64::MAX / primes[i] + 1`.
pub fn hash_table_size_prime_inv(index: usize) -> u64 {
    u64::MAX / HASH_TABLE_SIZE_PRIMES[index] as u64 + 1
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Values computed by replicating the C++ arithmetic with wrapping
    /// 32/64-bit integers (see `hashcalc.py` during the port).
    #[test]
    fn djb2_values() {
        assert_eq!(hash_djb2(b""), 5381);
        assert_eq!(hash_djb2(b"a"), 177604);
        assert_eq!(hash_djb2(b"abc"), 193409669);
        assert_eq!(hash_djb2(b"hello"), 178056679);
        // NUL terminates the C-string form.
        assert_eq!(hash_djb2(b"abc\0def"), hash_djb2(b"abc"));
        assert_eq!(hash_djb2_buffer(b"", 5381), 5381);
        assert_eq!(hash_djb2_buffer(b"abc", 5381), 193409669);
        assert_eq!(hash_djb2_one_32(0, 5381), 177573);
        assert_eq!(hash_djb2_one_32(1, 5381), 177572);
        assert_eq!(hash_djb2_one_32(77, 5), 232);
        assert_eq!(hash_djb2_one_64(0, 5381), 177573);
        assert_eq!(hash_djb2_one_64(u64::MAX, 0), u64::MAX);
    }

    #[test]
    fn wang_and_murmur64_values() {
        assert_eq!(hash_one_uint64(0), 720020139);
        assert_eq!(hash_one_uint64(1), 357654460);
        assert_eq!(hash_one_uint64(0x0123_4567_89ab_cdef), 2918899159);
        assert_eq!(hash64_murmur3_64(0, 0), 0);
        assert_eq!(hash64_murmur3_64(1, 0), 12994781566227106604);
        assert_eq!(
            hash64_murmur3_64(0x0123_4567_89ab_cdef, 0),
            9785191686031420650
        );
    }

    #[test]
    fn murmur3_word_values() {
        assert_eq!(hash_murmur3_one_32(0, HASH_MURMUR3_SEED), 873697370);
        assert_eq!(hash_murmur3_one_32(1, HASH_MURMUR3_SEED), 1446504008);
        assert_eq!(
            hash_murmur3_one_32(0xFFFF_FFFF, HASH_MURMUR3_SEED),
            639500531
        );
        assert_eq!(
            hash_murmur3_one_64(0x0123_4567_89ab_cdef, HASH_MURMUR3_SEED),
            3865115986
        );
        assert_eq!(hash_fmix32(0), 0);
        assert_eq!(hash_fmix32(1), 1364076727);
        assert_eq!(hash_rotl32(1, 15), 32768);
        assert_eq!(hash_rotl32(0x8000_0000, 1), 1);
    }

    #[test]
    fn murmur3_buffer_values() {
        assert_eq!(hash_murmur3_buffer(b"", HASH_MURMUR3_SEED), 2743165359);
        assert_eq!(hash_murmur3_buffer(b"abc", HASH_MURMUR3_SEED), 456045467);
        assert_eq!(
            hash_murmur3_buffer(b"The quick brown fox jumps over the lazy dog", HASH_MURMUR3_SEED),
            234077746
        );
        // Block + tail path: a length divisible by four and one that is not.
        assert_ne!(
            hash_murmur3_buffer(b"abcd", HASH_MURMUR3_SEED),
            hash_murmur3_buffer(b"abc", HASH_MURMUR3_SEED)
        );
    }

    #[test]
    fn float_hashing_normalizes_zero_and_nan() {
        // djB2 double form.
        assert_eq!(hash_djb2_one_float(0.0, 5381), 720197712);
        assert_eq!(hash_djb2_one_float(-0.0, 5381), 720197712);
        assert_eq!(hash_djb2_one_float(1.5, 5381), 3632008303);
        assert_eq!(hash_djb2_one_float_64(1.5, 5381), 4609434218613880229);

        // Murmur3 forms: 0.0 hashes as integer 0.
        assert_eq!(
            hash_murmur3_one_float(0.0, HASH_MURMUR3_SEED),
            hash_murmur3_one_32(0, HASH_MURMUR3_SEED)
        );
        assert_eq!(
            hash_murmur3_one_float(-0.0, HASH_MURMUR3_SEED),
            hash_murmur3_one_float(0.0, HASH_MURMUR3_SEED)
        );
        assert_eq!(
            hash_murmur3_one_double(0.0, HASH_MURMUR3_SEED),
            hash_murmur3_one_64(0, HASH_MURMUR3_SEED)
        );
        // NaNs with different payloads normalize to the canonical NaN.
        let nan_a = f32::from_bits(0x7fc0_0001);
        let nan_b = f32::from_bits(0xffc0_0002);
        assert_eq!(
            hash_murmur3_one_float(nan_a, HASH_MURMUR3_SEED),
            hash_murmur3_one_float(nan_b, HASH_MURMUR3_SEED)
        );
        let dn_a = f64::from_bits(0x7ff8_0000_0000_0001);
        let dn_b = f64::from_bits(0xfff8_0000_0000_0002);
        assert_eq!(
            hash_murmur3_one_double(dn_a, HASH_MURMUR3_SEED),
            hash_murmur3_one_double(dn_b, HASH_MURMUR3_SEED)
        );
    }

    #[test]
    fn prime_table_and_inverses() {
        assert_eq!(HASH_TABLE_SIZE_MAX, 29);
        assert_eq!(HASH_TABLE_SIZE_PRIMES[0], 5);
        assert_eq!(*HASH_TABLE_SIZE_PRIMES.last().unwrap(), 1610612741);
        for (index, prime) in HASH_TABLE_SIZE_PRIMES.iter().enumerate() {
            let inv = hash_table_size_prime_inv(index);
            assert_eq!(inv, u64::MAX / *prime as u64 + 1);
            // The table is the engine's formula, not a modular inverse: for a
            // divisor that divides u64::MAX exactly the product wraps to the
            // divisor itself.
            assert!(inv > 0);
        }
    }
}
