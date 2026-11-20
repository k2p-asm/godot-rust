//! `scene/2d/canvas_group.*`: the canvas group node's state.
//!
//! Defaults: fit margin 10, clear margin 10, mipmaps off. The margins are
//! `real_t` values passed straight through, like the header.
//!
//! Rendering (`RenderingServer` group creation, mipmap generation) is not ported.

#[derive(Clone, Debug)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults() {
        let group = CanvasGroupData::new();
        assert_eq!(group.get_fit_margin(), 10.0);
        assert_eq!(group.get_clear_margin(), 10.0);
        assert!(!group.is_using_mipmaps());
    }

    #[test]
    fn setters() {
        let mut group = CanvasGroupData::new();
        group.set_fit_margin(2.5);
        group.set_clear_margin(0.0);
        group.set_use_mipmaps(true);
        assert_eq!(group.get_fit_margin(), 2.5);
        assert_eq!(group.get_clear_margin(), 0.0);
        assert!(group.is_using_mipmaps());
    }
}
