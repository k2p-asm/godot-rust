//! `scene/2d/marker_2d.*`, `scene/3d/marker_3d.*` and
//! `scene/3d/visible_on_screen_notifier_3d.*` (state only).
//!
//! `Marker2D` defaults to 10 px gizmo extents and `Marker3D` to 0.25; both only
//! store the value.
//!
//! `VisibleOnScreenNotifier3D` holds the AABB (default 2×2×2 centred at the
//! origin, matching `AABB(Vector3(-1,-1,-1), Vector3(2,2,2))` from its
//! constructor) and the on-screen flag. `VisibleOnScreenEnabler3D` adds the
//! enable mode and its node path.
//!
//! Drawing, notifications and the visibility notifier plumbing are not ported.

use crate::math::aabb::Aabb;
use crate::math::vector3::Vector3;

#[derive(Clone, Copy, Debug)]
pub struct Marker2DData {
    gizmo_extents: f32,
}

impl Default for Marker2DData {
    fn default() -> Self {
        Self::new()
    }
}

impl Marker2DData {
    pub fn new() -> Self {
        Self {
            gizmo_extents: 10.0,
        }
    }

    pub fn set_gizmo_extents(&mut self, extents: f32) {
        self.gizmo_extents = extents;
    }

    pub fn get_gizmo_extents(&self) -> f32 {
        self.gizmo_extents
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Marker3DData {
    gizmo_extents: f32,
}

impl Default for Marker3DData {
    fn default() -> Self {
        Self::new()
    }
}

impl Marker3DData {
    pub fn new() -> Self {
        Self {
            gizmo_extents: 0.25,
        }
    }

    pub fn set_gizmo_extents(&mut self, extents: f32) {
        self.gizmo_extents = extents;
    }

    pub fn get_gizmo_extents(&self) -> f32 {
        self.gizmo_extents
    }
}

#[derive(Clone, Copy, Debug)]
pub struct VisibleOnScreenNotifier3DData {
    aabb: Aabb,
    on_screen: bool,
}

impl Default for VisibleOnScreenNotifier3DData {
    fn default() -> Self {
        Self::new()
    }
}

impl VisibleOnScreenNotifier3DData {
    pub fn new() -> Self {
        Self {
            aabb: Aabb::new(
                Vector3::new(-1.0, -1.0, -1.0),
                Vector3::new(2.0, 2.0, 2.0),
            ),
            on_screen: false,
        }
    }

    pub fn set_aabb(&mut self, aabb: Aabb) {
        self.aabb = aabb;
    }

    pub fn get_aabb(&self) -> Aabb {
        self.aabb
    }

    /// The notifier's own flag, set from the viewport's visibility test.
    pub fn set_on_screen(&mut self, on_screen: bool) {
        self.on_screen = on_screen;
    }

    pub fn is_on_screen(&self) -> bool {
        self.on_screen
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum EnableMode {
    Inherit,
    Always,
    WhenPaused,
}

#[derive(Clone, Debug)]
pub struct VisibleOnScreenEnabler3DData {
    pub notifier: VisibleOnScreenNotifier3DData,
    enable_mode: EnableMode,
    enable_node_path: String,
}

impl Default for VisibleOnScreenEnabler3DData {
    fn default() -> Self {
        Self::new()
    }
}

impl VisibleOnScreenEnabler3DData {
    pub fn new() -> Self {
        Self {
            notifier: VisibleOnScreenNotifier3DData::new(),
            enable_mode: EnableMode::Inherit,
            // The engine's default path is `../AnimationPlayer`.
            enable_node_path: "../AnimationPlayer".to_string(),
        }
    }

    pub fn set_enable_mode(&mut self, mode: EnableMode) {
        self.enable_mode = mode;
    }

    pub fn get_enable_mode(&self) -> EnableMode {
        self.enable_mode
    }

    pub fn set_enable_node_path(&mut self, path: &str) {
        self.enable_node_path = path.to_string();
    }

    pub fn get_enable_node_path(&self) -> &str {
        &self.enable_node_path
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn markers() {
        let mut m2 = Marker2DData::new();
        assert_eq!(m2.get_gizmo_extents(), 10.0);
        m2.set_gizmo_extents(2.0);
        assert_eq!(m2.get_gizmo_extents(), 2.0);

        let mut m3 = Marker3DData::new();
        assert_eq!(m3.get_gizmo_extents(), 0.25);
        m3.set_gizmo_extents(1.5);
        assert_eq!(m3.get_gizmo_extents(), 1.5);
    }

    #[test]
    fn notifier_defaults_and_flags() {
        let mut n = VisibleOnScreenNotifier3DData::new();
        assert_eq!(
            n.get_aabb(),
            Aabb::new(
                Vector3::new(-1.0, -1.0, -1.0),
                Vector3::new(2.0, 2.0, 2.0)
            )
        );
        assert!(!n.is_on_screen());

        n.set_aabb(Aabb::new(Vector3::ZERO, Vector3::ONE));
        n.set_on_screen(true);
        assert_eq!(n.get_aabb().size, Vector3::ONE);
        assert!(n.is_on_screen());
    }

    #[test]
    fn enabler_defaults() {
        let mut e = VisibleOnScreenEnabler3DData::new();
        assert_eq!(e.get_enable_mode(), EnableMode::Inherit);
        assert_eq!(e.get_enable_node_path(), "../AnimationPlayer");
        assert!(!e.notifier.is_on_screen());

        e.set_enable_mode(EnableMode::WhenPaused);
        e.set_enable_node_path("../Timer");
        assert_eq!(e.get_enable_mode(), EnableMode::WhenPaused);
        assert_eq!(e.get_enable_node_path(), "../Timer");

        assert_eq!(EnableMode::Inherit as i32, 0);
        assert_eq!(EnableMode::Always as i32, 1);
        assert_eq!(EnableMode::WhenPaused as i32, 2);
    }
}
