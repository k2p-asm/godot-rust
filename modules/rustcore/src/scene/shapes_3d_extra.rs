//! `scene/resources/3d/height_map_shape_3d.*` and
//! `separation_ray_shape_3d.*`: two more shape states.
//!
//! `HeightMapShape3D` keeps the 2×2 default grid, resizes its height array when
//! the width/depth changes (new cells are zeroed, like the engine), accepts
//! `map_data` only when it matches `width * depth`, and derives `min_height` /
//! `max_height` from the assigned data.
//!
//! `SeparationRayShape3D` stores its length (1.0) and the `slide_on_slope` flag.
//!
//! Debug meshes and physics shape updates are not ported.

#[derive(Clone, Debug)]
pub struct HeightMapShape3DData {
    map_width: i32,
    map_depth: i32,
    map_data: Vec<f32>,
    min_height: f32,
    max_height: f32,
}

impl Default for HeightMapShape3DData {
    fn default() -> Self {
        Self::new()
    }
}

impl HeightMapShape3DData {
    pub fn new() -> Self {
        Self {
            map_width: 2,
            map_depth: 2,
            map_data: Vec::new(),
            min_height: 0.0,
            max_height: 0.0,
        }
    }

    pub fn get_map_width(&self) -> i32 {
        self.map_width
    }

    /// `set_map_width`: values below 1 are ignored; the data is resized.
    pub fn set_map_width(&mut self, width: i32) {
        if width < 1 || width == self.map_width {
            return;
        }
        self.map_width = width;
        self.map_data
            .resize((self.map_width * self.map_depth) as usize, 0.0);
    }

    pub fn get_map_depth(&self) -> i32 {
        self.map_depth
    }

    pub fn set_map_depth(&mut self, depth: i32) {
        if depth < 1 || depth == self.map_depth {
            return;
        }
        self.map_depth = depth;
        self.map_data
            .resize((self.map_width * self.map_depth) as usize, 0.0);
    }

    pub fn get_map_data(&self) -> &[f32] {
        &self.map_data
    }

    /// `set_map_data`: mismatched sizes are ignored (the engine logs and
    /// returns); matching data updates the min/max heights.
    pub fn set_map_data(&mut self, data: &[f32]) -> bool {
        let size = (self.map_width * self.map_depth) as usize;
        if data.len() != size {
            return false;
        }
        self.map_data.clear();
        self.map_data.extend_from_slice(data);
        if let Some(first) = data.first() {
            self.min_height = *first;
            self.max_height = *first;
            for value in &data[1..] {
                self.min_height = self.min_height.min(*value);
                self.max_height = self.max_height.max(*value);
            }
        }
        true
    }

    pub fn get_min_height(&self) -> f32 {
        self.min_height
    }

    pub fn get_max_height(&self) -> f32 {
        self.max_height
    }
}

#[derive(Clone, Debug)]
pub struct SeparationRayShape3DData {
    length: f32,
    slide_on_slope: bool,
}

impl Default for SeparationRayShape3DData {
    fn default() -> Self {
        Self::new()
    }
}

impl SeparationRayShape3DData {
    pub fn new() -> Self {
        Self {
            length: 1.0,
            slide_on_slope: false,
        }
    }

    pub fn set_length(&mut self, length: f32) {
        self.length = length;
    }

    pub fn get_length(&self) -> f32 {
        self.length
    }

    pub fn set_slide_on_slope(&mut self, active: bool) {
        self.slide_on_slope = active;
    }

    pub fn get_slide_on_slope(&self) -> bool {
        self.slide_on_slope
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn height_map_defaults_and_resizing() {
        let mut shape = HeightMapShape3DData::new();
        assert_eq!(shape.get_map_width(), 2);
        assert_eq!(shape.get_map_depth(), 2);
        assert!(shape.get_map_data().is_empty());
        assert_eq!(shape.get_min_height(), 0.0);
        assert_eq!(shape.get_max_height(), 0.0);

        shape.set_map_width(3);
        assert_eq!(shape.get_map_width(), 3);
        assert_eq!(shape.get_map_data().len(), 6, "3×2 zeros");
        shape.set_map_depth(4);
        assert_eq!(shape.get_map_depth(), 4);
        assert_eq!(shape.get_map_data().len(), 12);
        // Invalid dimensions are ignored.
        shape.set_map_width(0);
        shape.set_map_depth(-1);
        assert_eq!(shape.get_map_width(), 3);
        assert_eq!(shape.get_map_depth(), 4);
    }

    #[test]
    fn height_map_data_and_bounds() {
        let mut shape = HeightMapShape3DData::new();
        // Wrong size is refused.
        assert!(!shape.set_map_data(&[1.0]));
        assert!(shape.get_map_data().is_empty());

        assert!(shape.set_map_data(&[1.0, -2.0, 3.0, 0.5]));
        assert_eq!(shape.get_map_data(), &[1.0, -2.0, 3.0, 0.5]);
        assert_eq!(shape.get_min_height(), -2.0);
        assert_eq!(shape.get_max_height(), 3.0);

        // Shrinking keeps the leading values; growing zero-fills.
        shape.set_map_width(1);
        assert_eq!(shape.get_map_data().len(), 2);
        assert_eq!(shape.get_map_data(), &[1.0, -2.0]);
        shape.set_map_width(3);
        assert_eq!(shape.get_map_data().len(), 6);
        assert_eq!(&shape.get_map_data()[2..], &[0.0, 0.0, 0.0, 0.0]);
        // Min/max are only updated by set_map_data (the engine's behaviour).
        assert_eq!(shape.get_min_height(), -2.0);
        assert_eq!(shape.get_max_height(), 3.0);
    }

    #[test]
    fn separation_ray_defaults_and_setters() {
        let mut ray = SeparationRayShape3DData::new();
        assert_eq!(ray.get_length(), 1.0);
        assert!(!ray.get_slide_on_slope());
        ray.set_length(2.5);
        ray.set_slide_on_slope(true);
        assert_eq!(ray.get_length(), 2.5);
        assert!(ray.get_slide_on_slope());
    }
}
