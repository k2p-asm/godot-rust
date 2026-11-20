//! `core/templates/safe_refcount.h`: atomic numbers, flags and ref counts.
//!
//! The engine's `SafeNumeric<T>` is a templated `std::atomic`. This port keeps
//! the same API on a `u32` (`SafeNumeric`), which is the only instantiation the
//! core uses, plus `SafeFlag` and `SafeRefCount`. All operations use the same
//! acquire-release ordering as the header: `set`/`set_to`/`clear` release,
//! `get`/`is_set` acquire, and the read-modify-write operations `AcqRel`.
//!
//! The engine's `DEV_ENABLED` crash when unreferencing a zero count is not
//! modelled; `unref`/`unrefval` simply saturate at zero like the atomic does.

use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

#[derive(Debug, Default)]
pub struct SafeNumeric {
    value: AtomicU32,
}

impl SafeNumeric {
    pub fn new(value: u32) -> Self {
        Self {
            value: AtomicU32::new(value),
        }
    }

    pub fn set(&self, value: u32) {
        self.value.store(value, Ordering::Release);
    }

    pub fn get(&self) -> u32 {
        self.value.load(Ordering::Acquire)
    }

    /// `increment`: the new value.
    pub fn increment(&self) -> u32 {
        self.value.fetch_add(1, Ordering::AcqRel) + 1
    }

    /// `postincrement`: the original value.
    pub fn postincrement(&self) -> u32 {
        self.value.fetch_add(1, Ordering::AcqRel)
    }

    /// `decrement`: the new value.
    pub fn decrement(&self) -> u32 {
        self.value.fetch_sub(1, Ordering::AcqRel).wrapping_sub(1)
    }

    /// `postdecrement`: the original value.
    pub fn postdecrement(&self) -> u32 {
        self.value.fetch_sub(1, Ordering::AcqRel)
    }

    pub fn add(&self, amount: u32) -> u32 {
        self.value.fetch_add(amount, Ordering::AcqRel) + amount
    }

    pub fn postadd(&self, amount: u32) -> u32 {
        self.value.fetch_add(amount, Ordering::AcqRel)
    }

    pub fn sub(&self, amount: u32) -> u32 {
        self.value.fetch_sub(amount, Ordering::AcqRel).wrapping_sub(amount)
    }

    pub fn postsub(&self, amount: u32) -> u32 {
        self.value.fetch_sub(amount, Ordering::AcqRel)
    }

    pub fn bit_or(&self, value: u32) -> u32 {
        self.value.fetch_or(value, Ordering::AcqRel)
    }

    pub fn bit_and(&self, value: u32) -> u32 {
        self.value.fetch_and(value, Ordering::AcqRel)
    }

    pub fn bit_xor(&self, value: u32) -> u32 {
        self.value.fetch_xor(value, Ordering::AcqRel)
    }

    /// `exchange_if_greater`: stores and returns `value` when it is larger than
    /// the current one, otherwise returns the current value.
    pub fn exchange_if_greater(&self, value: u32) -> u32 {
        loop {
            let current = self.value.load(Ordering::Acquire);
            if current >= value {
                return current;
            }
            if self
                .value
                .compare_exchange_weak(current, value, Ordering::AcqRel, Ordering::Acquire)
                .is_ok()
            {
                return value;
            }
        }
    }

    /// `conditional_increment`: 0 when the count is zero, otherwise the new value.
    pub fn conditional_increment(&self) -> u32 {
        loop {
            let current = self.value.load(Ordering::Acquire);
            if current == 0 {
                return 0;
            }
            if self
                .value
                .compare_exchange_weak(current, current + 1, Ordering::AcqRel, Ordering::Acquire)
                .is_ok()
            {
                return current + 1;
            }
        }
    }
}

#[derive(Debug, Default)]
pub struct SafeFlag {
    flag: AtomicBool,
}

impl SafeFlag {
    pub fn new(value: bool) -> Self {
        Self {
            flag: AtomicBool::new(value),
        }
    }

    pub fn is_set(&self) -> bool {
        self.flag.load(Ordering::Acquire)
    }

    pub fn set(&self) {
        self.flag.store(true, Ordering::Release);
    }

    /// `set_if_clear`: true when this call performed the transition.
    pub fn set_if_clear(&self) -> bool {
        !self.flag.swap(true, Ordering::AcqRel)
    }

    pub fn clear(&self) {
        self.flag.store(false, Ordering::Release);
    }

    /// `clear_if_set`: true when this call performed the transition.
    pub fn clear_if_set(&self) -> bool {
        self.flag.swap(false, Ordering::AcqRel)
    }

    pub fn set_to(&self, value: bool) {
        self.flag.store(value, Ordering::Release);
    }
}

#[derive(Debug)]
pub struct SafeRefCount {
    count: SafeNumeric,
}

impl SafeRefCount {
    /// `init(1)` plus the value setter.
    pub fn new(count: u32) -> Self {
        Self {
            count: SafeNumeric::new(count),
        }
    }

    /// `ref`: false when the count was already zero.
    pub fn ref_count(&self) -> bool {
        self.count.conditional_increment() != 0
    }

    /// `refval`: non-zero on success.
    pub fn refval(&self) -> u32 {
        self.count.conditional_increment()
    }

    /// `unref`: true when this call took the count to zero.
    pub fn unref(&self) -> bool {
        self.count.decrement() == 0
    }

    /// `unrefval`: 0 when the object must be disposed of.
    pub fn unrefval(&self) -> u32 {
        self.count.decrement()
    }

    pub fn get(&self) -> u32 {
        self.count.get()
    }

    /// `init`.
    pub fn init(&self, value: u32) {
        self.count.set(value);
    }
}

impl Default for SafeRefCount {
    fn default() -> Self {
        Self::new(1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numeric_arithmetic() {
        let numeric = SafeNumeric::new(5);
        assert_eq!(numeric.get(), 5);
        numeric.set(10);
        assert_eq!(numeric.get(), 10);
        assert_eq!(numeric.increment(), 11);
        assert_eq!(numeric.postincrement(), 11);
        assert_eq!(numeric.get(), 12);
        assert_eq!(numeric.decrement(), 11);
        assert_eq!(numeric.postdecrement(), 11);
        assert_eq!(numeric.get(), 10);
        assert_eq!(numeric.add(5), 15);
        assert_eq!(numeric.postadd(1), 15);
        assert_eq!(numeric.get(), 16);
        assert_eq!(numeric.sub(4), 12);
        assert_eq!(numeric.postsub(2), 12);
        assert_eq!(numeric.get(), 10);
    }

    #[test]
    fn numeric_bits_and_compare() {
        let numeric = SafeNumeric::new(0b1010);
        assert_eq!(numeric.bit_or(0b0100), 0b1010);
        assert_eq!(numeric.get(), 0b1110);
        assert_eq!(numeric.bit_and(0b0110), 0b1110);
        assert_eq!(numeric.get(), 0b0110);
        assert_eq!(numeric.bit_xor(0b0011), 0b0110);
        assert_eq!(numeric.get(), 0b0101);

        let numeric = SafeNumeric::new(4);
        assert_eq!(numeric.exchange_if_greater(3), 4);
        assert_eq!(numeric.get(), 4);
        assert_eq!(numeric.exchange_if_greater(9), 9);
        assert_eq!(numeric.get(), 9);
    }

    #[test]
    fn conditional_increment_respects_zero() {
        let numeric = SafeNumeric::new(0);
        assert_eq!(numeric.conditional_increment(), 0);
        assert_eq!(numeric.get(), 0);
        numeric.set(3);
        assert_eq!(numeric.conditional_increment(), 4);
        assert_eq!(numeric.get(), 4);
    }

    #[test]
    fn flags() {
        let flag = SafeFlag::new(false);
        assert!(!flag.is_set());
        assert!(flag.set_if_clear());
        assert!(!flag.set_if_clear());
        assert!(flag.is_set());
        assert!(flag.clear_if_set());
        assert!(!flag.clear_if_set());
        assert!(!flag.is_set());
        flag.set();
        assert!(flag.is_set());
        flag.clear();
        assert!(!flag.is_set());
        flag.set_to(true);
        assert!(flag.is_set());
    }

    #[test]
    fn ref_count_lifecycle() {
        let count = SafeRefCount::default();
        assert_eq!(count.get(), 1);
        assert!(count.ref_count());
        assert_eq!(count.refval(), 3);
        assert!(!count.unref());
        assert!(!count.unref());
        assert!(count.unref(), "the last reference");
        assert_eq!(count.get(), 0);
        // Referencing a dead object fails.
        assert!(!count.ref_count());
        assert_eq!(count.refval(), 0);

        // `unrefval` reports zero for disposal.
        let count = SafeRefCount::default();
        assert_eq!(count.unrefval(), 0);
        count.init(2);
        assert_eq!(count.unrefval(), 1);
        assert_eq!(count.get(), 1);
    }
}
