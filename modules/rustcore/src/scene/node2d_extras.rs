//! Small `scene/2d/` classes that only carry state.
//!
//! - `RemoteTransform2D`: remote path plus the four flags (all enabled by
//!   default). Applying the transform to the remote node needs the tree.
//! - `CanvasModulate`: a white colour; the "one active modulate per canvas"
//!   bookkeeping belongs to the canvas group.
//! - `BackBufferCopy`: `(-100,-100,200,200)` rect and `COPY_MODE_RECT`.
//! - `Marker2D`: `gizmo_extents = 10`.
//! - `CanvasGroup`: both margins 10, mipmaps off; negative margins are rejected
//!   (`ERR_FAIL_COND`), not clamped.

use crate::math::color::Color;
use crate::math::rect2::Rect2;
use crate::math::vector2::Vector2;

#[derive(Clone, Debug)]
pub struct RemoteTransform2DData {
    remote_node: String,
    use_global_coordinates: bool,
    update_position: bool,
    update_rotation: bool,
    update_scale: bool,
}

impl Default for RemoteTransform2DData {
    fn default() -> Self {
        Self::new()
    }
}

impl RemoteTransform2DData {
    pub fn new() -> Self {
        Self {
            remote_node: String::new(),
            use_global_coordinates: true,
            update_position: true,
            update_rotation: true,
            update_scale: true,
        }
    }

    /// Stored as the path text; the engine keeps a `NodePath`.
    pub fn set_remote_node(&mut self, path: &str) {
        self.remote_node = path.to_string();
    }

    pub fn get_remote_node(&self) -> &str {
        &self.remote_node
    }

    pub fn set_use_global_coordinates(&mut self, enable: bool) {
        self.use_global_coordinates = enable;
    }

    pub fn get_use_global_coordinates(&self) -> bool {
        self.use_global_coordinates
    }

    pub fn set_update_position(&mut self, update: bool) {
        self.update_position = update;
    }

    pub fn get_update_position(&self) -> bool {
        self.update_position
    }

    pub fn set_update_rotation(&mut self, update: bool) {
        self.update_rotation = update;
    }

    pub fn get_update_rotation(&self) -> bool {
        self.update_rotation
    }

    pub fn set_update_scale(&mut self, update: bool) {
        self.update_scale = update;
    }

    pub fn get_update_scale(&self) -> bool {
        self.update_scale
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CanvasModulateData {
    color: Color,
}

impl Default for CanvasModulateData {
    fn default() -> Self {
        Self::new()
    }
}

impl CanvasModulateData {
    pub fn new() -> Self {
        Self {
            color: Color::new(1.0, 1.0, 1.0, 1.0),
        }
    }

    pub fn set_color(&mut self, color: Color) {
        self.color = color;
    }

    pub fn get_color(&self) -> Color {
        self.color
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum CopyMode {
    Disabled,
    Rect,
    Viewport,
}

#[derive(Clone, Copy, Debug)]
pub struct BackBufferCopyData {
    rect: Rect2,
    copy_mode: CopyMode,
}

impl Default for BackBufferCopyData {
    fn default() -> Self {
        Self::new()
    }
}

impl BackBufferCopyData {
    pub fn new() -> Self {
        Self {
            rect: Rect2::new(Vector2::new(-100.0, -100.0), Vector2::new(200.0, 200.0)),
            copy_mode: CopyMode::Rect,
        }
    }

    pub fn set_rect(&mut self, rect: Rect2) {
        self.rect = rect;
    }

    pub fn get_rect(&self) -> Rect2 {
        self.rect
    }

    pub fn set_copy_mode(&mut self, mode: CopyMode) {
        self.copy_mode = mode;
    }

    pub fn get_copy_mode(&self) -> CopyMode {
        self.copy_mode
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Marker2DData {
    gizmo_extents: f32,
}

impl Default for Marker2DData {
    fn default() -> Self {
        Self::new()
    }
}

impl Marker2DData {
    pub fn new() -> Self {
        Self { gizmo_extents: 10.0 }
    }

    pub fn set_gizmo_extents(&mut self, extents: f32) {
        self.gizmo_extents = extents;
    }

    pub fn get_gizmo_extents(&self) -> f32 {
        self.gizmo_extents
    }
}

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

    /// Negative margins are rejected outright, keeping the old value.
    pub fn set_fit_margin(&mut self, margin: f32) -> bool {
        if margin < 0.0 {
            return false;
        }
        self.fit_margin = margin;
        true
    }

    pub fn get_fit_margin(&self) -> f32 {
        self.fit_margin
    }

    pub fn set_clear_margin(&mut self, margin: f32) -> bool {
        if margin < 0.0 {
            return false;
        }
        self.clear_margin = margin;
        true
    }

    pub fn get_clear_margin(&self) -> f32 {
        self.clear_margin
    }

    pub fn set_use_mipmaps(&mut self, use_mipmaps: bool) {
        self.use_mipmaps = use_mipmaps;
    }

    pub fn is_using_mipmaps(&self) -> bool {
        self.use_mipmaps
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn remote_transform_defaults() {
        let mut r = RemoteTransform2DData::new();
        assert_eq!(r.get_remote_node(), "");
        assert!(r.get_use_global_coordinates());
        assert!(r.get_update_position());
        assert!(r.get_update_rotation());
        assert!(r.get_update_scale());

        r.set_remote_node("../Player");
        assert_eq!(r.get_remote_node(), "../Player");
        r.set_use_global_coordinates(false);
        r.set_update_position(false);
        r.set_update_rotation(false);
        r.set_update_scale(false);
        assert!(!r.get_use_global_coordinates());
        assert!(!r.get_update_position());
        assert!(!r.get_update_rotation());
        assert!(!r.get_update_scale());
    }

    #[test]
    fn canvas_modulate_defaults_to_white() {
        let mut m = CanvasModulateData::new();
        assert_eq!(m.get_color(), Color::new(1.0, 1.0, 1.0, 1.0));
        m.set_color(Color::new(0.2, 0.3, 0.4, 1.0));
        assert_eq!(m.get_color(), Color::new(0.2, 0.3, 0.4, 1.0));
    }

    #[test]
    fn back_buffer_copy_defaults() {
        let mut b = BackBufferCopyData::new();
        assert_eq!(
            b.get_rect(),
            Rect2::new(Vector2::new(-100.0, -100.0), Vector2::new(200.0, 200.0))
        );
        assert_eq!(b.get_copy_mode(), CopyMode::Rect);

        b.set_rect(Rect2::new(Vector2::new(1.0, 2.0), Vector2::new(3.0, 4.0)));
        assert_eq!(b.get_rect().position, Vector2::new(1.0, 2.0));
        b.set_copy_mode(CopyMode::Viewport);
        assert_eq!(b.get_copy_mode(), CopyMode::Viewport);

        assert_eq!(CopyMode::Disabled as i32, 0);
        assert_eq!(CopyMode::Rect as i32, 1);
        assert_eq!(CopyMode::Viewport as i32, 2);
    }

    #[test]
    fn marker_and_canvas_group() {
        let mut m = Marker2DData::new();
        assert_eq!(m.get_gizmo_extents(), 10.0);
        m.set_gizmo_extents(4.5);
        assert_eq!(m.get_gizmo_extents(), 4.5);

        let mut g = CanvasGroupData::new();
        assert_eq!(g.get_fit_margin(), 10.0);
        assert_eq!(g.get_clear_margin(), 10.0);
        assert!(!g.is_using_mipmaps());

        assert!(g.set_fit_margin(0.0));
        assert!(g.set_clear_margin(2.5));
        assert!(!g.set_fit_margin(-1.0));
        assert!(!g.set_clear_margin(-1.0));
        assert_eq!(g.get_fit_margin(), 0.0);
        assert_eq!(g.get_clear_margin(), 2.5);

        g.set_use_mipmaps(true);
        assert!(g.is_using_mipmaps());
    }
}
