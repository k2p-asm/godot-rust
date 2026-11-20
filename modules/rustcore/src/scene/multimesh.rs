//! `scene/resources/multimesh.*`: multimesh allocation state.
//!
//! Defaults: 2D transform format, no colors/custom data, 0 instances, visible
//! count -1 (all), fast interpolation quality. The engine only lets the format
//! and the color/custom-data flags change while `instance_count == 0`;
//! `visible_instance_count` must be `-1..=instance_count` and
//! `instance_count` cannot be negative.
//!
//! The buffer layout and per-instance transforms are not ported.

use crate::math::aabb::Aabb;
use crate::servers::rid::Rid;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum TransformFormat {
    Transform2D,
    Transform3D,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum PhysicsInterpolationQuality {
    Fast,
    High,
}

#[derive(Clone, Debug)]
pub struct MultiMeshData {
    mesh: Option<Rid>,
    transform_format: TransformFormat,
    custom_aabb: Aabb,
    use_colors: bool,
    use_custom_data: bool,
    instance_count: i32,
    visible_instance_count: i32,
    physics_interpolation_quality: PhysicsInterpolationQuality,
}

impl Default for MultiMeshData {
    fn default() -> Self {
        Self::new()
    }
}

impl MultiMeshData {
    pub fn new() -> Self {
        Self {
            mesh: None,
            transform_format: TransformFormat::Transform2D,
            custom_aabb: Aabb::default(),
            use_colors: false,
            use_custom_data: false,
            instance_count: 0,
            visible_instance_count: -1,
            physics_interpolation_quality: PhysicsInterpolationQuality::Fast,
        }
    }

    pub fn set_mesh(&mut self, mesh: Option<Rid>) {
        self.mesh = mesh;
    }

    pub fn get_mesh(&self) -> Option<Rid> {
        self.mesh
    }

    /// Only while there are no instances.
    pub fn set_use_colors(&mut self, enable: bool) -> bool {
        if self.instance_count > 0 {
            return false;
        }
        self.use_colors = enable;
        true
    }

    pub fn is_using_colors(&self) -> bool {
        self.use_colors
    }

    /// Only while there are no instances.
    pub fn set_use_custom_data(&mut self, enable: bool) -> bool {
        if self.instance_count > 0 {
            return false;
        }
        self.use_custom_data = enable;
        true
    }

    pub fn is_using_custom_data(&self) -> bool {
        self.use_custom_data
    }

    /// Only while there are no instances.
    pub fn set_transform_format(&mut self, format: TransformFormat) -> bool {
        if self.instance_count > 0 {
            return false;
        }
        self.transform_format = format;
        true
    }

    pub fn get_transform_format(&self) -> TransformFormat {
        self.transform_format
    }

    /// Rejects negative counts; does not adjust the visible count.
    pub fn set_instance_count(&mut self, count: i32) -> bool {
        if count < 0 {
            return false;
        }
        self.instance_count = count;
        true
    }

    pub fn get_instance_count(&self) -> i32 {
        self.instance_count
    }

    /// Accepts `-1` (all) up to the instance count.
    pub fn set_visible_instance_count(&mut self, count: i32) -> bool {
        if count < -1 || count > self.instance_count {
            return false;
        }
        self.visible_instance_count = count;
        true
    }

    pub fn get_visible_instance_count(&self) -> i32 {
        self.visible_instance_count
    }

    pub fn set_custom_aabb(&mut self, aabb: Aabb) {
        self.custom_aabb = aabb;
    }

    pub fn get_custom_aabb(&self) -> Aabb {
        self.custom_aabb
    }

    pub fn set_physics_interpolation_quality(&mut self, quality: PhysicsInterpolationQuality) {
        self.physics_interpolation_quality = quality;
    }

    pub fn get_physics_interpolation_quality(&self) -> PhysicsInterpolationQuality {
        self.physics_interpolation_quality
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::vector3::Vector3;

    #[test]
    fn defaults() {
        let m = MultiMeshData::new();
        assert!(m.get_mesh().is_none());
        assert_eq!(m.get_transform_format(), TransformFormat::Transform2D);
        assert_eq!(m.get_custom_aabb(), Aabb::default());
        assert!(!m.is_using_colors());
        assert!(!m.is_using_custom_data());
        assert_eq!(m.get_instance_count(), 0);
        assert_eq!(m.get_visible_instance_count(), -1);
        assert_eq!(
            m.get_physics_interpolation_quality(),
            PhysicsInterpolationQuality::Fast
        );
    }

    #[test]
    fn format_and_flags_require_zero_instances() {
        let mut m = MultiMeshData::new();
        assert!(m.set_instance_count(4));
        assert!(!m.set_transform_format(TransformFormat::Transform3D));
        assert!(!m.set_use_colors(true));
        assert!(!m.set_use_custom_data(true));
        assert_eq!(m.get_transform_format(), TransformFormat::Transform2D);

        assert!(m.set_instance_count(0));
        assert!(m.set_transform_format(TransformFormat::Transform3D));
        assert!(m.set_use_colors(true));
        assert!(m.set_use_custom_data(true));
        assert_eq!(m.get_transform_format(), TransformFormat::Transform3D);
        assert!(m.is_using_colors());
        assert!(m.is_using_custom_data());
    }

    #[test]
    fn counts_are_validated() {
        let mut m = MultiMeshData::new();
        assert!(!m.set_instance_count(-1));
        assert!(m.set_instance_count(3));
        assert_eq!(m.get_instance_count(), 3);

        assert!(!m.set_visible_instance_count(-2));
        assert!(!m.set_visible_instance_count(4));
        assert!(m.set_visible_instance_count(-1));
        assert!(m.set_visible_instance_count(2));
        assert_eq!(m.get_visible_instance_count(), 2);

        // Lowering the instance count does not clamp the visible count.
        assert!(m.set_instance_count(1));
        assert_eq!(m.get_visible_instance_count(), 2);

        m.set_custom_aabb(Aabb::new(Vector3::ZERO, Vector3::ONE));
        m.set_physics_interpolation_quality(PhysicsInterpolationQuality::High);
        assert_eq!(m.get_custom_aabb().size, Vector3::ONE);
        assert_eq!(
            m.get_physics_interpolation_quality(),
            PhysicsInterpolationQuality::High
        );

        assert_eq!(TransformFormat::Transform2D as i32, 0);
        assert_eq!(TransformFormat::Transform3D as i32, 1);
        assert_eq!(PhysicsInterpolationQuality::Fast as i32, 0);
        assert_eq!(PhysicsInterpolationQuality::High as i32, 1);
    }
}
