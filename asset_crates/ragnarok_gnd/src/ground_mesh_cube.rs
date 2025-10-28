use std::io::Read;

use ragnarok_rebuild_common::reader_ext::ReaderExt;

use crate::triangle_normal;

#[derive(Debug)]
pub struct GroundMeshCube {
    pub bottom_left_height: f32,
    pub bottom_right_height: f32,
    pub top_left_height: f32,
    pub top_right_height: f32,
    pub upwards_facing_surface: i32,
    pub north_facing_surface: i32,
    pub east_facing_surface: i32,
}

impl GroundMeshCube {
    pub fn from_reader(mut reader: &mut dyn Read) -> Result<Self, super::Error> {
        let bottom_left_height = reader.read_le_f32()?;
        let bottom_right_height = reader.read_le_f32()?;
        let top_left_height = reader.read_le_f32()?;
        let top_right_height = reader.read_le_f32()?;
        let upwards_facing_surface = reader.read_le_i32()?;
        let north_facing_surface = reader.read_le_i32()?;
        let east_facing_surface = reader.read_le_i32()?;

        Ok(Self {
            bottom_left_height,
            bottom_right_height,
            top_left_height,
            top_right_height,
            upwards_facing_surface,
            north_facing_surface,
            east_facing_surface,
        })
    }

    /// Return the normals of the top face of the cube.
    ///
    /// The order is
    /// ```ignore
    /// + ----- +
    /// | \  1  |
    /// |   \   |
    /// | 0   \ |
    /// + ----- +
    /// ```
    pub fn calculate_normals(&self, scale: f32) -> [[f32; 3]; 2] {
        let heights = [
            self.bottom_left_height,
            self.bottom_right_height,
            self.top_left_height,
            self.top_right_height,
        ];

        let zero = triangle_normal(
            [-0.5 * scale, heights[0], -0.5 * scale],
            [0.5 * scale, heights[1], -0.5 * scale],
            [-0.5 * scale, heights[2], 0.5 * scale],
        );
        let one = triangle_normal(
            [0.5 * scale, heights[1], -0.5 * scale],
            [0.5 * scale, heights[3], 0.5 * scale],
            [-0.5 * scale, heights[2], 0.5 * scale],
        );

        [zero, one]
    }

    /// Tests if the top of the [`GroundMeshCube`] is connected to the bottom
    /// of another [`GroundMeshCube`]
    pub fn is_connected_top(&self, top: &Self) -> bool {
        let left_connected = (top.bottom_left_height - self.top_left_height).abs() < f32::EPSILON;
        let right_connected =
            (top.bottom_right_height - self.top_right_height).abs() < f32::EPSILON;

        left_connected || right_connected
    }

    /// Tests if the bottom of the [`GroundMeshCube`] is connected to the top
    /// of another [`GroundMeshCube`]
    #[inline(always)]
    pub fn is_connected_bottom(&self, bottom: &Self) -> bool {
        bottom.is_connected_top(self)
    }

    /// Tests if the top of the [`GroundMeshCube`] is connected to the bottom
    /// of another [`GroundMeshCube`]
    pub fn is_connected_right(&self, right: &Self) -> bool {
        let top_connected = (self.top_right_height - right.top_left_height).abs() < f32::EPSILON;
        let bottom_connected =
            (self.bottom_right_height - right.bottom_left_height).abs() < f32::EPSILON;

        top_connected || bottom_connected
    }

    /// Tests if the bottom of the [`GroundMeshCube`] is connected to the top
    /// of another [`GroundMeshCube`]
    #[inline(always)]
    pub fn is_connected_left(&self, left: &Self) -> bool {
        left.is_connected_right(self)
    }
}
