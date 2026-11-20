//! `scene/resources/world_2d.*` and `scene/resources/3d/world_3d.*` (state).
//!
//! `World2D` owns the canvas, navigation-map and physics-space handles (lazily
//! created in the engine) and the viewport list.
//!
//! `World3D` owns the scenario, space and navigation-map handles plus the
//! environment, fallback environment, camera attributes and compositor, and the
//! registered cameras (stored as handles here).
//!
//! Server creation, lazy map creation and camera registration signals are not
//! ported; the handles are passed in by the caller.

use crate::servers::rid::Rid;

#[derive(Clone, Debug, Default)]
pub struct World2DData {
    canvas: Option<Rid>,
    navigation_map: Option<Rid>,
    space: Option<Rid>,
    viewport_count: usize,
}

impl World2DData {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_canvas(&mut self, canvas: Option<Rid>) {
        self.canvas = canvas;
    }

    pub fn get_canvas(&self) -> Option<Rid> {
        self.canvas
    }

    pub fn set_navigation_map(&mut self, map: Option<Rid>) {
        self.navigation_map = map;
    }

    pub fn get_navigation_map(&self) -> Option<Rid> {
        self.navigation_map
    }

    pub fn set_space(&mut self, space: Option<Rid>) {
        self.space = space;
    }

    pub fn get_space(&self) -> Option<Rid> {
        self.space
    }

    /// The engine does not expose a viewport count; this tracks the registrations
    /// so callers can assert on them.
    pub fn register_viewport(&mut self) {
        self.viewport_count += 1;
    }

    pub fn unregister_viewport(&mut self) {
        self.viewport_count = self.viewport_count.saturating_sub(1);
    }

    pub fn get_viewport_count(&self) -> usize {
        self.viewport_count
    }
}

#[derive(Clone, Debug, Default)]
pub struct World3DData {
    scenario: Option<Rid>,
    space: Option<Rid>,
    navigation_map: Option<Rid>,
    environment: Option<Rid>,
    fallback_environment: Option<Rid>,
    camera_attributes: Option<Rid>,
    compositor: Option<Rid>,
    cameras: Vec<Rid>,
}

impl World3DData {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_scenario(&mut self, scenario: Option<Rid>) {
        self.scenario = scenario;
    }

    pub fn get_scenario(&self) -> Option<Rid> {
        self.scenario
    }

    pub fn set_space(&mut self, space: Option<Rid>) {
        self.space = space;
    }

    pub fn get_space(&self) -> Option<Rid> {
        self.space
    }

    pub fn set_navigation_map(&mut self, map: Option<Rid>) {
        self.navigation_map = map;
    }

    pub fn get_navigation_map(&self) -> Option<Rid> {
        self.navigation_map
    }

    pub fn set_environment(&mut self, environment: Option<Rid>) {
        self.environment = environment;
    }

    pub fn get_environment(&self) -> Option<Rid> {
        self.environment
    }

    pub fn set_fallback_environment(&mut self, environment: Option<Rid>) {
        self.fallback_environment = environment;
    }

    pub fn get_fallback_environment(&self) -> Option<Rid> {
        self.fallback_environment
    }

    pub fn set_camera_attributes(&mut self, attributes: Option<Rid>) {
        self.camera_attributes = attributes;
    }

    pub fn get_camera_attributes(&self) -> Option<Rid> {
        self.camera_attributes
    }

    pub fn set_compositor(&mut self, compositor: Option<Rid>) {
        self.compositor = compositor;
    }

    pub fn get_compositor(&self) -> Option<Rid> {
        self.compositor
    }

    /// `_register_camera` / `_remove_camera`, kept as a handle list.
    pub fn register_camera(&mut self, camera: Rid) {
        if !self.cameras.contains(&camera) {
            self.cameras.push(camera);
        }
    }

    pub fn remove_camera(&mut self, camera: Rid) {
        self.cameras.retain(|other| *other != camera);
    }

    pub fn get_cameras(&self) -> &[Rid] {
        &self.cameras
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn world_2d_handles() {
        let mut w = World2DData::new();
        assert!(w.get_canvas().is_none());
        assert!(w.get_navigation_map().is_none());
        assert!(w.get_space().is_none());
        assert_eq!(w.get_viewport_count(), 0);

        w.set_canvas(Some(Rid::from_u64(1)));
        w.set_navigation_map(Some(Rid::from_u64(2)));
        w.set_space(Some(Rid::from_u64(3)));
        assert_eq!(w.get_canvas(), Some(Rid::from_u64(1)));
        assert_eq!(w.get_navigation_map(), Some(Rid::from_u64(2)));
        assert_eq!(w.get_space(), Some(Rid::from_u64(3)));

        w.register_viewport();
        w.register_viewport();
        assert_eq!(w.get_viewport_count(), 2);
        w.unregister_viewport();
        assert_eq!(w.get_viewport_count(), 1);
        w.unregister_viewport();
        w.unregister_viewport();
        assert_eq!(w.get_viewport_count(), 0, "never goes below zero");
    }

    #[test]
    fn world_3d_handles_and_cameras() {
        let mut w = World3DData::new();
        assert!(w.get_scenario().is_none());
        assert!(w.get_space().is_none());
        assert!(w.get_navigation_map().is_none());
        assert!(w.get_environment().is_none());
        assert!(w.get_fallback_environment().is_none());
        assert!(w.get_camera_attributes().is_none());
        assert!(w.get_compositor().is_none());
        assert!(w.get_cameras().is_empty());

        w.set_scenario(Some(Rid::from_u64(4)));
        w.set_space(Some(Rid::from_u64(5)));
        w.set_navigation_map(Some(Rid::from_u64(6)));
        w.set_environment(Some(Rid::from_u64(7)));
        w.set_fallback_environment(Some(Rid::from_u64(8)));
        w.set_camera_attributes(Some(Rid::from_u64(9)));
        w.set_compositor(Some(Rid::from_u64(10)));

        assert_eq!(w.get_scenario(), Some(Rid::from_u64(4)));
        assert_eq!(w.get_space(), Some(Rid::from_u64(5)));
        assert_eq!(w.get_navigation_map(), Some(Rid::from_u64(6)));
        assert_eq!(w.get_environment(), Some(Rid::from_u64(7)));
        assert_eq!(w.get_fallback_environment(), Some(Rid::from_u64(8)));
        assert_eq!(w.get_camera_attributes(), Some(Rid::from_u64(9)));
        assert_eq!(w.get_compositor(), Some(Rid::from_u64(10)));

        w.register_camera(Rid::from_u64(11));
        w.register_camera(Rid::from_u64(12));
        w.register_camera(Rid::from_u64(11));
        assert_eq!(w.get_cameras().len(), 2, "no duplicates");
        w.remove_camera(Rid::from_u64(11));
        assert_eq!(w.get_cameras(), &[Rid::from_u64(12)]);
        w.remove_camera(Rid::from_u64(99));
        assert_eq!(w.get_cameras().len(), 1);
    }
}
