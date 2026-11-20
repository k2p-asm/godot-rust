//! `scene/resources/physics_material.*` and `color_palette.*`.
//!
//! `PhysicsMaterial` defaults to friction 1, bounce 0 and neither `rough` nor
//! `absorbent`; `computed_friction`/`computed_bounce` negate the value when the
//! flag is set. `ColorPalette` is just a colour array.
//!
//! The `Resource` plumbing and the `changed` notification are not ported.

use crate::math::color::Color;

#[derive(Clone, Copy, Debug)]
pub struct PhysicsMaterialData {
    friction: f32,
    rough: bool,
    bounce: f32,
    absorbent: bool,
}

impl Default for PhysicsMaterialData {
    fn default() -> Self {
        Self::new()
    }
}

impl PhysicsMaterialData {
    pub fn new() -> Self {
        Self {
            friction: 1.0,
            rough: false,
            bounce: 0.0,
            absorbent: false,
        }
    }

    pub fn set_friction(&mut self, friction: f32) {
        self.friction = friction;
    }

    pub fn get_friction(&self) -> f32 {
        self.friction
    }

    pub fn set_rough(&mut self, rough: bool) {
        self.rough = rough;
    }

    pub fn is_rough(&self) -> bool {
        self.rough
    }

    /// `rough ? -friction : friction`.
    pub fn computed_friction(&self) -> f32 {
        if self.rough {
            -self.friction
        } else {
            self.friction
        }
    }

    pub fn set_bounce(&mut self, bounce: f32) {
        self.bounce = bounce;
    }

    pub fn get_bounce(&self) -> f32 {
        self.bounce
    }

    pub fn set_absorbent(&mut self, absorbent: bool) {
        self.absorbent = absorbent;
    }

    pub fn is_absorbent(&self) -> bool {
        self.absorbent
    }

    /// `absorbent ? -bounce : bounce`.
    pub fn computed_bounce(&self) -> f32 {
        if self.absorbent {
            -self.bounce
        } else {
            self.bounce
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct ColorPaletteData {
    colors: Vec<Color>,
}

impl ColorPaletteData {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_colors(&mut self, colors: Vec<Color>) {
        self.colors = colors;
    }

    pub fn get_colors(&self) -> &[Color] {
        &self.colors
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn physics_material_defaults_and_computed_values() {
        let mut m = PhysicsMaterialData::new();
        assert_eq!(m.get_friction(), 1.0);
        assert_eq!(m.get_bounce(), 0.0);
        assert!(!m.is_rough());
        assert!(!m.is_absorbent());
        assert_eq!(m.computed_friction(), 1.0);
        assert_eq!(m.computed_bounce(), 0.0);

        m.set_friction(0.25);
        m.set_bounce(0.75);
        assert_eq!(m.computed_friction(), 0.25);
        assert_eq!(m.computed_bounce(), 0.75);

        m.set_rough(true);
        m.set_absorbent(true);
        assert_eq!(m.computed_friction(), -0.25);
        assert_eq!(m.computed_bounce(), -0.75);

        // No clamping on either value.
        m.set_friction(-2.0);
        m.set_rough(false);
        assert_eq!(m.computed_friction(), -2.0);
    }

    #[test]
    fn color_palette_round_trip() {
        let mut p = ColorPaletteData::new();
        assert!(p.get_colors().is_empty());
        p.set_colors(vec![Color::rgb(1.0, 0.0, 0.0), Color::rgb(0.0, 1.0, 0.0)]);
        assert_eq!(p.get_colors().len(), 2);
        assert_eq!(p.get_colors()[0], Color::rgb(1.0, 0.0, 0.0));
        p.set_colors(Vec::new());
        assert!(p.get_colors().is_empty());
    }
}
