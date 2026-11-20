//! `scene/2d/navigation/navigation_link_2d.*` and `navigation_obstacle_2d.*`
//! (state only).
//!
//! `NavigationLink2D` starts enabled and bidirectional on layer 1 with zero
//! offsets and the usual 0/1 enter/travel costs (both reject negatives with the
//! engine's "must be positive" messages).
//!
//! `NavigationObstacle2D` starts with radius 0 (negative rejected), no vertices,
//! avoidance on layer 1, world-positive vertex flags and the velocity latch. The
//! derived vertex flags use the ported `Geometry2D` helpers exactly as the node
//! does: the obstacle reports the inverse of `is_polygon_clockwise`, and validity
//! is a successful `triangulate_polygon`.
//!
//! Servers, maps, transforms and debug drawing are not ported.

use crate::math::geometry_2d;
use crate::math::vector2::Vector2;
use crate::servers::rid::Rid;

// -- NavigationLink2D -------------------------------------------------------

#[derive(Clone, Debug)]
pub struct NavigationLink2DData {
    enabled: bool,
    bidirectional: bool,
    navigation_layers: u32,
    start_position: Vector2,
    end_position: Vector2,
    enter_cost: f32,
    travel_cost: f32,
    navigation_map: Option<Rid>,
}

impl Default for NavigationLink2DData {
    fn default() -> Self {
        Self::new()
    }
}

impl NavigationLink2DData {
    pub fn new() -> Self {
        Self {
            enabled: true,
            bidirectional: true,
            navigation_layers: 1,
            start_position: Vector2::ZERO,
            end_position: Vector2::ZERO,
            enter_cost: 0.0,
            travel_cost: 1.0,
            navigation_map: None,
        }
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn set_bidirectional(&mut self, bidirectional: bool) {
        self.bidirectional = bidirectional;
    }

    pub fn is_bidirectional(&self) -> bool {
        self.bidirectional
    }

    pub fn set_navigation_layers(&mut self, layers: u32) {
        self.navigation_layers = layers;
    }

    pub fn get_navigation_layers(&self) -> u32 {
        self.navigation_layers
    }

    pub fn set_start_position(&mut self, position: Vector2) {
        self.start_position = position;
    }

    pub fn get_start_position(&self) -> Vector2 {
        self.start_position
    }

    pub fn set_end_position(&mut self, position: Vector2) {
        self.end_position = position;
    }

    pub fn get_end_position(&self) -> Vector2 {
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

// -- NavigationObstacle2D ---------------------------------------------------

#[derive(Clone, Debug)]
pub struct NavigationObstacle2DData {
    radius: f32,
    vertices: Vec<Vector2>,
    vertices_are_clockwise: bool,
    vertices_are_valid: bool,
    avoidance_enabled: bool,
    avoidance_layers: u32,
    velocity: Vector2,
    previous_velocity: Vector2,
    velocity_submitted: bool,
    affect_navigation_mesh: bool,
    carve_navigation_mesh: bool,
    navigation_map: Option<Rid>,
}

impl Default for NavigationObstacle2DData {
    fn default() -> Self {
        Self::new()
    }
}

impl NavigationObstacle2DData {
    pub fn new() -> Self {
        Self {
            radius: 0.0,
            vertices: Vec::new(),
            vertices_are_clockwise: true,
            vertices_are_valid: true,
            avoidance_enabled: true,
            avoidance_layers: 1,
            velocity: Vector2::ZERO,
            previous_velocity: Vector2::ZERO,
            velocity_submitted: false,
            affect_navigation_mesh: false,
            carve_navigation_mesh: false,
            navigation_map: None,
        }
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

    pub fn set_vertices(&mut self, vertices: Vec<Vector2>) {
        self.vertices_are_clockwise = !geometry_2d::is_polygon_clockwise(&vertices);
        self.vertices_are_valid = !geometry_2d::triangulate_polygon(&vertices).is_empty();
        self.vertices = vertices;
    }

    pub fn get_vertices(&self) -> &[Vector2] {
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

    pub fn set_velocity(&mut self, velocity: Vector2) {
        self.previous_velocity = self.velocity;
        self.velocity = velocity;
        self.velocity_submitted = true;
    }

    pub fn get_velocity(&self) -> Vector2 {
        self.velocity
    }

    pub fn get_previous_velocity(&self) -> Vector2 {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn link_defaults_and_costs() {
        let mut l = NavigationLink2DData::new();
        assert!(l.is_enabled());
        assert!(l.is_bidirectional());
        assert_eq!(l.get_navigation_layers(), 1);
        assert_eq!(l.get_start_position(), Vector2::ZERO);
        assert_eq!(l.get_end_position(), Vector2::ZERO);
        assert_eq!(l.get_enter_cost(), 0.0);
        assert_eq!(l.get_travel_cost(), 1.0);
        assert!(l.get_navigation_map().is_none());

        assert!(!l.set_enter_cost(-0.5));
        assert!(!l.set_travel_cost(-0.5));
        assert!(l.set_enter_cost(1.5));
        assert!(l.set_travel_cost(0.0));
        assert_eq!(l.get_enter_cost(), 1.5);
        assert_eq!(l.get_travel_cost(), 0.0);

        l.set_enabled(false);
        l.set_bidirectional(false);
        l.set_navigation_layers(2);
        l.set_start_position(Vector2::new(1.0, 2.0));
        l.set_end_position(Vector2::new(3.0, 4.0));
        l.set_navigation_map(Some(Rid::from_u64(5)));
        assert!(!l.is_enabled());
        assert!(!l.is_bidirectional());
        assert_eq!(l.get_navigation_layers(), 2);
        assert_eq!(l.get_start_position(), Vector2::new(1.0, 2.0));
        assert_eq!(l.get_end_position(), Vector2::new(3.0, 4.0));
        assert_eq!(l.get_navigation_map(), Some(Rid::from_u64(5)));
    }

    #[test]
    fn obstacle_defaults() {
        let mut o = NavigationObstacle2DData::new();
        assert_eq!(o.get_radius(), 0.0);
        assert!(o.get_avoidance_enabled());
        assert_eq!(o.get_avoidance_layers(), 1);
        assert!(!o.is_velocity_submitted());
        assert!(!o.get_affect_navigation_mesh());
        assert!(!o.get_carve_navigation_mesh());
        assert!(o.get_vertices_are_clockwise());
        assert!(o.get_vertices_are_valid());

        assert!(!o.set_radius(-1.0));
        assert!(o.set_radius(2.0));
        assert_eq!(o.get_radius(), 2.0);

        o.set_avoidance_enabled(false);
        o.set_avoidance_layers(4);
        o.set_affect_navigation_mesh(true);
        o.set_carve_navigation_mesh(true);
        o.set_navigation_map(Some(Rid::from_u64(1)));
        assert!(!o.get_avoidance_enabled());
        assert_eq!(o.get_avoidance_layers(), 4);
        assert!(o.get_affect_navigation_mesh());
        assert!(o.get_carve_navigation_mesh());
        assert_eq!(o.get_navigation_map(), Some(Rid::from_u64(1)));
    }

    #[test]
    fn obstacle_vertex_flags_match_geometry_2d() {
        let mut o = NavigationObstacle2DData::new();
        // Clockwise in screen coordinates, so the obstacle's inverted test is off.
        o.set_vertices(vec![
            Vector2::ZERO,
            Vector2::new(0.0, 1.0),
            Vector2::new(1.0, 1.0),
            Vector2::new(1.0, 0.0),
        ]);
        assert!(!o.get_vertices_are_clockwise());
        assert!(o.get_vertices_are_valid());
        assert_eq!(o.get_vertices().len(), 4);

        // The opposite winding is reported as clockwise.
        o.set_vertices(vec![
            Vector2::ZERO,
            Vector2::new(1.0, 0.0),
            Vector2::new(1.0, 1.0),
        ]);
        assert!(o.get_vertices_are_clockwise());

        // Too few points cannot be triangulated.
        o.set_vertices(vec![Vector2::ZERO, Vector2::ONE]);
        assert!(!o.get_vertices_are_valid());
    }

    #[test]
    fn obstacle_velocity_latch() {
        let mut o = NavigationObstacle2DData::new();
        o.set_velocity(Vector2::new(1.0, 0.0));
        assert!(o.is_velocity_submitted());
        assert_eq!(o.get_velocity(), Vector2::new(1.0, 0.0));
        assert_eq!(o.get_previous_velocity(), Vector2::ZERO);
        o.set_velocity(Vector2::new(0.0, 2.0));
        assert_eq!(o.get_previous_velocity(), Vector2::new(1.0, 0.0));
    }
}
