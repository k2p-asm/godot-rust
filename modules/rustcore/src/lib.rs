//! Rust reimplementation of Godot Engine 4.7.2 core subsystems.
//!
//! Source of truth: the C++ files in this same fork (`core/math/*.{h,cpp}`).
//! Every ported function must keep the exact numeric semantics of the original,
//! which is verified by the differential harness in `parity/dump.cpp` and the
//! fixture-based tests in `tests/parity.rs`.
//!
//! Boundary rule for this crate: only POD types cross the FFI (`#[repr(C)]`),
//! no `Variant`/`String`/`Object` ever appears here.

pub mod ffi;
pub mod math;
pub mod string;
pub mod object;
pub mod variant;
pub mod templates;
