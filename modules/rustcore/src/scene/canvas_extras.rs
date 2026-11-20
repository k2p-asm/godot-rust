//! 2D helpers: `scene/2d/canvas_group.*`,
//! `scene/2d/visible_on_screen_notifier_2d.*` and
//! `scene/2d/navigation/navigation_region_2d.*` (state only).
//!
//! `CanvasGroup` defaults to 10 px fit/clear margins with mipmaps off.
//!
//! `VisibleOnScreenNotifier2D` starts with the engine's 2D rect
//! `(-10, -10, 20, 20)`, drawing that rect in the editor and not on screen; the
//! enabler adds the enable mode and `../AnimationPlayer` path.
//!
//! `NavigationRegion2D` mirrors the 3D region: enabled with edge connections,
//! layer 1, enter cost 0 and travel cost 1 with the engine's "must be positive"
//! rejections, plus the polygon handle and cached bounds.
//!
//! Drawing, viewport visibility tests, the navigation server and map plumbing are
//! not ported.

use crate::math::rect2::Rect2;
use crate::math::vector2::Vector2;
use crate::servers::rid::Rid;

// -- CanvasGroup ------------------------------------------------------------

#[derive(Clone, Copy, Debug)]
pub struct CanvasGroupData {
    fit_margin: f32,
    clear_margin: f32,
    use_mipmaps: bool,
}

impl Default for CanvasGroupData {
    fn default() -> Self {
        Self::new()
    }
}

impl CanvasGroupData {
    pub fn new() -> Self {
        Self {
            fit_margin: 10.0,
            clear_margin: 10.0,
            use_mipmaps: false,
        }
    }

    pub fn set_fit_margin(&mut self, margin: f32) {
        self.fit_margin = margin;
    }

    pub fn get_fit_margin(&self) -> f32 {
        self.fit_margin
    }

    pub fn set_clear_margin(&mut self, margin: f32) {
        self.clear_margin = margin;
    }

    pub fn get_clear_margin(&self) -> f32 {
        self.clear_margin
    }

    pub fn set_use_mipmaps(&mut self, enabled: bool) {
        self.use_mipmaps = enabled;
    }

    pub fn is_using_mipmaps(&self) -> bool {
        self.use_mipmaps
    }
}

// -- VisibleOnScreenNotifier2D ---------------------------------------------

#[derive(Clone, Copy, Debug)]
pub struct VisibleOnScreenNotifier2DData {
    rect: Rect2,
    show_rect: bool,
    on_screen: bool,
}

impl Default for VisibleOnScreenNotifier2DData {
    fn default() -> Self {
        Self::new()
    }
}

impl VisibleOnScreenNotifier2DData {
    pub fn new() -> Self {
        Self {
            rect: Rect2::new(Vector2::new(-10.0, -10.0), Vector2::new(20.0, 20.0)),
            show_rect: true,
            on_screen: false,
        }
    }

    pub fn set_rect(&mut self, rect: Rect2) {
        self.rect = rect;
    }

    pub fn get_rect(&self) -> Rect2 {
        self.rect
    }

    /// Editor-only flag (`show_rect`); it does not affect the visibility test.
    pub fn set_show_rect(&mut self, show: bool) {
        self.show_rect = show;
    }

    pub fn is_showing_rect(&self) -> bool {
        self.show_rect
    }

    pub fn set_on_screen(&mut self, on_screen: bool) {
        self.on_screen = on_screen;
    }

    pub fn is_on_screen(&self) -> bool {
        self.on_screen
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum EnableMode {
    Inherit,
    Always,
    WhenPaused,
}

#[derive(Clone, Debug)]
pub struct VisibleOnScreenEnabler2DData {
    pub notifier: VisibleOnScreenNotifier2DData,
    enable_mode: EnableMode,
    enable_node_path: String,
}

impl Default for VisibleOnScreenEnabler2DData {
    fn default() -> Self {
        Self::new()
    }
}

impl VisibleOnScreenEnabler2DData {
    pub fn new() -> Self {
        Self {
            notifier: VisibleOnScreenNotifier2DData::new(),
            enable_mode: EnableMode::Inherit,
            enable_node_path: "../AnimationPlayer".to_string(),
        }
    }

    pub fn set_enable_mode(&mut self, mode: EnableMode) {
        self.enable_mode = mode;
    }

    pub fn get_enable_mode(&self) -> EnableMode {
        self.enable_mode
    }

    pub fn set_enable_node_path(&mut self, path: &str) {
        self.enable_node_path = path.to_string();
    }

    pub fn get_enable_node_path(&self) -> &str {
        &self.enable_node_path
    }
}

// -- NavigationRegion2D -----------------------------------------------------

#[derive(Clone, Debug)]
pub struct NavigationRegion2DData {
    enabled: bool,
    use_edge_connections: bool,
    navigation_layers: u32,
    enter_cost: f32,
    travel_cost: f32,
    navigation_polygon: Option<Rid>,
    navigation_map: Option<Rid>,
    bounds: Rect2,
}

impl Default for NavigationRegion2DData {
    fn default() -> Self {
        Self::new()
    }
}

impl NavigationRegion2DData {
    pub fn new() -> Self {
        Self {
            enabled: true,
            use_edge_connections: true,
            navigation_layers: 1,
            enter_cost: 0.0,
            travel_cost: 1.0,
            navigation_polygon: None,
            navigation_map: None,
            bounds: Rect2::default(),
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

    pub fn set_navigation_polygon(&mut self, polygon: Option<Rid>) {
        self.navigation_polygon = polygon;
    }

    pub fn get_navigation_polygon(&self) -> Option<Rid> {
        self.navigation_polygon
    }

    pub fn set_navigation_map(&mut self, map: Option<Rid>) {
        self.navigation_map = map;
    }

    pub fn get_navigation_map(&self) -> Option<Rid> {
        self.navigation_map
    }

    pub fn get_bounds(&self) -> Rect2 {
        self.bounds
    }

    pub fn set_bounds(&mut self, bounds: Rect2) {
        self.bounds = bounds;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canvas_group_defaults() {
        let mut g = CanvasGroupData::new();
        assert_eq!(g.get_fit_margin(), 10.0);
        assert_eq!(g.get_clear_margin(), 10.0);
        assert!(!g.is_using_mipmaps());

        g.set_fit_margin(2.0);
        g.set_clear_margin(4.0);
        g.set_use_mipmaps(true);
        assert_eq!(g.get_fit_margin(), 2.0);
        assert_eq!(g.get_clear_margin(), 4.0);
        assert!(g.is_using_mipmaps());
    }

    #[test]
    fn notifier_2d_defaults() {
        let mut n = VisibleOnScreenNotifier2DData::new();
        assert_eq!(
            n.get_rect(),
            Rect2::new(Vector2::new(-10.0, -10.0), Vector2::new(20.0, 20.0))
        );
        assert!(n.is_showing_rect());
        assert!(!n.is_on_screen());

        n.set_rect(Rect2::new(Vector2::ZERO, Vector2::ONE));
        n.set_show_rect(false);
        n.set_on_screen(true);
        assert_eq!(n.get_rect().size, Vector2::ONE);
        assert!(!n.is_showing_rect());
        assert!(n.is_on_screen());
    }

    #[test]
    fn enabler_2d_defaults() {
        let mut e = VisibleOnScreenEnabler2DData::new();
        assert_eq!(e.get_enable_mode(), EnableMode::Inherit);
        assert_eq!(e.get_enable_node_path(), "../AnimationPlayer");
        e.set_enable_mode(EnableMode::Always);
        assert_eq!(e.get_enable_mode(), EnableMode::Always);
        assert_eq!(EnableMode::Inherit as i32, 0);
        assert_eq!(EnableMode::Always as i32, 1);
        assert_eq!(EnableMode::WhenPaused as i32, 2);
    }

    #[test]
    fn navigation_region_2d_defaults() {
        let mut r = NavigationRegion2DData::new();
        assert!(r.get_enabled());
        assert!(r.get_use_edge_connections());
        assert_eq!(r.get_navigation_layers(), 1);
        assert_eq!(r.get_enter_cost(), 0.0);
        assert_eq!(r.get_travel_cost(), 1.0);
        assert!(r.get_navigation_polygon().is_none());
        assert_eq!(r.get_bounds(), Rect2::default());

        assert!(!r.set_enter_cost(-1.0));
        assert!(!r.set_travel_cost(-1.0));
        assert!(r.set_enter_cost(2.0));
        assert!(r.set_travel_cost(0.0));
        assert_eq!(r.get_enter_cost(), 2.0);
        assert_eq!(r.get_travel_cost(), 0.0);

        r.set_enabled(false);
        r.set_use_edge_connections(false);
        r.set_navigation_layers(6);
        r.set_navigation_polygon(Some(Rid::from_u64(1)));
        r.set_navigation_map(Some(Rid::from_u64(2)));
        assert!(!r.get_enabled());
        assert!(!r.get_use_edge_connections());
        assert_eq!(r.get_navigation_layers(), 6);
        assert_eq!(r.get_navigation_polygon(), Some(Rid::from_u64(1)));
        assert_eq!(r.get_navigation_map(), Some(Rid::from_u64(2)));
    }
}
