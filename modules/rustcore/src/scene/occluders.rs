//! `scene/3d/occluder_instance_3d.*` (state only).
//!
//! `Occluder3D` keeps the baked vertex/index arrays and their AABB;
//! `ArrayOccluder3D` fills them from caller data, while `QuadOccluder3D`,
//! `BoxOccluder3D`, `SphereOccluder3D` and `PolygonOccluder3D` keep their shape
//! parameters (size `(1,1)` / `(1,1,1)`, radius 1, empty polygon).
//!
//! `OccluderInstance3D` defaults to no occluder, `0xFFFFFFFF` bake mask and a
//! 0.1 bake simplification distance, with the `1..=32` mask bit helpers.
//!
//! Baking (`bake_scene`), the server RIDs and the mesh/array generation of the
//! primitive occluders are not ported.

use crate::math::aabb::Aabb;
use crate::math::vector2::Vector2;
use crate::math::vector3::Vector3;
use crate::servers::rid::Rid;

/// `Occluder3D`'s baked geometry.
#[derive(Clone, Debug, Default)]
pub struct Occluder3DData {
    vertices: Vec<Vector3>,
    indices: Vec<i32>,
    aabb: Aabb,
}

impl Occluder3DData {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_vertices(&self) -> &[Vector3] {
        &self.vertices
    }

    pub fn get_indices(&self) -> &[i32] {
        &self.indices
    }

    pub fn get_aabb(&self) -> Aabb {
        self.aabb
    }

    /// `set_arrays`: stores the data and recomputes the AABB from the vertices.
    pub fn set_arrays(&mut self, vertices: Vec<Vector3>, indices: Vec<i32>) {
        let mut aabb = Aabb::default();
        for (index, vertex) in vertices.iter().enumerate() {
            if index == 0 {
                aabb = Aabb::new(*vertex, Vector3::ZERO);
            } else {
                aabb.expand_to(*vertex);
            }
        }
        self.vertices = vertices;
        self.indices = indices;
        self.aabb = aabb;
    }
}

#[derive(Clone, Debug, Default)]
pub struct ArrayOccluder3DData {
    pub occluder: Occluder3DData,
}

impl ArrayOccluder3DData {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_arrays(&mut self, vertices: Vec<Vector3>, indices: Vec<i32>) {
        self.occluder.set_arrays(vertices, indices);
    }

    pub fn set_vertices(&mut self, vertices: Vec<Vector3>) {
        let indices = self.occluder.indices.clone();
        self.occluder.set_arrays(vertices, indices);
    }

    pub fn set_indices(&mut self, indices: Vec<i32>) {
        let vertices = self.occluder.vertices.clone();
        self.occluder.set_arrays(vertices, indices);
    }
}

#[derive(Clone, Debug)]
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

    pub fn get_size(&self) -> Vector2 {
        self.size
    }

    pub fn set_size(&mut self, size: Vector2) {
        self.size = size;
    }
}

#[derive(Clone, Debug)]
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

    pub fn get_size(&self) -> Vector3 {
        self.size
    }

    pub fn set_size(&mut self, size: Vector3) {
        self.size = size;
    }
}

pub const SPHERE_OCCLUDER_RINGS: i32 = 7;
pub const SPHERE_OCCLUDER_RADIAL_SEGMENTS: i32 = 7;

#[derive(Clone, Debug)]
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

    pub fn get_radius(&self) -> f32 {
        self.radius
    }

    pub fn set_radius(&mut self, radius: f32) {
        self.radius = radius;
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
pub enum OccluderBakeError {
    Ok,
    NoSavePath,
    NoMeshes,
    CantSave,
}

#[derive(Clone, Debug)]
pub struct OccluderInstance3DData {
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

    pub fn set_bake_mask_value(&mut self, layer: i32, value: bool) -> bool {
        if !(1..=32).contains(&layer) {
            return false;
        }
        let bit = 1u32 << (layer - 1);
        if value {
            self.bake_mask |= bit;
        } else {
            self.bake_mask &= !bit;
        }
        true
    }

    pub fn get_bake_mask_value(&self, layer: i32) -> bool {
        if !(1..=32).contains(&layer) {
            return false;
        }
        self.bake_mask & (1u32 << (layer - 1)) != 0
    }

    pub fn set_bake_simplification_distance(&mut self, distance: f32) {
        self.bake_simplification_distance = distance;
    }

    pub fn get_bake_simplification_distance(&self) -> f32 {
        self.bake_simplification_distance
    }

    /// `get_aabb`: the occluder's bounds (zero without an occluder handle).
    pub fn get_aabb(&self) -> Aabb {
        Aabb::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn occluder_arrays_and_aabb() {
        let mut occluder = Occluder3DData::new();
        assert!(occluder.get_vertices().is_empty());
        assert!(occluder.get_indices().is_empty());
        assert_eq!(occluder.get_aabb(), Aabb::default());

        occluder.set_arrays(
            vec![
                Vector3::new(0.0, 0.0, 0.0),
                Vector3::new(1.0, 0.0, 0.0),
                Vector3::new(0.0, 1.0, 0.0),
            ],
            vec![0, 1, 2],
        );
        assert_eq!(occluder.get_vertices().len(), 3);
        assert_eq!(occluder.get_indices(), &[0, 1, 2]);
        let aabb = occluder.get_aabb();
        assert_eq!(aabb.position, Vector3::ZERO);
        assert_eq!(aabb.size, Vector3::new(1.0, 1.0, 0.0));

        let mut array = ArrayOccluder3DData::new();
        array.set_arrays(
            vec![Vector3::ZERO, Vector3::ONE],
            vec![0, 1, 0],
        );
        assert_eq!(array.occluder.get_vertices().len(), 2);
        array.set_indices(vec![0, 1, 1]);
        assert_eq!(array.occluder.get_indices(), &[0, 1, 1]);
        assert_eq!(array.occluder.get_vertices().len(), 2);
        array.set_vertices(vec![Vector3::new(2.0, 2.0, 2.0)]);
        assert_eq!(array.occluder.get_aabb().position, Vector3::new(2.0, 2.0, 2.0));
        assert_eq!(array.occluder.get_indices(), &[0, 1, 1]);
    }

    #[test]
    fn primitive_occluders() {
        let mut quad = QuadOccluder3DData::new();
        assert_eq!(quad.get_size(), Vector2::new(1.0, 1.0));
        quad.set_size(Vector2::new(2.0, 3.0));
        assert_eq!(quad.get_size(), Vector2::new(2.0, 3.0));

        let mut boxed = BoxOccluder3DData::new();
        assert_eq!(boxed.get_size(), Vector3::new(1.0, 1.0, 1.0));
        boxed.set_size(Vector3::new(2.0, 4.0, 6.0));
        assert_eq!(boxed.get_size(), Vector3::new(2.0, 4.0, 6.0));

        let mut sphere = SphereOccluder3DData::new();
        assert_eq!(sphere.get_radius(), 1.0);
        sphere.set_radius(2.5);
        assert_eq!(sphere.get_radius(), 2.5);
        assert_eq!(SPHERE_OCCLUDER_RINGS, 7);
        assert_eq!(SPHERE_OCCLUDER_RADIAL_SEGMENTS, 7);

        let mut polygon = PolygonOccluder3DData::new();
        assert!(polygon.get_polygon().is_empty());
        polygon.set_polygon(vec![Vector2::ZERO, Vector2::ONE, Vector2::new(1.0, 0.0)]);
        assert_eq!(polygon.get_polygon().len(), 3);
    }

    #[test]
    fn instance_defaults_and_mask() {
        let mut instance = OccluderInstance3DData::new();
        assert!(instance.get_occluder().is_none());
        assert_eq!(instance.get_bake_mask(), 0xFFFFFFFF);
        assert_eq!(instance.get_bake_simplification_distance(), 0.1);
        assert_eq!(instance.get_aabb(), Aabb::default());

        instance.set_occluder(Some(Rid::from_u64(2)));
        instance.set_bake_simplification_distance(0.5);
        assert_eq!(instance.get_occluder(), Some(Rid::from_u64(2)));
        assert_eq!(instance.get_bake_simplification_distance(), 0.5);

        instance.set_bake_mask(0);
        assert!(instance.set_bake_mask_value(1, true));
        assert!(instance.set_bake_mask_value(32, true));
        assert_eq!(instance.get_bake_mask(), 1 | (1 << 31));
        assert!(instance.get_bake_mask_value(32));
        assert!(!instance.set_bake_mask_value(0, true));
        assert!(!instance.set_bake_mask_value(33, true));
        assert!(!instance.get_bake_mask_value(33));
        assert_eq!(OccluderBakeError::Ok as i32, 0);
        assert_eq!(OccluderBakeError::CantSave as i32, 3);
    }
}
