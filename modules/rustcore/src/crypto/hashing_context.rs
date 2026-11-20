//! `core/crypto/hashing_context.*`: the streaming hashing wrapper.
//!
//! Ported from `hashing_context.cpp`: `start(type)` creates the MD5/SHA-1/SHA-256
//! context (`ERR_ALREADY_IN_USE` when one is active), `update` feeds chunks
//! (`ERR_UNCONFIGURED` without a context) and `finish` returns the digest and
//! drops the context (an empty array when nothing was started).
//!
//! The engine stores mbedtls contexts behind a `void *`; this port dispatches on
//! the ported digest contexts directly.

use crate::crypto::md5::Md5Context;
use crate::crypto::sha1::Sha1Context;
use crate::crypto::sha256::Sha256Context;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum HashType {
    Md5 = 0,
    Sha1 = 1,
    Sha256 = 2,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HashError {
    Ok,
    /// `ERR_ALREADY_IN_USE`.
    AlreadyInUse,
    /// `ERR_UNCONFIGURED`.
    Unconfigured,
}

#[derive(Clone, Debug)]
enum Context {
    Md5(Md5Context),
    Sha1(Sha1Context),
    Sha256(Sha256Context),
}

#[derive(Clone, Debug, Default)]
pub struct HashingContext {
    context: Option<Context>,
    hash_type: Option<HashType>,
}

impl HashingContext {
    pub fn new() -> Self {
        Self::default()
    }

    /// `start`.
    pub fn start(&mut self, hash_type: HashType) -> HashError {
        if self.context.is_some() {
            return HashError::AlreadyInUse;
        }
        self.context = Some(match hash_type {
            HashType::Md5 => Context::Md5(Md5Context::new()),
            HashType::Sha1 => Context::Sha1(Sha1Context::new()),
            HashType::Sha256 => Context::Sha256(Sha256Context::new()),
        });
        self.hash_type = Some(hash_type);
        HashError::Ok
    }

    /// `update`.
    pub fn update(&mut self, chunk: &[u8]) -> HashError {
        match self.context.as_mut() {
            Some(Context::Md5(context)) => {
                context.update(chunk);
                HashError::Ok
            }
            Some(Context::Sha1(context)) => {
                context.update(chunk);
                HashError::Ok
            }
            Some(Context::Sha256(context)) => {
                context.update(chunk);
                HashError::Ok
            }
            None => HashError::Unconfigured,
        }
    }

    /// `finish`: the digest, or an empty array without a context.
    pub fn finish(&mut self) -> Vec<u8> {
        let context = self.context.take();
        self.hash_type = None;
        match context {
            Some(Context::Md5(context)) => context.finish().to_vec(),
            Some(Context::Sha1(context)) => context.finish().to_vec(),
            Some(Context::Sha256(context)) => context.finish().to_vec(),
            None => Vec::new(),
        }
    }

    pub fn is_started(&self) -> bool {
        self.context.is_some()
    }

    pub fn get_hash_type(&self) -> Option<HashType> {
        self.hash_type
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::md5::md5;
    use crate::crypto::sha1::sha1;
    use crate::crypto::sha256::sha256;

    #[test]
    fn known_digests() {
        for (hash_type, expected) in [
            (HashType::Md5, md5(b"abc").to_vec()),
            (HashType::Sha1, sha1(b"abc").to_vec()),
            (HashType::Sha256, sha256(b"abc").to_vec()),
        ] {
            let mut context = HashingContext::new();
            assert_eq!(context.start(hash_type), HashError::Ok);
            assert!(context.is_started());
            assert_eq!(context.get_hash_type(), Some(hash_type));
            assert_eq!(context.update(b"abc"), HashError::Ok);
            assert_eq!(context.finish(), expected);
            assert!(!context.is_started());
            assert_eq!(context.get_hash_type(), None);
        }
        assert_eq!(HashType::Md5 as i32, 0);
        assert_eq!(HashType::Sha1 as i32, 1);
        assert_eq!(HashType::Sha256 as i32, 2);
    }

    #[test]
    fn chunked_updates_match_one_shot() {
        let data: Vec<u8> = (0..1000u32).map(|value| (value % 251) as u8).collect();
        let mut context = HashingContext::new();
        context.start(HashType::Sha256);
        for chunk in data.chunks(7) {
            assert_eq!(context.update(chunk), HashError::Ok);
        }
        assert_eq!(context.finish(), sha256(&data).to_vec());
    }

    #[test]
    fn state_errors() {
        let mut context = HashingContext::new();
        // Update/finish without a context.
        assert_eq!(context.update(b"x"), HashError::Unconfigured);
        assert!(context.finish().is_empty());

        // A second start is refused until finish.
        assert_eq!(context.start(HashType::Md5), HashError::Ok);
        assert_eq!(context.start(HashType::Sha1), HashError::AlreadyInUse);
        assert_eq!(context.get_hash_type(), Some(HashType::Md5));
        assert!(!context.finish().is_empty());
        assert_eq!(context.start(HashType::Sha1), HashError::Ok);
        assert_eq!(context.finish(), sha1(b"").to_vec());
    }
}
