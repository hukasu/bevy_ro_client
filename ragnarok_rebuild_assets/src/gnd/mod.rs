mod error;
mod ground_mesh_cube;
mod lightmap;
mod surface;

use std::io::Read;

use ragnarok_rebuild_common::reader_ext::ReaderExt;

use crate::read_n_euc_kr_strings;

use super::common::{Version, WaterPlane};

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
        let version = Version::gnd_version_from_reader(reader)?;

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
}
