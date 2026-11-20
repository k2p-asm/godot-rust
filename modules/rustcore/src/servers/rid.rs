//! `servers/rendering/rendering_server_globals.h`'s RID idea: a handle that
//! survives slot reuse.
//!
//! The engine hands out `RID`s from per-type owners; a freed id can have its
//! slot reused but the old handle must never become valid again. This port uses
//! the classic generation + index packing:
//!
//! `rid = (generation << 32) | (index + 1)`, with generation starting at 1 and
//! `0` reserved as the invalid RID. Freeing bumps the generation, so a stale
//! handle fails `get` even if the slot is reassigned.
//!
//! Not ported: typed RID owners per server resource, the chunked allocator, and
//! the `ObjectID`-style registry.

use std::collections::HashMap;

/// A 64-bit handle; `0` is invalid.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Rid(u64);

impl Rid {
    pub const INVALID: Rid = Rid(0);

    pub fn is_valid(self) -> bool {
        self.0 != 0
    }

    pub fn as_u64(self) -> u64 {
        self.0
    }

    pub fn from_u64(v: u64) -> Self {
        Rid(v)
    }
}

struct Slot<T> {
    generation: u32,
    value: Option<T>,
}

pub struct RidOwner<T> {
    slots: Vec<Slot<T>>,
    free: Vec<usize>,
    live: HashMap<u64, usize>,
    len: usize,
}

impl<T> Default for RidOwner<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> RidOwner<T> {
    pub fn new() -> Self {
        Self {
            slots: Vec::new(),
            free: Vec::new(),
            live: HashMap::new(),
            len: 0,
        }
    }

    #[inline]
    fn make(index: usize, generation: u32) -> Rid {
        Rid(((generation as u64) << 32) | (index as u64 + 1))
    }

    #[inline]
    fn split(rid: Rid) -> (usize, u32) {
        let generation = (rid.0 >> 32) as u32;
        let index = (rid.0 & 0xFFFF_FFFF) as usize;
        (index.wrapping_sub(1), generation)
    }

    /// Allocates a slot for `value`. The returned handle stays valid until
    /// `free` is called on it.
    pub fn alloc(&mut self, value: T) -> Rid {
        let generation;
        let index;

        if let Some(i) = self.free.pop() {
            self.slots[i].generation = self.slots[i].generation.wrapping_add(1).max(1);
            self.slots[i].value = Some(value);
            generation = self.slots[i].generation;
            index = i;
        } else {
            index = self.slots.len();
            self.slots.push(Slot {
                generation: 1,
                value: Some(value),
            });
            generation = 1;
        }

        let rid = Self::make(index, generation);
        self.live.insert(rid.0, index);
        self.len += 1;
        rid
    }

    /// Frees `rid`; a stale or invalid handle returns false.
    pub fn free(&mut self, rid: Rid) -> bool {
        if !rid.is_valid() {
            return false;
        }
        let (index, generation) = Self::split(rid);
        if index >= self.slots.len() || self.slots[index].generation != generation {
            return false;
        }
        if self.slots[index].value.is_none() {
            return false;
        }
        self.slots[index].value = None;
        self.slots[index].generation = generation.wrapping_add(1).max(1);
        self.live.remove(&rid.0);
        self.free.push(index);
        self.len -= 1;
        true
    }

    pub fn get(&self, rid: Rid) -> Option<&T> {
        if !rid.is_valid() {
            return None;
        }
        let (index, generation) = Self::split(rid);
        let slot = self.slots.get(index)?;
        if slot.generation != generation {
            return None;
        }
        slot.value.as_ref()
    }

    pub fn get_mut(&mut self, rid: Rid) -> Option<&mut T> {
        if !rid.is_valid() {
            return None;
        }
        let (index, generation) = Self::split(rid);
        let slot = self.slots.get_mut(index)?;
        if slot.generation != generation {
            return None;
        }
        slot.value.as_mut()
    }

    pub fn is_valid(&self, rid: Rid) -> bool {
        self.get(rid).is_some()
    }

    pub fn count(&self) -> usize {
        self.len
    }

    /// Live handles, in allocation-independent (numeric) order.
    pub fn live_rids(&self) -> Vec<Rid> {
        let mut v: Vec<Rid> = self.live.keys().map(|k| Rid(*k)).collect();
        v.sort_unstable();
        v
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alloc_get_free() {
        let mut owner: RidOwner<i32> = RidOwner::new();
        let a = owner.alloc(10);
        assert!(a.is_valid());
        assert_eq!(owner.get(a), Some(&10));
        assert_eq!(owner.count(), 1);

        *owner.get_mut(a).unwrap() = 11;
        assert_eq!(owner.get(a), Some(&11));

        assert!(owner.free(a));
        assert_eq!(owner.get(a), None);
        assert_eq!(owner.count(), 0);
        // Freeing twice fails.
        assert!(!owner.free(a));
    }

    #[test]
    fn invalid_rid_is_rejected() {
        let owner: RidOwner<i32> = RidOwner::new();
        assert!(owner.get(Rid::INVALID).is_none());
        assert!(!owner.is_valid(Rid::from_u64(12345)));
    }

    #[test]
    fn freed_slots_reuse_the_index_with_a_new_generation() {
        let mut owner: RidOwner<&str> = RidOwner::new();
        let a = owner.alloc("first");
        let index_a = a.as_u64() & 0xFFFF_FFFF;
        assert!(owner.free(a));

        let b = owner.alloc("second");
        let index_b = b.as_u64() & 0xFFFF_FFFF;
        assert_eq!(index_a, index_b, "the slot should be reused");
        // But the handle differs: the old one must stay dead.
        assert_ne!(a, b);
        assert!(owner.get(a).is_none(), "stale handle became valid");
        assert_eq!(owner.get(b), Some(&"second"));
    }

    #[test]
    fn many_allocations_are_independent() {
        let mut owner: RidOwner<usize> = RidOwner::new();
        let rids: Vec<Rid> = (0..100).map(|i| owner.alloc(i)).collect();
        assert_eq!(owner.count(), 100);

        let mut unique = std::collections::HashSet::new();
        for (i, r) in rids.iter().enumerate() {
            assert!(unique.insert(*r), "duplicate RID");
            assert_eq!(owner.get(*r), Some(&i));
        }

        // Free every other one and allocate replacements.
        for r in rids.iter().step_by(2) {
            assert!(owner.free(*r));
        }
        let fresh: Vec<Rid> = (0..50).map(|i| owner.alloc(1000 + i)).collect();
        for r in &fresh {
            assert!(owner.is_valid(*r));
        }
        // The stale ones stay invalid.
        for r in rids.iter().step_by(2) {
            assert!(!owner.is_valid(*r));
        }
        assert_eq!(owner.count(), 100);
    }

    #[test]
    fn generation_wraps_without_producing_invalid_zero() {
        let mut owner: RidOwner<()> = RidOwner::new();
        let mut r = owner.alloc(());
        // Force the generation to the top of its range.
        let index = (r.as_u64() & 0xFFFF_FFFF) as usize - 1;
        owner.slots[index].generation = u32::MAX;
        r = RidOwner::<()>::make(index, u32::MAX);
        assert!(owner.get(r).is_some());
        assert!(owner.free(r));

        let next = owner.alloc(());
        assert!(next.is_valid(), "wrap produced the invalid RID");
        assert!(owner.get(next).is_some());
    }
}
