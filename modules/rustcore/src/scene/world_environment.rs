//! `scene/3d/world_environment.*`: the node's three world resources.
//!
//! `WorldEnvironment` only holds references to an `Environment`, camera
//! attributes and a compositor. They are reduced to `Rid` handles here; the
//! viewport propagation is not ported.

use crate::servers::rid::Rid;

#[derive(Clone, Copy, Debug, Default)]
pub struct WorldEnvironmentData {
    environment: Option<Rid>,
    camera_attributes: Option<Rid>,
    compositor: Option<Rid>,
}

impl WorldEnvironmentData {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_environment(&mut self, environment: Option<Rid>) {
        self.environment = environment;
    }

    pub fn get_environment(&self) -> Option<Rid> {
        self.environment
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_and_round_trip() {
        let mut w = WorldEnvironmentData::new();
        assert!(w.get_environment().is_none());
        assert!(w.get_camera_attributes().is_none());
        assert!(w.get_compositor().is_none());

        w.set_environment(Some(Rid::from_u64(1)));
        w.set_camera_attributes(Some(Rid::from_u64(2)));
        w.set_compositor(Some(Rid::from_u64(3)));
        assert_eq!(w.get_environment(), Some(Rid::from_u64(1)));
        assert_eq!(w.get_camera_attributes(), Some(Rid::from_u64(2)));
        assert_eq!(w.get_compositor(), Some(Rid::from_u64(3)));

        // Each reference can be cleared independently.
        w.set_environment(None);
        assert!(w.get_environment().is_none());
        assert_eq!(w.get_camera_attributes(), Some(Rid::from_u64(2)));
    }
}
