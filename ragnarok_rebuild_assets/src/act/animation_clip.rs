use std::io::Read;

use ragnarok_rebuild_common::{reader_ext::ReaderExt, Color, Version};

#[derive(Debug)]
pub struct AnimationClip {
    pub animation_frames: Box<[AnimationFrame]>,
}

#[derive(Debug)]
pub struct AnimationFrame {
    pub sprite_layers: Box<[SpriteLayer]>,
    pub animation_event_id: i32,
    pub sprite_anchors: Box<[SpriteAnchor]>,
}

#[derive(Debug)]
pub struct SpriteLayer {
    pub position_u: i32,
    pub position_v: i32,
    pub spritesheet_cell_index: i32,
    pub is_flipped_v: bool,
    pub tint: Color,
    pub scale_u: f32,
    pub scale_v: f32,
    pub rotation: i32,
    pub image_type_id: i32,
    pub image_width: i32,
    pub image_height: i32,
}

#[derive(Debug)]
pub struct SpriteAnchor {
    pub position_u: i32,
    pub position_v: i32,
}

impl AnimationClip {
    pub fn from_reader(mut reader: &mut dyn Read, version: &Version) -> Result<Self, super::Error> {
        let animation_frame_count = reader.read_le_u32()?;

        let animation_frames = (0..animation_frame_count)
            .map(|_| AnimationFrame::from_reader(reader, version))
            .collect::<Result<Box<[_]>, _>>()?;

        Ok(Self { animation_frames })
    }
}

impl AnimationFrame {
    pub fn from_reader(reader: &mut dyn Read, version: &Version) -> Result<Self, super::Error> {
        let sprite_layers = Self::load_sprite_layers(reader, version)?;
        let animation_event_id = Self::load_animation_event_id(reader, version)?;
        let sprite_anchors = Self::load_sprite_anchors(reader, version)?;

        Ok(Self {
            sprite_layers,
            animation_event_id,
            sprite_anchors,
        })
    }

    fn load_sprite_layers(
        mut reader: &mut dyn Read,
        version: &Version,
    ) -> Result<Box<[SpriteLayer]>, super::Error> {
        let _unused: [u8; 32] = reader.read_array()?;

        let sprite_layer_count = reader.read_le_u32()?;
        (0..sprite_layer_count)
            .map(|_| SpriteLayer::from_reader(reader, version))
            .collect::<Result<Box<[_]>, _>>()
    }

    fn load_animation_event_id(
        mut reader: &mut dyn Read,
        version: &Version,
    ) -> Result<i32, super::Error> {
        match version {
            Version(2, 0, 0) => {
                // Frame has 4 bytes that seems to be animation_event_id, but it is discarded for v2.0
                let _unused = reader.read_le_i32()?;
                Ok(-1)
            }
            Version(2, 1, 0) | Version(2, 3, 0) | Version(2, 4, 0) | Version(2, 5, 0) => {
                reader.read_le_i32().map_err(super::Error::from)
            }
            version => Err(super::Error::UnsupportedVersion(*version)),
        }
    }

    fn load_sprite_anchors(
        mut reader: &mut dyn Read,
        version: &Version,
    ) -> Result<Box<[SpriteAnchor]>, super::Error> {
        match version {
            Version(2, 0, 0) | Version(2, 1, 0) => Ok(Box::new([])),
            Version(2, 3, 0) | Version(2, 4, 0) | Version(2, 5, 0) => {
                let sprite_anchor_count = reader.read_le_u32()?;
                (0..sprite_anchor_count)
                    .map(|_| SpriteAnchor::from_reader(reader))
                    .collect::<Result<Box<[_]>, _>>()
            }
            version => Err(super::Error::UnsupportedVersion(*version)),
        }
    }
}

impl SpriteLayer {
    pub fn from_reader(reader: &mut dyn Read, version: &Version) -> Result<Self, super::Error> {
        let (position_u, position_v) = Self::load_position(reader)?;
        let spritesheet_cell_index = Self::load_spritesheet_id(reader)?;
        let is_flipped_v = Self::load_spritesheet_flags(reader)?;
        let tint = Self::load_tint(reader)?;
        let (scale_u, scale_v) = Self::load_scale(reader, version)?;
        let rotation = Self::load_rotation(reader)?;
        let image_type_id = Self::load_image_type(reader)?;
        let (image_width, image_height) = Self::load_image_dimensions(reader, version)?;

        Ok(Self {
            position_u,
            position_v,
            spritesheet_cell_index,
            is_flipped_v,
            tint,
            scale_u,
            scale_v,
            rotation,
            image_type_id,
            image_width,
            image_height,
        })
    }

    fn load_position(mut reader: &mut dyn Read) -> Result<(i32, i32), super::Error> {
        Ok((reader.read_le_i32()?, reader.read_le_i32()?))
    }

    fn load_spritesheet_id(mut reader: &mut dyn Read) -> Result<i32, super::Error> {
        reader.read_le_i32().map_err(super::Error::from)
    }

    fn load_spritesheet_flags(mut reader: &mut dyn Read) -> Result<bool, super::Error> {
        match reader.read_le_i32()? {
            0 => Ok(false),
            1 => Ok(true),
            flag => Err(super::Error::UnknownSpritesheetFlag(flag)),
        }
    }

    fn load_tint(mut reader: &mut dyn Read) -> Result<Color, super::Error> {
        Ok(Color {
            red: reader.read_u8()?,
            green: reader.read_u8()?,
            blue: reader.read_u8()?,
            alpha: reader.read_u8()?,
        })
    }

    fn load_scale(
        mut reader: &mut dyn Read,
        version: &Version,
    ) -> Result<(f32, f32), super::Error> {
        match version {
            Version(2, 0, 0) | Version(2, 1, 0) | Version(2, 3, 0) => {
                let scale = reader.read_le_f32()?;
                Ok((scale, scale))
            }
            Version(2, 4, 0) | Version(2, 5, 0) => {
                Ok((reader.read_le_f32()?, reader.read_le_f32()?))
            }
            version => Err(super::Error::UnsupportedVersion(*version)),
        }
    }

    fn load_rotation(mut reader: &mut dyn Read) -> Result<i32, super::Error> {
        Ok(reader.read_le_i32()?)
    }

    fn load_image_type(mut reader: &mut dyn Read) -> Result<i32, super::Error> {
        match reader.read_le_i32()? {
            a if (0..=1).contains(&a) => Ok(a),
            id => Err(super::Error::UnknownImageType(id)),
        }
    }

    fn load_image_dimensions(
        mut reader: &mut dyn Read,
        version: &Version,
    ) -> Result<(i32, i32), super::Error> {
        match version {
            Version(2, 0, 0) | Version(2, 1, 0) | Version(2, 3, 0) | Version(2, 4, 0) => {
                Ok((-1, -1))
            }
            Version(2, 5, 0) => Ok((reader.read_le_i32()?, reader.read_le_i32()?)),
            version => Err(super::Error::UnsupportedVersion(*version)),
        }
    }
}

impl SpriteAnchor {
    pub fn from_reader(mut reader: &mut dyn Read) -> Result<Self, super::Error> {
        let _unused: [u8; 4] = reader.read_array()?;
        let position_u = reader.read_le_i32()?;
        let position_v = reader.read_le_i32()?;
        let _unused: [u8; 4] = reader.read_array()?;

        Ok(Self {
            position_u,
            position_v,
        })
    }
}
