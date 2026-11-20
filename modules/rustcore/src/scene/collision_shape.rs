//! `scene/2d/physics/collision_shape_2d.*` and
//! `scene/3d/physics/collision_shape_3d.*` (state only).
//!
//! `CollisionShape2D` defaults: no shape, the editor rect `(-10,-10,20,20)`,
//! not disabled, one-way collision off with margin 1 and direction `(0,1)`, and a
//! transparent debug colour. `CollisionShape3D` keeps the shape handle, the
//! disabled flag and the debug colour.
//!
//! The shape owner registration with the parent `CollisionObject`, the debug
//! drawing and the physics server are not ported.

use crate::math::color::Color;
use crate::math::rect2::Rect2;
use crate::math::vector2::Vector2;
use crate::servers::rid::Rid;

#[derive(Clone, Debug)]
pub struct CollisionShape2DData {
    shape: Option<Rid>,
    rect: Rect2,
    disabled: bool,
    one_way_collision: bool,
    one_way_collision_margin: f32,
    one_way_collision_direction: Vector2,
    debug_color: Color,
}

impl Default for CollisionShape2DData {
    fn default() -> Self {
        Self::new()
    }
}

impl CollisionShape2DData {
    pub fn new() -> Self {
        Self {
            shape: None,
            rect: Rect2::new(Vector2::new(-10.0, -10.0), Vector2::new(20.0, 20.0)),
            disabled: false,
            one_way_collision: false,
            one_way_collision_margin: 1.0,
            one_way_collision_direction: Vector2::new(0.0, 1.0),
            debug_color: Color::new(0.0, 0.0, 0.0, 0.0),
        }
    }

    pub fn set_shape(&mut self, shape: Option<Rid>) {
        self.shape = shape;
    }

    pub fn get_shape(&self) -> Option<Rid> {
        self.shape
    }

    /// The editor helper rect (the engine updates it from the shape).
    pub fn get_rect(&self) -> Rect2 {
        self.rect
    }

    pub fn set_rect(&mut self, rect: Rect2) {
        self.rect = rect;
    }

    pub fn set_disabled(&mut self, disabled: bool) {
        self.disabled = disabled;
    }

    pub fn is_disabled(&self) -> bool {
        self.disabled
    }

    pub fn set_one_way_collision(&mut self, enabled: bool) {
        self.one_way_collision = enabled;
    }

    pub fn is_one_way_collision_enabled(&self) -> bool {
        self.one_way_collision
    }

    pub fn set_one_way_collision_margin(&mut self, margin: f32) {
        self.one_way_collision_margin = margin;
    }

    pub fn get_one_way_collision_margin(&self) -> f32 {
        self.one_way_collision_margin
    }

    pub fn set_one_way_collision_direction(&mut self, direction: Vector2) {
        self.one_way_collision_direction = direction;
    }

    pub fn get_one_way_collision_direction(&self) -> Vector2 {
        self.one_way_collision_direction
    }

    /// A fully transparent debug colour means "use the default".
    pub fn set_debug_color(&mut self, color: Color) {
        self.debug_color = color;
    }

    pub fn get_debug_color(&self) -> Color {
        self.debug_color
    }
}

#[derive(Clone, Debug)]
pub struct CollisionShape3DData {
    shape: Option<Rid>,
    disabled: bool,
    debug_color: Color,
}

impl Default for CollisionShape3DData {
    fn default() -> Self {
        Self::new()
    }
}

impl CollisionShape3DData {
    pub fn new() -> Self {
        Self {
            shape: None,
            disabled: false,
            debug_color: Color::new(0.0, 0.0, 0.0, 0.0),
        }
    }

    pub fn set_shape(&mut self, shape: Option<Rid>) {
        self.shape = shape;
    }

    pub fn get_shape(&self) -> Option<Rid> {
        self.shape
    }

    pub fn set_disabled(&mut self, disabled: bool) {
        self.disabled = disabled;
    }

    pub fn is_disabled(&self) -> bool {
        self.disabled
    }

    pub fn set_debug_color(&mut self, color: Color) {
        self.debug_color = color;
    }

    pub fn get_debug_color(&self) -> Color {
        self.debug_color
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shape_2d_defaults() {
        let mut s = CollisionShape2DData::new();
        assert!(s.get_shape().is_none());
        assert_eq!(
            s.get_rect(),
            Rect2::new(Vector2::new(-10.0, -10.0), Vector2::new(20.0, 20.0))
        );
        assert!(!s.is_disabled());
        assert!(!s.is_one_way_collision_enabled());
        assert_eq!(s.get_one_way_collision_margin(), 1.0);
        assert_eq!(
            s.get_one_way_collision_direction(),
            Vector2::new(0.0, 1.0)
        );
        assert_eq!(s.get_debug_color(), Color::new(0.0, 0.0, 0.0, 0.0));

        s.set_shape(Some(Rid::from_u64(4)));
        s.set_rect(Rect2::new(Vector2::ZERO, Vector2::ONE));
        s.set_disabled(true);
        s.set_one_way_collision(true);
        s.set_one_way_collision_margin(2.0);
        s.set_one_way_collision_direction(Vector2::UP);
        s.set_debug_color(Color::new(1.0, 0.0, 0.0, 1.0));

        assert_eq!(s.get_shape(), Some(Rid::from_u64(4)));
        assert_eq!(s.get_rect().size, Vector2::ONE);
        assert!(s.is_disabled());
        assert!(s.is_one_way_collision_enabled());
        assert_eq!(s.get_one_way_collision_margin(), 2.0);
        assert_eq!(s.get_one_way_collision_direction(), Vector2::UP);
        assert_eq!(s.get_debug_color(), Color::new(1.0, 0.0, 0.0, 1.0));
    }

    #[test]
    fn shape_3d_defaults_and_setters() {
        let mut s = CollisionShape3DData::new();
        assert!(s.get_shape().is_none());
        assert!(!s.is_disabled());
        assert_eq!(s.get_debug_color(), Color::new(0.0, 0.0, 0.0, 0.0));

        s.set_shape(Some(Rid::from_u64(9)));
        s.set_disabled(true);
        s.set_debug_color(Color::new(0.0, 0.0, 1.0, 1.0));
        assert_eq!(s.get_shape(), Some(Rid::from_u64(9)));
        assert!(s.is_disabled());
        assert_eq!(s.get_debug_color(), Color::new(0.0, 0.0, 1.0, 1.0));

        s.set_shape(None);
        assert!(s.get_shape().is_none());
    }
}
