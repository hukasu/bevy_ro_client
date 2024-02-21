use std::io::Read;

use crate::reader_ext::ReaderExt;

pub struct Surface {
    pub bottom_left: (f32, f32),
    pub bottom_right: (f32, f32),
    pub top_left: (f32, f32),
    pub top_right: (f32, f32),
    pub texture_id: i16,
    pub lightmap_id: i16,
    pub bottom_left_vertex_color: [u8; 4],
}

impl Surface {
    pub fn from_reader(mut reader: &mut dyn Read) -> Result<Self, super::Error> {
        let [bottom_left, bottom_right, top_left, top_right] = Self::read_uvs(reader)?;

        let texture_id = reader.read_le_i16()?;
        let lightmap_id = reader.read_le_i16()?;

        let bottom_left_vertex_color = Self::read_vertex_color(reader)?;

        Ok(Self {
            bottom_left,
            bottom_right,
            top_left,
            top_right,
            texture_id,
            lightmap_id,
            bottom_left_vertex_color,
        })
    }

    fn read_uvs(mut reader: &mut dyn Read) -> Result<[(f32, f32); 4], super::Error> {
        let blu = reader.read_le_f32()?;
        let bru = reader.read_le_f32()?;
        let tlu = reader.read_le_f32()?;
        let tru = reader.read_le_f32()?;
        let blv = reader.read_le_f32()?;
        let brv = reader.read_le_f32()?;
        let tlv = reader.read_le_f32()?;
        let trv = reader.read_le_f32()?;
        Ok([(blu, blv), (bru, brv), (tlu, tlv), (tru, trv)])
    }

    fn read_vertex_color(mut reader: &mut dyn Read) -> Result<[u8; 4], super::Error> {
        let b = reader.read_u8()?;
        let g = reader.read_u8()?;
        let r = reader.read_u8()?;
        let a = reader.read_u8()?;
        Ok([r, g, b, a])
    }
}
