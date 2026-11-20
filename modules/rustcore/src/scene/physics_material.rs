//! `scene/resources/physics_material.*`: friction/bounce settings.
//!
//! Defaults: friction 1, rough off, bounce 0, absorbent off. The setters are
//! plain assignments in the C++ (no clamping), and
//! `computed_friction()` is `rough ? -friction : friction` — the negative value
//! makes the backend treat the surface as rough.

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

    pub fn set_friction(&mut self, value: f32) {
        self.friction = value;
    }

    pub fn get_friction(&self) -> f32 {
        self.friction
    }

    pub fn set_rough(&mut self, value: bool) {
        self.rough = value;
    }

    pub fn is_rough(&self) -> bool {
        self.rough
    }

    pub fn set_bounce(&mut self, value: f32) {
        self.bounce = value;
    }

    pub fn get_bounce(&self) -> f32 {
        self.bounce
    }

    pub fn set_absorbent(&mut self, value: bool) {
        self.absorbent = value;
    }

    pub fn is_absorbent(&self) -> bool {
        self.absorbent
    }

    /// `computed_friction`: negated when `rough` is set.
    pub fn computed_friction(&self) -> f32 {
        if self.rough {
            -self.friction
        } else {
            self.friction
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults() {
        let m = PhysicsMaterialData::new();
        assert_eq!(m.get_friction(), 1.0);
        assert!(!m.is_rough());
        assert_eq!(m.get_bounce(), 0.0);
        assert!(!m.is_absorbent());
        assert_eq!(m.computed_friction(), 1.0);
    }

    #[test]
    fn setters_and_computed_friction() {
        let mut m = PhysicsMaterialData::new();
        m.set_friction(0.25);
        m.set_bounce(0.75);
        m.set_absorbent(true);
        assert_eq!(m.get_friction(), 0.25);
        assert_eq!(m.get_bounce(), 0.75);
        assert!(m.is_absorbent());
        assert_eq!(m.computed_friction(), 0.25);

        m.set_rough(true);
        assert!(m.is_rough());
        assert_eq!(m.computed_friction(), -0.25);

        // Values are stored unclamped, like the C++.
        m.set_bounce(5.0);
        m.set_friction(-2.0);
        assert_eq!(m.get_bounce(), 5.0);
        assert_eq!(m.computed_friction(), 2.0);
    }
}
