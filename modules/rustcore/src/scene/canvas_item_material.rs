//! `scene/resources/canvas_item_material.*`: the 2D material parameters.
//!
//! Defaults: mix blend, normal light mode, no particles animation, but the
//! constructor still sets the frame counts to 1 and loop off. All setters
//! assign directly; the shader/Variant plumbing and the RenderingServer material
//! are not ported.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum BlendMode {
    Mix,
    Add,
    Sub,
    Mul,
    PremultAlpha,
    Disabled,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum LightMode {
    Normal,
    Unshaded,
    LightOnly,
}

#[derive(Clone, Copy, Debug)]
pub struct CanvasItemMaterialData {
    blend_mode: BlendMode,
    light_mode: LightMode,
    particles_animation: bool,
    particles_anim_h_frames: i32,
    particles_anim_v_frames: i32,
    particles_anim_loop: bool,
}

impl Default for CanvasItemMaterialData {
    fn default() -> Self {
        Self::new()
    }
}

impl CanvasItemMaterialData {
    pub fn new() -> Self {
        Self {
            blend_mode: BlendMode::Mix,
            light_mode: LightMode::Normal,
            particles_animation: false,
            // Proper values set in the constructor.
            particles_anim_h_frames: 1,
            particles_anim_v_frames: 1,
            particles_anim_loop: false,
        }
    }

    pub fn set_blend_mode(&mut self, mode: BlendMode) {
        self.blend_mode = mode;
    }

    pub fn get_blend_mode(&self) -> BlendMode {
        self.blend_mode
    }

    pub fn set_light_mode(&mut self, mode: LightMode) {
        self.light_mode = mode;
    }

    pub fn get_light_mode(&self) -> LightMode {
        self.light_mode
    }

    pub fn set_particles_animation(&mut self, enabled: bool) {
        self.particles_animation = enabled;
    }

    pub fn get_particles_animation(&self) -> bool {
        self.particles_animation
    }

    pub fn set_particles_anim_h_frames(&mut self, frames: i32) {
        self.particles_anim_h_frames = frames;
    }

    pub fn get_particles_anim_h_frames(&self) -> i32 {
        self.particles_anim_h_frames
    }

    pub fn set_particles_anim_v_frames(&mut self, frames: i32) {
        self.particles_anim_v_frames = frames;
    }

    pub fn get_particles_anim_v_frames(&self) -> i32 {
        self.particles_anim_v_frames
    }

    pub fn set_particles_anim_loop(&mut self, looped: bool) {
        self.particles_anim_loop = looped;
    }

    pub fn get_particles_anim_loop(&self) -> bool {
        self.particles_anim_loop
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults() {
        let m = CanvasItemMaterialData::new();
        assert_eq!(m.get_blend_mode(), BlendMode::Mix);
        assert_eq!(m.get_light_mode(), LightMode::Normal);
        assert!(!m.get_particles_animation());
        assert_eq!(m.get_particles_anim_h_frames(), 1);
        assert_eq!(m.get_particles_anim_v_frames(), 1);
        assert!(!m.get_particles_anim_loop());
    }

    #[test]
    fn setters_round_trip_and_enum_values() {
        let mut m = CanvasItemMaterialData::new();
        m.set_blend_mode(BlendMode::PremultAlpha);
        m.set_light_mode(LightMode::LightOnly);
        m.set_particles_animation(true);
        m.set_particles_anim_h_frames(4);
        m.set_particles_anim_v_frames(2);
        m.set_particles_anim_loop(true);

        assert_eq!(m.get_blend_mode(), BlendMode::PremultAlpha);
        assert_eq!(m.get_light_mode(), LightMode::LightOnly);
        assert!(m.get_particles_animation());
        assert_eq!(m.get_particles_anim_h_frames(), 4);
        assert_eq!(m.get_particles_anim_v_frames(), 2);
        assert!(m.get_particles_anim_loop());

        assert_eq!(BlendMode::Mix as i32, 0);
        assert_eq!(BlendMode::Add as i32, 1);
        assert_eq!(BlendMode::Sub as i32, 2);
        assert_eq!(BlendMode::Mul as i32, 3);
        assert_eq!(BlendMode::PremultAlpha as i32, 4);
        assert_eq!(BlendMode::Disabled as i32, 5);

        assert_eq!(LightMode::Normal as i32, 0);
        assert_eq!(LightMode::Unshaded as i32, 1);
        assert_eq!(LightMode::LightOnly as i32, 2);
    }
}
