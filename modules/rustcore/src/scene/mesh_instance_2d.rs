//! `scene/2d/mesh_instance_2d.*` and `multimesh_instance_2d.*`: resource refs.
//!
//! Both classes only hold a mesh-like resource and an optional texture; the
//! resources are reduced to `Rid` handles because the mesh/resource layer is not
//! ported. Drawing, the deprecated `normal_map` alias and the navigation-mesh
//! geometry callback are not ported.

use crate::servers::rid::Rid;

#[derive(Clone, Copy, Debug, Default)]
pub struct MeshInstance2DData {
    mesh: Option<Rid>,
    texture: Option<Rid>,
}

impl MeshInstance2DData {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_mesh(&mut self, mesh: Option<Rid>) {
        self.mesh = mesh;
    }

    pub fn get_mesh(&self) -> Option<Rid> {
        self.mesh
    }

    pub fn set_texture(&mut self, texture: Option<Rid>) {
        self.texture = texture;
    }

    pub fn get_texture(&self) -> Option<Rid> {
        self.texture
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct MultiMeshInstance2DData {
    multimesh: Option<Rid>,
    texture: Option<Rid>,
}

impl MultiMeshInstance2DData {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_multimesh(&mut self, multimesh: Option<Rid>) {
        self.multimesh = multimesh;
    }

    pub fn get_multimesh(&self) -> Option<Rid> {
        self.multimesh
    }

    pub fn set_texture(&mut self, texture: Option<Rid>) {
        self.texture = texture;
    }

    pub fn get_texture(&self) -> Option<Rid> {
        self.texture
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mesh_instance_defaults_and_refs() {
        let mut m = MeshInstance2DData::new();
        assert!(m.get_mesh().is_none());
        assert!(m.get_texture().is_none());

        let mesh = Rid::from_u64(0x1234);
        let tex = Rid::from_u64(0x5678);
        m.set_mesh(Some(mesh));
        m.set_texture(Some(tex));
        assert_eq!(m.get_mesh(), Some(mesh));
        assert_eq!(m.get_texture(), Some(tex));

        m.set_mesh(None);
        assert!(m.get_mesh().is_none());
    }

    #[test]
    fn multimesh_instance_defaults_and_refs() {
        let mut m = MultiMeshInstance2DData::new();
        assert!(m.get_multimesh().is_none());
        assert!(m.get_texture().is_none());

        let multimesh = Rid::from_u64(0xABCD);
        m.set_multimesh(Some(multimesh));
        m.set_texture(Some(Rid::from_u64(1)));
        assert_eq!(m.get_multimesh(), Some(multimesh));
        assert_eq!(m.get_texture(), Some(Rid::from_u64(1)));
    }
}
