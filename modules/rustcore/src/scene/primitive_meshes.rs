//! `scene/resources/3d/primitive_meshes.*`: the primitive mesh parameters.
//!
//! `PrimitiveMesh` adds `flip_faces` (off), `add_uv2` (off), `uv2_padding` (2.0)
//! and the lightmap `texel_size` (0.2) to `Mesh`.
//!
//! Every concrete primitive's constructor defaults are reproduced, and the
//! setters keep the engine's guards:
//!
//! - `CapsuleMesh`/`CylinderMesh`: `rings >= 0`;
//! - `SphereMesh`: `rings >= 1`;
//! - `TorusMesh`: `rings >= 3`, `ring_segments >= 3`, and `inner_radius` must
//!   differ from `outer_radius` ("Inner radius and outer radius cannot be the
//!   same.").
//!
//! Everything else (radii, heights, subdivisions, `left_to_right`) is stored
//! unvalidated, like the C++.
//!
//! Mesh generation (`_create_mesh_array`), surface creation and the lightmap UV2
//! layout are not ported; the parameters and their guards are.

use crate::math::vector3::Vector3;

/// `PrimitiveMesh`'s shared parameters.
#[derive(Clone, Copy, Debug)]
pub struct PrimitiveMeshParams {
    pub flip_faces: bool,
    pub add_uv2: bool,
    pub uv2_padding: f32,
    pub texel_size: f32,
}

impl Default for PrimitiveMeshParams {
    fn default() -> Self {
        Self {
            flip_faces: false,
            add_uv2: false,
            uv2_padding: 2.0,
            texel_size: 0.2,
        }
    }
}

#[derive(Clone, Debug)]
pub struct CapsuleMeshData {
    pub params: PrimitiveMeshParams,
    radius: f32,
    height: f32,
    radial_segments: i32,
    rings: i32,
}

impl Default for CapsuleMeshData {
    fn default() -> Self {
        Self::new()
    }
}

impl CapsuleMeshData {
    pub fn new() -> Self {
        Self {
            params: PrimitiveMeshParams::default(),
            radius: 0.5,
            height: 2.0,
            radial_segments: 64,
            rings: 8,
        }
    }

    pub fn set_radius(&mut self, radius: f32) {
        self.radius = radius;
    }

    pub fn get_radius(&self) -> f32 {
        self.radius
    }

    pub fn set_height(&mut self, height: f32) {
        self.height = height;
    }

    pub fn get_height(&self) -> f32 {
        self.height
    }

    pub fn set_radial_segments(&mut self, segments: i32) {
        self.radial_segments = segments;
    }

    pub fn get_radial_segments(&self) -> i32 {
        self.radial_segments
    }

    /// `ERR_FAIL_COND(p_rings < 0)`.
    pub fn set_rings(&mut self, rings: i32) -> bool {
        if rings < 0 {
            return false;
        }
        self.rings = rings;
        true
    }

    pub fn get_rings(&self) -> i32 {
        self.rings
    }
}

#[derive(Clone, Debug)]
pub struct BoxMeshData {
    pub params: PrimitiveMeshParams,
    size: Vector3,
    subdivide_w: i32,
    subdivide_h: i32,
    subdivide_d: i32,
}

impl Default for BoxMeshData {
    fn default() -> Self {
        Self::new()
    }
}

impl BoxMeshData {
    pub fn new() -> Self {
        Self {
            params: PrimitiveMeshParams::default(),
            size: Vector3::new(1.0, 1.0, 1.0),
            subdivide_w: 0,
            subdivide_h: 0,
            subdivide_d: 0,
        }
    }

    pub fn set_size(&mut self, size: Vector3) {
        self.size = size;
    }

    pub fn get_size(&self) -> Vector3 {
        self.size
    }

    pub fn set_subdivide_width(&mut self, subdivide: i32) {
        self.subdivide_w = subdivide;
    }

    pub fn get_subdivide_width(&self) -> i32 {
        self.subdivide_w
    }

    pub fn set_subdivide_height(&mut self, subdivide: i32) {
        self.subdivide_h = subdivide;
    }

    pub fn get_subdivide_height(&self) -> i32 {
        self.subdivide_h
    }

    pub fn set_subdivide_depth(&mut self, subdivide: i32) {
        self.subdivide_d = subdivide;
    }

    pub fn get_subdivide_depth(&self) -> i32 {
        self.subdivide_d
    }
}

#[derive(Clone, Debug)]
pub struct CylinderMeshData {
    pub params: PrimitiveMeshParams,
    top_radius: f32,
    bottom_radius: f32,
    height: f32,
    radial_segments: i32,
    rings: i32,
    cap_top: bool,
    cap_bottom: bool,
}

impl Default for CylinderMeshData {
    fn default() -> Self {
        Self::new()
    }
}

impl CylinderMeshData {
    pub fn new() -> Self {
        Self {
            params: PrimitiveMeshParams::default(),
            top_radius: 0.5,
            bottom_radius: 0.5,
            height: 2.0,
            radial_segments: 64,
            rings: 4,
            cap_top: true,
            cap_bottom: true,
        }
    }

    pub fn set_top_radius(&mut self, radius: f32) {
        self.top_radius = radius;
    }

    pub fn get_top_radius(&self) -> f32 {
        self.top_radius
    }

    pub fn set_bottom_radius(&mut self, radius: f32) {
        self.bottom_radius = radius;
    }

    pub fn get_bottom_radius(&self) -> f32 {
        self.bottom_radius
    }

    pub fn set_height(&mut self, height: f32) {
        self.height = height;
    }

    pub fn get_height(&self) -> f32 {
        self.height
    }

    pub fn set_radial_segments(&mut self, segments: i32) {
        self.radial_segments = segments;
    }

    pub fn get_radial_segments(&self) -> i32 {
        self.radial_segments
    }

    /// `ERR_FAIL_COND(p_rings < 0)`.
    pub fn set_rings(&mut self, rings: i32) -> bool {
        if rings < 0 {
            return false;
        }
        self.rings = rings;
        true
    }

    pub fn get_rings(&self) -> i32 {
        self.rings
    }

    pub fn set_cap_top(&mut self, enabled: bool) {
        self.cap_top = enabled;
    }

    pub fn is_cap_top(&self) -> bool {
        self.cap_top
    }

    pub fn set_cap_bottom(&mut self, enabled: bool) {
        self.cap_bottom = enabled;
    }

    pub fn is_cap_bottom(&self) -> bool {
        self.cap_bottom
    }
}

#[derive(Clone, Debug)]
pub struct PlaneMeshData {
    pub params: PrimitiveMeshParams,
    size: crate::math::vector2::Vector2,
    subdivide_w: i32,
    subdivide_d: i32,
    center_offset: Vector3,
}

impl Default for PlaneMeshData {
    fn default() -> Self {
        Self::new()
    }
}

impl PlaneMeshData {
    pub fn new() -> Self {
        Self {
            params: PrimitiveMeshParams::default(),
            size: crate::math::vector2::Vector2::new(2.0, 2.0),
            subdivide_w: 0,
            subdivide_d: 0,
            center_offset: Vector3::ZERO,
        }
    }

    pub fn set_size(&mut self, size: crate::math::vector2::Vector2) {
        self.size = size;
    }

    pub fn get_size(&self) -> crate::math::vector2::Vector2 {
        self.size
    }

    pub fn set_subdivide_width(&mut self, subdivide: i32) {
        self.subdivide_w = subdivide;
    }

    pub fn get_subdivide_width(&self) -> i32 {
        self.subdivide_w
    }

    pub fn set_subdivide_depth(&mut self, subdivide: i32) {
        self.subdivide_d = subdivide;
    }

    pub fn get_subdivide_depth(&self) -> i32 {
        self.subdivide_d
    }

    pub fn set_center_offset(&mut self, offset: Vector3) {
        self.center_offset = offset;
    }

    pub fn get_center_offset(&self) -> Vector3 {
        self.center_offset
    }
}

/// `QuadMesh` is a `PlaneMesh` facing +Z with zero center offset.
#[derive(Clone, Debug)]
pub struct QuadMeshData {
    pub plane: PlaneMeshData,
}

impl Default for QuadMeshData {
    fn default() -> Self {
        Self::new()
    }
}

impl QuadMeshData {
    pub fn new() -> Self {
        let mut plane = PlaneMeshData::new();
        plane.set_center_offset(Vector3::ZERO);
        Self { plane }
    }
}

#[derive(Clone, Debug)]
pub struct PrismMeshData {
    pub params: PrimitiveMeshParams,
    left_to_right: f32,
    size: Vector3,
    subdivide_w: i32,
    subdivide_h: i32,
    subdivide_d: i32,
}

impl Default for PrismMeshData {
    fn default() -> Self {
        Self::new()
    }
}

impl PrismMeshData {
    pub fn new() -> Self {
        Self {
            params: PrimitiveMeshParams::default(),
            left_to_right: 0.5,
            size: Vector3::new(1.0, 1.0, 1.0),
            subdivide_w: 0,
            subdivide_h: 0,
            subdivide_d: 0,
        }
    }

    pub fn set_left_to_right(&mut self, value: f32) {
        self.left_to_right = value;
    }

    pub fn get_left_to_right(&self) -> f32 {
        self.left_to_right
    }

    pub fn set_size(&mut self, size: Vector3) {
        self.size = size;
    }

    pub fn get_size(&self) -> Vector3 {
        self.size
    }

    pub fn set_subdivide_width(&mut self, subdivide: i32) {
        self.subdivide_w = subdivide;
    }

    pub fn get_subdivide_width(&self) -> i32 {
        self.subdivide_w
    }

    pub fn set_subdivide_height(&mut self, subdivide: i32) {
        self.subdivide_h = subdivide;
    }

    pub fn get_subdivide_height(&self) -> i32 {
        self.subdivide_h
    }

    pub fn set_subdivide_depth(&mut self, subdivide: i32) {
        self.subdivide_d = subdivide;
    }

    pub fn get_subdivide_depth(&self) -> i32 {
        self.subdivide_d
    }
}

#[derive(Clone, Debug)]
pub struct SphereMeshData {
    pub params: PrimitiveMeshParams,
    radius: f32,
    height: f32,
    radial_segments: i32,
    rings: i32,
    is_hemisphere: bool,
}

impl Default for SphereMeshData {
    fn default() -> Self {
        Self::new()
    }
}

impl SphereMeshData {
    pub fn new() -> Self {
        Self {
            params: PrimitiveMeshParams::default(),
            radius: 0.5,
            height: 1.0,
            radial_segments: 64,
            rings: 32,
            is_hemisphere: false,
        }
    }

    pub fn set_radius(&mut self, radius: f32) {
        self.radius = radius;
    }

    pub fn get_radius(&self) -> f32 {
        self.radius
    }

    pub fn set_height(&mut self, height: f32) {
        self.height = height;
    }

    pub fn get_height(&self) -> f32 {
        self.height
    }

    pub fn set_radial_segments(&mut self, segments: i32) {
        self.radial_segments = segments;
    }

    pub fn get_radial_segments(&self) -> i32 {
        self.radial_segments
    }

    /// `ERR_FAIL_COND(p_rings < 1)`.
    pub fn set_rings(&mut self, rings: i32) -> bool {
        if rings < 1 {
            return false;
        }
        self.rings = rings;
        true
    }

    pub fn get_rings(&self) -> i32 {
        self.rings
    }

    pub fn set_is_hemisphere(&mut self, hemisphere: bool) {
        self.is_hemisphere = hemisphere;
    }

    pub fn get_is_hemisphere(&self) -> bool {
        self.is_hemisphere
    }
}

#[derive(Clone, Debug)]
pub struct TorusMeshData {
    pub params: PrimitiveMeshParams,
    inner_radius: f32,
    outer_radius: f32,
    rings: i32,
    ring_segments: i32,
}

impl Default for TorusMeshData {
    fn default() -> Self {
        Self::new()
    }
}

impl TorusMeshData {
    pub fn new() -> Self {
        Self {
            params: PrimitiveMeshParams::default(),
            inner_radius: 0.5,
            outer_radius: 1.0,
            rings: 64,
            ring_segments: 32,
        }
    }

    /// Equal radii are refused ("Inner radius and outer radius cannot be the
    /// same.").
    pub fn set_inner_radius(&mut self, radius: f32) -> bool {
        if radius == self.outer_radius {
            return false;
        }
        self.inner_radius = radius;
        true
    }

    pub fn get_inner_radius(&self) -> f32 {
        self.inner_radius
    }

    pub fn set_outer_radius(&mut self, radius: f32) -> bool {
        if radius == self.inner_radius {
            return false;
        }
        self.outer_radius = radius;
        true
    }

    pub fn get_outer_radius(&self) -> f32 {
        self.outer_radius
    }

    /// `ERR_FAIL_COND(p_rings < 3)`.
    pub fn set_rings(&mut self, rings: i32) -> bool {
        if rings < 3 {
            return false;
        }
        self.rings = rings;
        true
    }

    pub fn get_rings(&self) -> i32 {
        self.rings
    }

    /// `ERR_FAIL_COND(p_ring_segments < 3)`.
    pub fn set_ring_segments(&mut self, segments: i32) -> bool {
        if segments < 3 {
            return false;
        }
        self.ring_segments = segments;
        true
    }

    pub fn get_ring_segments(&self) -> i32 {
        self.ring_segments
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::vector2::Vector2;

    #[test]
    fn base_parameters() {
        let p = PrimitiveMeshParams::default();
        assert!(!p.flip_faces);
        assert!(!p.add_uv2);
        assert_eq!(p.uv2_padding, 2.0);
        assert_eq!(p.texel_size, 0.2);
    }

    #[test]
    fn capsule_defaults_and_rings() {
        let mut c = CapsuleMeshData::new();
        assert_eq!(c.get_radius(), 0.5);
        assert_eq!(c.get_height(), 2.0);
        assert_eq!(c.get_radial_segments(), 64);
        assert_eq!(c.get_rings(), 8);

        assert!(!c.set_rings(-1));
        assert!(c.set_rings(0));
        assert_eq!(c.get_rings(), 0);
        c.set_radius(1.5);
        c.set_height(4.0);
        c.set_radial_segments(8);
        assert_eq!(c.get_radius(), 1.5);
        assert_eq!(c.get_height(), 4.0);
        assert_eq!(c.get_radial_segments(), 8);
    }

    #[test]
    fn box_and_plane_defaults() {
        let mut b = BoxMeshData::new();
        assert_eq!(b.get_size(), Vector3::new(1.0, 1.0, 1.0));
        assert_eq!(b.get_subdivide_width(), 0);
        assert_eq!(b.get_subdivide_height(), 0);
        assert_eq!(b.get_subdivide_depth(), 0);
        b.set_size(Vector3::new(2.0, 3.0, 4.0));
        b.set_subdivide_width(2);
        b.set_subdivide_height(3);
        b.set_subdivide_depth(4);
        assert_eq!(b.get_size(), Vector3::new(2.0, 3.0, 4.0));
        assert_eq!(b.get_subdivide_width(), 2);

        let mut p = PlaneMeshData::new();
        assert_eq!(p.get_size(), Vector2::new(2.0, 2.0));
        assert_eq!(p.get_center_offset(), Vector3::ZERO);
        p.set_size(Vector2::new(4.0, 6.0));
        p.set_subdivide_width(1);
        p.set_subdivide_depth(2);
        p.set_center_offset(Vector3::new(0.0, 1.0, 0.0));
        assert_eq!(p.get_size(), Vector2::new(4.0, 6.0));
        assert_eq!(p.get_subdivide_depth(), 2);
        assert_eq!(p.get_center_offset(), Vector3::new(0.0, 1.0, 0.0));

        let q = QuadMeshData::new();
        assert_eq!(q.plane.get_center_offset(), Vector3::ZERO);
        assert_eq!(q.plane.get_size(), Vector2::new(2.0, 2.0));
    }

    #[test]
    fn cylinder_defaults_and_rings() {
        let mut c = CylinderMeshData::new();
        assert_eq!(c.get_top_radius(), 0.5);
        assert_eq!(c.get_bottom_radius(), 0.5);
        assert_eq!(c.get_height(), 2.0);
        assert_eq!(c.get_radial_segments(), 64);
        assert_eq!(c.get_rings(), 4);
        assert!(c.is_cap_top());
        assert!(c.is_cap_bottom());

        assert!(!c.set_rings(-5));
        assert!(c.set_rings(0));
        c.set_top_radius(1.0);
        c.set_bottom_radius(2.0);
        c.set_cap_top(false);
        c.set_cap_bottom(false);
        assert_eq!(c.get_top_radius(), 1.0);
        assert_eq!(c.get_bottom_radius(), 2.0);
        assert!(!c.is_cap_top());
        assert!(!c.is_cap_bottom());
    }

    #[test]
    fn prism_defaults() {
        let mut p = PrismMeshData::new();
        assert_eq!(p.get_left_to_right(), 0.5);
        assert_eq!(p.get_size(), Vector3::new(1.0, 1.0, 1.0));
        p.set_left_to_right(0.25);
        p.set_size(Vector3::new(2.0, 2.0, 2.0));
        p.set_subdivide_width(1);
        p.set_subdivide_height(2);
        p.set_subdivide_depth(3);
        assert_eq!(p.get_left_to_right(), 0.25);
        assert_eq!(p.get_subdivide_height(), 2);
        assert_eq!(p.get_subdivide_depth(), 3);
    }

    #[test]
    fn sphere_defaults_and_rings() {
        let mut s = SphereMeshData::new();
        assert_eq!(s.get_radius(), 0.5);
        assert_eq!(s.get_height(), 1.0);
        assert_eq!(s.get_radial_segments(), 64);
        assert_eq!(s.get_rings(), 32);
        assert!(!s.get_is_hemisphere());

        assert!(!s.set_rings(0), "at least one ring");
        assert!(s.set_rings(1));
        assert_eq!(s.get_rings(), 1);
        s.set_is_hemisphere(true);
        assert!(s.get_is_hemisphere());
    }

    #[test]
    fn torus_defaults_and_guards() {
        let mut t = TorusMeshData::new();
        assert_eq!(t.get_inner_radius(), 0.5);
        assert_eq!(t.get_outer_radius(), 1.0);
        assert_eq!(t.get_rings(), 64);
        assert_eq!(t.get_ring_segments(), 32);

        // Equal radii are refused in both directions.
        assert!(!t.set_inner_radius(1.0));
        assert!(!t.set_outer_radius(0.5));
        assert!(t.set_inner_radius(0.25));
        assert_eq!(t.get_inner_radius(), 0.25);
        assert!(t.set_outer_radius(2.0));
        assert_eq!(t.get_outer_radius(), 2.0);

        assert!(!t.set_rings(2));
        assert!(t.set_rings(3));
        assert!(!t.set_ring_segments(2));
        assert!(t.set_ring_segments(3));
        assert_eq!(t.get_rings(), 3);
        assert_eq!(t.get_ring_segments(), 3);
    }
}
