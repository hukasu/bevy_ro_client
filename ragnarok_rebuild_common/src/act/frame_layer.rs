use crate::reader_ext::ReaderExt;

#[derive(Debug)]
pub struct FrameLayer {
    x: i32,
    y: i32,
    sprite_id: i32,
    flags: u32,
    color: [u8; 4],
    x_scale: f32,
    y_scale: f32,
    rotation: f32,
    sprite_type: i32,
    width: i32,
    height: i32,
}

impl FrameLayer {
    pub fn from_bytes(bytes: &mut &[u8]) -> Result<Self, std::io::Error> {
        let x = bytes.read_le_i32()?;
        let y = bytes.read_le_i32()?;

        let sprite_id = bytes.read_le_i32()?;
        let flags = bytes.read_le_u32()?;
        let color = bytes.read_array()?;

        let x_scale = bytes.read_le_f32()?;
        let y_scale = bytes.read_le_f32()?;
        let rotation = bytes.read_le_f32()?;

        let sprite_type = bytes.read_le_i32()?;
        let width = bytes.read_le_i32()?;
        let height = bytes.read_le_i32()?;

        Ok(Self {
            x,
            y,
            sprite_id,
            flags,
            color,
            x_scale,
            y_scale,
            rotation,
            sprite_type,
            width,
            height,
        })
    }

    pub fn is_y_mirrored(&self) -> bool {
        (self.flags & 0x00000001) != 0
    }

    pub fn is_palette_sprite(&self) -> bool {
        self.sprite_type == 0
    }

    pub fn is_truecolor_sprite(&self) -> bool {
        self.sprite_type == 0
    }
}
