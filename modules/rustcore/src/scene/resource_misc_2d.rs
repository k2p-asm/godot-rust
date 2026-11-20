//! Miscellaneous resource/effect states: `Shape2D`, the 2D navigation source
//! geometry, `MultiMeshInstance2D` and `CharFXTransform`.
//!
//! - `Shape2D`: the custom solver bias (0.0). Collision queries go through
//!   `PhysicsServer2D`, which is not ported.
//! - `NavigationMeshSourceGeometryData2D`: traversable/obstruction outlines, the
//!   root-node transform, projected obstructions (vertices + carve), and a
//!   bounds rectangle computed as the union of all outline points.
//! - `MultiMeshInstance2D`: the multimesh and texture handles.
//! - `CharFXTransform`: the per-glyph rich-text effect state (transform, range,
//!   visibility, outline, offset, colour, elapsed time, environment pairs,
//!   glyph index/flags/count, relative index and font handle).

use crate::math::color::Color;
use crate::math::rect2::Rect2;
use crate::math::transform_2d::Transform2D;
use crate::math::vector2::Vector2;
use crate::math::vector2i::Vector2i;
use crate::servers::rid::Rid;
use crate::variant::Variant;

// -- Shape2D ----------------------------------------------------------------

#[derive(Clone, Debug, Default)]
pub struct Shape2DData {
    custom_bias: f32,
}

impl Shape2DData {
    pub fn new() -> Self {
        Self::default()
    }

    /// `set_custom_solver_bias`.
    pub fn set_custom_solver_bias(&mut self, bias: f32) {
        self.custom_bias = bias;
    }

    pub fn get_custom_solver_bias(&self) -> f32 {
        self.custom_bias
    }
}

// -- NavigationMeshSourceGeometryData2D -------------------------------------

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ProjectedObstruction2D {
    pub vertices: Vec<f32>,
    pub carve: bool,
}

#[derive(Clone, Debug)]
pub struct NavigationMeshSourceGeometryData2D {
    traversable_outlines: Vec<Vec<Vector2>>,
    obstruction_outlines: Vec<Vec<Vector2>>,
    root_node_transform: Transform2D,
    projected_obstructions: Vec<ProjectedObstruction2D>,
}

impl Default for NavigationMeshSourceGeometryData2D {
    fn default() -> Self {
        Self::new()
    }
}

impl NavigationMeshSourceGeometryData2D {
    pub fn new() -> Self {
        Self {
            traversable_outlines: Vec::new(),
            obstruction_outlines: Vec::new(),
            root_node_transform: Transform2D::default(),
            projected_obstructions: Vec::new(),
        }
    }

    pub fn set_traversable_outlines(&mut self, outlines: Vec<Vec<Vector2>>) {
        self.traversable_outlines = outlines;
    }

    pub fn get_traversable_outlines(&self) -> &[Vec<Vector2>] {
        &self.traversable_outlines
    }

    pub fn append_traversable_outlines(&mut self, outlines: &[Vec<Vector2>]) {
        self.traversable_outlines.extend_from_slice(outlines);
    }

    pub fn add_traversable_outline(&mut self, outline: Vec<Vector2>) {
        self.traversable_outlines.push(outline);
    }

    pub fn clear_traversable_outlines(&mut self) {
        self.traversable_outlines.clear();
    }

    pub fn set_obstruction_outlines(&mut self, outlines: Vec<Vec<Vector2>>) {
        self.obstruction_outlines = outlines;
    }

    pub fn get_obstruction_outlines(&self) -> &[Vec<Vector2>] {
        &self.obstruction_outlines
    }

    pub fn append_obstruction_outlines(&mut self, outlines: &[Vec<Vector2>]) {
        self.obstruction_outlines.extend_from_slice(outlines);
    }

    pub fn add_obstruction_outline(&mut self, outline: Vec<Vector2>) {
        self.obstruction_outlines.push(outline);
    }

    pub fn clear_obstruction_outlines(&mut self) {
        self.obstruction_outlines.clear();
    }

    /// `has_data`: any outline or obstruction.
    pub fn has_data(&self) -> bool {
        !self.traversable_outlines.is_empty()
            || !self.obstruction_outlines.is_empty()
            || !self.projected_obstructions.is_empty()
    }

    pub fn clear(&mut self) {
        self.traversable_outlines.clear();
        self.obstruction_outlines.clear();
        self.projected_obstructions.clear();
        self.root_node_transform = Transform2D::default();
    }

    pub fn set_root_node_transform(&mut self, transform: Transform2D) {
        self.root_node_transform = transform;
    }

    pub fn get_root_node_transform(&self) -> Transform2D {
        self.root_node_transform
    }

    /// `get_bounds`: the union of every outlined point (empty when there are no
    /// outlines).
    pub fn get_bounds(&self) -> Rect2 {
        let mut bounds: Option<Rect2> = None;
        for outline in self
            .traversable_outlines
            .iter()
            .chain(self.obstruction_outlines.iter())
        {
            for point in outline {
                bounds = Some(match bounds {
                    None => Rect2::new(*point, Vector2::ZERO),
                    Some(mut rect) => {
                        rect = rect.expand(*point);
                        rect
                    }
                });
            }
        }
        bounds.unwrap_or_default()
    }

    pub fn add_projected_obstruction(&mut self, vertices: Vec<f32>, carve: bool) {
        self.projected_obstructions
            .push(ProjectedObstruction2D { vertices, carve });
    }

    pub fn get_projected_obstructions(&self) -> &[ProjectedObstruction2D] {
        &self.projected_obstructions
    }

    pub fn clear_projected_obstructions(&mut self) {
        self.projected_obstructions.clear();
    }
}

// -- MultiMeshInstance2D ----------------------------------------------------

#[derive(Clone, Debug, Default)]
pub struct MultiMeshInstance2DData {
    multimesh: Option<Rid>,
    texture: Option<Rid>,
}

impl MultiMeshInstance2DData {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_multimesh(&mut self, multimesh: Option<Rid>) {
        self.multimesh = multimesh;
    }

    pub fn get_multimesh(&self) -> Option<Rid> {
        self.multimesh
    }

    pub fn set_texture(&mut self, texture: Option<Rid>) {
        self.texture = texture;
    }

    pub fn get_texture(&self) -> Option<Rid> {
        self.texture
    }
}

// -- CharFXTransform --------------------------------------------------------

#[derive(Clone, Debug)]
pub struct CharFxTransform {
    pub transform: Transform2D,
    pub range: Vector2i,
    pub visibility: bool,
    pub outline: bool,
    pub offset: Vector2,
    pub color: Color,
    pub elapsed_time: f64,
    /// The engine's `Dictionary`; this port keeps insertion pairs.
    pub environment: Vec<(String, Variant)>,
    pub glyph_index: u32,
    pub glyph_flags: u16,
    pub glyph_count: u8,
    pub relative_index: i32,
    pub font: Option<Rid>,
}

impl Default for CharFxTransform {
    fn default() -> Self {
        Self::new()
    }
}

impl CharFxTransform {
    pub fn new() -> Self {
        Self {
            transform: Transform2D::default(),
            range: Vector2i::ZERO,
            visibility: true,
            outline: false,
            offset: Vector2::ZERO,
            // `Color()` is opaque black in the engine.
            color: Color::new(0.0, 0.0, 0.0, 1.0),
            elapsed_time: 0.0,
            environment: Vec::new(),
            glyph_index: 0,
            glyph_flags: 0,
            glyph_count: 0,
            relative_index: 0,
            font: None,
        }
    }

    pub fn set_environment(&mut self, environment: Vec<(String, Variant)>) {
        self.environment = environment;
    }

    pub fn get_environment(&self) -> &[(String, Variant)] {
        &self.environment
    }

    pub fn get_environment_value(&self, name: &str) -> Option<&Variant> {
        self.environment
            .iter()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value)
    }

    pub fn set_font(&mut self, font: Option<Rid>) {
        self.font = font;
    }

    pub fn get_font(&self) -> Option<Rid> {
        self.font
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::string::gstring::GoString;

    #[test]
    fn shape2d_bias() {
        let mut shape = Shape2DData::new();
        assert_eq!(shape.get_custom_solver_bias(), 0.0);
        shape.set_custom_solver_bias(0.5);
        assert_eq!(shape.get_custom_solver_bias(), 0.5);
        shape.set_custom_solver_bias(-0.25);
        assert_eq!(shape.get_custom_solver_bias(), -0.25);
    }

    #[test]
    fn navigation_2d_outlines_and_bounds() {
        let mut geometry = NavigationMeshSourceGeometryData2D::new();
        assert!(!geometry.has_data());
        assert!(geometry.get_traversable_outlines().is_empty());
        assert_eq!(geometry.get_bounds(), Rect2::default());

        geometry.add_traversable_outline(vec![
            Vector2::new(0.0, 0.0),
            Vector2::new(2.0, 0.0),
            Vector2::new(2.0, 1.0),
        ]);
        geometry.add_obstruction_outline(vec![Vector2::new(-1.0, 3.0)]);
        assert!(geometry.has_data());
        assert_eq!(geometry.get_traversable_outlines().len(), 1);
        assert_eq!(geometry.get_obstruction_outlines().len(), 1);
        let bounds = geometry.get_bounds();
        assert_eq!(bounds.position, Vector2::new(-1.0, 0.0));
        assert_eq!(bounds.size, Vector2::new(3.0, 3.0));

        geometry.append_traversable_outlines(&[vec![Vector2::new(5.0, 5.0)]]);
        assert_eq!(geometry.get_traversable_outlines().len(), 2);
        assert_eq!(geometry.get_bounds().size, Vector2::new(6.0, 5.0));

        // Projected obstructions participate in `has_data` and clearing.
        geometry.add_projected_obstruction(vec![0.0, 0.0, 1.0, 1.0], true);
        assert_eq!(geometry.get_projected_obstructions().len(), 1);
        assert!(geometry.get_projected_obstructions()[0].carve);
        geometry.clear_projected_obstructions();
        assert!(geometry.get_projected_obstructions().is_empty());

        let transform = Transform2D::default();
        geometry.set_root_node_transform(transform);
        assert_eq!(geometry.get_root_node_transform(), transform);

        geometry.clear();
        assert!(!geometry.has_data());
        assert_eq!(geometry.get_bounds(), Rect2::default());
    }

    #[test]
    fn multimesh_instance_state() {
        let mut instance = MultiMeshInstance2DData::new();
        assert!(instance.get_multimesh().is_none());
        assert!(instance.get_texture().is_none());
        instance.set_multimesh(Some(Rid::from_u64(3)));
        instance.set_texture(Some(Rid::from_u64(4)));
        assert_eq!(instance.get_multimesh(), Some(Rid::from_u64(3)));
        assert_eq!(instance.get_texture(), Some(Rid::from_u64(4)));
        instance.set_multimesh(None);
        assert!(instance.get_multimesh().is_none());
    }

    #[test]
    fn char_fx_transform_state() {
        let mut effect = CharFxTransform::new();
        assert_eq!(effect.transform, Transform2D::default());
        assert_eq!(effect.range, Vector2i::ZERO);
        assert!(effect.visibility);
        assert!(!effect.outline);
        assert_eq!(effect.offset, Vector2::ZERO);
        assert_eq!(effect.color, Color::new(0.0, 0.0, 0.0, 1.0));
        assert_eq!(effect.elapsed_time, 0.0);
        assert!(effect.environment.is_empty());
        assert_eq!(effect.glyph_index, 0);
        assert_eq!(effect.glyph_flags, 0);
        assert_eq!(effect.glyph_count, 0);
        assert_eq!(effect.relative_index, 0);
        assert!(effect.font.is_none());

        effect.range = Vector2i::new(2, 5);
        effect.visibility = false;
        effect.outline = true;
        effect.offset = Vector2::new(1.0, -1.0);
        effect.color = Color::new(1.0, 0.5, 0.0, 1.0);
        effect.elapsed_time = 1.25;
        effect.environment = vec![(
            "speed".to_string(),
            Variant::Float(2.0),
        )];
        effect.glyph_index = 42;
        effect.glyph_flags = 0b101;
        effect.glyph_count = 3;
        effect.relative_index = 1;
        effect.set_font(Some(Rid::from_u64(9)));

        assert_eq!(effect.range, Vector2i::new(2, 5));
        assert!(!effect.visibility);
        assert!(effect.outline);
        assert_eq!(effect.offset, Vector2::new(1.0, -1.0));
        assert_eq!(effect.color, Color::new(1.0, 0.5, 0.0, 1.0));
        assert_eq!(effect.elapsed_time, 1.25);
        assert_eq!(effect.glyph_index, 42);
        assert_eq!(effect.glyph_flags, 0b101);
        assert_eq!(effect.glyph_count, 3);
        assert_eq!(effect.relative_index, 1);
        assert_eq!(effect.get_font(), Some(Rid::from_u64(9)));
        assert_eq!(
            effect.get_environment_value("speed"),
            Some(&Variant::Float(2.0))
        );
        assert!(effect.get_environment_value("missing").is_none());
        let _ = GoString::from_str("x");
    }
}
