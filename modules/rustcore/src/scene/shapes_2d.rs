//! `scene/resources/2d/*_shape_2d.*`: the physical 2D shapes.
//!
//! Each shape carries its dimensions plus `get_rect` and `get_enclosing_radius`,
//! copied operation by operation from the C++ class. Validation matches the
//! engine setters:
//!
//! - rectangle / circle / capsule reject negative dimensions;
//! - the capsule keeps `height >= radius * 2` in both directions and exposes
//!   `mid_height = height - 2 * radius`;
//! - the world boundary rejects an approximately zero normal but accepts a
//!   non-unit one.
//!
//! The physics server side (`shape_set_data`) and the editor `draw` helpers are
//! not ported.

use crate::math::funcs;
use crate::math::rect2::Rect2;
use crate::math::vector2::Vector2;

#[derive(Clone, Copy, Debug)]
pub struct RectangleShape2D {
    size: Vector2,
}

impl Default for RectangleShape2D {
    fn default() -> Self {
        Self::new()
    }
}

impl RectangleShape2D {
    pub fn new() -> Self {
        Self {
            size: Vector2::new(20.0, 20.0),
        }
    }

    /// Rejects a negative component, keeping the old size.
    pub fn set_size(&mut self, size: Vector2) -> bool {
        if size.x < 0.0 || size.y < 0.0 {
            return false;
        }
        self.size = size;
        true
    }

    pub fn get_size(&self) -> Vector2 {
        self.size
    }

    pub fn get_rect(&self) -> Rect2 {
        Rect2::new(-self.size * 0.5, self.size)
    }

    pub fn get_enclosing_radius(&self) -> f32 {
        self.size.length() / 2.0
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CircleShape2D {
    radius: f32,
}

impl Default for CircleShape2D {
    fn default() -> Self {
        Self::new()
    }
}

impl CircleShape2D {
    pub fn new() -> Self {
        Self { radius: 10.0 }
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

    pub fn get_rect(&self) -> Rect2 {
        Rect2::new(
            Vector2::new(-self.radius, -self.radius),
            Vector2::new(self.radius, self.radius) * 2.0,
        )
    }

    pub fn get_enclosing_radius(&self) -> f32 {
        self.radius
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CapsuleShape2D {
    height: f32,
    radius: f32,
}

impl Default for CapsuleShape2D {
    fn default() -> Self {
        Self::new()
    }
}

impl CapsuleShape2D {
    pub fn new() -> Self {
        Self {
            height: 30.0,
            radius: 10.0,
        }
    }

    /// Rejects a negative radius; grows the height if it is now too small.
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

    /// Rejects a negative height; shrinks the radius if it is now too large.
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

    /// `height = mid_height + 2 * radius`; negative mid-heights are rejected.
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

    pub fn get_rect(&self) -> Rect2 {
        let half_size = Vector2::new(self.radius, self.height * 0.5);
        Rect2::new(-half_size, half_size * 2.0)
    }

    pub fn get_enclosing_radius(&self) -> f32 {
        self.height * 0.5
    }
}

#[derive(Clone, Copy, Debug)]
pub struct SegmentShape2D {
    a: Vector2,
    b: Vector2,
}

impl Default for SegmentShape2D {
    fn default() -> Self {
        Self::new()
    }
}

impl SegmentShape2D {
    pub fn new() -> Self {
        Self {
            a: Vector2::ZERO,
            b: Vector2::new(0.0, 10.0),
        }
    }

    pub fn set_a(&mut self, a: Vector2) {
        self.a = a;
    }

    pub fn get_a(&self) -> Vector2 {
        self.a
    }

    pub fn set_b(&mut self, b: Vector2) {
        self.b = b;
    }

    pub fn get_b(&self) -> Vector2 {
        self.b
    }

    pub fn get_rect(&self) -> Rect2 {
        let mut rect = Rect2::new(self.a, Vector2::ZERO);
        rect.expand_to(self.b);
        rect
    }

    /// Note: the engine returns the length of `a + b`, not of `b - a`.
    pub fn get_enclosing_radius(&self) -> f32 {
        (self.a + self.b).length()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct WorldBoundaryShape2D {
    normal: Vector2,
    distance: f32,
}

impl Default for WorldBoundaryShape2D {
    fn default() -> Self {
        Self::new()
    }
}

impl WorldBoundaryShape2D {
    pub fn new() -> Self {
        Self {
            normal: Vector2::new(0.0, -1.0),
            distance: 0.0,
        }
    }

    /// A zero (approximately) normal is rejected; non-unit normals are allowed.
    pub fn set_normal(&mut self, normal: Vector2) -> bool {
        if normal.is_zero_approx() {
            return false;
        }
        self.normal = normal;
        true
    }

    pub fn get_normal(&self) -> Vector2 {
        self.normal
    }

    pub fn set_distance(&mut self, distance: f32) {
        self.distance = distance;
    }

    pub fn get_distance(&self) -> f32 {
        self.distance
    }

    pub fn get_rect(&self) -> Rect2 {
        let point = self.normal * self.distance;
        let ortho = self.normal.orthogonal();
        let l1 = [point - ortho * 100.0, point + ortho * 100.0];
        let l2 = [point, point + self.normal * 30.0];

        let mut rect = Rect2::new(l1[0], Vector2::ZERO);
        rect.expand_to(l1[1]);
        rect.expand_to(l2[0]);
        rect.expand_to(l2[1]);
        rect
    }

    /// The engine returns `distance` itself here.
    pub fn get_enclosing_radius(&self) -> f32 {
        self.distance
    }
}

#[derive(Clone, Copy, Debug)]
pub struct SeparationRayShape2D {
    length: f32,
    slide_on_slope: bool,
}

impl Default for SeparationRayShape2D {
    fn default() -> Self {
        Self::new()
    }
}

impl SeparationRayShape2D {
    pub fn new() -> Self {
        Self {
            length: 20.0,
            slide_on_slope: false,
        }
    }

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

    /// Segment from the origin to `(0, length)`, grown by `SQRT12 * 4`.
    pub fn get_rect(&self) -> Rect2 {
        let mut rect = Rect2::new(Vector2::ZERO, Vector2::ZERO);
        rect.expand_to(Vector2::new(0.0, self.length));
        rect.grow(funcs::SQRT12 as f32 * 4.0)
    }

    pub fn get_enclosing_radius(&self) -> f32 {
        self.length
    }
}

#[derive(Clone, Debug, Default)]
pub struct ConvexPolygonShape2D {
    points: Vec<Vector2>,
}

impl ConvexPolygonShape2D {
    pub fn new() -> Self {
        Self::default()
    }

    /// Computes the hull first; fewer than three hull points is rejected and
    /// the stored polygon is left untouched (`ERR_FAIL_COND(hull.size() < 3)`).
    pub fn set_point_cloud(&mut self, cloud: &[Vector2]) -> bool {
        let hull = crate::math::geometry_2d::convex_hull(cloud);
        if hull.len() < 3 {
            return false;
        }
        self.points = hull;
        true
    }

    /// `set_points` stores the vertices as given, with no validation.
    pub fn set_points(&mut self, points: Vec<Vector2>) {
        self.points = points;
    }

    pub fn get_points(&self) -> &[Vector2] {
        &self.points
    }
}

#[derive(Clone, Debug, Default)]
pub struct ConcavePolygonShape2D {
    segments: Vec<Vector2>,
}

impl ConcavePolygonShape2D {
    pub fn new() -> Self {
        Self::default()
    }

    /// The engine forwards the pairs untouched; an odd count is only skipped
    /// when drawing.
    pub fn set_segments(&mut self, segments: Vec<Vector2>) {
        self.segments = segments;
    }

    pub fn get_segments(&self) -> &[Vector2] {
        &self.segments
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rectangle_shape() {
        let mut r = RectangleShape2D::new();
        assert_eq!(r.get_size(), Vector2::new(20.0, 20.0));
        assert_eq!(r.get_rect(), Rect2::new(Vector2::new(-10.0, -10.0), Vector2::new(20.0, 20.0)));
        assert!((r.get_enclosing_radius() - 200.0f32.sqrt()).abs() < 1e-4);

        assert!(r.set_size(Vector2::new(4.0, 6.0)));
        assert_eq!(r.get_rect(), Rect2::new(Vector2::new(-2.0, -3.0), Vector2::new(4.0, 6.0)));
        assert!((r.get_enclosing_radius() - (52.0f32).sqrt() / 2.0).abs() < 1e-4);

        assert!(!r.set_size(Vector2::new(-1.0, 2.0)));
        assert_eq!(r.get_size(), Vector2::new(4.0, 6.0));
        // Zero is allowed.
        assert!(r.set_size(Vector2::ZERO));
    }

    #[test]
    fn circle_shape() {
        let mut c = CircleShape2D::new();
        assert_eq!(c.get_radius(), 10.0);
        assert_eq!(c.get_rect(), Rect2::new(Vector2::new(-10.0, -10.0), Vector2::new(20.0, 20.0)));
        assert_eq!(c.get_enclosing_radius(), 10.0);

        assert!(!c.set_radius(-1.0));
        assert_eq!(c.get_radius(), 10.0);
        assert!(c.set_radius(0.0));
        assert_eq!(c.get_rect(), Rect2::new(Vector2::ZERO, Vector2::ZERO));
    }

    #[test]
    fn capsule_shape_keeps_height_and_radius_consistent() {
        let mut c = CapsuleShape2D::new();
        assert_eq!(c.get_height(), 30.0);
        assert_eq!(c.get_radius(), 10.0);
        assert_eq!(c.get_mid_height(), 10.0);
        assert_eq!(c.get_rect(), Rect2::new(Vector2::new(-10.0, -15.0), Vector2::new(20.0, 30.0)));
        assert_eq!(c.get_enclosing_radius(), 15.0);

        // A radius that would make the capsule shorter than two radii grows it.
        assert!(c.set_radius(25.0));
        assert_eq!(c.get_radius(), 25.0);
        assert_eq!(c.get_height(), 50.0);
        assert_eq!(c.get_mid_height(), 0.0);

        // A height smaller than the radius shrinks the radius.
        assert!(c.set_height(20.0));
        assert_eq!(c.get_height(), 20.0);
        assert_eq!(c.get_radius(), 10.0);

        // Mid-height sets the straight section.
        assert!(c.set_mid_height(40.0));
        assert_eq!(c.get_height(), 60.0);
        assert_eq!(c.get_mid_height(), 40.0);

        assert!(!c.set_radius(-1.0));
        assert!(!c.set_height(-1.0));
        assert!(!c.set_mid_height(-1.0));
    }

    #[test]
    fn segment_shape() {
        let mut s = SegmentShape2D::new();
        assert_eq!(s.get_a(), Vector2::ZERO);
        assert_eq!(s.get_b(), Vector2::new(0.0, 10.0));
        assert_eq!(s.get_rect(), Rect2::new(Vector2::ZERO, Vector2::new(0.0, 10.0)));
        assert_eq!(s.get_enclosing_radius(), 10.0);

        s.set_a(Vector2::new(5.0, 5.0));
        s.set_b(Vector2::new(-5.0, 15.0));
        assert_eq!(s.get_rect(), Rect2::new(Vector2::new(-5.0, 5.0), Vector2::new(10.0, 10.0)));
        // (a + b).length(), not the segment length.
        assert_eq!(s.get_enclosing_radius(), Vector2::new(0.0, 20.0).length());
    }

    #[test]
    fn world_boundary_shape() {
        let mut w = WorldBoundaryShape2D::new();
        assert_eq!(w.get_normal(), Vector2::new(0.0, -1.0));
        assert_eq!(w.get_distance(), 0.0);
        assert_eq!(w.get_enclosing_radius(), 0.0);

        // Default: the line is the X axis, the arrow points down. Edges are
        // exclusive in `has_point`, so sample the interior.
        let r = w.get_rect();
        assert!((r.position.x + 100.0).abs() < 1e-4, "{:?}", r.position);
        assert!((r.position.y + 30.0).abs() < 1e-4, "{:?}", r.position);
        assert!((r.size.x - 200.0).abs() < 1e-4, "{:?}", r.size);
        assert!((r.size.y - 30.0).abs() < 1e-4, "{:?}", r.size);
        assert!(r.has_point(Vector2::new(0.0, -15.0)));
        assert!(r.has_point(Vector2::new(-50.0, -10.0)));
        assert!(r.has_point(Vector2::new(50.0, -20.0)));

        assert!(!w.set_normal(Vector2::ZERO));
        assert_eq!(w.get_normal(), Vector2::new(0.0, -1.0));
        // Non-unit normals are accepted.
        assert!(w.set_normal(Vector2::new(0.0, -2.0)));
        assert_eq!(w.get_normal(), Vector2::new(0.0, -2.0));

        w.set_distance(5.0);
        assert_eq!(w.get_distance(), 5.0);
        assert_eq!(w.get_enclosing_radius(), 5.0);
    }

    #[test]
    fn convex_and_concave_polygon_shapes() {
        let mut c = ConvexPolygonShape2D::new();
        assert!(c.get_points().is_empty());
        // Two points cannot form a hull: rejected, nothing stored.
        assert!(!c.set_point_cloud(&[Vector2::ZERO, Vector2::new(1.0, 0.0)]));
        assert!(c.get_points().is_empty());

        // Square plus an interior point: the hull keeps the four corners, and
        // `Geometry2D::convex_hull` repeats the first point at the end.
        let cloud = [
            Vector2::ZERO,
            Vector2::new(4.0, 0.0),
            Vector2::new(4.0, 4.0),
            Vector2::new(0.0, 4.0),
            Vector2::new(2.0, 2.0),
        ];
        assert!(c.set_point_cloud(&cloud));
        assert_eq!(c.get_points().len(), 5);
        assert_eq!(c.get_points()[0], *c.get_points().last().unwrap());
        for p in &cloud[..4] {
            assert!(c.get_points().contains(p), "missing {p:?}");
        }
        assert!(!c.get_points().contains(&Vector2::new(2.0, 2.0)));

        // set_points itself accepts anything.
        c.set_points(vec![Vector2::ZERO, Vector2::new(1.0, 1.0)]);
        assert_eq!(c.get_points().len(), 2);

        let mut k = ConcavePolygonShape2D::new();
        assert!(k.get_segments().is_empty());
        k.set_segments(vec![
            Vector2::ZERO,
            Vector2::new(1.0, 0.0),
            Vector2::new(1.0, 0.0),
            Vector2::new(1.0, 1.0),
        ]);
        assert_eq!(k.get_segments().len(), 4);
        assert_eq!(k.get_segments()[3], Vector2::new(1.0, 1.0));
    }

    #[test]
    fn separation_ray_shape() {
        let mut s = SeparationRayShape2D::new();
        assert_eq!(s.get_length(), 20.0);
        assert!(!s.get_slide_on_slope());
        assert_eq!(s.get_enclosing_radius(), 20.0);

        let grow = funcs::SQRT12 as f32 * 4.0;
        let r = s.get_rect();
        assert!((r.position.x + grow).abs() < 1e-4, "{:?}", r.position);
        assert!((r.position.y + grow).abs() < 1e-4, "{:?}", r.position);
        assert!((r.size.y - (20.0 + grow * 2.0)).abs() < 1e-4, "{:?}", r.size);

        s.set_length(5.0);
        s.set_slide_on_slope(true);
        assert_eq!(s.get_length(), 5.0);
        assert!(s.get_slide_on_slope());
    }
}
