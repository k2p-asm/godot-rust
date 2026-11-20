//! `scene/resources/3d/navigation_mesh_source_geometry_data_3d.*` (state).
//!
//! Ported: the interleaved vertex/index arrays, the cached bounds (recomputed
//! from the vertex triplets), the root-node transform, the projected-obstruction
//! list (vertex floats, elevation, height, carve flag), `append_arrays`,
//! `merge`, `has_data`, `clear` and the obstruction clear/add accessors.
//!
//! Mesh parsing (`add_mesh`/`add_faces`) and the navigation bake are not ported.

use crate::math::aabb::Aabb;
use crate::math::transform_3d::Transform3D;
use crate::math::vector3::Vector3;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ProjectedObstruction {
    pub vertices: Vec<f32>,
    pub elevation: f32,
    pub height: f32,
    pub carve: bool,
}

#[derive(Clone, Debug)]
pub struct NavigationMeshSourceGeometryData3D {
    vertices: Vec<f32>,
    indices: Vec<i32>,
    bounds: Aabb,
    bounds_dirty: bool,
    projected_obstructions: Vec<ProjectedObstruction>,
    root_node_transform: Transform3D,
}

impl Default for NavigationMeshSourceGeometryData3D {
    fn default() -> Self {
        Self::new()
    }
}

impl NavigationMeshSourceGeometryData3D {
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            indices: Vec::new(),
            bounds: Aabb::default(),
            bounds_dirty: true,
            projected_obstructions: Vec::new(),
            root_node_transform: Transform3D::default(),
        }
    }

    pub fn set_vertices(&mut self, vertices: Vec<f32>) {
        self.vertices = vertices;
        self.bounds_dirty = true;
    }

    pub fn get_vertices(&self) -> &[f32] {
        &self.vertices
    }

    pub fn set_indices(&mut self, indices: Vec<i32>) {
        self.indices = indices;
    }

    pub fn get_indices(&self) -> &[i32] {
        &self.indices
    }

    /// `append_arrays`: appends the vertex floats and the indices.
    pub fn append_arrays(&mut self, vertices: &[f32], indices: &[i32]) {
        self.vertices.extend_from_slice(vertices);
        self.indices.extend_from_slice(indices);
        self.bounds_dirty = true;
    }

    /// `has_data`: any vertex or index data.
    pub fn has_data(&self) -> bool {
        !self.vertices.is_empty() || !self.indices.is_empty()
    }

    pub fn clear(&mut self) {
        self.vertices.clear();
        self.indices.clear();
        self.projected_obstructions.clear();
        self.bounds = Aabb::default();
        self.bounds_dirty = true;
    }

    pub fn set_root_node_transform(&mut self, transform: Transform3D) {
        self.root_node_transform = transform;
    }

    pub fn get_root_node_transform(&self) -> Transform3D {
        self.root_node_transform
    }

    /// `get_bounds`: recomputed from the vertex triplets when dirty; an empty
    /// geometry reports the default AABB.
    pub fn get_bounds(&mut self) -> Aabb {
        if self.bounds_dirty {
            self.bounds_dirty = false;
            let mut bounds: Option<Aabb> = None;
            for chunk in self.vertices.chunks_exact(3) {
                let point = Vector3::new(chunk[0], chunk[1], chunk[2]);
                bounds = Some(match bounds {
                    None => Aabb::new(point, Vector3::ZERO),
                    Some(mut current) => {
                        current.expand_to(point);
                        current
                    }
                });
            }
            self.bounds = bounds.unwrap_or_default();
        }
        self.bounds
    }

    /// `add_projected_obstruction`.
    pub fn add_projected_obstruction(
        &mut self,
        vertices: Vec<f32>,
        elevation: f32,
        height: f32,
        carve: bool,
    ) {
        self.projected_obstructions.push(ProjectedObstruction {
            vertices,
            elevation,
            height,
            carve,
        });
    }

    pub fn get_projected_obstructions(&self) -> &[ProjectedObstruction] {
        &self.projected_obstructions
    }

    pub fn set_projected_obstructions(&mut self, obstructions: Vec<ProjectedObstruction>) {
        self.projected_obstructions = obstructions;
    }

    pub fn clear_projected_obstructions(&mut self) {
        self.projected_obstructions.clear();
    }

    /// `merge`: appends the other geometry's arrays and obstructions.
    pub fn merge(&mut self, other: &NavigationMeshSourceGeometryData3D) {
        self.vertices.extend_from_slice(&other.vertices);
        self.indices.extend_from_slice(&other.indices);
        self.projected_obstructions
            .extend_from_slice(&other.projected_obstructions);
        self.bounds_dirty = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_and_basic_data() {
        let mut geometry = NavigationMeshSourceGeometryData3D::new();
        assert!(!geometry.has_data());
        assert!(geometry.get_vertices().is_empty());
        assert!(geometry.get_indices().is_empty());
        assert!(geometry.get_projected_obstructions().is_empty());

        geometry.set_vertices(vec![0.0, 0.0, 0.0, 1.0, 2.0, 3.0]);
        geometry.set_indices(vec![0, 1, 0]);
        assert!(geometry.has_data());
        assert_eq!(geometry.get_vertices().len(), 6);
        assert_eq!(geometry.get_indices(), &[0, 1, 0]);
        let bounds = geometry.get_bounds();
        assert_eq!(bounds.position, Vector3::ZERO);
        assert_eq!(bounds.size, Vector3::new(1.0, 2.0, 3.0));

        geometry.clear();
        assert!(!geometry.has_data());
        assert_eq!(geometry.get_bounds(), Aabb::default());
    }

    #[test]
    fn append_and_merge() {
        let mut geometry = NavigationMeshSourceGeometryData3D::new();
        geometry.append_arrays(&[0.0, 0.0, 0.0], &[0]);
        geometry.append_arrays(&[1.0, 1.0, 1.0], &[1, 0]);
        assert_eq!(geometry.get_vertices().len(), 6);
        assert_eq!(geometry.get_indices(), &[0, 1, 0]);
        assert_eq!(geometry.get_bounds().size, Vector3::ONE);

        let mut other = NavigationMeshSourceGeometryData3D::new();
        other.append_arrays(&[-1.0, -1.0, -1.0], &[3]);
        other.add_projected_obstruction(vec![0.0, 0.0, 1.0, 0.0, 1.0], 0.5, 2.0, true);
        geometry.merge(&other);
        assert_eq!(geometry.get_vertices().len(), 9);
        assert_eq!(geometry.get_indices(), &[0, 1, 0, 3]);
        assert_eq!(geometry.get_projected_obstructions().len(), 1);
        let bounds = geometry.get_bounds();
        assert_eq!(bounds.position, Vector3::new(-1.0, -1.0, -1.0));
        assert_eq!(bounds.size, Vector3::new(2.0, 2.0, 2.0));
    }

    #[test]
    fn obstructions_and_transform() {
        let mut geometry = NavigationMeshSourceGeometryData3D::new();
        geometry.add_projected_obstruction(vec![1.0, 2.0], 0.25, 1.5, false);
        geometry.add_projected_obstruction(vec![3.0, 4.0], 0.5, 2.5, true);
        assert_eq!(geometry.get_projected_obstructions().len(), 2);
        assert_eq!(geometry.get_projected_obstructions()[0].elevation, 0.25);
        assert!(!geometry.get_projected_obstructions()[0].carve);
        assert!(geometry.get_projected_obstructions()[1].carve);

        let mut replacement = vec![ProjectedObstruction {
            vertices: vec![9.0],
            elevation: 1.0,
            height: 3.0,
            carve: true,
        }];
        geometry.set_projected_obstructions(replacement.clone());
        assert_eq!(geometry.get_projected_obstructions(), &replacement[..]);
        geometry.clear_projected_obstructions();
        assert!(geometry.get_projected_obstructions().is_empty());

        let transform = Transform3D::default();
        geometry.set_root_node_transform(transform);
        assert_eq!(geometry.get_root_node_transform(), transform);
        replacement.clear();
    }

    #[test]
    fn bounds_ignore_non_triplet_tails() {
        let mut geometry = NavigationMeshSourceGeometryData3D::new();
        geometry.set_vertices(vec![2.0, 3.0, 4.0, 0.0, 0.0, 0.0, 7.0]);
        let bounds = geometry.get_bounds();
        assert_eq!(bounds.position, Vector3::ZERO);
        assert_eq!(bounds.size, Vector3::new(2.0, 3.0, 4.0));
        // Repeated calls reuse the cache.
        let again = geometry.get_bounds();
        assert_eq!(again, bounds);
    }
}
