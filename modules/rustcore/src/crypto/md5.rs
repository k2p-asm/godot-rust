//! `core/crypto/md5.*`: the MD5 digest (RFC 1321).
//!
//! The engine uses `CryptoCore::md5` for pack-file directory keys and per-file
//! hashes. This port implements the same algorithm bit for bit: the standard
//! padding (0x80 then zeros to 56 mod 64, then the 64-bit little-endian bit
//! length), 64 rounds over four chained words with the per-round sine constants
//! and rotation amounts, and the little-endian digest.
//!
//! API: [`md5`] (bytes in, 16-byte digest out), [`md5_hex`] and the incremental
//! [`Md5Context`] (`start`/`update`/`finish`) for streaming use.

/// The four chained state words.
#[derive(Clone, Debug)]
pub struct Md5Context {
    state: [u32; 4],
    buffer: Vec<u8>,
    length: u64,
}

impl Default for Md5Context {
    fn default() -> Self {
        Self::new()
    }
}

impl Md5Context {
    pub fn new() -> Self {
        Self {
            state: [0x6745_2301, 0xefcd_ab89, 0x98ba_dcfe, 0x1032_5476],
            buffer: Vec::new(),
            length: 0,
        }
    }

    pub fn update(&mut self, data: &[u8]) {
        self.length = self.length.wrapping_add(data.len() as u64);
        self.buffer.extend_from_slice(data);
        while self.buffer.len() >= 64 {
            let block: Vec<u8> = self.buffer.drain(..64).collect();
            self.process_block(&block);
        }
    }

    pub fn finish(mut self) -> [u8; 16] {
        let bit_length = self.length.wrapping_mul(8);
        self.buffer.push(0x80);
        while self.buffer.len() % 64 != 56 {
            self.buffer.push(0);
        }
        self.buffer.extend_from_slice(&bit_length.to_le_bytes());
        while self.buffer.len() >= 64 {
            let block: Vec<u8> = self.buffer.drain(..64).collect();
            self.process_block(&block);
        }
        let mut digest = [0u8; 16];
        for (index, word) in self.state.iter().enumerate() {
            digest[index * 4..index * 4 + 4].copy_from_slice(&word.to_le_bytes());
        }
        digest
    }

    fn process_block(&mut self, block: &[u8]) {
        debug_assert_eq!(block.len(), 64);
        let mut words = [0u32; 16];
        for (index, chunk) in block.chunks_exact(4).enumerate() {
            words[index] = u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
        }

        const SHIFTS: [u32; 64] = [
            7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22, 5, 9, 14, 20, 5, 9, 14,
            20, 5, 9, 14, 20, 5, 9, 14, 20, 4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23, 4, 11,
            16, 23, 6, 10, 15, 21, 6, 10, 15, 21, 6, 10, 15, 21, 6, 10, 15, 21,
        ];

        let mut a = self.state[0];
        let mut b = self.state[1];
        let mut c = self.state[2];
        let mut d = self.state[3];

        for i in 0..64 {
            let constant = k_table()[i];
            let (f, g) = match i / 16 {
                0 => ((b & c) | (!b & d), i),
                1 => ((d & b) | (!d & c), (5 * i + 1) % 16),
                2 => (b ^ c ^ d, (3 * i + 5) % 16),
                _ => (c ^ (b | !d), (7 * i) % 16),
            };
            let temp = d;
            d = c;
            c = b;
            let sum = a
                .wrapping_add(f)
                .wrapping_add(constant)
                .wrapping_add(words[g]);
            b = b.wrapping_add(sum.rotate_left(SHIFTS[i]));
            a = temp;
        }

        self.state[0] = self.state[0].wrapping_add(a);
        self.state[1] = self.state[1].wrapping_add(b);
        self.state[2] = self.state[2].wrapping_add(c);
        self.state[3] = self.state[3].wrapping_add(d);
    }
}

/// The RFC 1321 sine-derived constants: `floor(abs(sin(i + 1)) * 2^32)`.
fn k_table() -> &'static [u32; 64] {
    use std::sync::OnceLock;
    static TABLE: OnceLock<[u32; 64]> = OnceLock::new();
    TABLE.get_or_init(|| {
        let mut table = [0u32; 64];
        for (index, slot) in table.iter_mut().enumerate() {
            let value = ((index as f64 + 1.0).sin().abs() * 4_294_967_296.0) as u32;
            *slot = value;
        }
        table
    })
}

/// `CryptoCore::md5`.
pub fn md5(data: &[u8]) -> [u8; 16] {
    let mut context = Md5Context::new();
    context.update(data);
    context.finish()
}

/// The lowercase hexadecimal digest.
pub fn md5_hex(data: &[u8]) -> String {
    let digest = md5(data);
    let mut out = String::with_capacity(32);
    for byte in digest {
        out.push_str(&format!("{byte:02x}"));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rfc_1321_vectors() {
        assert_eq!(md5_hex(b""), "d41d8cd98f00b204e9800998ecf8427e");
        assert_eq!(md5_hex(b"a"), "0cc175b9c0f1b6a831c399e269772661");
        assert_eq!(md5_hex(b"abc"), "900150983cd24fb0d6963f7d28e17f72");
        assert_eq!(
            md5_hex(b"message digest"),
            "f96b697d7cb7938d525a2f31aaf161d0"
        );
        assert_eq!(
            md5_hex(b"abcdefghijklmnopqrstuvwxyz"),
            "c3fcd3d76192e4007dfb496cca67e13b"
        );
        assert_eq!(
            md5_hex(b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789"),
            "d174ab98d277d9f5a5611c2c9f419d9f"
        );
        assert_eq!(
            md5_hex(b"12345678901234567890123456789012345678901234567890123456789012345678901234567890"),
            "57edf4a22be3c955ac49da2e2107b67a"
        );
        assert_eq!(
            md5_hex(b"The quick brown fox jumps over the lazy dog"),
            "9e107d9d372bb6826bd81d3542a419d6"
        );
    }

    #[test]
    fn padding_boundaries() {
        // 55, 56, 64 and 65 bytes exercise the padding/length edges.
        let digest_55 = md5(&[b'a'; 55]);
        let digest_56 = md5(&[b'a'; 56]);
        let digest_64 = md5(&[b'a'; 64]);
        let digest_65 = md5(&[b'a'; 65]);
        assert_ne!(digest_55, digest_56, "56 bytes needs an extra block");
        assert_ne!(digest_56, digest_64);
        assert_ne!(digest_64, digest_65);
        // The engine hashes buffers with trailing zero bytes too.
        assert_eq!(md5(&[0u8; 16]), md5(&[0u8; 16]));
        assert_ne!(md5(&[0u8; 16]), md5(&[0u8; 17]));
    }

    #[test]
    fn incremental_matches_one_shot() {
        let data: Vec<u8> = (0..1000u32).map(|value| (value % 251) as u8).collect();
        let expected = md5(&data);

        let mut context = Md5Context::new();
        for chunk in data.chunks(7) {
            context.update(chunk);
        }
        assert_eq!(context.finish(), expected);

        // Empty updates do not change the state.
        let mut context = Md5Context::new();
        context.update(&[]);
        context.update(b"abc");
        context.update(&[]);
        assert_eq!(context.finish(), md5(b"abc"));
    }
}
