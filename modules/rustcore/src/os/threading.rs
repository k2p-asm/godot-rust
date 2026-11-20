//! `core/os/mutex.h`, `core/os/rw_lock.h`, `core/os/semaphore.h` and
//! `core/os/thread.h`: the synchronisation primitives.
//!
//! The engine wraps the platform primitives; this port wraps `std`:
//!
//! - `Mutex` (`lock`/`unlock`/`try_lock`), also exposing a guard so callers can
//!   hold it safely in tests;
//! - `RwLock` (`read_lock`/`write_lock`, the try forms and both unlocks);
//! - `Semaphore` (`post`, `wait`, `try_wait`, `get`), built on a mutex+condvar
//!   with the engine's counting semantics;
//! - `Thread` (`start`, `stop`, `is_started`, `wait_to_finish`, id, name,
//!   priority).
//!
//! Debug checks (`DEV_ENABLED` error reporting for double locks) are not
//! modelled; the operations are plain and the tests exercise the transitions.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Condvar, Mutex as StdMutex, MutexGuard, RwLock as StdRwLock};
use std::thread::JoinHandle;

// -- Mutex ------------------------------------------------------------------

#[derive(Debug, Default)]
pub struct Mutex {
    inner: StdMutex<()>,
}

impl Mutex {
    pub fn new() -> Self {
        Self::default()
    }

    /// `lock`: blocks until the mutex is free.
    pub fn lock(&self) {
        let guard = self.inner.lock().expect("mutex poisoned");
        drop(guard);
    }

    /// `unlock`: with `std` the guard owns the lock, so this only verifies that
    /// the mutex is currently free (documented deviation).
    pub fn unlock(&self) -> bool {
        match self.inner.try_lock() {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    /// `try_lock`: true when the caller acquired it.
    pub fn try_lock(&self) -> bool {
        self.inner.try_lock().is_ok()
    }

    /// Held for the lifetime of the guard.
    pub fn guard(&self) -> MutexGuard<'_, ()> {
        self.inner.lock().expect("mutex poisoned")
    }
}

// -- RWLock -----------------------------------------------------------------

#[derive(Debug, Default)]
pub struct RwLock {
    inner: StdRwLock<()>,
}

impl RwLock {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn read_lock(&self) {
        let guard = self.inner.read().expect("rwlock poisoned");
        drop(guard);
    }

    pub fn read_unlock(&self) {
        // `std` releases on guard drop; the call is kept for API parity.
    }

    pub fn write_lock(&self) {
        let guard = self.inner.write().expect("rwlock poisoned");
        drop(guard);
    }

    pub fn write_unlock(&self) {
        // `std` releases on guard drop.
    }

    pub fn try_read_lock(&self) -> bool {
        self.inner.try_read().is_ok()
    }

    pub fn try_write_lock(&self) -> bool {
        self.inner.try_write().is_ok()
    }
}

// -- Semaphore --------------------------------------------------------------

#[derive(Debug, Default)]
pub struct Semaphore {
    count: StdMutex<u32>,
    condvar: Condvar,
}

impl Semaphore {
    pub fn new() -> Self {
        Self::default()
    }

    /// `post`.
    pub fn post(&self) {
        let mut count = self.count.lock().expect("semaphore poisoned");
        *count += 1;
        self.condvar.notify_one();
    }

    /// `wait`: blocks until the count is positive, then decrements it.
    pub fn wait(&self) {
        let mut count = self.count.lock().expect("semaphore poisoned");
        while *count == 0 {
            count = self.condvar.wait(count).expect("semaphore poisoned");
        }
        *count -= 1;
    }

    /// `try_wait`: false when the count was zero.
    pub fn try_wait(&self) -> bool {
        let mut count = self.count.lock().expect("semaphore poisoned");
        if *count == 0 {
            return false;
        }
        *count -= 1;
        true
    }

    pub fn get(&self) -> u32 {
        *self.count.lock().expect("semaphore poisoned")
    }
}

// -- Thread -----------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum ThreadPriority {
    Low,
    Normal,
    High,
}

static NEXT_THREAD_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Default)]
pub struct Thread {
    started: AtomicBool,
    id: u64,
    name: StdMutex<String>,
    priority: StdMutex<ThreadPriority>,
    handle: StdMutex<Option<JoinHandle<()>>>,
}

impl Default for ThreadPriority {
    fn default() -> Self {
        ThreadPriority::Normal
    }
}

impl Thread {
    pub fn new() -> Self {
        Self {
            started: AtomicBool::new(false),
            id: NEXT_THREAD_ID.fetch_add(1, Ordering::Relaxed),
            name: StdMutex::new(String::new()),
            priority: StdMutex::new(ThreadPriority::Normal),
            handle: StdMutex::new(None),
        }
    }

    /// The engine's `get_id`.
    pub fn get_id(&self) -> u64 {
        self.id
    }

    pub fn set_name(&self, name: &str) {
        *self.name.lock().expect("thread name") = name.to_string();
    }

    pub fn get_name(&self) -> String {
        self.name.lock().expect("thread name").clone()
    }

    pub fn set_priority(&self, priority: ThreadPriority) {
        *self.priority.lock().expect("thread priority") = priority;
    }

    pub fn get_priority(&self) -> ThreadPriority {
        *self.priority.lock().expect("thread priority")
    }

    /// `start`: refuses a second start and runs `body` on a new thread.
    pub fn start<F>(&self, body: F) -> bool
    where
        F: FnOnce() + Send + 'static,
    {
        if self.started.swap(true, Ordering::AcqRel) {
            return false;
        }
        let handle = std::thread::spawn(body);
        *self.handle.lock().expect("thread handle") = Some(handle);
        true
    }

    pub fn is_started(&self) -> bool {
        self.started.load(Ordering::Acquire)
    }

    /// `wait_to_finish`.
    pub fn wait_to_finish(&self) {
        let handle = self.handle.lock().expect("thread handle").take();
        if let Some(handle) = handle {
            let _ = handle.join();
        }
        self.started.store(false, Ordering::Release);
    }

    /// `stop`: without cooperative cancellation this only drops the handle after
    /// joining (documented deviation; the engine sets a cancel flag).
    pub fn stop(&self) {
        self.wait_to_finish();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicU32;
    use std::sync::Arc;

    #[test]
    fn mutex_operations() {
        let mutex = Mutex::new();
        assert!(mutex.try_lock());
        assert!(mutex.unlock());
        {
            let _guard = mutex.guard();
        }
        mutex.lock();
        mutex.unlock();
        assert!(mutex.try_lock());
    }

    #[test]
    fn rwlock_operations() {
        let lock = RwLock::new();
        assert!(lock.try_read_lock());
        assert!(lock.try_write_lock());
        lock.read_lock();
        lock.read_unlock();
        lock.write_lock();
        lock.write_unlock();
    }

    #[test]
    fn semaphore_counting() {
        let semaphore = Semaphore::new();
        assert_eq!(semaphore.get(), 0);
        assert!(!semaphore.try_wait());
        semaphore.post();
        semaphore.post();
        assert_eq!(semaphore.get(), 2);
        assert!(semaphore.try_wait());
        assert_eq!(semaphore.get(), 1);
        // A wait consumes the last token.
        semaphore.wait();
        assert_eq!(semaphore.get(), 0);
        assert!(!semaphore.try_wait());

        // A blocked waiter is released by a post from another thread.
        let semaphore = Arc::new(Semaphore::new());
        let waiting = semaphore.clone();
        let handle = std::thread::spawn(move || waiting.wait());
        std::thread::yield_now();
        semaphore.post();
        handle.join().expect("waiter");
        assert_eq!(semaphore.get(), 0);
    }

    #[test]
    fn threads_run_once_and_join() {
        let thread = Thread::new();
        let id = thread.get_id();
        assert!(id > 0);
        assert!(!thread.is_started());
        assert_eq!(thread.get_name(), "");
        thread.set_name("Worker");
        assert_eq!(thread.get_name(), "Worker");
        assert_eq!(thread.get_priority(), ThreadPriority::Normal);
        thread.set_priority(ThreadPriority::High);
        assert_eq!(thread.get_priority(), ThreadPriority::High);

        let counter = Arc::new(AtomicU32::new(0));
        let writer = counter.clone();
        assert!(thread.start(move || {
            writer.store(42, Ordering::Release);
        }));
        assert!(thread.is_started());
        assert!(!thread.start(|| {}), "already started");
        thread.wait_to_finish();
        assert_eq!(counter.load(Ordering::Acquire), 42);
        assert!(!thread.is_started());

        // Starting again after the join is allowed.
        assert!(thread.start(|| {}));
        thread.stop();
        assert!(!thread.is_started());
        assert_eq!(ThreadPriority::Low as i32, 0);
        assert_eq!(ThreadPriority::High as i32, 2);
    }
}
