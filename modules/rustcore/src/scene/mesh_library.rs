//! `scene/resources/3d/mesh_library.*`: integer-keyed mesh items.
//!
//! Each item stores its name, mesh `Rid`, mesh transform, shadow-casting mode
//! (on by default), preview `Rid`, navigation mesh `Rid`/transform/layers (1 by
//! default) and optional shape `(Rid, Transform3D)` pairs. Items live in a
//! sorted map, so `get_item_list` is ascending and `get_last_unused_item_id` is
//! the highest key plus one.
//!
//! `create_item` rejects negative or duplicate ids; every other setter/getter
//! requires an existing item. Missing reads return the engine defaults.

use std::collections::BTreeMap;

use crate::math::transform_3d::Transform3D;
use crate::servers::rid::Rid;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum ShadowCastingSetting {
    Off,
    On,
    DoubleSided,
    ShadowsOnly,
}

#[derive(Clone, Debug)]
pub struct MeshLibraryShape {
    pub shape: Option<Rid>,
    pub local_transform: Transform3D,
}

#[derive(Clone, Debug)]
pub struct MeshLibraryItem {
    pub name: String,
    pub mesh: Option<Rid>,
    pub mesh_transform: Transform3D,
    pub mesh_cast_shadow: ShadowCastingSetting,
    pub shapes: Vec<MeshLibraryShape>,
    pub preview: Option<Rid>,
    pub navigation_mesh: Option<Rid>,
    pub navigation_mesh_transform: Transform3D,
    pub navigation_layers: u32,
}

impl Default for MeshLibraryItem {
    fn default() -> Self {
        Self {
            name: String::new(),
            mesh: None,
            mesh_transform: Transform3D::IDENTITY,
            mesh_cast_shadow: ShadowCastingSetting::On,
            shapes: Vec::new(),
            preview: None,
            navigation_mesh: None,
            navigation_mesh_transform: Transform3D::IDENTITY,
            navigation_layers: 1,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct MeshLibraryData {
    items: BTreeMap<i32, MeshLibraryItem>,
}

impl MeshLibraryData {
    pub fn new() -> Self {
        Self::default()
    }

    /// Rejects negative ids and ids already present.
    pub fn create_item(&mut self, item: i32) -> bool {
        if item < 0 || self.items.contains_key(&item) {
            return false;
        }
        self.items.insert(item, MeshLibraryItem::default());
        true
    }

    pub fn has_item(&self, item: i32) -> bool {
        self.items.contains_key(&item)
    }

    pub fn remove_item(&mut self, item: i32) -> bool {
        self.items.remove(&item).is_some()
    }

    pub fn clear(&mut self) {
        self.items.clear();
    }

    pub fn get_item_count(&self) -> usize {
        self.items.len()
    }

    /// Ascending ids (`RBMap` order).
    pub fn get_item_list(&self) -> Vec<i32> {
        self.items.keys().copied().collect()
    }

    /// Highest id + 1, or 0 when empty.
    pub fn get_last_unused_item_id(&self) -> i32 {
        match self.items.keys().next_back() {
            Some(last) => last + 1,
            None => 0,
        }
    }

    pub fn find_item_by_name(&self, name: &str) -> i32 {
        for (id, item) in &self.items {
            if item.name == name {
                return *id;
            }
        }
        -1
    }

    fn item_mut(&mut self, item: i32) -> Option<&mut MeshLibraryItem> {
        self.items.get_mut(&item)
    }

    pub fn set_item_name(&mut self, item: i32, name: &str) -> bool {
        match self.item_mut(item) {
            Some(i) => {
                i.name = name.to_string();
                true
            }
            None => false,
        }
    }

    pub fn get_item_name(&self, item: i32) -> String {
        self.items
            .get(&item)
            .map(|i| i.name.clone())
            .unwrap_or_default()
    }

    pub fn set_item_mesh(&mut self, item: i32, mesh: Option<Rid>) -> bool {
        match self.item_mut(item) {
            Some(i) => {
                i.mesh = mesh;
                true
            }
            None => false,
        }
    }

    pub fn get_item_mesh(&self, item: i32) -> Option<Rid> {
        self.items.get(&item).and_then(|i| i.mesh)
    }

    pub fn set_item_mesh_transform(&mut self, item: i32, transform: Transform3D) -> bool {
        match self.item_mut(item) {
            Some(i) => {
                i.mesh_transform = transform;
                true
            }
            None => false,
        }
    }

    pub fn get_item_mesh_transform(&self, item: i32) -> Transform3D {
        self.items
            .get(&item)
            .map(|i| i.mesh_transform)
            .unwrap_or(Transform3D::IDENTITY)
    }

    pub fn set_item_mesh_cast_shadow(&mut self, item: i32, setting: ShadowCastingSetting) -> bool {
        match self.item_mut(item) {
            Some(i) => {
                i.mesh_cast_shadow = setting;
                true
            }
            None => false,
        }
    }

    pub fn get_item_mesh_cast_shadow(&self, item: i32) -> ShadowCastingSetting {
        self.items
            .get(&item)
            .map(|i| i.mesh_cast_shadow)
            .unwrap_or(ShadowCastingSetting::On)
    }

    pub fn set_item_shapes(&mut self, item: i32, shapes: Vec<MeshLibraryShape>) -> bool {
        match self.item_mut(item) {
            Some(i) => {
                i.shapes = shapes;
                true
            }
            None => false,
        }
    }

    pub fn get_item_shapes(&self, item: i32) -> &[MeshLibraryShape] {
        self.items
            .get(&item)
            .map(|i| i.shapes.as_slice())
            .unwrap_or(&[])
    }

    pub fn set_item_preview(&mut self, item: i32, preview: Option<Rid>) -> bool {
        match self.item_mut(item) {
            Some(i) => {
                i.preview = preview;
                true
            }
            None => false,
        }
    }

    pub fn get_item_preview(&self, item: i32) -> Option<Rid> {
        self.items.get(&item).and_then(|i| i.preview)
    }

    pub fn set_item_navigation_mesh(&mut self, item: i32, mesh: Option<Rid>) -> bool {
        match self.item_mut(item) {
            Some(i) => {
                i.navigation_mesh = mesh;
                true
            }
            None => false,
        }
    }

    pub fn get_item_navigation_mesh(&self, item: i32) -> Option<Rid> {
        self.items.get(&item).and_then(|i| i.navigation_mesh)
    }

    pub fn set_item_navigation_mesh_transform(
        &mut self,
        item: i32,
        transform: Transform3D,
    ) -> bool {
        match self.item_mut(item) {
            Some(i) => {
                i.navigation_mesh_transform = transform;
                true
            }
            None => false,
        }
    }

    pub fn get_item_navigation_mesh_transform(&self, item: i32) -> Transform3D {
        self.items
            .get(&item)
            .map(|i| i.navigation_mesh_transform)
            .unwrap_or(Transform3D::IDENTITY)
    }

    pub fn set_item_navigation_layers(&mut self, item: i32, layers: u32) -> bool {
        match self.item_mut(item) {
            Some(i) => {
                i.navigation_layers = layers;
                true
            }
            None => false,
        }
    }

    pub fn get_item_navigation_layers(&self, item: i32) -> u32 {
        self.items.get(&item).map(|i| i.navigation_layers).unwrap_or(1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::vector3::Vector3;

    #[test]
    fn item_creation_and_ordering() {
        let mut m = MeshLibraryData::new();
        assert_eq!(m.get_last_unused_item_id(), 0);
        assert!(!m.create_item(-1));
        assert!(m.create_item(2));
        assert!(m.create_item(0));
        assert!(!m.create_item(2), "duplicate id");
        assert!(m.create_item(5));

        assert_eq!(m.get_item_list(), vec![0, 2, 5]);
        assert_eq!(m.get_item_count(), 3);
        assert_eq!(m.get_last_unused_item_id(), 6);
        assert!(m.has_item(2));

        assert!(m.remove_item(2));
        assert!(!m.remove_item(2));
        assert_eq!(m.get_item_list(), vec![0, 5]);
    }

    #[test]
    fn defaults_for_new_and_missing_items() {
        let mut m = MeshLibraryData::new();
        m.create_item(1);
        assert_eq!(m.get_item_name(1), "");
        assert!(m.get_item_mesh(1).is_none());
        assert_eq!(m.get_item_mesh_transform(1), Transform3D::IDENTITY);
        assert_eq!(m.get_item_mesh_cast_shadow(1), ShadowCastingSetting::On);
        assert!(m.get_item_shapes(1).is_empty());
        assert_eq!(m.get_item_navigation_layers(1), 1);
        assert!(m.get_item_preview(1).is_none());
        assert!(m.get_item_navigation_mesh(1).is_none());

        // Missing items read as the defaults and reject writes.
        assert_eq!(m.get_item_name(9), "");
        assert_eq!(m.get_item_mesh_transform(9), Transform3D::IDENTITY);
        assert_eq!(m.get_item_navigation_layers(9), 1);
        assert!(!m.set_item_name(9, "x"));
        assert!(!m.set_item_mesh(9, None));
        assert!(!m.set_item_shapes(9, Vec::new()));
    }

    #[test]
    fn item_values_round_trip_and_find_by_name() {
        let mut m = MeshLibraryData::new();
        m.create_item(1);
        m.create_item(3);
        let transform = Transform3D {
            basis: crate::math::basis::Basis::IDENTITY,
            origin: Vector3::new(1.0, 2.0, 3.0),
        };

        assert!(m.set_item_name(1, "crate"));
        assert!(m.set_item_mesh(1, Some(Rid::from_u64(10))));
        assert!(m.set_item_mesh_transform(1, transform));
        assert!(m.set_item_mesh_cast_shadow(1, ShadowCastingSetting::DoubleSided));
        assert!(m.set_item_preview(1, Some(Rid::from_u64(11))));
        assert!(m.set_item_navigation_mesh(1, Some(Rid::from_u64(12))));
        assert!(m.set_item_navigation_mesh_transform(1, transform));
        assert!(m.set_item_navigation_layers(1, 0b101));
        assert!(m.set_item_shapes(
            1,
            vec![MeshLibraryShape {
                shape: Some(Rid::from_u64(13)),
                local_transform: transform,
            }]
        ));

        assert_eq!(m.get_item_name(1), "crate");
        assert_eq!(m.get_item_mesh(1), Some(Rid::from_u64(10)));
        assert_eq!(m.get_item_mesh_transform(1).origin, Vector3::new(1.0, 2.0, 3.0));
        assert_eq!(m.get_item_mesh_cast_shadow(1), ShadowCastingSetting::DoubleSided);
        assert_eq!(m.get_item_preview(1), Some(Rid::from_u64(11)));
        assert_eq!(m.get_item_navigation_mesh(1), Some(Rid::from_u64(12)));
        assert_eq!(m.get_item_navigation_layers(1), 0b101);
        assert_eq!(m.get_item_shapes(1).len(), 1);
        assert_eq!(m.get_item_shapes(1)[0].shape, Some(Rid::from_u64(13)));

        assert_eq!(m.find_item_by_name("crate"), 1);
        assert_eq!(m.find_item_by_name("missing"), -1);

        assert_eq!(ShadowCastingSetting::Off as i32, 0);
        assert_eq!(ShadowCastingSetting::On as i32, 1);
        assert_eq!(ShadowCastingSetting::DoubleSided as i32, 2);
        assert_eq!(ShadowCastingSetting::ShadowsOnly as i32, 3);

        m.clear();
        assert_eq!(m.get_item_count(), 0);
        assert_eq!(m.get_last_unused_item_id(), 0);
    }
}
