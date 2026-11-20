//! `servers/rendering_server.*`: the canvas-item half as stored state.
//!
//! The engine's `RenderingServer` is an abstract interface backed by Vulkan,
//! D3D12, Metal or the software renderer. This module keeps the part that is
//! *not* backend-specific: the per-item state that `canvas_item_*` calls write,
//! stored in a [`crate::servers::rid::RidOwner`] so handles stay safe across
//! frees.
//!
//! Not ported: everything that reaches a GPU (textures, meshes, materials,
//! draw commands), and the singleton/threading model.

use crate::math::color::Color;
use crate::math::transform_2d::Transform2D;
use crate::math::vector2i::Vector2i;
use crate::servers::rendering_enums::TextureType;
use crate::servers::rid::{Rid, RidOwner};

#[derive(Clone, Debug, PartialEq)]
pub enum CanvasCommand {
    Rect {
        rect: crate::math::rect2::Rect2,
        color: Color,
        filled: bool,
        width: f32,
    },
    Line {
        from: crate::math::vector2::Vector2,
        to: crate::math::vector2::Vector2,
        color: Color,
        width: f32,
        antialiased: bool,
    },
    Polyline {
        points: Vec<crate::math::vector2::Vector2>,
        color: Color,
        width: f32,
    },
    Polygon {
        points: Vec<crate::math::vector2::Vector2>,
        colors: Vec<Color>,
    },
    TextureRect {
        texture: Rid,
        rect: crate::math::rect2::Rect2,
        modulate: Color,
        transpose: bool,
    },
}

#[derive(Clone, Debug)]
pub struct CanvasItemState {
    pub transform: Transform2D,
    pub modulate: Color,
    pub self_modulate: Color,
    pub z_index: i32,
    pub visible: bool,
    pub commands: Vec<CanvasCommand>,
}

impl Default for CanvasItemState {
    fn default() -> Self {
        Self {
            transform: Transform2D::IDENTITY,
            modulate: Color::new(1.0, 1.0, 1.0, 1.0),
            self_modulate: Color::new(1.0, 1.0, 1.0, 1.0),
            z_index: 0,
            visible: true,
            commands: Vec::new(),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ViewportState {
    pub canvas_transform: Transform2D,
    pub size: Vector2i,
    pub transparent_background: bool,
    pub active: bool,
}

impl Default for ViewportState {
    fn default() -> Self {
        Self {
            canvas_transform: Transform2D::IDENTITY,
            size: Vector2i::ZERO,
            transparent_background: false,
            active: true,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct TextureState {
    pub size: Vector2i,
    pub format: i32,
    pub layers: i32,
    pub texture_type: TextureType,
    pub flags: u32,
}

impl Default for TextureState {
    fn default() -> Self {
        Self {
            size: Vector2i::ZERO,
            format: 0,
            layers: 1,
            texture_type: TextureType::Type2D,
            flags: 0,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct MaterialState {
    pub shading_mode: i32,
    pub params: std::collections::HashMap<crate::string::string_name::StringName, crate::variant::Variant>,
}

#[derive(Clone, Copy, Debug)]
pub struct MeshSurface {
    pub material: Rid,
    pub aabb: crate::math::aabb::Aabb,
    pub primitive: i32,
}

#[derive(Clone, Debug, Default)]
pub struct MeshState {
    pub surfaces: Vec<MeshSurface>,
}

#[derive(Default)]
pub struct RenderingServerData {
    canvas_items: RidOwner<CanvasItemState>,
    viewports: RidOwner<ViewportState>,
    textures: RidOwner<TextureState>,
    materials: RidOwner<MaterialState>,
    meshes: RidOwner<MeshState>,
}

impl RenderingServerData {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn canvas_item_create(&mut self) -> Rid {
        self.canvas_items.alloc(CanvasItemState::default())
    }

    pub fn canvas_item_free(&mut self, item: Rid) -> bool {
        self.canvas_items.free(item)
    }

    pub fn canvas_item_get(&self, item: Rid) -> Option<&CanvasItemState> {
        self.canvas_items.get(item)
    }

    pub fn canvas_item_set_transform(&mut self, item: Rid, transform: Transform2D) -> bool {
        match self.canvas_items.get_mut(item) {
            Some(s) => {
                s.transform = transform;
                true
            }
            None => false,
        }
    }

    pub fn canvas_item_set_modulate(&mut self, item: Rid, color: Color) -> bool {
        match self.canvas_items.get_mut(item) {
            Some(s) => {
                s.modulate = color;
                true
            }
            None => false,
        }
    }

    pub fn canvas_item_set_self_modulate(&mut self, item: Rid, color: Color) -> bool {
        match self.canvas_items.get_mut(item) {
            Some(s) => {
                s.self_modulate = color;
                true
            }
            None => false,
        }
    }

    /// The server stores the raw value; the ±4096 validation lives in
    /// `CanvasItem::set_z_index`.
    pub fn canvas_item_set_z_index(&mut self, item: Rid, z_index: i32) -> bool {
        match self.canvas_items.get_mut(item) {
            Some(s) => {
                s.z_index = z_index;
                true
            }
            None => false,
        }
    }

    pub fn canvas_item_set_visible(&mut self, item: Rid, visible: bool) -> bool {
        match self.canvas_items.get_mut(item) {
            Some(s) => {
                s.visible = visible;
                true
            }
            None => false,
        }
    }

    pub fn canvas_item_count(&self) -> usize {
        self.canvas_items.count()
    }

    fn push_command(&mut self, item: Rid, command: CanvasCommand) -> bool {
        match self.canvas_items.get_mut(item) {
            Some(s) => {
                s.commands.push(command);
                true
            }
            None => false,
        }
    }

    pub fn canvas_item_add_rect(
        &mut self,
        item: Rid,
        rect: crate::math::rect2::Rect2,
        color: Color,
        filled: bool,
        width: f32,
    ) -> bool {
        self.push_command(item, CanvasCommand::Rect { rect, color, filled, width })
    }

    pub fn canvas_item_add_line(
        &mut self,
        item: Rid,
        from: crate::math::vector2::Vector2,
        to: crate::math::vector2::Vector2,
        color: Color,
        width: f32,
        antialiased: bool,
    ) -> bool {
        self.push_command(item, CanvasCommand::Line { from, to, color, width, antialiased })
    }

    pub fn canvas_item_add_polyline(
        &mut self,
        item: Rid,
        points: Vec<crate::math::vector2::Vector2>,
        color: Color,
        width: f32,
    ) -> bool {
        self.push_command(item, CanvasCommand::Polyline { points, color, width })
    }

    pub fn canvas_item_add_polygon(
        &mut self,
        item: Rid,
        points: Vec<crate::math::vector2::Vector2>,
        colors: Vec<Color>,
    ) -> bool {
        self.push_command(item, CanvasCommand::Polygon { points, colors })
    }

    pub fn canvas_item_add_texture_rect(
        &mut self,
        item: Rid,
        texture: Rid,
        rect: crate::math::rect2::Rect2,
        modulate: Color,
        transpose: bool,
    ) -> bool {
        self.push_command(item, CanvasCommand::TextureRect { texture, rect, modulate, transpose })
    }

    /// canvas_item_clear: drops the recorded commands, keeps the item state.
    pub fn canvas_item_clear(&mut self, item: Rid) -> bool {
        match self.canvas_items.get_mut(item) {
            Some(s) => {
                s.commands.clear();
                true
            }
            None => false,
        }
    }

    pub fn canvas_item_commands(&self, item: Rid) -> Option<&[CanvasCommand]> {
        self.canvas_items.get(item).map(|s| s.commands.as_slice())
    }

    pub fn viewport_create(&mut self) -> Rid {
        self.viewports.alloc(ViewportState::default())
    }

    pub fn viewport_free(&mut self, viewport: Rid) -> bool {
        self.viewports.free(viewport)
    }

    pub fn viewport_get(&self, viewport: Rid) -> Option<&ViewportState> {
        self.viewports.get(viewport)
    }

    pub fn viewport_set_canvas_transform(&mut self, viewport: Rid, transform: Transform2D) -> bool {
        match self.viewports.get_mut(viewport) {
            Some(v) => {
                v.canvas_transform = transform;
                true
            }
            None => false,
        }
    }

    pub fn viewport_set_size(&mut self, viewport: Rid, size: Vector2i) -> bool {
        match self.viewports.get_mut(viewport) {
            Some(v) => {
                v.size = size;
                true
            }
            None => false,
        }
    }

    pub fn viewport_set_transparent_background(&mut self, viewport: Rid, enabled: bool) -> bool {
        match self.viewports.get_mut(viewport) {
            Some(v) => {
                v.transparent_background = enabled;
                true
            }
            None => false,
        }
    }

    pub fn viewport_set_active(&mut self, viewport: Rid, active: bool) -> bool {
        match self.viewports.get_mut(viewport) {
            Some(v) => {
                v.active = active;
                true
            }
            None => false,
        }
    }

    pub fn viewport_count(&self) -> usize {
        self.viewports.count()
    }

    /// 	exture_2d_create: stores the descriptor; pixels live in the backend.
    pub fn texture_2d_create(&mut self, format: i32, size: Vector2i) -> Rid {
        self.textures.alloc(TextureState {
            size,
            format,
            texture_type: TextureType::Type2D,
            ..TextureState::default()
        })
    }

    pub fn texture_free(&mut self, texture: Rid) -> bool {
        self.textures.free(texture)
    }

    pub fn texture_get(&self, texture: Rid) -> Option<&TextureState> {
        self.textures.get(texture)
    }

    pub fn texture_set_flags(&mut self, texture: Rid, flags: u32) -> bool {
        match self.textures.get_mut(texture) {
            Some(t) => {
                t.flags = flags;
                true
            }
            None => false,
        }
    }

    pub fn texture_count(&self) -> usize {
        self.textures.count()
    }

    pub fn material_create(&mut self) -> Rid {
        self.materials.alloc(MaterialState::default())
    }

    pub fn material_free(&mut self, material: Rid) -> bool {
        self.materials.free(material)
    }

    pub fn material_get(&self, material: Rid) -> Option<&MaterialState> {
        self.materials.get(material)
    }

    /// material_set_param; the engine stores the Variant as-is.
    pub fn material_set_param(&mut self, material: Rid, name: &str, value: crate::variant::Variant) -> bool {
        match self.materials.get_mut(material) {
            Some(m) => {
                m.params
                    .insert(crate::string::string_name::StringName::new(name), value);
                true
            }
            None => false,
        }
    }

    /// material_get_param: None for a freed material or an absent key.
    pub fn material_get_param(&self, material: Rid, name: &str) -> Option<crate::variant::Variant> {
        self.materials
            .get(material)
            .and_then(|m| m.params.get(&crate::string::string_name::StringName::new(name)).cloned())
    }

    pub fn material_set_shading_mode(&mut self, material: Rid, mode: i32) -> bool {
        match self.materials.get_mut(material) {
            Some(m) => {
                m.shading_mode = mode;
                true
            }
            None => false,
        }
    }

    pub fn material_count(&self) -> usize {
        self.materials.count()
    }

    pub fn mesh_create(&mut self) -> Rid {
        self.meshes.alloc(MeshState::default())
    }

    pub fn mesh_free(&mut self, mesh: Rid) -> bool {
        self.meshes.free(mesh)
    }

    pub fn mesh_get(&self, mesh: Rid) -> Option<&MeshState> {
        self.meshes.get(mesh)
    }

    /// mesh_add_surface; returns the new surface index.
    pub fn mesh_add_surface(&mut self, mesh: Rid, surface: MeshSurface) -> Option<usize> {
        let m = self.meshes.get_mut(mesh)?;
        m.surfaces.push(surface);
        Some(m.surfaces.len() - 1)
    }

    pub fn mesh_add_surface_material(&mut self, mesh: Rid, material: Rid) -> Option<usize> {
        self.mesh_add_surface(
            mesh,
            MeshSurface {
                material,
                aabb: crate::math::aabb::Aabb::new(
                    crate::math::vector3::Vector3::ZERO,
                    crate::math::vector3::Vector3::ZERO,
                ),
                primitive: 0,
            },
        )
    }

    pub fn mesh_surface_count(&self, mesh: Rid) -> usize {
        self.meshes.get(mesh).map(|m| m.surfaces.len()).unwrap_or(0)
    }

    /// mesh_surface_set_material; out-of-range indices return false.
    pub fn mesh_surface_set_material(&mut self, mesh: Rid, surface: usize, material: Rid) -> bool {
        match self.meshes.get_mut(mesh) {
            Some(m) => match m.surfaces.get_mut(surface) {
                Some(s) => {
                    s.material = material;
                    true
                }
                None => false,
            },
            None => false,
        }
    }

    pub fn mesh_count(&self) -> usize {
        self.meshes.count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_starts_with_identity_and_white() {
        let mut rs = RenderingServerData::new();
        let item = rs.canvas_item_create();
        let s = rs.canvas_item_get(item).expect("created item exists");
        assert_eq!(s.transform, Transform2D::IDENTITY);
        assert_eq!(s.modulate, Color::new(1.0, 1.0, 1.0, 1.0));
        assert_eq!(s.z_index, 0);
        assert!(s.visible);
        assert_eq!(rs.canvas_item_count(), 1);
    }

    #[test]
    fn setters_write_the_state() {
        let mut rs = RenderingServerData::new();
        let item = rs.canvas_item_create();

        let mut t = Transform2D::IDENTITY;
        t.columns[2] = crate::math::vector2::Vector2::new(3.0, 4.0);
        assert!(rs.canvas_item_set_transform(item, t));
        assert!(rs.canvas_item_set_modulate(item, Color::new(0.5, 0.5, 0.5, 1.0)));
        assert!(rs.canvas_item_set_self_modulate(item, Color::new(1.0, 0.0, 0.0, 1.0)));
        assert!(rs.canvas_item_set_z_index(item, -12));
        assert!(rs.canvas_item_set_visible(item, false));

        let s = rs.canvas_item_get(item).unwrap();
        assert_eq!(s.transform.columns[2], crate::math::vector2::Vector2::new(3.0, 4.0));
        assert_eq!(s.modulate, Color::new(0.5, 0.5, 0.5, 1.0));
        assert_eq!(s.self_modulate, Color::new(1.0, 0.0, 0.0, 1.0));
        assert_eq!(s.z_index, -12);
        assert!(!s.visible);
    }

    #[test]
    fn freed_items_reject_writes() {
        let mut rs = RenderingServerData::new();
        let item = rs.canvas_item_create();
        assert!(rs.canvas_item_free(item));
        assert!(rs.canvas_item_get(item).is_none());

        assert!(!rs.canvas_item_set_transform(item, Transform2D::IDENTITY));
        assert!(!rs.canvas_item_set_visible(item, true));
        assert!(!rs.canvas_item_set_z_index(item, 1));
        assert_eq!(rs.canvas_item_count(), 0);
    }

    #[test]
    fn server_stores_z_outside_the_scene_range() {
        // The scene layer validates; the server does not.
        let mut rs = RenderingServerData::new();
        let item = rs.canvas_item_create();
        assert!(rs.canvas_item_set_z_index(item, 999_999));
        assert_eq!(rs.canvas_item_get(item).unwrap().z_index, 999_999);
    }

    #[test]
    fn viewport_defaults_and_setters() {
        let mut rs = RenderingServerData::new();
        let vp = rs.viewport_create();
        let v = rs.viewport_get(vp).unwrap();
        assert_eq!(v.canvas_transform, Transform2D::IDENTITY);
        assert_eq!(v.size, Vector2i::ZERO);
        assert!(!v.transparent_background);
        assert!(v.active);

        let mut t = Transform2D::IDENTITY;
        t.columns[2] = crate::math::vector2::Vector2::new(5.0, 6.0);
        assert!(rs.viewport_set_canvas_transform(vp, t));
        assert!(rs.viewport_set_size(vp, Vector2i::new(1280, 720)));
        assert!(rs.viewport_set_transparent_background(vp, true));
        assert!(rs.viewport_set_active(vp, false));

        let v = rs.viewport_get(vp).unwrap();
        assert_eq!(v.canvas_transform.columns[2], crate::math::vector2::Vector2::new(5.0, 6.0));
        assert_eq!(v.size, Vector2i::new(1280, 720));
        assert!(v.transparent_background);
        assert!(!v.active);
    }

    #[test]
    fn freed_viewports_reject_writes() {
        let mut rs = RenderingServerData::new();
        let vp = rs.viewport_create();
        assert_eq!(rs.viewport_count(), 1);
        assert!(rs.viewport_free(vp));

        assert!(!rs.viewport_set_size(vp, Vector2i::ONE));
        assert!(!rs.viewport_set_canvas_transform(vp, Transform2D::IDENTITY));
        assert!(!rs.viewport_set_active(vp, true));
        assert!(rs.viewport_get(vp).is_none());
        assert_eq!(rs.viewport_count(), 0);
        // The canvas-item owner is independent.
        assert_eq!(rs.canvas_item_count(), 0);
    }

    #[test]
    fn texture_descriptors_are_stored() {
        let mut rs = RenderingServerData::new();
        let tex = rs.texture_2d_create(5, Vector2i::new(64, 32));
        let t = rs.texture_get(tex).unwrap();
        assert_eq!(t.size, Vector2i::new(64, 32));
        assert_eq!(t.format, 5);
        assert_eq!(t.texture_type, TextureType::Type2D);
        assert_eq!(t.layers, 1);
        assert_eq!(t.flags, 0);

        assert!(rs.texture_set_flags(tex, 0b101));
        assert_eq!(rs.texture_get(tex).unwrap().flags, 0b101);
    }

    #[test]
    fn material_params_and_shading_mode() {
        use crate::string::gstring::GoString;
        use crate::variant::Variant;

        let mut rs = RenderingServerData::new();
        let mat = rs.material_create();
        assert_eq!(rs.material_get(mat).unwrap().shading_mode, 0);
        assert_eq!(rs.material_get_param(mat, "albedo"), None);

        assert!(rs.material_set_param(mat, "albedo", Variant::Color(crate::math::color::Color::new(1.0, 0.0, 0.0, 1.0))));
        assert!(rs.material_set_param(mat, "roughness", Variant::Float(0.25)));
        assert!(rs.material_set_param(mat, "name", Variant::String(GoString::from_str("mat"))));
        assert!(rs.material_set_shading_mode(mat, 2));

        assert_eq!(
            rs.material_get_param(mat, "albedo"),
            Some(Variant::Color(crate::math::color::Color::new(1.0, 0.0, 0.0, 1.0)))
        );
        assert_eq!(rs.material_get_param(mat, "roughness"), Some(Variant::Float(0.25)));
        assert_eq!(rs.material_get(mat).unwrap().shading_mode, 2);

        assert!(rs.material_free(mat));
        assert_eq!(rs.material_get_param(mat, "roughness"), None);
        assert!(!rs.material_set_param(mat, "x", Variant::Int(1)));
    }

    #[test]
    fn draw_commands_are_recorded_in_order() {
        use crate::math::rect2::Rect2;
        use crate::math::vector2::Vector2;

        let mut rs = RenderingServerData::new();
        let item = rs.canvas_item_create();
        let tex = rs.texture_2d_create(0, Vector2i::new(4, 4));

        assert!(rs.canvas_item_add_rect(
            item,
            Rect2::new(Vector2::ZERO, Vector2::new(10.0, 5.0)),
            Color::new(1.0, 0.0, 0.0, 1.0),
            true,
            0.0
        ));
        assert!(rs.canvas_item_add_line(
            item,
            Vector2::ZERO,
            Vector2::new(1.0, 1.0),
            Color::new(0.0, 1.0, 0.0, 1.0),
            1.5,
            true
        ));
        assert!(rs.canvas_item_add_polyline(
            item,
            vec![Vector2::ZERO, Vector2::new(2.0, 0.0)],
            Color::new(0.0, 0.0, 1.0, 1.0),
            2.0
        ));
        assert!(rs.canvas_item_add_polygon(
            item,
            vec![Vector2::ZERO, Vector2::new(1.0, 0.0), Vector2::new(0.0, 1.0)],
            vec![Color::new(1.0, 1.0, 1.0, 1.0)]
        ));
        assert!(rs.canvas_item_add_texture_rect(
            item,
            tex,
            Rect2::new(Vector2::ZERO, Vector2::new(4.0, 4.0)),
            Color::new(1.0, 1.0, 1.0, 1.0),
            false
        ));

        let cmds = rs.canvas_item_commands(item).unwrap();
        assert_eq!(cmds.len(), 5);
        assert!(matches!(cmds[0], CanvasCommand::Rect { filled: true, width: 0.0, .. }));
        assert!(matches!(cmds[1], CanvasCommand::Line { width, antialiased: true, .. } if (width - 1.5).abs() < 1e-6));
        assert!(matches!(&cmds[2], CanvasCommand::Polyline { points, .. } if points.len() == 2));
        assert!(matches!(&cmds[3], CanvasCommand::Polygon { points, colors } if points.len() == 3 && colors.len() == 1));
        assert!(matches!(cmds[4], CanvasCommand::TextureRect { texture, transpose: false, .. } if texture == tex));

        // Clearing drops the commands but keeps the item itself.
        assert!(rs.canvas_item_clear(item));
        assert_eq!(rs.canvas_item_commands(item).unwrap().len(), 0);
        assert_eq!(rs.canvas_item_count(), 1);
    }

    #[test]
    fn freed_items_reject_draw_commands() {
        use crate::math::rect2::Rect2;
        use crate::math::vector2::Vector2;

        let mut rs = RenderingServerData::new();
        let item = rs.canvas_item_create();
        assert!(rs.canvas_item_free(item));

        assert!(!rs.canvas_item_add_rect(
            item,
            Rect2::new(Vector2::ZERO, Vector2::ONE),
            Color::new(1.0, 1.0, 1.0, 1.0),
            true,
            0.0
        ));
        assert!(!rs.canvas_item_clear(item));
        assert!(rs.canvas_item_commands(item).is_none());
    }

    #[test]
    fn mesh_surfaces_and_materials() {
        let mut rs = RenderingServerData::new();
        let mesh = rs.mesh_create();
        assert_eq!(rs.mesh_surface_count(mesh), 0);

        let mat = rs.material_create();
        assert_eq!(rs.mesh_add_surface_material(mesh, mat), Some(0));
        let aabb = crate::math::aabb::Aabb::new(
            crate::math::vector3::Vector3::new(-1.0, -1.0, -1.0),
            crate::math::vector3::Vector3::new(2.0, 2.0, 2.0),
        );
        let s = rs.mesh_add_surface(
            mesh,
            MeshSurface {
                material: Rid::INVALID,
                aabb,
                primitive: 3,
            },
        );
        assert_eq!(s, Some(1));
        assert_eq!(rs.mesh_surface_count(mesh), 2);

        let m = rs.mesh_get(mesh).unwrap();
        assert_eq!(m.surfaces[0].material, mat);
        assert_eq!(m.surfaces[1].primitive, 3);
        assert_eq!(m.surfaces[1].aabb, aabb);

        // Reassign the second surface's material.
        assert!(rs.mesh_surface_set_material(mesh, 1, mat));
        assert_eq!(rs.mesh_get(mesh).unwrap().surfaces[1].material, mat);
        assert!(!rs.mesh_surface_set_material(mesh, 9, mat));
    }

    #[test]
    fn freed_meshes_reject_surfaces() {
        let mut rs = RenderingServerData::new();
        let mesh = rs.mesh_create();
        assert!(rs.mesh_free(mesh));
        assert_eq!(rs.mesh_add_surface_material(mesh, Rid::INVALID), None);
        assert_eq!(rs.mesh_surface_count(mesh), 0);
        assert!(!rs.mesh_surface_set_material(mesh, 0, Rid::INVALID));
        assert_eq!(rs.mesh_count(), 0);
    }

    #[test]
    fn materials_do_not_touch_other_owners() {
        let mut rs = RenderingServerData::new();
        let _ = rs.material_create();
        let _ = rs.material_create();
        assert_eq!(rs.material_count(), 2);
        assert_eq!(rs.canvas_item_count(), 0);
        assert_eq!(rs.viewport_count(), 0);
        assert_eq!(rs.texture_count(), 0);
    }

    #[test]
    fn textures_are_independent_and_freed_handles_reject_writes() {
        let mut rs = RenderingServerData::new();
        let a = rs.texture_2d_create(1, Vector2i::new(8, 8));
        let b = rs.texture_2d_create(2, Vector2i::new(16, 16));
        assert_eq!(rs.texture_count(), 2);

        assert!(rs.texture_free(a));
        assert!(!rs.texture_set_flags(a, 1));
        assert!(rs.texture_get(a).is_none());
        // The other texture keeps its own descriptor.
        assert_eq!(rs.texture_get(b).unwrap().size, Vector2i::new(16, 16));
        assert_eq!(rs.texture_count(), 1);

        // Freeing a texture does not touch the other owners.
        assert_eq!(rs.canvas_item_count(), 0);
        assert_eq!(rs.viewport_count(), 0);
    }
}
