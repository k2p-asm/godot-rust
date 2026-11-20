//! `core/crypto/sha1.*`: SHA-1 (FIPS 180-1), as exposed by `CryptoCore::sha1`.
//!
//! The engine links mbedtls' SHA-1; this port implements the same algorithm:
//! 80 rounds over five chained words, the standard 0x80 padding with a 64-bit
//! big-endian bit length, and the big-endian digest. SHA-1 is only kept for
//! compatibility (pack signatures, UID checks), so the implementation mirrors
//! the reference without shortcuts.
//!
//! Verified against the FIPS/RFC 3174 vectors.

#[derive(Clone, Debug)]
pub struct Sha1Context {
    state: [u32; 5],
    buffer: Vec<u8>,
    length: u64,
}

impl Default for Sha1Context {
    fn default() -> Self {
        Self::new()
    }
}

impl Sha1Context {
    pub fn new() -> Self {
        Self {
            state: [0x67452301, 0xefcdab89, 0x98badcfe, 0x10325476, 0xc3d2e1f0],
            buffer: Vec::new(),
            length: 0,
        }
    }

    pub fn update(&mut self, data: &[u8]) {
        self.length = self.length.wrapping_add(data.len() as u64);
        self.buffer.extend_from_slice(data);
        while self.buffer.len() >= 64 {
            let block: Vec<u8> = self.buffer.drain(..64).collect();
            self.compress(&block);
        }
    }

    pub fn finish(mut self) -> [u8; 20] {
        let bit_length = self.length.wrapping_mul(8);
        self.buffer.push(0x80);
        while self.buffer.len() % 64 != 56 {
            self.buffer.push(0);
        }
        self.buffer.extend_from_slice(&bit_length.to_be_bytes());
        while self.buffer.len() >= 64 {
            let block: Vec<u8> = self.buffer.drain(..64).collect();
            self.compress(&block);
        }
        let mut digest = [0u8; 20];
        for (index, word) in self.state.iter().enumerate() {
            digest[index * 4..index * 4 + 4].copy_from_slice(&word.to_be_bytes());
        }
        digest
    }

    fn compress(&mut self, block: &[u8]) {
        debug_assert_eq!(block.len(), 64);
        let mut w = [0u32; 80];
        for (index, chunk) in block.chunks_exact(4).enumerate() {
            w[index] = u32::from_be_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
        }
        for index in 16..80 {
            w[index] = (w[index - 3] ^ w[index - 8] ^ w[index - 14] ^ w[index - 16])
                .rotate_left(1);
        }

        let mut a = self.state[0];
        let mut b = self.state[1];
        let mut c = self.state[2];
        let mut d = self.state[3];
        let mut e = self.state[4];

        for (index, word) in w.iter().enumerate() {
            let (f, k) = match index {
                0..=19 => ((b & c) | (!b & d), 0x5a827999),
                20..=39 => (b ^ c ^ d, 0x6ed9eba1),
                40..=59 => ((b & c) | (b & d) | (c & d), 0x8f1bbcdc),
                _ => (b ^ c ^ d, 0xca62c1d6),
            };
            let temp = a
                .rotate_left(5)
                .wrapping_add(f)
                .wrapping_add(e)
                .wrapping_add(k)
                .wrapping_add(*word);
            e = d;
            d = c;
            c = b.rotate_left(30);
            b = a;
            a = temp;
        }

        self.state[0] = self.state[0].wrapping_add(a);
        self.state[1] = self.state[1].wrapping_add(b);
        self.state[2] = self.state[2].wrapping_add(c);
        self.state[3] = self.state[3].wrapping_add(d);
        self.state[4] = self.state[4].wrapping_add(e);
    }
}

/// `CryptoCore::sha1`.
pub fn sha1(data: &[u8]) -> [u8; 20] {
    let mut context = Sha1Context::new();
    context.update(data);
    context.finish()
}

pub fn sha1_hex(data: &[u8]) -> String {
    let digest = sha1(data);
    let mut out = String::with_capacity(40);
    for byte in digest {
        out.push_str(&format!("{byte:02x}"));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reference_vectors() {
        assert_eq!(sha1_hex(b""), "da39a3ee5e6b4b0d3255bfef95601890afd80709");
        assert_eq!(sha1_hex(b"abc"), "a9993e364706816aba3e25717850c26c9cd0d89d");
        assert_eq!(
            sha1_hex(b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq"),
            "84983e441c3bd26ebaae4aa1f95129e5e54670f1"
        );
        assert_eq!(
            sha1_hex(b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789"),
            "761c457bf73b14d27e9e9265c46f4b4dda11f940"
        );
        assert_eq!(
            sha1_hex(b"12345678901234567890123456789012345678901234567890123456789012345678901234567890"),
            "50abf5706a150990a08b2c5ea40fa0e585554732"
        );
    }

    #[test]
    fn padding_and_incremental() {
        // Known 55-byte digest: the last size before an extra padding block.
        assert_eq!(
            sha1_hex(&[b'a'; 55]),
            "c1c8bbdc22796e28c0e15163d20899b65621d65a"
        );
        let data: Vec<u8> = (0..1000u32).map(|value| (value % 251) as u8).collect();
        let expected = sha1(&data);
        let mut context = Sha1Context::new();
        for chunk in data.chunks(11) {
            context.update(chunk);
        }
        assert_eq!(context.finish(), expected);

        let mut context = Sha1Context::new();
        context.update(&[]);
        context.update(b"abc");
        assert_eq!(context.finish(), sha1(b"abc"));
    }
}
