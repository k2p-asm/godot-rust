//! Object identity and reference counting (`core/object/object.h` +
//! `ref_counted.*`), reduced to the part that does not need the class database.
//!
//! In the engine an `ObjectID` is a 64-bit handle that encodes a validity
//! check: a stale id must not resolve to a live object. Here that is modelled
//! with a monotonically increasing counter plus a registry of live ids, which
//! gives the same observable rules:
//!
//! - ids are never reused, so a stale handle can never alias a new object;
//! - `ObjectId::is_valid()` is false once the object is gone;
//! - `RefCounted` frees itself when the count reaches zero.
//!
//! NOT ported (they need `ClassDB`, `Variant` dispatch and the binding layer):
//! signals, properties, `get_class()`, `_notification` and script instances.
//! These tests are therefore **non-differential**: `object.cpp` cannot be
//! linked into the parity harness.

pub mod class_db;
pub mod message_queue;
pub mod resource;

use std::collections::HashSet;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};

/// 64-bit object handle. `NONE` (`0`) is what a null reference becomes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ObjectId(u64);

impl Default for ObjectId {
    #[inline]
    fn default() -> Self {
        ObjectId::NONE
    }
}

impl ObjectId {
    pub const NONE: ObjectId = ObjectId(0);

    #[inline]
    pub fn is_null(self) -> bool {
        self.0 == 0
    }

    #[inline]
    pub fn as_u64(self) -> u64 {
        self.0
    }

    #[inline]
    pub fn from_u64(v: u64) -> Self {
        ObjectId(v)
    }

    /// True while the object behind this handle is still alive.
    pub fn is_valid(self) -> bool {
        if self.is_null() {
            return false;
        }
        live().lock().unwrap().contains(&self.0)
    }
}

fn live() -> &'static Mutex<HashSet<u64>> {
    static LIVE: OnceLock<Mutex<HashSet<u64>>> = OnceLock::new();
    LIVE.get_or_init(|| Mutex::new(HashSet::new()))
}

fn next_id() -> u64 {
    static NEXT: AtomicU64 = AtomicU64::new(1);
    NEXT.fetch_add(1, Ordering::Relaxed)
}

/// Base object: an identity that exists while the value is alive.
#[derive(Debug)]
pub struct Object {
    id: ObjectId,
}

impl Object {
    pub fn new() -> Self {
        let id = ObjectId(next_id());
        live().lock().unwrap().insert(id.0);
        Self { id }
    }

    #[inline]
    pub fn id(&self) -> ObjectId {
        self.id
    }

    /// Always true for a live handle; exists to mirror `Object::is_instance_valid`.
    #[inline]
    pub fn is_valid(&self) -> bool {
        self.id.is_valid()
    }
}

impl Drop for Object {
    fn drop(&mut self) {
        live().lock().unwrap().remove(&self.id.0);
    }
}

impl Default for Object {
    fn default() -> Self {
        Self::new()
    }
}

/// Reference-counted object. `ref()` returns the new count, `unref()` returns
/// whether the object was destroyed (`RefCounted::unreference`).
#[derive(Debug, Default)]
pub struct RefCounted {
    /// `None` once destroyed: the engine deletes the object at zero refs.
    object: Option<Object>,
    id: ObjectId,
    refcount: i32,
    destroyed: bool,
}

impl RefCounted {
    pub fn new() -> Self {
        let object = Object::new();
        let id = object.id();
        Self { object: Some(object), id, refcount: 0, destroyed: false }
    }

    #[inline]
    pub fn id(&self) -> ObjectId {
        self.id
    }

    /// `RefCounted::reference()`.
    pub fn ref_(&mut self) -> i32 {
        assert!(!self.destroyed, "reference() on a destroyed object");
        self.refcount += 1;
        self.refcount
    }

    /// `RefCounted::unreference()`: true means the object must be released.
    pub fn unref(&mut self) -> bool {
        assert!(self.refcount > 0, "unreference() with a zero count");
        self.refcount -= 1;
        if self.refcount == 0 {
            self.destroyed = true;
            // Drop the object right away, so its id becomes stale.
            self.object = None;
            return true;
        }
        false
    }

    #[inline]
    pub fn refcount(&self) -> i32 {
        self.refcount
    }

    #[inline]
    pub fn is_destroyed(&self) -> bool {
        self.destroyed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_are_unique_and_never_reused() {
        let a = Object::new();
        let id_a = a.id();
        let b = Object::new();
        assert_ne!(id_a, b.id());

        assert!(id_a.is_valid());
        drop(a);
        // The handle is stale and must not alias anything new.
        assert!(!id_a.is_valid());
        assert!(!id_a.is_null());
        assert!(ObjectId::NONE.is_null());
        assert!(!ObjectId::NONE.is_valid());
    }

    #[test]
    fn validity_tracks_liveness() {
        let obj = Object::new();
        let id = obj.id();
        assert!(id.is_valid());
        assert!(obj.is_valid());
        drop(obj);
        assert!(!id.is_valid());
    }

    #[test]
    fn refcount_lifecycle() {
        let mut rc = RefCounted::new();
        assert_eq!(rc.refcount(), 0);
        assert_eq!(rc.ref_(), 1);
        assert_eq!(rc.ref_(), 2);
        assert!(!rc.unref());
        assert!(!rc.is_destroyed());
        assert!(rc.unref()); // Count reached zero.
        assert!(rc.is_destroyed());
        assert_eq!(rc.refcount(), 0);
    }

    #[test]
    fn destroyed_object_drops_its_id() {
        let mut rc = RefCounted::new();
        let id = rc.id();
        rc.ref_();
        assert!(id.is_valid());
        assert!(rc.unref());
        assert!(!id.is_valid());
    }

    #[test]
    #[should_panic(expected = "unreference() with a zero count")]
    fn unref_below_zero_panics() {
        let mut rc = RefCounted::new();
        rc.unref();
    }
}
