//! `scene/3d/navigation/navigation_region_3d.*`, `navigation_link_3d.*` and
//! `navigation_obstacle_3d.*`: node-side navigation parameters.
//!
//! Defaults come from the headers: regions are enabled with edge connections,
//! layer 1, enter cost 0 and travel cost 1; links are enabled and bidirectional
//! with the same costs and zero start/end offsets; obstacles are 1 m tall with a
//! zero radius, avoidance on layer 1 in 2D mode, and neither affect nor carve
//! the navigation mesh.
//!
//! Validation matches the C++: `enter_cost`/`travel_cost` (region and link) and
//! the obstacle's `radius`/`height` reject negatives with the engine's messages.
//!
//! `NavigationObstacle3D::set_vertices` keeps the engine's two derived flags: the
//! clockwise test is the shoelace sign over the `(x, z)` projection (the engine
//! reports the *inverse* of `Geometry2D::is_polygon_clockwise`), and validity is
//! "triangulable". The triangle test here is the simpler `>= 3 vertices` rule,
//! because the port has no `triangulate_polygon`; that is the one approximation.
//!
//! NavigationServer3D calls, map plumbing, transforms and debug meshes are not
//! ported.

use crate::math::aabb::Aabb;
use crate::math::vector3::Vector3;
use crate::servers::rid::Rid;

// -- NavigationRegion3D -----------------------------------------------------

#[derive(Clone, Debug)]
pub struct NavigationRegion3DData {
    enabled: bool,
    use_edge_connections: bool,
    navigation_layers: u32,
    enter_cost: f32,
    travel_cost: f32,
    navigation_mesh: Option<Rid>,
    navigation_map: Option<Rid>,
    bounds: Aabb,
}

impl Default for NavigationRegion3DData {
    fn default() -> Self {
        Self::new()
    }
}

impl NavigationRegion3DData {
    pub fn new() -> Self {
        Self {
            enabled: true,
            use_edge_connections: true,
            navigation_layers: 1,
            enter_cost: 0.0,
            travel_cost: 1.0,
            navigation_mesh: None,
            navigation_map: None,
            bounds: Aabb::default(),
        }
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn get_enabled(&self) -> bool {
        self.enabled
    }

    pub fn set_use_edge_connections(&mut self, enabled: bool) {
        self.use_edge_connections = enabled;
    }

    pub fn get_use_edge_connections(&self) -> bool {
        self.use_edge_connections
    }

    pub fn set_navigation_layers(&mut self, layers: u32) {
        self.navigation_layers = layers;
    }

    pub fn get_navigation_layers(&self) -> u32 {
        self.navigation_layers
    }

    /// `ERR_FAIL_COND_MSG(enter_cost < 0, "The enter_cost must be positive.")`.
    pub fn set_enter_cost(&mut self, cost: f32) -> bool {
        if cost < 0.0 {
            return false;
        }
        self.enter_cost = cost;
        true
    }

    pub fn get_enter_cost(&self) -> f32 {
        self.enter_cost
    }

    /// `ERR_FAIL_COND_MSG(travel_cost < 0, "The travel_cost must be positive.")`.
    pub fn set_travel_cost(&mut self, cost: f32) -> bool {
        if cost < 0.0 {
            return false;
        }
        self.travel_cost = cost;
        true
    }

    pub fn get_travel_cost(&self) -> f32 {
        self.travel_cost
    }

    pub fn set_navigation_mesh(&mut self, mesh: Option<Rid>) {
        self.navigation_mesh = mesh;
    }

    pub fn get_navigation_mesh(&self) -> Option<Rid> {
        self.navigation_mesh
    }

    pub fn set_navigation_map(&mut self, map: Option<Rid>) {
        self.navigation_map = map;
    }

    pub fn get_navigation_map(&self) -> Option<Rid> {
        self.navigation_map
    }

    /// `get_bounds`: the region's cached AABB (computed from the mesh by the
    /// engine's server; stored here so callers can set it).
    pub fn get_bounds(&self) -> Aabb {
        self.bounds
    }

    pub fn set_bounds(&mut self, bounds: Aabb) {
        self.bounds = bounds;
    }
}

// -- NavigationLink3D -------------------------------------------------------

#[derive(Clone, Debug)]
pub struct NavigationLink3DData {
    enabled: bool,
    bidirectional: bool,
    navigation_layers: u32,
    start_position: Vector3,
    end_position: Vector3,
    enter_cost: f32,
    travel_cost: f32,
    navigation_map: Option<Rid>,
}

impl Default for NavigationLink3DData {
    fn default() -> Self {
        Self::new()
    }
}

impl NavigationLink3DData {
    pub fn new() -> Self {
        Self {
            enabled: true,
            bidirectional: true,
            navigation_layers: 1,
            start_position: Vector3::ZERO,
            end_position: Vector3::ZERO,
            enter_cost: 0.0,
            travel_cost: 1.0,
            navigation_map: None,
        }
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn get_enabled(&self) -> bool {
        self.enabled
    }

    pub fn set_bidirectional(&mut self, bidirectional: bool) {
        self.bidirectional = bidirectional;
    }

    pub fn get_bidirectional(&self) -> bool {
        self.bidirectional
    }

    pub fn set_navigation_layers(&mut self, layers: u32) {
        self.navigation_layers = layers;
    }

    pub fn get_navigation_layers(&self) -> u32 {
        self.navigation_layers
    }

    pub fn set_start_position(&mut self, position: Vector3) {
        self.start_position = position;
    }

    pub fn get_start_position(&self) -> Vector3 {
        self.start_position
    }

    pub fn set_end_position(&mut self, position: Vector3) {
        self.end_position = position;
    }

    pub fn get_end_position(&self) -> Vector3 {
        self.end_position
    }

    /// `ERR_FAIL_COND_MSG(enter_cost < 0, "The enter_cost must be positive.")`.
    pub fn set_enter_cost(&mut self, cost: f32) -> bool {
        if cost < 0.0 {
            return false;
        }
        self.enter_cost = cost;
        true
    }

    pub fn get_enter_cost(&self) -> f32 {
        self.enter_cost
    }

    /// `ERR_FAIL_COND_MSG(travel_cost < 0, "The travel_cost must be positive.")`.
    pub fn set_travel_cost(&mut self, cost: f32) -> bool {
        if cost < 0.0 {
            return false;
        }
        self.travel_cost = cost;
        true
    }

    pub fn get_travel_cost(&self) -> f32 {
        self.travel_cost
    }

    pub fn set_navigation_map(&mut self, map: Option<Rid>) {
        self.navigation_map = map;
    }

    pub fn get_navigation_map(&self) -> Option<Rid> {
        self.navigation_map
    }
}

// -- NavigationObstacle3D ---------------------------------------------------

#[derive(Clone, Debug)]
pub struct NavigationObstacle3DData {
    height: f32,
    radius: f32,
    vertices: Vec<Vector3>,
    vertices_are_clockwise: bool,
    vertices_are_valid: bool,
    avoidance_enabled: bool,
    avoidance_layers: u32,
    use_3d_avoidance: bool,
    velocity: Vector3,
    previous_velocity: Vector3,
    velocity_submitted: bool,
    affect_navigation_mesh: bool,
    carve_navigation_mesh: bool,
    navigation_map: Option<Rid>,
}

impl Default for NavigationObstacle3DData {
    fn default() -> Self {
        Self::new()
    }
}

impl NavigationObstacle3DData {
    pub fn new() -> Self {
        Self {
            height: 1.0,
            radius: 0.0,
            vertices: Vec::new(),
            vertices_are_clockwise: true,
            vertices_are_valid: true,
            avoidance_enabled: true,
            avoidance_layers: 1,
            use_3d_avoidance: false,
            velocity: Vector3::ZERO,
            previous_velocity: Vector3::ZERO,
            velocity_submitted: false,
            affect_navigation_mesh: false,
            carve_navigation_mesh: false,
            navigation_map: None,
        }
    }

    /// `ERR_FAIL_COND_MSG(height < 0, "Height must be positive.")`.
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

    /// `ERR_FAIL_COND_MSG(radius < 0, "Radius must be positive.")`.
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

    /// Stores the outline and refreshes the two derived flags.
    pub fn set_vertices(&mut self, vertices: Vec<Vector3>) {
        self.vertices = vertices;
        let projected: Vec<(f32, f32)> = self.vertices.iter().map(|v| (v.x, v.z)).collect();
        self.vertices_are_clockwise = !is_polygon_clockwise(&projected);
        self.vertices_are_valid = self.vertices.len() >= 3;
    }

    pub fn get_vertices(&self) -> &[Vector3] {
        &self.vertices
    }

    pub fn get_vertices_are_clockwise(&self) -> bool {
        self.vertices_are_clockwise
    }

    pub fn get_vertices_are_valid(&self) -> bool {
        self.vertices_are_valid
    }

    pub fn set_avoidance_enabled(&mut self, enabled: bool) {
        self.avoidance_enabled = enabled;
    }

    pub fn get_avoidance_enabled(&self) -> bool {
        self.avoidance_enabled
    }

    pub fn set_avoidance_layers(&mut self, layers: u32) {
        self.avoidance_layers = layers;
    }

    pub fn get_avoidance_layers(&self) -> u32 {
        self.avoidance_layers
    }

    pub fn set_use_3d_avoidance(&mut self, enabled: bool) {
        self.use_3d_avoidance = enabled;
    }

    pub fn get_use_3d_avoidance(&self) -> bool {
        self.use_3d_avoidance
    }

    /// `set_velocity` also latches `velocity_submitted`.
    pub fn set_velocity(&mut self, velocity: Vector3) {
        self.previous_velocity = self.velocity;
        self.velocity = velocity;
        self.velocity_submitted = true;
    }

    pub fn get_velocity(&self) -> Vector3 {
        self.velocity
    }

    pub fn get_previous_velocity(&self) -> Vector3 {
        self.previous_velocity
    }

    pub fn is_velocity_submitted(&self) -> bool {
        self.velocity_submitted
    }

    pub fn set_affect_navigation_mesh(&mut self, enabled: bool) {
        self.affect_navigation_mesh = enabled;
    }

    pub fn get_affect_navigation_mesh(&self) -> bool {
        self.affect_navigation_mesh
    }

    pub fn set_carve_navigation_mesh(&mut self, enabled: bool) {
        self.carve_navigation_mesh = enabled;
    }

    pub fn get_carve_navigation_mesh(&self) -> bool {
        self.carve_navigation_mesh
    }

    pub fn set_navigation_map(&mut self, map: Option<Rid>) {
        self.navigation_map = map;
    }

    pub fn get_navigation_map(&self) -> Option<Rid> {
        self.navigation_map
    }
}

/// The shoelace sign over the `(x, z)` projection: positive for the winding the
/// engine's `Geometry2D::is_polygon_clockwise` calls clockwise.
fn is_polygon_clockwise(points: &[(f32, f32)]) -> bool {
    if points.len() < 3 {
        return false;
    }
    // Godot's own formulation: sum += (x2 - x1) * (y2 + y1), clockwise when
    // the sum is positive (screen coordinates, y down).
    let mut sum = 0.0f32;
    for i in 0..points.len() {
        let (x1, y1) = points[i];
        let (x2, y2) = points[(i + 1) % points.len()];
        sum += (x2 - x1) * (y2 + y1);
    }
    sum > 0.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn region_defaults_and_costs() {
        let mut r = NavigationRegion3DData::new();
        assert!(r.get_enabled());
        assert!(r.get_use_edge_connections());
        assert_eq!(r.get_navigation_layers(), 1);
        assert_eq!(r.get_enter_cost(), 0.0);
        assert_eq!(r.get_travel_cost(), 1.0);
        assert!(r.get_navigation_mesh().is_none());
        assert!(r.get_navigation_map().is_none());
        assert_eq!(r.get_bounds(), Aabb::default());

        assert!(!r.set_enter_cost(-0.1));
        assert!(r.set_enter_cost(0.0));
        assert!(r.set_enter_cost(2.5));
        assert_eq!(r.get_enter_cost(), 2.5);
        assert!(!r.set_travel_cost(-3.0));
        assert!(r.set_travel_cost(0.0));
        assert_eq!(r.get_travel_cost(), 0.0);

        r.set_enabled(false);
        r.set_use_edge_connections(false);
        r.set_navigation_layers(0b1010);
        r.set_navigation_mesh(Some(Rid::from_u64(1)));
        r.set_navigation_map(Some(Rid::from_u64(2)));
        r.set_bounds(Aabb::new(Vector3::ZERO, Vector3::ONE));
        assert!(!r.get_enabled());
        assert!(!r.get_use_edge_connections());
        assert_eq!(r.get_navigation_layers(), 0b1010);
        assert_eq!(r.get_navigation_mesh(), Some(Rid::from_u64(1)));
        assert_eq!(r.get_navigation_map(), Some(Rid::from_u64(2)));
        assert_eq!(r.get_bounds().size, Vector3::ONE);
    }

    #[test]
    fn link_defaults_and_costs() {
        let mut l = NavigationLink3DData::new();
        assert!(l.get_enabled());
        assert!(l.get_bidirectional());
        assert_eq!(l.get_navigation_layers(), 1);
        assert_eq!(l.get_start_position(), Vector3::ZERO);
        assert_eq!(l.get_end_position(), Vector3::ZERO);
        assert_eq!(l.get_enter_cost(), 0.0);
        assert_eq!(l.get_travel_cost(), 1.0);

        assert!(!l.set_enter_cost(-1.0));
        assert!(!l.set_travel_cost(-1.0));
        assert!(l.set_enter_cost(1.0));
        assert!(l.set_travel_cost(2.0));

        l.set_enabled(false);
        l.set_bidirectional(false);
        l.set_navigation_layers(4);
        l.set_start_position(Vector3::new(1.0, 2.0, 3.0));
        l.set_end_position(Vector3::new(-1.0, 0.0, 5.0));
        assert!(!l.get_enabled());
        assert!(!l.get_bidirectional());
        assert_eq!(l.get_navigation_layers(), 4);
        assert_eq!(l.get_start_position(), Vector3::new(1.0, 2.0, 3.0));
        assert_eq!(l.get_end_position(), Vector3::new(-1.0, 0.0, 5.0));
    }

    #[test]
    fn obstacle_defaults_and_bounds() {
        let mut o = NavigationObstacle3DData::new();
        assert_eq!(o.get_height(), 1.0);
        assert_eq!(o.get_radius(), 0.0);
        assert!(o.get_avoidance_enabled());
        assert_eq!(o.get_avoidance_layers(), 1);
        assert!(!o.get_use_3d_avoidance());
        assert_eq!(o.get_velocity(), Vector3::ZERO);
        assert!(!o.is_velocity_submitted());
        assert!(!o.get_affect_navigation_mesh());
        assert!(!o.get_carve_navigation_mesh());
        assert!(o.get_vertices().is_empty());
        assert!(o.get_vertices_are_clockwise());
        assert!(o.get_vertices_are_valid());

        assert!(!o.set_radius(-0.5));
        assert!(!o.set_height(-0.5));
        assert!(o.set_radius(0.0));
        assert!(o.set_height(0.0));
        o.set_radius(1.5);
        o.set_height(2.5);
        assert_eq!(o.get_radius(), 1.5);
        assert_eq!(o.get_height(), 2.5);
    }

    #[test]
    fn obstacle_vertex_flags() {
        let mut o = NavigationObstacle3DData::new();
        // In screen coordinates (y down) this winding reads clockwise, and the
        // engine inverts its own test, so the obstacle reports *not* clockwise.
        o.set_vertices(vec![
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(0.0, 0.0, 1.0),
            Vector3::new(1.0, 0.0, 1.0),
            Vector3::new(1.0, 0.0, 0.0),
        ]);
        assert!(!o.get_vertices_are_clockwise());
        assert!(o.get_vertices_are_valid());
        assert_eq!(o.get_vertices().len(), 4);

        // The opposite winding reads as clockwise for the obstacle.
        o.set_vertices(vec![
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(1.0, 0.0, 0.0),
            Vector3::new(1.0, 0.0, 1.0),
        ]);
        assert!(o.get_vertices_are_clockwise());

        // Fewer than three points cannot be triangulated.
        o.set_vertices(vec![Vector3::ZERO, Vector3::ONE]);
        assert!(!o.get_vertices_are_valid());
        o.set_vertices(Vec::new());
        assert!(!o.get_vertices_are_valid());
    }

    #[test]
    fn obstacle_flags_and_velocity() {
        let mut o = NavigationObstacle3DData::new();
        o.set_avoidance_enabled(false);
        o.set_avoidance_layers(8);
        o.set_use_3d_avoidance(true);
        o.set_affect_navigation_mesh(true);
        o.set_carve_navigation_mesh(true);
        o.set_navigation_map(Some(Rid::from_u64(3)));
        assert!(!o.get_avoidance_enabled());
        assert_eq!(o.get_avoidance_layers(), 8);
        assert!(o.get_use_3d_avoidance());
        assert!(o.get_affect_navigation_mesh());
        assert!(o.get_carve_navigation_mesh());
        assert_eq!(o.get_navigation_map(), Some(Rid::from_u64(3)));

        o.set_velocity(Vector3::new(1.0, 0.0, 0.0));
        assert!(o.is_velocity_submitted());
        assert_eq!(o.get_velocity(), Vector3::new(1.0, 0.0, 0.0));
        assert_eq!(o.get_previous_velocity(), Vector3::ZERO);
        o.set_velocity(Vector3::new(0.0, 0.0, 2.0));
        assert_eq!(o.get_previous_velocity(), Vector3::new(1.0, 0.0, 0.0));
    }
}
