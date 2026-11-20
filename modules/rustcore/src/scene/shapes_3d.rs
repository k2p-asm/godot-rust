//! `scene/resources/3d/*_shape_3d.*`: physics shape resources (state only).
//!
//! Defaults come from each constructor:
//!
//! - `BoxShape3D` `(1,1,1)`, `SphereShape3D` radius 0.5,
//! - `CapsuleShape3D` radius 0.5 / height 2, `CylinderShape3D` the same,
//! - `SeparationRayShape3D` length 1 with `slide_on_slope` off,
//! - `WorldBoundaryShape3D` plane `(0, 1, 0, 0)`.
//!
//! Validation mirrors the setters: box rejects any negative component, sphere
//! and cylinder reject negative radius/height, the capsule keeps
//! `height >= radius * 2` in both directions and exposes
//! `mid_height = height - 2 * radius`.
//!
//! `HeightMapShape3D` resizes its data to `width * depth`, ignores sizes below
//! 1 and refuses a data array whose length does not match. `ConcavePolygonShape3D`
//! stores the triangle soup and the backface flag; `ConvexPolygonShape3D` stores
//! its points as given.
//!
//! The physics server (`shape_set_data`) and debug meshes are not ported.

use crate::math::plane::Plane;
use crate::math::vector3::Vector3;

#[derive(Clone, Copy, Debug)]
pub struct BoxShape3D {
    size: Vector3,
}

impl Default for BoxShape3D {
    fn default() -> Self {
        Self::new()
    }
}

impl BoxShape3D {
    pub fn new() -> Self {
        Self {
            size: Vector3::new(1.0, 1.0, 1.0),
        }
    }

    /// Rejects any negative component, keeping the old size.
    pub fn set_size(&mut self, size: Vector3) -> bool {
        if size.x < 0.0 || size.y < 0.0 || size.z < 0.0 {
            return false;
        }
        self.size = size;
        true
    }

    pub fn get_size(&self) -> Vector3 {
        self.size
    }
}

#[derive(Clone, Copy, Debug)]
pub struct SphereShape3D {
    radius: f32,
}

impl Default for SphereShape3D {
    fn default() -> Self {
        Self::new()
    }
}

impl SphereShape3D {
    pub fn new() -> Self {
        Self { radius: 0.5 }
    }

    pub fn set_radius(&mut self, radius: f32) -> bool {
        if radius < 0.0 {
            return false;
        }
        self.radius = radius;
        true
    }

    pub fn get_radius(&self) -> f32 {
        self.radius
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CapsuleShape3D {
    radius: f32,
    height: f32,
}

impl Default for CapsuleShape3D {
    fn default() -> Self {
        Self::new()
    }
}

impl CapsuleShape3D {
    pub fn new() -> Self {
        Self {
            radius: 0.5,
            height: 2.0,
        }
    }

    /// Negative radii are rejected; the height grows to twice the radius.
    pub fn set_radius(&mut self, radius: f32) -> bool {
        if radius < 0.0 {
            return false;
        }
        self.radius = radius;
        if self.height < radius * 2.0 {
            self.height = radius * 2.0;
        }
        true
    }

    pub fn get_radius(&self) -> f32 {
        self.radius
    }

    /// Negative heights are rejected; the radius shrinks to half the height.
    pub fn set_height(&mut self, height: f32) -> bool {
        if height < 0.0 {
            return false;
        }
        self.height = height;
        if self.radius > height * 0.5 {
            self.radius = height * 0.5;
        }
        true
    }

    pub fn get_height(&self) -> f32 {
        self.height
    }

    /// `height = mid_height + 2 * radius`; negatives are rejected.
    pub fn set_mid_height(&mut self, mid_height: f32) -> bool {
        if mid_height < 0.0 {
            return false;
        }
        self.height = mid_height + self.radius * 2.0;
        true
    }

    pub fn get_mid_height(&self) -> f32 {
        self.height - self.radius * 2.0
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CylinderShape3D {
    radius: f32,
    height: f32,
}

impl Default for CylinderShape3D {
    fn default() -> Self {
        Self::new()
    }
}

impl CylinderShape3D {
    pub fn new() -> Self {
        Self {
            radius: 0.5,
            height: 2.0,
        }
    }

    pub fn set_radius(&mut self, radius: f32) -> bool {
        if radius < 0.0 {
            return false;
        }
        self.radius = radius;
        true
    }

    pub fn get_radius(&self) -> f32 {
        self.radius
    }

    pub fn set_height(&mut self, height: f32) -> bool {
        if height < 0.0 {
            return false;
        }
        self.height = height;
        true
    }

    pub fn get_height(&self) -> f32 {
        self.height
    }
}

#[derive(Clone, Copy, Debug)]
pub struct SeparationRayShape3D {
    length: f32,
    slide_on_slope: bool,
}

impl Default for SeparationRayShape3D {
    fn default() -> Self {
        Self::new()
    }
}

impl SeparationRayShape3D {
    pub fn new() -> Self {
        Self {
            length: 1.0,
            slide_on_slope: false,
        }
    }

    /// No validation, like the engine.
    pub fn set_length(&mut self, length: f32) {
        self.length = length;
    }

    pub fn get_length(&self) -> f32 {
        self.length
    }

    pub fn set_slide_on_slope(&mut self, active: bool) {
        self.slide_on_slope = active;
    }

    pub fn get_slide_on_slope(&self) -> bool {
        self.slide_on_slope
    }
}

#[derive(Clone, Copy, Debug)]
pub struct WorldBoundaryShape3D {
    plane: Plane,
}

impl Default for WorldBoundaryShape3D {
    fn default() -> Self {
        Self::new()
    }
}

impl WorldBoundaryShape3D {
    pub fn new() -> Self {
        Self {
            plane: Plane::new(0.0, 1.0, 0.0, 0.0),
        }
    }

    pub fn set_plane(&mut self, plane: Plane) {
        self.plane = plane;
    }

    pub fn get_plane(&self) -> Plane {
        self.plane
    }
}

#[derive(Clone, Debug, Default)]
pub struct ConvexPolygonShape3D {
    points: Vec<Vector3>,
}

impl ConvexPolygonShape3D {
    pub fn new() -> Self {
        Self::default()
    }

    /// `set_points` stores the points as given; the engine computes nothing.
    pub fn set_points(&mut self, points: Vec<Vector3>) {
        self.points = points;
    }

    pub fn get_points(&self) -> &[Vector3] {
        &self.points
    }
}

#[derive(Clone, Debug, Default)]
pub struct ConcavePolygonShape3D {
    faces: Vec<Vector3>,
    backface_collision: bool,
}

impl ConcavePolygonShape3D {
    pub fn new() -> Self {
        Self::default()
    }

    /// Triangle soup: three vertices per triangle, stored untouched.
    pub fn set_faces(&mut self, faces: Vec<Vector3>) {
        self.faces = faces;
    }

    pub fn get_faces(&self) -> &[Vector3] {
        &self.faces
    }

    pub fn set_backface_collision_enabled(&mut self, enabled: bool) {
        self.backface_collision = enabled;
    }

    pub fn is_backface_collision_enabled(&self) -> bool {
        self.backface_collision
    }
}

#[derive(Clone, Debug)]
pub struct HeightMapShape3D {
    width: i32,
    depth: i32,
    data: Vec<f32>,
}

impl Default for HeightMapShape3D {
    fn default() -> Self {
        Self::new()
    }
}

impl HeightMapShape3D {
    pub fn new() -> Self {
        Self {
            width: 2,
            depth: 2,
            data: vec![0.0; 4],
        }
    }

    /// Sizes below 1 are ignored; otherwise the data is resized with zeros.
    pub fn set_map_width(&mut self, width: i32) {
        if width < 1 || width == self.width {
            return;
        }
        self.width = width;
        self.data.resize((self.width * self.depth) as usize, 0.0);
    }

    pub fn get_map_width(&self) -> i32 {
        self.width
    }

    pub fn set_map_depth(&mut self, depth: i32) {
        if depth < 1 || depth == self.depth {
            return;
        }
        self.depth = depth;
        self.data.resize((self.width * self.depth) as usize, 0.0);
    }

    pub fn get_map_depth(&self) -> i32 {
        self.depth
    }

    /// Refused unless the length is exactly `width * depth`.
    pub fn set_map_data(&mut self, data: Vec<f32>) -> bool {
        if data.len() != (self.width * self.depth) as usize {
            return false;
        }
        self.data = data;
        true
    }

    pub fn get_map_data(&self) -> &[f32] {
        &self.data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn primitive_defaults_and_validation() {
        let mut b = BoxShape3D::new();
        assert_eq!(b.get_size(), Vector3::new(1.0, 1.0, 1.0));
        assert!(!b.set_size(Vector3::new(1.0, -1.0, 1.0)));
        assert_eq!(b.get_size(), Vector3::new(1.0, 1.0, 1.0));
        assert!(b.set_size(Vector3::new(2.0, 3.0, 4.0)));
        assert!(b.set_size(Vector3::ZERO));

        let mut s = SphereShape3D::new();
        assert_eq!(s.get_radius(), 0.5);
        assert!(!s.set_radius(-0.1));
        assert!(s.set_radius(0.0));

        let mut c = CylinderShape3D::new();
        assert_eq!(c.get_radius(), 0.5);
        assert_eq!(c.get_height(), 2.0);
        assert!(!c.set_radius(-1.0));
        assert!(!c.set_height(-1.0));
        // No coupling between radius and height for a cylinder.
        assert!(c.set_height(0.5));
        assert_eq!(c.get_radius(), 0.5);
    }

    #[test]
    fn capsule_keeps_height_and_radius_consistent() {
        let mut c = CapsuleShape3D::new();
        assert_eq!(c.get_radius(), 0.5);
        assert_eq!(c.get_height(), 2.0);
        assert_eq!(c.get_mid_height(), 1.0);

        assert!(c.set_radius(2.0));
        assert_eq!(c.get_height(), 4.0);
        assert_eq!(c.get_mid_height(), 0.0);

        assert!(c.set_height(1.0));
        assert_eq!(c.get_radius(), 0.5);
        assert_eq!(c.get_mid_height(), 0.0);

        assert!(c.set_mid_height(3.0));
        assert_eq!(c.get_height(), 4.0);
        assert_eq!(c.get_mid_height(), 3.0);

        assert!(!c.set_radius(-1.0));
        assert!(!c.set_height(-1.0));
        assert!(!c.set_mid_height(-1.0));
    }

    #[test]
    fn ray_and_boundary_shapes() {
        let mut r = SeparationRayShape3D::new();
        assert_eq!(r.get_length(), 1.0);
        assert!(!r.get_slide_on_slope());
        r.set_length(-2.0);
        assert_eq!(r.get_length(), -2.0);
        r.set_slide_on_slope(true);
        assert!(r.get_slide_on_slope());

        let mut w = WorldBoundaryShape3D::new();
        assert_eq!(w.get_plane().normal, Vector3::UP);
        assert_eq!(w.get_plane().d, 0.0);
        w.set_plane(Plane::new(1.0, 0.0, 0.0, -5.0));
        assert_eq!(w.get_plane().d, -5.0);
    }

    #[test]
    fn polygon_shapes_store_as_given() {
        let mut convex = ConvexPolygonShape3D::new();
        assert!(convex.get_points().is_empty());
        convex.set_points(vec![Vector3::ZERO, Vector3::ONE]);
        assert_eq!(convex.get_points().len(), 2);

        let mut concave = ConcavePolygonShape3D::new();
        assert!(concave.get_faces().is_empty());
        assert!(!concave.is_backface_collision_enabled());
        concave.set_faces(vec![Vector3::ZERO, Vector3::ONE, Vector3::UP]);
        concave.set_backface_collision_enabled(true);
        assert_eq!(concave.get_faces().len(), 3);
        assert!(concave.is_backface_collision_enabled());
    }

    #[test]
    fn heightmap_sizing() {
        let mut h = HeightMapShape3D::new();
        assert_eq!(h.get_map_width(), 2);
        assert_eq!(h.get_map_depth(), 2);
        assert_eq!(h.get_map_data(), &[0.0; 4]);

        // Width/depth below 1 are ignored.
        h.set_map_width(0);
        h.set_map_depth(-3);
        assert_eq!((h.get_map_width(), h.get_map_depth()), (2, 2));

        h.set_map_width(3);
        assert_eq!(h.get_map_data().len(), 6);
        h.set_map_depth(4);
        assert_eq!(h.get_map_data().len(), 12);

        // Data must match the map size exactly.
        assert!(!h.set_map_data(vec![1.0; 11]));
        assert!(h.set_map_data(vec![2.0; 12]));
        assert_eq!(h.get_map_data()[11], 2.0);
        assert_eq!(h.get_map_data()[0], 2.0);
    }
}
