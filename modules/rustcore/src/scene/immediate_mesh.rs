//! `scene/resources/immediate_mesh.*`: immediate-mode vertex capture.
//!
//! Ported: the surface lifecycle (`surface_begin`/`surface_end`, rejecting a
//! second begin or an empty end), the attribute setters with the engine's
//! "retro-fill existing vertices when first enabled" rule, 2D/3D vertex mixing
//! rejection, per-surface primitive/material, and `clear_surfaces`.
//!
//! The packed vertex/attribute buffers, `ArrayFormat` flags and the AABB are not
//! ported; `array_len` here is the vertex count the engine stores.

use crate::math::color::Color;
use crate::math::plane::Plane;
use crate::math::vector2::Vector2;
use crate::math::vector3::Vector3;
use crate::servers::rid::Rid;

/// `Mesh::PrimitiveType`, matching `RenderingServer`'s values.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum PrimitiveType {
    Points,
    Lines,
    LineStrip,
    Triangles,
    TriangleStrip,
    Max,
}

#[derive(Clone, Debug, Default)]
pub struct ImmediateSurface {
    pub primitive: PrimitiveType,
    pub material: Option<Rid>,
    pub vertex_2d: bool,
    /// Number of vertices (`Surface::array_len` in the engine).
    pub array_len: usize,
    pub vertices: Vec<Vector3>,
    pub colors: Vec<Color>,
    pub normals: Vec<Vector3>,
    pub tangents: Vec<Plane>,
    pub uvs: Vec<Vector2>,
    pub uv2s: Vec<Vector2>,
}

impl Default for PrimitiveType {
    fn default() -> Self {
        PrimitiveType::Points
    }
}

#[derive(Clone, Debug)]
pub struct ImmediateMeshData {
    surfaces: Vec<ImmediateSurface>,
    uses_colors: bool,
    uses_normals: bool,
    uses_tangents: bool,
    uses_uvs: bool,
    uses_uv2s: bool,
    current_color: Color,
    current_normal: Vector3,
    current_tangent: Plane,
    current_uv: Vector2,
    current_uv2: Vector2,
    vertex_2d: bool,
    active_primitive: PrimitiveType,
    active_material: Option<Rid>,
    surface_active: bool,
    vertices: Vec<Vector3>,
    colors: Vec<Color>,
    normals: Vec<Vector3>,
    tangents: Vec<Plane>,
    uvs: Vec<Vector2>,
    uv2s: Vec<Vector2>,
}

impl Default for ImmediateMeshData {
    fn default() -> Self {
        Self::new()
    }
}

impl ImmediateMeshData {
    pub fn new() -> Self {
        Self {
            surfaces: Vec::new(),
            uses_colors: false,
            uses_normals: false,
            uses_tangents: false,
            uses_uvs: false,
            uses_uv2s: false,
            current_color: Color::ZERO,
            current_normal: Vector3::ZERO,
            current_tangent: Plane::default(),
            current_uv: Vector2::ZERO,
            current_uv2: Vector2::ZERO,
            vertex_2d: false,
            active_primitive: PrimitiveType::Points,
            active_material: None,
            surface_active: false,
            vertices: Vec::new(),
            colors: Vec::new(),
            normals: Vec::new(),
            tangents: Vec::new(),
            uvs: Vec::new(),
            uv2s: Vec::new(),
        }
    }

    /// Fails when a surface is already being built.
    pub fn surface_begin(&mut self, primitive: PrimitiveType, material: Option<Rid>) -> bool {
        if self.surface_active {
            return false;
        }
        self.active_primitive = primitive;
        self.active_material = material;
        self.surface_active = true;
        true
    }

    pub fn surface_set_color(&mut self, color: Color) -> bool {
        if !self.surface_active {
            return false;
        }
        if !self.uses_colors {
            self.colors = vec![color; self.vertices.len()];
            self.uses_colors = true;
        }
        self.current_color = color;
        true
    }

    pub fn surface_set_normal(&mut self, normal: Vector3) -> bool {
        if !self.surface_active {
            return false;
        }
        if !self.uses_normals {
            self.normals = vec![normal; self.vertices.len()];
            self.uses_normals = true;
        }
        self.current_normal = normal;
        true
    }

    pub fn surface_set_tangent(&mut self, tangent: Plane) -> bool {
        if !self.surface_active {
            return false;
        }
        if !self.uses_tangents {
            self.tangents = vec![tangent; self.vertices.len()];
            self.uses_tangents = true;
        }
        self.current_tangent = tangent;
        true
    }

    pub fn surface_set_uv(&mut self, uv: Vector2) -> bool {
        if !self.surface_active {
            return false;
        }
        if !self.uses_uvs {
            self.uvs = vec![uv; self.vertices.len()];
            self.uses_uvs = true;
        }
        self.current_uv = uv;
        true
    }

    pub fn surface_set_uv2(&mut self, uv2: Vector2) -> bool {
        if !self.surface_active {
            return false;
        }
        if !self.uses_uv2s {
            self.uv2s = vec![uv2; self.vertices.len()];
            self.uses_uv2s = true;
        }
        self.current_uv2 = uv2;
        true
    }

    fn push_attributes(&mut self) {
        if self.uses_colors {
            self.colors.push(self.current_color);
        }
        if self.uses_normals {
            self.normals.push(self.current_normal);
        }
        if self.uses_tangents {
            self.tangents.push(self.current_tangent);
        }
        if self.uses_uvs {
            self.uvs.push(self.current_uv);
        }
        if self.uses_uv2s {
            self.uv2s.push(self.current_uv2);
        }
    }

    /// Fails when no surface is active or when the surface already has 2D
    /// vertices (mixing is rejected).
    pub fn surface_add_vertex(&mut self, vertex: Vector3) -> bool {
        if !self.surface_active {
            return false;
        }
        if !self.vertices.is_empty() && self.vertex_2d {
            return false;
        }
        self.push_attributes();
        self.vertices.push(vertex);
        true
    }

    /// Stores `(x, y, 0)` and marks the surface as 2D.
    pub fn surface_add_vertex_2d(&mut self, vertex: Vector2) -> bool {
        if !self.surface_active {
            return false;
        }
        if !self.vertices.is_empty() && !self.vertex_2d {
            return false;
        }
        self.push_attributes();
        self.vertices.push(Vector3::new(vertex.x, vertex.y, 0.0));
        self.vertex_2d = true;
        true
    }

    /// Finishes the surface; fails when inactive or when no vertex was added.
    /// Returns the new surface index.
    pub fn surface_end(&mut self) -> Option<usize> {
        if !self.surface_active || self.vertices.is_empty() {
            return None;
        }

        let surface = ImmediateSurface {
            primitive: self.active_primitive,
            material: self.active_material,
            vertex_2d: self.vertex_2d,
            array_len: self.vertices.len(),
            vertices: std::mem::take(&mut self.vertices),
            colors: std::mem::take(&mut self.colors),
            normals: std::mem::take(&mut self.normals),
            tangents: std::mem::take(&mut self.tangents),
            uvs: std::mem::take(&mut self.uvs),
            uv2s: std::mem::take(&mut self.uv2s),
        };
        self.surfaces.push(surface);

        self.uses_colors = false;
        self.uses_normals = false;
        self.uses_tangents = false;
        self.uses_uvs = false;
        self.uses_uv2s = false;
        self.vertex_2d = false;
        self.surface_active = false;
        Some(self.surfaces.len() - 1)
    }

    pub fn clear_surfaces(&mut self) {
        self.surfaces.clear();
        self.surface_active = false;
        self.vertices.clear();
        self.colors.clear();
        self.normals.clear();
        self.tangents.clear();
        self.uvs.clear();
        self.uv2s.clear();
        self.uses_colors = false;
        self.uses_normals = false;
        self.uses_tangents = false;
        self.uses_uvs = false;
        self.uses_uv2s = false;
        self.vertex_2d = false;
    }

    pub fn get_surface_count(&self) -> usize {
        self.surfaces.len()
    }

    pub fn get_surface(&self, index: usize) -> Option<&ImmediateSurface> {
        self.surfaces.get(index)
    }

    pub fn surface_set_material(&mut self, index: usize, material: Option<Rid>) -> bool {
        match self.surfaces.get_mut(index) {
            Some(s) => {
                s.material = material;
                true
            }
            None => false,
        }
    }

    pub fn surface_get_material(&self, index: usize) -> Option<Rid> {
        self.surfaces.get(index).and_then(|s| s.material)
    }

    pub fn surface_get_primitive_type(&self, index: usize) -> Option<PrimitiveType> {
        self.surfaces.get(index).map(|s| s.primitive)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn surface_lifecycle() {
        let mut m = ImmediateMeshData::new();
        assert_eq!(m.get_surface_count(), 0);
        // Ending without beginning fails.
        assert!(m.surface_end().is_none());
        assert!(m.surface_begin(PrimitiveType::Triangles, None));
        // A second begin while active fails.
        assert!(!m.surface_begin(PrimitiveType::Lines, None));
        // Empty surfaces are rejected and stay active.
        assert!(m.surface_end().is_none());

        assert!(m.surface_add_vertex(Vector3::ZERO));
        assert_eq!(m.surface_end(), Some(0));
        assert_eq!(m.get_surface_count(), 1);
        let s = m.get_surface(0).unwrap();
        assert_eq!(s.primitive, PrimitiveType::Triangles);
        assert_eq!(s.array_len, 1);
        assert_eq!(s.vertices, vec![Vector3::ZERO]);

        // After ending, the builder is reusable.
        assert!(m.surface_begin(PrimitiveType::Lines, Some(Rid::from_u64(4))));
        assert!(m.surface_add_vertex(Vector3::new(1.0, 0.0, 0.0)));
        assert_eq!(m.surface_end(), Some(1));
        assert_eq!(m.surface_get_material(1), Some(Rid::from_u64(4)));
        assert_eq!(m.surface_get_primitive_type(1), Some(PrimitiveType::Lines));
    }

    #[test]
    fn attributes_fill_existing_vertices_when_first_enabled() {
        let mut m = ImmediateMeshData::new();
        m.surface_begin(PrimitiveType::Points, None);
        m.surface_add_vertex(Vector3::ZERO);
        m.surface_add_vertex(Vector3::new(1.0, 0.0, 0.0));

        // Enabling the colour late back-fills the two existing vertices.
        assert!(m.surface_set_color(Color::rgb(1.0, 0.0, 0.0)));
        m.surface_add_vertex(Vector3::new(2.0, 0.0, 0.0));
        m.surface_set_color(Color::rgb(0.0, 1.0, 0.0));
        m.surface_add_vertex(Vector3::new(3.0, 0.0, 0.0));
        m.surface_end();

        let s = m.get_surface(0).unwrap();
        assert_eq!(
            s.colors,
            vec![
                Color::rgb(1.0, 0.0, 0.0),
                Color::rgb(1.0, 0.0, 0.0),
                Color::rgb(1.0, 0.0, 0.0),
                Color::rgb(0.0, 1.0, 0.0),
            ]
        );
        assert_eq!(s.vertices.len(), 4);
    }

    #[test]
    fn other_attributes_and_material_edits() {
        let mut m = ImmediateMeshData::new();
        m.surface_begin(PrimitiveType::TriangleStrip, None);
        assert!(m.surface_set_normal(Vector3::UP));
        assert!(m.surface_set_tangent(Plane::new(1.0, 0.0, 0.0, 0.5)));
        assert!(m.surface_set_uv(Vector2::new(0.25, 0.75)));
        assert!(m.surface_set_uv2(Vector2::new(1.0, 1.0)));
        m.surface_add_vertex(Vector3::ZERO);
        m.surface_add_vertex(Vector3::new(1.0, 0.0, 0.0));
        m.surface_end();

        let s = m.get_surface(0).unwrap();
        assert_eq!(s.normals, vec![Vector3::UP, Vector3::UP]);
        assert_eq!(s.tangents.len(), 2);
        assert_eq!(s.tangents[0].d, 0.5);
        assert_eq!(s.uvs, vec![Vector2::new(0.25, 0.75); 2]);
        assert_eq!(s.uv2s, vec![Vector2::new(1.0, 1.0); 2]);

        assert!(m.surface_set_material(0, Some(Rid::from_u64(8))));
        assert_eq!(m.surface_get_material(0), Some(Rid::from_u64(8)));
        assert!(!m.surface_set_material(9, None));
    }

    #[test]
    fn two_d_and_three_d_vertices_cannot_mix() {
        let mut m = ImmediateMeshData::new();

        m.surface_begin(PrimitiveType::Points, None);
        assert!(m.surface_add_vertex(Vector3::ZERO));
        assert!(!m.surface_add_vertex_2d(Vector2::ZERO));
        m.surface_end();

        m.surface_begin(PrimitiveType::Points, None);
        assert!(m.surface_add_vertex_2d(Vector2::new(1.0, 2.0)));
        assert!(!m.surface_add_vertex(Vector3::ZERO));
        assert!(m.surface_end().is_some());
        let s = m.get_surface(1).unwrap();
        assert!(s.vertex_2d);
        assert_eq!(s.vertices, vec![Vector3::new(1.0, 2.0, 0.0)]);

        // Setters are rejected outside a surface.
        assert!(!m.surface_set_color(Color::ZERO));
        assert!(!m.surface_add_vertex(Vector3::ZERO));
    }

    #[test]
    fn clear_surfaces_resets_everything() {
        let mut m = ImmediateMeshData::new();
        m.surface_begin(PrimitiveType::Lines, None);
        m.surface_set_uv(Vector2::ONE);
        m.surface_add_vertex(Vector3::ZERO);
        m.surface_end();
        assert_eq!(m.get_surface_count(), 1);

        m.clear_surfaces();
        assert_eq!(m.get_surface_count(), 0);
        assert!(m.surface_end().is_none());

        // A new surface starts without attributes from the cleared one.
        m.surface_begin(PrimitiveType::Points, None);
        m.surface_add_vertex(Vector3::ZERO);
        m.surface_end();
        let s = m.get_surface(0).unwrap();
        assert!(s.uvs.is_empty());
        assert_eq!(s.primitive, PrimitiveType::Points);
    }
}
