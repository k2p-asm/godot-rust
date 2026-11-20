//! `scene/3d/multimesh_instance_3d.*`: multimesh instance state.
//!
//! `MultiMeshInstance3D` extends `GeometryInstance3D` with a single `MultiMesh`
//! handle (reduced to a `Rid`). The mesh expansion and navigation callbacks are
//! not ported.

use crate::scene::visual_instance_3d::GeometryInstance3DData;
use crate::servers::rid::Rid;

#[derive(Clone, Debug, Default)]
pub struct MultiMeshInstance3DData {
    pub geometry: GeometryInstance3DData,
    multimesh: Option<Rid>,
}

impl MultiMeshInstance3DData {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_multimesh(&mut self, multimesh: Option<Rid>) {
        self.multimesh = multimesh;
    }

    pub fn get_multimesh(&self) -> Option<Rid> {
        self.multimesh
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scene::mesh_library::ShadowCastingSetting;

    #[test]
    fn defaults_and_round_trip() {
        let mut m = MultiMeshInstance3DData::new();
        assert!(m.get_multimesh().is_none());
        assert_eq!(m.geometry.get_cast_shadows_setting(), ShadowCastingSetting::On);

        m.set_multimesh(Some(Rid::from_u64(4)));
        assert_eq!(m.get_multimesh(), Some(Rid::from_u64(4)));
        m.set_multimesh(None);
        assert!(m.get_multimesh().is_none());
    }
}
