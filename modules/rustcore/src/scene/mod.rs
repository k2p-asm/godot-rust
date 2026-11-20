//! Scene-layer data structures ported without the object system.
//!
//! The engine's nodes are `Object`s; these modules carry only the numeric state
//! those nodes own, so they can be verified before the object layer exists.

pub mod camera_2d;
pub mod animation;
pub mod animation_library;
pub mod animation_mixer;
pub mod animation_player;
pub mod box_container;
pub mod canvas_item;
pub mod control;
pub mod curve;
pub mod curve_2d;
pub mod curve_3d;
pub mod gradient;
pub mod gradient_texture;
pub mod grid_container;
pub mod gui_indicators;
pub mod gui_misc;
pub mod light_2d;
pub mod line_2d;
pub mod canvas_layer;
pub mod margin_center;
pub mod node;
pub mod polygon_2d;
pub mod range;
pub mod scene_tree;
pub mod shapes_2d;
pub mod scroll_container;
pub mod node_2d;
pub mod node_3d;
pub mod node2d_extras;
pub mod node_tree;
pub mod parallax_layer;
pub mod path_2d;
pub mod sprite_2d;
pub mod sprite_frames;
pub mod style_box;
pub mod texture_misc;
pub mod theme;
pub mod tile_map_layer;
pub mod tile_set;
pub mod timer;
