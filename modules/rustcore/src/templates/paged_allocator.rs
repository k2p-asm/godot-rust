//! `core/templates/paged_allocator.h`: a page-backed slot pool.
//!
//! The engine hands out raw pointers from power-of-two pages with a free list.
//! The port replaces the pointers with slot indices:
//!
//! - `configure(page_size)` rounds the page size up to a power of two and is
//!   refused once configured (the engine's `page_pool != nullptr` guard);
//! - `alloc(value)` returns a slot index, growing by whole pages and taking the
//!   last slot of a new page first, like the engine's `allocs_available--`;
//! - `free(index)` returns the value and recycles the slot;
//! - `reset(allow_unfreed)` refuses to drop a pool with live allocations unless
//!   `allow_unfreed` is set (the engine's `allocs_available < pages * page_size`
//!   check);
//! - `get`/`get_mut`, `is_configured`, `get_page_size`, `get_pages_allocated`.
//!
//! The default page size is 4096 slots.

pub const DEFAULT_PAGE_SIZE: usize = 4096;

#[derive(Clone, Debug)]
pub struct PagedAllocator<T> {
    page_size: usize,
    slots: Vec<Option<T>>,
    free_slots: Vec<usize>,
    pages_allocated: usize,
    live: usize,
}

impl<T> Default for PagedAllocator<T> {
    fn default() -> Self {
        Self::new(DEFAULT_PAGE_SIZE)
    }
}

impl<T> PagedAllocator<T> {
    /// `PagedAllocator(page_size = 4096)`.
    pub fn new(page_size: usize) -> Self {
        let mut allocator = Self {
            page_size: 0,
            slots: Vec::new(),
            free_slots: Vec::new(),
            pages_allocated: 0,
            live: 0,
        };
        allocator.configure(page_size);
        allocator
    }

    /// `configure`: rounds up to a power of two; false when already configured
    /// or when `page_size` is zero.
    pub fn configure(&mut self, page_size: usize) -> bool {
        if self.page_size != 0 || page_size == 0 {
            return false;
        }
        self.page_size = nearest_power_of_two(page_size);
        true
    }

    fn add_page(&mut self) {
        let base = self.slots.len();
        self.slots.resize_with(base + self.page_size, || None);
        for index in 0..self.page_size {
            self.free_slots.push(base + index);
        }
        self.pages_allocated += 1;
    }

    /// `alloc`: grows by pages and takes the last free slot.
    pub fn alloc(&mut self, value: T) -> usize {
        if self.free_slots.is_empty() {
            self.add_page();
        }
        let index = self.free_slots.pop().expect("page just added");
        self.slots[index] = Some(value);
        self.live += 1;
        index
    }

    /// `free`: returns the value when the slot was live.
    pub fn free(&mut self, index: usize) -> Option<T> {
        let slot = self.slots.get_mut(index)?;
        let value = slot.take()?;
        self.free_slots.push(index);
        self.live -= 1;
        Some(value)
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        self.slots.get(index).and_then(|slot| slot.as_ref())
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        self.slots.get_mut(index).and_then(|slot| slot.as_mut())
    }

    /// `reset`: drops the pages; false when allocations remain and
    /// `allow_unfreed` is off.
    pub fn reset(&mut self, allow_unfreed: bool) -> bool {
        if self.live != 0 && !allow_unfreed {
            return false;
        }
        self.slots.clear();
        self.free_slots.clear();
        self.pages_allocated = 0;
        self.live = 0;
        true
    }

    pub fn is_configured(&self) -> bool {
        self.page_size > 0
    }

    pub fn get_page_size(&self) -> usize {
        self.page_size
    }

    pub fn get_pages_allocated(&self) -> usize {
        self.pages_allocated
    }

    pub fn get_live_count(&self) -> usize {
        self.live
    }

    pub fn get_free_slot_count(&self) -> usize {
        self.free_slots.len()
    }
}

/// `Math::nearest_power_of_2_templated` (rounds up, minimum 1).
fn nearest_power_of_two(value: usize) -> usize {
    if value <= 1 {
        return 1;
    }
    let mut power = 1usize;
    while power < value {
        power <<= 1;
    }
    power
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn configuration_rounds_and_locks() {
        let mut pool: PagedAllocator<u32> = PagedAllocator::new(100);
        assert!(pool.is_configured());
        assert_eq!(pool.get_page_size(), 128, "rounded up to a power of two");
        assert!(!pool.configure(256), "already configured");
        assert_eq!(pool.get_page_size(), 128);

        assert_eq!(PagedAllocator::<u32>::new(0).get_page_size(), 0);
        assert_eq!(PagedAllocator::<u32>::new(1).get_page_size(), 1);
        assert_eq!(PagedAllocator::<u32>::default().get_page_size(), 4096);
    }

    #[test]
    fn allocation_grows_pages_and_reuses_slots() {
        let mut pool: PagedAllocator<u32> = PagedAllocator::new(2);
        assert_eq!(pool.get_pages_allocated(), 0);
        let a = pool.alloc(10);
        // The engine takes the last slot of a fresh page first.
        assert_eq!(a, 1);
        let b = pool.alloc(20);
        assert_eq!(b, 0);
        assert_eq!(pool.get_pages_allocated(), 1);
        assert_eq!(pool.get_live_count(), 2);

        // A third allocation grows the pool.
        let c = pool.alloc(30);
        assert_eq!(c, 3);
        assert_eq!(pool.get_pages_allocated(), 2);
        assert_eq!(pool.get_page_size(), 2);

        assert_eq!(pool.get(a), Some(&10));
        assert_eq!(pool.get(b), Some(&20));
        assert_eq!(pool.get(c), Some(&30));

        // Freed slots are recycled (last freed first).
        assert_eq!(pool.free(b), Some(20));
        assert_eq!(pool.free(b), None, "double free is harmless");
        let d = pool.alloc(40);
        assert_eq!(d, 0);
        assert_eq!(pool.get_live_count(), 3);

        *pool.get_mut(d).unwrap() = 41;
        assert_eq!(pool.get(d), Some(&41));
        assert_eq!(pool.get(99), None);
        assert_eq!(pool.get_mut(99), None);
        assert_eq!(pool.free(99), None);
    }

    #[test]
    fn reset_requires_freed_slots() {
        let mut pool: PagedAllocator<u32> = PagedAllocator::new(4);
        let a = pool.alloc(1);
        let b = pool.alloc(2);
        assert!(!pool.reset(false), "live allocations block reset");
        assert_eq!(pool.get_live_count(), 2);
        assert!(pool.reset(true), "allow_unfreed drops them");
        assert_eq!(pool.get_pages_allocated(), 0);
        assert_eq!(pool.get_live_count(), 0);
        assert_eq!(pool.get_free_slot_count(), 0);
        assert!(pool.get_page_size() > 0, "configuration is kept");

        // A fully freed pool resets without the flag.
        let c = pool.alloc(3);
        assert_eq!(pool.free(c), Some(3));
        assert!(pool.reset(false));
        assert_eq!(pool.get_live_count(), 0);
        let _ = (a, b);
    }
}
