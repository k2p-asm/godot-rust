//! `scene/resources/compositor.*`: compositor effects.
//!
//! `CompositorEffect` stores the callback stage (default post-transparent), the
//! `enabled` flag (on) and the four feature-access flags (off).
//! `Compositor` keeps an ordered effect list. The render-data callbacks and RIDs
//! are not ported.

use crate::servers::rid::Rid;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum EffectCallbackType {
    PreOpaque,
    PostOpaque,
    PostSky,
    PreTransparent,
    PostTransparent,
    Max,
}

#[derive(Clone, Debug)]
pub struct CompositorEffectData {
    enabled: bool,
    effect_callback_type: EffectCallbackType,
    access_resolved_color: bool,
    access_resolved_depth: bool,
    needs_motion_vectors: bool,
    needs_normal_roughness: bool,
    needs_separate_specular: bool,
}

impl Default for CompositorEffectData {
    fn default() -> Self {
        Self::new()
    }
}

impl CompositorEffectData {
    pub fn new() -> Self {
        Self {
            enabled: true,
            effect_callback_type: EffectCallbackType::PostTransparent,
            access_resolved_color: false,
            access_resolved_depth: false,
            needs_motion_vectors: false,
            needs_normal_roughness: false,
            needs_separate_specular: false,
        }
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn get_enabled(&self) -> bool {
        self.enabled
    }

    pub fn set_effect_callback_type(&mut self, callback_type: EffectCallbackType) {
        self.effect_callback_type = callback_type;
    }

    pub fn get_effect_callback_type(&self) -> EffectCallbackType {
        self.effect_callback_type
    }

    pub fn set_access_resolved_color(&mut self, enabled: bool) {
        self.access_resolved_color = enabled;
    }

    pub fn get_access_resolved_color(&self) -> bool {
        self.access_resolved_color
    }

    pub fn set_access_resolved_depth(&mut self, enabled: bool) {
        self.access_resolved_depth = enabled;
    }

    pub fn get_access_resolved_depth(&self) -> bool {
        self.access_resolved_depth
    }

    pub fn set_needs_motion_vectors(&mut self, enabled: bool) {
        self.needs_motion_vectors = enabled;
    }

    pub fn get_needs_motion_vectors(&self) -> bool {
        self.needs_motion_vectors
    }

    pub fn set_needs_normal_roughness(&mut self, enabled: bool) {
        self.needs_normal_roughness = enabled;
    }

    pub fn get_needs_normal_roughness(&self) -> bool {
        self.needs_normal_roughness
    }

    pub fn set_needs_separate_specular(&mut self, enabled: bool) {
        self.needs_separate_specular = enabled;
    }

    pub fn get_needs_separate_specular(&self) -> bool {
        self.needs_separate_specular
    }
}

#[derive(Clone, Debug, Default)]
pub struct CompositorData {
    effects: Vec<CompositorEffectData>,
}

impl CompositorData {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_compositor_effects(&mut self, effects: Vec<CompositorEffectData>) {
        self.effects = effects;
    }

    pub fn get_compositor_effects(&self) -> &[CompositorEffectData] {
        &self.effects
    }

    pub fn effect_count(&self) -> usize {
        self.effects.len()
    }

    /// RIDs are not owned here; kept for API parity with the resource layer.
    pub fn get_rid(&self) -> Option<Rid> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn effect_defaults_and_flags() {
        let mut e = CompositorEffectData::new();
        assert!(e.get_enabled());
        assert_eq!(e.get_effect_callback_type(), EffectCallbackType::PostTransparent);
        assert!(!e.get_access_resolved_color());
        assert!(!e.get_access_resolved_depth());
        assert!(!e.get_needs_motion_vectors());
        assert!(!e.get_needs_normal_roughness());
        assert!(!e.get_needs_separate_specular());

        e.set_enabled(false);
        e.set_effect_callback_type(EffectCallbackType::PreOpaque);
        e.set_access_resolved_color(true);
        e.set_access_resolved_depth(true);
        e.set_needs_motion_vectors(true);
        e.set_needs_normal_roughness(true);
        e.set_needs_separate_specular(true);
        assert!(!e.get_enabled());
        assert_eq!(e.get_effect_callback_type(), EffectCallbackType::PreOpaque);
        assert!(e.get_access_resolved_color());
        assert!(e.get_access_resolved_depth());
        assert!(e.get_needs_motion_vectors());
        assert!(e.get_needs_normal_roughness());
        assert!(e.get_needs_separate_specular());

        assert_eq!(EffectCallbackType::PreOpaque as i32, 0);
        assert_eq!(EffectCallbackType::PostOpaque as i32, 1);
        assert_eq!(EffectCallbackType::PostSky as i32, 2);
        assert_eq!(EffectCallbackType::PreTransparent as i32, 3);
        assert_eq!(EffectCallbackType::PostTransparent as i32, 4);
        assert_eq!(EffectCallbackType::Max as i32, 5);
    }

    #[test]
    fn compositor_effect_ordering() {
        let mut c = CompositorData::new();
        assert_eq!(c.effect_count(), 0);
        assert!(c.get_rid().is_none());

        let mut first = CompositorEffectData::new();
        first.set_effect_callback_type(EffectCallbackType::PostOpaque);
        let mut second = CompositorEffectData::new();
        second.set_effect_callback_type(EffectCallbackType::PreTransparent);
        c.set_compositor_effects(vec![first, second]);

        assert_eq!(c.effect_count(), 2);
        assert_eq!(
            c.get_compositor_effects()[0].get_effect_callback_type(),
            EffectCallbackType::PostOpaque
        );
        assert_eq!(
            c.get_compositor_effects()[1].get_effect_callback_type(),
            EffectCallbackType::PreTransparent
        );

        c.set_compositor_effects(Vec::new());
        assert_eq!(c.effect_count(), 0);
    }
}
