//! `scene/3d/remote_transform_3d.*`: remote target flags.
//!
//! The remote path is stored as text (the engine keeps a `NodePath`); global
//! coordinates and the three update flags are all on by default. Applying the
//! transform to the target node needs the tree and is not ported.

#[derive(Clone, Debug)]
pub struct RemoteTransform3DData {
    remote_node: String,
    use_global_coordinates: bool,
    update_position: bool,
    update_rotation: bool,
    update_scale: bool,
}

impl Default for RemoteTransform3DData {
    fn default() -> Self {
        Self::new()
    }
}

impl RemoteTransform3DData {
    pub fn new() -> Self {
        Self {
            remote_node: String::new(),
            use_global_coordinates: true,
            update_position: true,
            update_rotation: true,
            update_scale: true,
        }
    }

    pub fn set_remote_node(&mut self, path: &str) {
        self.remote_node = path.to_string();
    }

    pub fn get_remote_node(&self) -> &str {
        &self.remote_node
    }

    pub fn set_use_global_coordinates(&mut self, enable: bool) {
        self.use_global_coordinates = enable;
    }

    pub fn get_use_global_coordinates(&self) -> bool {
        self.use_global_coordinates
    }

    pub fn set_update_position(&mut self, update: bool) {
        self.update_position = update;
    }

    pub fn get_update_position(&self) -> bool {
        self.update_position
    }

    pub fn set_update_rotation(&mut self, update: bool) {
        self.update_rotation = update;
    }

    pub fn get_update_rotation(&self) -> bool {
        self.update_rotation
    }

    pub fn set_update_scale(&mut self, update: bool) {
        self.update_scale = update;
    }

    pub fn get_update_scale(&self) -> bool {
        self.update_scale
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults() {
        let r = RemoteTransform3DData::new();
        assert_eq!(r.get_remote_node(), "");
        assert!(r.get_use_global_coordinates());
        assert!(r.get_update_position());
        assert!(r.get_update_rotation());
        assert!(r.get_update_scale());
    }

    #[test]
    fn setters_round_trip() {
        let mut r = RemoteTransform3DData::new();
        r.set_remote_node("../Target");
        r.set_use_global_coordinates(false);
        r.set_update_position(false);
        r.set_update_rotation(false);
        r.set_update_scale(false);

        assert_eq!(r.get_remote_node(), "../Target");
        assert!(!r.get_use_global_coordinates());
        assert!(!r.get_update_position());
        assert!(!r.get_update_rotation());
        assert!(!r.get_update_scale());

        r.set_remote_node("");
        assert_eq!(r.get_remote_node(), "");
    }
}
