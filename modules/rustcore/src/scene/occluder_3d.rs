//! `scene/3d/occluder_instance_3d.*`: occluder resources and the instance.
//!
//! Marker3D: `gizmo_extents = 0.25`.
//!
//! Occluders: `ArrayOccluder3D` stores vertices/indices as given; `Quad` and
//! `Box` clamp their sizes to `>= 0`; `Sphere` clamps the radius to `>= 0`;
//! `Polygon` stores the 2D outline unchanged.
//!
//! `OccluderInstance3D`: occluder handle, bake mask (`0xffffffff`) with layer
//! numbers 1..20, and bake simplification distance 0.1.
//!
//! Baking, debug meshes and RIDs are not ported.

use crate::math::vector2::Vector2;
use crate::math::vector3::Vector3;
use crate::scene::visual_instance_3d::VisualInstance3DData;
use crate::servers::rid::Rid;

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
        Self { gizmo_extents: 0.25 }
    }

    pub fn set_gizmo_extents(&mut self, extents: f32) {
        self.gizmo_extents = extents;
    }

    pub fn get_gizmo_extents(&self) -> f32 {
        self.gizmo_extents
    }
}

#[derive(Clone, Debug, Default)]
pub struct ArrayOccluder3DData {
    vertices: Vec<Vector3>,
    indices: Vec<i32>,
}

impl ArrayOccluder3DData {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_arrays(&mut self, vertices: Vec<Vector3>, indices: Vec<i32>) {
        self.vertices = vertices;
        self.indices = indices;
    }

    pub fn set_vertices(&mut self, vertices: Vec<Vector3>) {
        self.vertices = vertices;
    }

    pub fn set_indices(&mut self, indices: Vec<i32>) {
        self.indices = indices;
    }

    pub fn get_vertices(&self) -> &[Vector3] {
        &self.vertices
    }

    pub fn get_indices(&self) -> &[i32] {
        &self.indices
    }
}

#[derive(Clone, Copy, Debug)]
pub struct QuadOccluder3DData {
    size: Vector2,
}

impl Default for QuadOccluder3DData {
    fn default() -> Self {
        Self::new()
    }
}

impl QuadOccluder3DData {
    pub fn new() -> Self {
        Self {
            size: Vector2::new(1.0, 1.0),
        }
    }

    /// Components are clamped to `>= 0` (`maxf(0)`).
    pub fn set_size(&mut self, size: Vector2) {
        self.size = Vector2::new(size.x.max(0.0), size.y.max(0.0));
    }

    pub fn get_size(&self) -> Vector2 {
        self.size
    }
}

#[derive(Clone, Copy, Debug)]
pub struct BoxOccluder3DData {
    size: Vector3,
}

impl Default for BoxOccluder3DData {
    fn default() -> Self {
        Self::new()
    }
}

impl BoxOccluder3DData {
    pub fn new() -> Self {
        Self {
            size: Vector3::new(1.0, 1.0, 1.0),
        }
    }

    pub fn set_size(&mut self, size: Vector3) {
        self.size = Vector3::new(size.x.max(0.0), size.y.max(0.0), size.z.max(0.0));
    }

    pub fn get_size(&self) -> Vector3 {
        self.size
    }
}

#[derive(Clone, Copy, Debug)]
pub struct SphereOccluder3DData {
    radius: f32,
}

impl Default for SphereOccluder3DData {
    fn default() -> Self {
        Self::new()
    }
}

impl SphereOccluder3DData {
    pub fn new() -> Self {
        Self { radius: 1.0 }
    }

    pub fn set_radius(&mut self, radius: f32) {
        self.radius = radius.max(0.0);
    }

    pub fn get_radius(&self) -> f32 {
        self.radius
    }
}

#[derive(Clone, Debug, Default)]
pub struct PolygonOccluder3DData {
    polygon: Vec<Vector2>,
}

impl PolygonOccluder3DData {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_polygon(&mut self, polygon: Vec<Vector2>) {
        self.polygon = polygon;
    }

    pub fn get_polygon(&self) -> &[Vector2] {
        &self.polygon
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum BakeError {
    Ok,
    NoSavePath,
    NoMeshes,
    CantSave,
}

#[derive(Clone, Debug)]
pub struct OccluderInstance3DData {
    pub visual: VisualInstance3DData,
    occluder: Option<Rid>,
    bake_mask: u32,
    bake_simplification_distance: f32,
}

impl Default for OccluderInstance3DData {
    fn default() -> Self {
        Self::new()
    }
}

impl OccluderInstance3DData {
    pub fn new() -> Self {
        Self {
            visual: VisualInstance3DData::new(),
            occluder: None,
            bake_mask: 0xFFFFFFFF,
            bake_simplification_distance: 0.1,
        }
    }

    pub fn set_occluder(&mut self, occluder: Option<Rid>) {
        self.occluder = occluder;
    }

    pub fn get_occluder(&self) -> Option<Rid> {
        self.occluder
    }

    pub fn set_bake_mask(&mut self, mask: u32) {
        self.bake_mask = mask;
    }

    pub fn get_bake_mask(&self) -> u32 {
        self.bake_mask
    }

    /// Layer numbers are `1..=20`.
    pub fn set_bake_mask_value(&mut self, layer: i32, enabled: bool) -> bool {
        if !(1..=20).contains(&layer) {
            return false;
        }
        let bit = 1u32 << (layer - 1);
        if enabled {
            self.bake_mask |= bit;
        } else {
            self.bake_mask &= !bit;
        }
        true
    }

    pub fn get_bake_mask_value(&self, layer: i32) -> Option<bool> {
        if !(1..=20).contains(&layer) {
            return None;
        }
        Some((self.bake_mask & (1u32 << (layer - 1))) != 0)
    }

    pub fn set_bake_simplification_distance(&mut self, distance: f32) {
        self.bake_simplification_distance = distance;
    }

    pub fn get_bake_simplification_distance(&self) -> f32 {
        self.bake_simplification_distance
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn marker3d_defaults() {
        let mut m = Marker3DData::new();
        assert_eq!(m.get_gizmo_extents(), 0.25);
        m.set_gizmo_extents(2.0);
        assert_eq!(m.get_gizmo_extents(), 2.0);
    }

    #[test]
    fn array_occluder_stores_as_given() {
        let mut a = ArrayOccluder3DData::new();
        assert!(a.get_vertices().is_empty());
        assert!(a.get_indices().is_empty());

        a.set_arrays(
            vec![Vector3::ZERO, Vector3::new(1.0, 0.0, 0.0), Vector3::new(0.0, 1.0, 0.0)],
            vec![0, 1, 2],
        );
        assert_eq!(a.get_vertices().len(), 3);
        assert_eq!(a.get_indices(), &[0, 1, 2]);

        a.set_vertices(vec![Vector3::ZERO]);
        a.set_indices(vec![7]);
        assert_eq!(a.get_vertices().len(), 1);
        assert_eq!(a.get_indices(), &[7]);
    }

    #[test]
    fn primitive_occluders_clamp() {
        let mut q = QuadOccluder3DData::new();
        assert_eq!(q.get_size(), Vector2::new(1.0, 1.0));
        q.set_size(Vector2::new(-2.0, 3.0));
        assert_eq!(q.get_size(), Vector2::new(0.0, 3.0));

        let mut b = BoxOccluder3DData::new();
        assert_eq!(b.get_size(), Vector3::new(1.0, 1.0, 1.0));
        b.set_size(Vector3::new(2.0, -1.0, -3.0));
        assert_eq!(b.get_size(), Vector3::new(2.0, 0.0, 0.0));

        let mut s = SphereOccluder3DData::new();
        assert_eq!(s.get_radius(), 1.0);
        s.set_radius(-4.0);
        assert_eq!(s.get_radius(), 0.0);
        s.set_radius(2.5);
        assert_eq!(s.get_radius(), 2.5);

        let mut p = PolygonOccluder3DData::new();
        p.set_polygon(vec![Vector2::ZERO, Vector2::new(1.0, 0.0), Vector2::new(0.0, 1.0)]);
        assert_eq!(p.get_polygon().len(), 3);
    }

    #[test]
    fn occluder_instance_defaults_and_mask() {
        let mut o = OccluderInstance3DData::new();
        assert!(o.get_occluder().is_none());
        assert_eq!(o.get_bake_mask(), 0xFFFFFFFF);
        assert_eq!(o.get_bake_simplification_distance(), 0.1);
        assert_eq!(o.visual.get_layer_mask(), 1);

        o.set_occluder(Some(Rid::from_u64(5)));
        assert_eq!(o.get_occluder(), Some(Rid::from_u64(5)));

        o.set_bake_mask(0);
        assert!(o.set_bake_mask_value(1, true));
        assert!(o.set_bake_mask_value(20, true));
        assert_eq!(o.get_bake_mask(), 1 | (1 << 19));
        assert_eq!(o.get_bake_mask_value(1), Some(true));
        assert!(!o.set_bake_mask_value(0, true));
        assert!(!o.set_bake_mask_value(21, true));
        assert_eq!(o.get_bake_mask_value(21), None);
        assert!(o.set_bake_mask_value(20, false));
        assert_eq!(o.get_bake_mask(), 1);

        o.set_bake_simplification_distance(0.5);
        assert_eq!(o.get_bake_simplification_distance(), 0.5);

        assert_eq!(BakeError::Ok as i32, 0);
        assert_eq!(BakeError::NoSavePath as i32, 1);
        assert_eq!(BakeError::NoMeshes as i32, 2);
        assert_eq!(BakeError::CantSave as i32, 3);
    }
}
