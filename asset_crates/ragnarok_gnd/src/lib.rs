mod error;
mod ground_mesh_cube;
mod lightmap;
mod surface;
#[cfg(feature = "warning")]
pub mod warnings;

use std::io::Read;

use ragnarok_rebuild_common::{Version, euc_kr::read_n_euc_kr_strings, reader_ext::ReaderExt};
use ragnarok_water_plane::WaterPlane;

pub use self::{
    error::Error, ground_mesh_cube::GroundMeshCube, lightmap::Lightmap, surface::Surface,
};

#[derive(Debug)]
pub struct Gnd {
    pub signature: Box<str>,
    pub version: Version,
    pub width: u32,
    pub height: u32,
    pub scale: f32,
    pub textures: Box<[Box<str>]>,
    pub lightmap: Lightmap,
    pub surfaces: Box<[Surface]>,
    pub ground_mesh_cubes: Box<[GroundMeshCube]>,
    pub water_planes: Box<[WaterPlane]>,
}

impl Gnd {
    pub fn from_reader(mut reader: &mut dyn Read) -> Result<Self, Error> {
        let signature = Self::read_signature(reader)?;
        let version = Self::read_version(reader)?;

        if version < Version(1, 7, 0) || version >= Version(1, 10, 0) {
            return Err(Error::UnknownVersion(version));
        }

        let width = reader.read_le_u32()?;
        let height = reader.read_le_u32()?;
        let scale = reader.read_le_f32()?;

        let texture_count = reader.read_le_u32()?;
        let texture_path_len = reader.read_le_u32()?;
        let textures =
            read_n_euc_kr_strings(reader, texture_count, Some(texture_path_len as usize))?;

        let lightmap = lightmap::Lightmap::from_reader(reader)?;

        let surface_count = reader.read_le_u32()?;
        let surfaces = (0..surface_count)
            .map(|_| surface::Surface::from_reader(reader))
            .collect::<Result<Box<[_]>, Error>>()?;

        let ground_mesh_cubes = (0..(width * height))
            .map(|_| ground_mesh_cube::GroundMeshCube::from_reader(reader))
            .collect::<Result<Box<[_]>, Error>>()?;

        let water_planes = Self::read_water_planes(reader, &version)?;

        let mut rest = vec![];
        reader.read_to_end(&mut rest)?;
        if !rest.is_empty() {
            return Err(Error::IncompleteRead(version, rest.len()));
        }

        Ok(Self {
            signature,
            version,
            width,
            height,
            scale,
            textures,
            lightmap,
            surfaces,
            ground_mesh_cubes,
            water_planes,
        })
    }

    fn read_signature(mut reader: &mut dyn Read) -> Result<Box<str>, error::Error> {
        let signature = {
            let buffer: [u8; 4] = reader.read_array()?;
            String::from_utf8(buffer.to_vec())
                .map_err(|_| {
                    std::io::Error::new(std::io::ErrorKind::InvalidData, "Read invalid Utf8.")
                })?
                .into_boxed_str()
        };
        if (*signature).eq("GRGN") {
            Ok(signature)
        } else {
            Err(Error::InvalidSignature(signature))
        }
    }

    fn read_version(mut reader: &mut dyn Read) -> Result<Version, error::Error> {
        let major = reader.read_u8()?;
        let minor = reader.read_u8()?;
        Ok(Version(major, minor, 0))
    }

    fn read_water_planes(
        mut reader: &mut dyn Read,
        version: &Version,
    ) -> Result<Box<[WaterPlane]>, Error> {
        if version < &Version(1, 8, 0) {
            Ok(Box::new([]))
        } else if version < &Version(1, 9, 0) {
            let base_water_plane = WaterPlane::from_reader(reader)?;
            let horizontal = reader.read_le_i32()?;
            let vertical = reader.read_le_i32()?;
            let extras = (0..(horizontal * vertical))
                .map(|_| {
                    let level = reader.read_le_f32()?;
                    let mut water_plane = base_water_plane;
                    water_plane.water_level = level;
                    Ok(water_plane)
                })
                .collect::<Result<Vec<WaterPlane>, Error>>()?;
            let base_water_level = base_water_plane.water_level;
            Ok([base_water_plane]
                .into_iter()
                .chain(
                    extras.into_iter().filter(|plane| {
                        (plane.water_level - base_water_level).abs() > f32::EPSILON
                    }),
                )
                .collect())
        } else {
            let base_water_plane = WaterPlane::from_reader(reader)?;
            let horizontal = reader.read_le_i32()?;
            let vertical = reader.read_le_i32()?;
            let extras = (0..(horizontal * vertical))
                .map(|_| WaterPlane::from_reader(reader).map_err(Error::from))
                .collect::<Result<Vec<WaterPlane>, Error>>()?;
            let base_water_level = base_water_plane.water_level;
            Ok([base_water_plane]
                .into_iter()
                .chain(
                    extras.into_iter().filter(|plane| {
                        (plane.water_level - base_water_level).abs() > f32::EPSILON
                    }),
                )
                .collect())
        }
    }

    /// Return the normals of the top face of a cube
    ///
    /// The order is
    /// ```ignore
    /// 2 ----- 3
    /// | \     |
    /// |     \ |
    /// 0 ----- 1
    /// ```
    pub fn calculate_normals(&self, x: usize, z: usize) -> Option<[[f32; 3]; 4]> {
        let Ok(width) = usize::try_from(self.width) else {
            unreachable!("Width must fit on usize");
        };
        let Ok(height) = usize::try_from(self.height) else {
            unreachable!("Height must fit on usize");
        };

        if x >= width || z >= height {
            return None;
        }

        let Some(heights) = self.get_top_face_heights(x, z) else {
            unreachable!("Must be a valid face.");
        };

        let zero = Self::triangle_normal(
            [-0.5, heights[0], -0.5],
            [0.5, heights[1], -0.5],
            [-0.5, heights[2], 0.5],
        );
        let three = Self::triangle_normal(
            [0.5, heights[1], -0.5],
            [0.5, heights[3], 0.5],
            [-0.5, heights[2], 0.5],
        );

        let shared = {
            let sum = [zero[0] + three[0], zero[1] + three[1], zero[2] + three[2]];
            let length = (sum[0].powi(2) + sum[1].powi(2) + sum[2].powi(2)).sqrt();
            [sum[0] / length, sum[1] / length, sum[2] / length]
        };

        Some([zero, shared, shared, three])
    }

    #[inline(always)]
    fn triangle_normal(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> [f32; 3] {
        let x = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
        let y = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
        [
            x[1] * y[2] - x[2] * y[1],
            x[2] * y[0] - x[0] - y[2],
            x[0] * y[1] - x[1] * y[0],
        ]
    }

    /// Return the heights of the top face.
    ///
    /// The order is
    /// ```ignore
    /// 2 ----- 3
    /// |       |
    /// |       |
    /// 0 ----- 1
    /// ```
    pub fn get_top_face_heights(&self, x: usize, z: usize) -> Option<[f32; 4]> {
        let Ok(width) = usize::try_from(self.width) else {
            unreachable!("Width must fit on usize");
        };
        let Ok(height) = usize::try_from(self.height) else {
            unreachable!("Height must fit on usize");
        };

        if x >= width || z >= height {
            return None;
        }

        let cube = &self.ground_mesh_cubes[x + z * width];
        Some([
            cube.bottom_left_height,
            cube.bottom_right_height,
            cube.top_left_height,
            cube.top_right_height,
        ])
    }

    /// Return the heights of the east face.
    ///
    /// The order is
    /// ```ignore
    ///           3
    ///          /|
    /// + ----- 2 |
    /// |       | 1
    /// |       |/
    /// + ----- 0
    /// ```
    pub fn get_east_face_heights(&self, x: usize, z: usize) -> Option<[f32; 4]> {
        let Ok(width) = usize::try_from(self.width) else {
            unreachable!("Width must fit on usize");
        };
        let Ok(height) = usize::try_from(self.height) else {
            unreachable!("Height must fit on usize");
        };

        if x + 1 >= width || z >= height {
            return None;
        }

        let cur_cube = &self.ground_mesh_cubes[x + z * width];
        let east_cube = &self.ground_mesh_cubes[(x + 1) + z * width];
        Some([
            cur_cube.bottom_right_height,
            east_cube.bottom_left_height,
            cur_cube.top_right_height,
            east_cube.top_left_height,
        ])
    }

    /// Return the heights of the north face.
    ///
    /// The order is
    /// ```ignore
    ///   2 ----- 3
    ///  /       /|
    /// 0 ----- 1 |
    /// |       | +
    /// |       |/
    /// + ----- +
    /// ```
    pub fn get_north_face_heights(&self, x: usize, z: usize) -> Option<[f32; 4]> {
        let Ok(width) = usize::try_from(self.width) else {
            unreachable!("Width must fit on usize");
        };
        let Ok(height) = usize::try_from(self.height) else {
            unreachable!("Height must fit on usize");
        };

        if x >= width || z + 1 >= height {
            return None;
        }

        let cur_cube = &self.ground_mesh_cubes[x + z * width];
        let north_cube = &self.ground_mesh_cubes[x + (z + 1) * width];
        Some([
            cur_cube.top_left_height,
            cur_cube.top_right_height,
            north_cube.bottom_left_height,
            north_cube.bottom_right_height,
        ])
    }
}
