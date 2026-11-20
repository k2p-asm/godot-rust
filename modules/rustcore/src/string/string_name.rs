//! `StringName`: interned strings, where equality is identity.
//!
//! In the engine a `StringName` holds a pointer into a global table
//! (`StringName::_table`), so `==` is a pointer compare and hashing is free.
//! Here the pointer is a `u64` id handed out by a process-wide interner, which
//! gives the same observable semantics: equal text -> same id, `==` on ids.
//!
//! The pool never frees, exactly like the engine's table, which keeps ids
//! stable for the lifetime of the process.

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use crate::templates::hashfuncs;

#[derive(Default)]
struct Pool {
    ids: HashMap<String, u64>,
    texts: Vec<String>,
}

fn pool() -> &'static Mutex<Pool> {
    static POOL: OnceLock<Mutex<Pool>> = OnceLock::new();
    POOL.get_or_init(|| Mutex::new(Pool::default()))
}

/// An interned string handle. Copying it is free, like copying the C++ type.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct StringName {
    id: u64,
}

impl StringName {
    /// Interns `text`, returning the shared handle (`StringName(const String &)`).
    pub fn new(text: &str) -> Self {
        let mut p = pool().lock().unwrap();
        if let Some(id) = p.ids.get(text) {
            return Self { id: *id };
        }
        let id = p.texts.len() as u64;
        p.texts.push(text.to_string());
        p.ids.insert(text.to_string(), id);
        Self { id }
    }

    /// The interning id. Two names are equal iff their ids are equal.
    #[inline]
    pub fn id(&self) -> u64 {
        self.id
    }

    /// Recovers the text (`StringName::operator String()`).
    pub fn as_string(&self) -> String {
        let p = pool().lock().unwrap();
        p.texts[self.id as usize].clone()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.as_string().is_empty()
    }

    /// `StringName::hash()` — the djb2 hash of the text.
    pub fn hash(&self) -> u32 {
        hashfuncs::hash_djb2(self.as_string().as_bytes())
    }

    /// Number of interned names; used by tests and diagnostics.
    pub fn pool_size() -> usize {
        pool().lock().unwrap().texts.len()
    }
}

impl Default for StringName {
    fn default() -> Self {
        Self::new("")
    }
}

impl std::fmt::Display for StringName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_string())
    }
}

impl From<&str> for StringName {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interning_is_identity_based() {
        let a = StringName::new("position");
        let b = StringName::new("position");
        // Same text: same handle, and equality is by id.
        assert_eq!(a, b);
        assert_eq!(a.id(), b.id());

        let c = StringName::new("rotation");
        assert_ne!(a, c);
    }

    #[test]
    fn comparison_is_case_sensitive() {
        let lower = StringName::new("text");
        let upper = StringName::new("Text");
        assert_ne!(lower, upper);
        // They must not collide in the pool either.
        assert_ne!(lower.id(), upper.id());
    }

    #[test]
    fn text_roundtrips_and_hash_is_stable() {
        let n = StringName::new("mesh_instance_3d");
        assert_eq!(n.as_string(), "mesh_instance_3d");
        assert_eq!(n.hash(), StringName::new("mesh_instance_3d").hash());
        assert_ne!(n.hash(), StringName::new("mesh_instance_2d").hash());
        assert_eq!(n.to_string(), "mesh_instance_3d");
    }

    #[test]
    fn repeat_intern_returns_same_handle() {
        // The pool is global and tests run in parallel, so this checks handle
        // stability rather than the pool size.
        let first = StringName::new("__pool_probe__");
        for _ in 0..10 {
            assert_eq!(StringName::new("__pool_probe__").id(), first.id());
        }
    }
    #[test]
    fn empty_default() {
        let e = StringName::default();
        assert!(e.is_empty());
        assert_eq!(e, StringName::new(""));
    }
}
