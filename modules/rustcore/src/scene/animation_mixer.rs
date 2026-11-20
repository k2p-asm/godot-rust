//! `scene/animation/animation_mixer.*`: applying tracks to node state.
//!
//! `AnimationMixer` resolves each track's `Node/Path:property` and writes the
//! interpolated value. This port keeps the 2D transform it writes into per node
//! id (`NodeTree` owns names and hierarchy, not transforms), and supports the
//! properties that exist in 2D: `position`, `rotation`, `scale`.
//!
//! Track kinds: `Value` (Float or Vector2), `Position3D`, `Rotation3D`
//! (quaternion converted to a Z angle) and `Scale3D`.
//!
//! Not ported: method/bezier/audio tracks, blending and `AnimationTree`.

use crate::math::funcs;
use crate::math::transform_2d::Transform2D;
use crate::math::vector2::Vector2;
use crate::scene::animation::{Animation, TrackType};
use crate::scene::node_tree::NodeTree;
use crate::variant::Variant;
use std::collections::HashMap;

#[derive(Default)]
pub struct AnimationMixerData {
    transforms: HashMap<u64, Transform2D>,
}

impl AnimationMixerData {
    pub fn new() -> Self {
        Self::default()
    }

    /// The transform the mixer has written for `node` (identity if none yet).
    pub fn get_transform(&self, node: u64) -> Transform2D {
        self.transforms.get(&node).copied().unwrap_or(Transform2D::IDENTITY)
    }

    fn transform_mut(&mut self, node: u64) -> &mut Transform2D {
        self.transforms.entry(node).or_insert(Transform2D::IDENTITY)
    }

    /// Applies every track of `animation` at `time`. Returns how many tracks
    /// were written; unresolved paths and unsupported properties are skipped.
    pub fn apply(
        &mut self,
        animation: &Animation,
        tree: &NodeTree,
        root: u64,
        time: f64,
    ) -> usize {
        let mut applied = 0;

        for track_index in 0..animation.get_track_count() {
            let Some(track) = animation.get_track(track_index) else {
                continue;
            };
            let path = track.path.as_string();
            let Some((node_path, property)) = path.split_once(':') else {
                continue;
            };
            let Some(node) = tree.resolve_path(root, node_path) else {
                continue;
            };

            match track.track_type {
                TrackType::Position3D => {
                    if let Some(p) = animation.position_track_interpolate(track_index, time) {
                        let mut t = *self.transform_mut(node);
                        t.set_origin(Vector2::new(p.x, p.y));
                        *self.transform_mut(node) = t;
                        applied += 1;
                    }
                }
                TrackType::Scale3D => {
                    if let Some(s) = animation.scale_track_interpolate(track_index, time) {
                        let mut t = *self.transform_mut(node);
                        t.set_scale(Vector2::new(s.x, s.y));
                        *self.transform_mut(node) = t;
                        applied += 1;
                    }
                }
                TrackType::Rotation3D => {
                    if let Some(q) = animation.rotation_track_interpolate(track_index, time) {
                        // Rotation around Z expressed as a quaternion.
                        let angle = 2.0 * funcs::atan2(q.z as f64, q.w as f64) as f32;
                        let mut t = *self.transform_mut(node);
                        t.set_rotation(angle);
                        *self.transform_mut(node) = t;
                        applied += 1;
                    }
                }
                TrackType::Value => {
                    let Some(value) = animation.value_track_interpolate(track_index, time)
                    else {
                        continue;
                    };
                    let mut t = *self.transform_mut(node);
                    let ok = match (property, &value) {
                        ("position", Variant::Vector2(v)) => {
                            t.set_origin(*v);
                            true
                        }
                        ("rotation", Variant::Float(a)) => {
                            t.set_rotation(*a as f32);
                            true
                        }
                        ("scale", Variant::Vector2(v)) => {
                            t.set_scale(*v);
                            true
                        }
                        _ => false,
                    };
                    if ok {
                        *self.transform_mut(node) = t;
                        applied += 1;
                    }
                }
                _ => {}
            }
        }

        applied
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scene::animation::{Animation, InterpolationType, TrackType};

    fn setup() -> (NodeTree, u64, u64) {
        let mut tree = NodeTree::new();
        let root = tree.add_node(None, "Root").unwrap();
        let player = tree.add_node(Some(root), "Player").unwrap();
        (tree, root, player)
    }

    #[test]
    fn value_position_track_moves_the_node() {
        let (tree, root, player) = setup();
        let mut anim = Animation::new();
        let t = anim.add_track(TrackType::Value, -1);
        anim.track_set_path(t, "Player:position");
        anim.track_insert_key(t, 0.0, Variant::Vector2(Vector2::ZERO), 1.0);
        anim.track_insert_key(
            t,
            1.0,
            Variant::Vector2(Vector2::new(10.0, -4.0)),
            1.0,
        );

        let mut mixer = AnimationMixerData::new();
        let applied = mixer.apply(&anim, &tree, root, 0.5);
        assert_eq!(applied, 1);
        let origin = mixer.get_transform(player).columns[2];
        assert!((origin - Vector2::new(5.0, -2.0)).length() < 1e-5, "{origin:?}");
    }

    #[test]
    fn rotation_and_scale_tracks() {
        let (tree, root, player) = setup();
        let mut anim = Animation::new();

        let rot = anim.add_track(TrackType::Value, -1);
        anim.track_set_path(rot, "Player:rotation");
        anim.track_insert_key(rot, 0.0, Variant::Float(0.0), 1.0);
        anim.track_insert_key(rot, 1.0, Variant::Float(1.0), 1.0);

        let scale = anim.add_track(TrackType::Value, -1);
        anim.track_set_path(scale, "Player:scale");
        anim.track_insert_key(scale, 0.0, Variant::Vector2(Vector2::ONE), 1.0);
        anim.track_insert_key(
            scale,
            1.0,
            Variant::Vector2(Vector2::new(3.0, 3.0)),
            1.0,
        );

        let mut mixer = AnimationMixerData::new();
        assert_eq!(mixer.apply(&anim, &tree, root, 0.5), 2);
        let t = mixer.get_transform(player);
        assert!((t.get_rotation() - 0.5).abs() < 1e-5);
        // Column lengths are the scale (get_scale also carries the determinant sign).
        assert!((t.columns[0].length() - 2.0).abs() < 1e-5, "{:?}", t.columns[0]);
        assert!((t.columns[1].length() - 2.0).abs() < 1e-5, "{:?}", t.columns[1]);
    }

    #[test]
    fn transform_tracks_apply_without_a_property_name() {
        let (tree, root, player) = setup();
        let mut anim = Animation::new();

        let pos = anim.add_track(TrackType::Position3D, -1);
        anim.track_set_path(pos, "Player:position");
        anim.position_track_insert_key(pos, 0.0, crate::math::vector3::Vector3::ZERO);
        anim.position_track_insert_key(
            pos,
            2.0,
            crate::math::vector3::Vector3::new(0.0, 8.0, 99.0),
        );

        let scl = anim.add_track(TrackType::Scale3D, -1);
        anim.track_set_path(scl, "Player:scale");
        anim.scale_track_insert_key(scl, 0.0, crate::math::vector3::Vector3::ONE);
        anim.scale_track_insert_key(
            scl,
            2.0,
            crate::math::vector3::Vector3::new(9.0, 9.0, 9.0),
        );

        let mut mixer = AnimationMixerData::new();
        assert_eq!(mixer.apply(&anim, &tree, root, 1.0), 2);
        let t = mixer.get_transform(player);
        // Z is ignored in 2D.
        assert!((t.columns[2] - Vector2::new(0.0, 4.0)).length() < 1e-5);
        assert!((t.get_scale() - Vector2::new(5.0, 5.0)).length() < 1e-5);
    }

    #[test]
    fn unknown_paths_and_properties_are_skipped() {
        let (tree, root, _player) = setup();
        let mut anim = Animation::new();

        let missing = anim.add_track(TrackType::Value, -1);
        anim.track_set_path(missing, "Ghost:position");
        anim.track_insert_key(missing, 0.0, Variant::Vector2(Vector2::ZERO), 1.0);

        let bad_prop = anim.add_track(TrackType::Value, -1);
        anim.track_set_path(bad_prop, "Player:modulate");
        anim.track_insert_key(bad_prop, 0.0, Variant::Int(1), 1.0);

        // A path with no ':' at all is ignored too.
        let no_colon = anim.add_track(TrackType::Value, -1);
        anim.track_set_path(no_colon, "Player");
        anim.track_insert_key(no_colon, 0.0, Variant::Int(1), 1.0);

        let mut mixer = AnimationMixerData::new();
        assert_eq!(mixer.apply(&anim, &tree, root, 0.0), 0);
    }

    #[test]
    fn nearest_interpolation_is_respected() {
        let (tree, root, player) = setup();
        let mut anim = Animation::new();
        let t = anim.add_track(TrackType::Value, -1);
        anim.track_set_path(t, "Player:position");
        anim.track_set_interpolation_type(t, InterpolationType::Nearest);
        anim.track_insert_key(t, 0.0, Variant::Vector2(Vector2::ZERO), 1.0);
        anim.track_insert_key(t, 1.0, Variant::Vector2(Vector2::new(10.0, 0.0)), 1.0);

        let mut mixer = AnimationMixerData::new();
        mixer.apply(&anim, &tree, root, 0.9);
        // Nearest takes the previous key, so the value is still 0.
        assert!(mixer.get_transform(player).columns[2].length() < 1e-5);
    }
}
