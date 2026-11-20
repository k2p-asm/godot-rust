//! Scene-layer data structures ported without the object system.
//!
//! The engine's nodes are `Object`s; these modules carry only the numeric state
//! those nodes own, so they can be verified before the object layer exists.

pub mod camera_2d;
pub mod canvas_item;
pub mod curve;
pub mod curve_2d;
pub mod curve_3d;
pub mod gradient;
pub mod canvas_layer;
pub mod node;
pub mod node_2d;
pub mod sprite_2d;
