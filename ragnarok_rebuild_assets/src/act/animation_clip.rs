use std::io::Read;

use ragnarok_rebuild_common::reader_ext::ReaderExt;

use crate::common::{Color, Version};

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
    pub is_flipped_v: i32,
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
        match version {
            Version(2, 0, 0) => Self::load_version_2_0(reader, version),
            Version(2, 1, 0) => Self::load_version_2_1(reader, version),
            Version(2, 3, 0) | Version(2, 4, 0) | Version(2, 5, 0) => {
                Self::load_version_2_3(reader, version)
            }
            version => Err(super::Error::UnsupportedVersion(*version)),
        }
    }

    fn load_version_2_0(
        mut reader: &mut dyn Read,
        version: &Version,
    ) -> Result<Self, super::Error> {
        let _unused: [u8; 32] = reader.read_array()?;

        let sprite_layer_count = reader.read_le_u32()?;
        let sprite_layers = (0..sprite_layer_count)
            .map(|_| SpriteLayer::from_reader(reader, version))
            .collect::<Result<Box<[_]>, _>>()?;

        // Frame has 4 bytes that seems to be animation_event_id, but it is discarded for v2.0
        let _unused = reader.read_le_i32()?;
        let animation_event_id = -1;

        Ok(Self {
            sprite_layers,
            animation_event_id,
            sprite_anchors: Box::new([]),
        })
    }

    fn load_version_2_1(
        mut reader: &mut dyn Read,
        version: &Version,
    ) -> Result<Self, super::Error> {
        let _unused: [u8; 32] = reader.read_array()?;

        let sprite_layer_count = reader.read_le_u32()?;
        let sprite_layers = (0..sprite_layer_count)
            .map(|_| SpriteLayer::from_reader(reader, version))
            .collect::<Result<Box<[_]>, _>>()?;

        let animation_event_id = reader.read_le_i32()?;

        Ok(Self {
            sprite_layers,
            animation_event_id,
            sprite_anchors: Box::new([]),
        })
    }

    fn load_version_2_3(
        mut reader: &mut dyn Read,
        version: &Version,
    ) -> Result<Self, super::Error> {
        let _unused: [u8; 32] = reader.read_array()?;

        let sprite_layer_count = reader.read_le_u32()?;
        let sprite_layers = (0..sprite_layer_count)
            .map(|_| SpriteLayer::from_reader(reader, version))
            .collect::<Result<Box<[_]>, _>>()?;

        let animation_event_id = reader.read_le_i32()?;

        let sprite_anchor_count = reader.read_le_u32()?;
        let sprite_anchors = (0..sprite_anchor_count)
            .map(|_| SpriteAnchor::from_reader(reader))
            .collect::<Result<Box<[_]>, _>>()?;

        Ok(Self {
            sprite_layers,
            animation_event_id,
            sprite_anchors,
        })
    }
}

impl SpriteLayer {
    pub fn from_reader(reader: &mut dyn Read, version: &Version) -> Result<Self, super::Error> {
        match version {
            Version(2, 0, 0) | Version(2, 1, 0) | Version(2, 3, 0) => {
                Self::load_version_2_0(reader)
            }
            Version(2, 4, 0) => Self::load_version_2_4(reader),
            Version(2, 5, 0) => Self::load_version_2_5(reader),
            version => Err(super::Error::UnsupportedVersion(*version)),
        }
    }

    fn load_version_2_0(mut reader: &mut dyn Read) -> Result<Self, super::Error> {
        let position_u: i32 = reader.read_le_i32()?;
        let position_v: i32 = reader.read_le_i32()?;
        let spritesheet_cell_index: i32 = reader.read_le_i32()?;
        let is_flipped_v: i32 = reader.read_le_i32()?;
        let tint: Color = Color {
            red: reader.read_u8()?,
            green: reader.read_u8()?,
            blue: reader.read_u8()?,
            alpha: reader.read_u8()?,
        };
        let scale_u: f32 = reader.read_le_f32()?;
        let scale_v = scale_u;
        let rotation: i32 = reader.read_le_i32()?;
        let image_type_id: i32 = reader.read_le_i32()?;

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
            image_width: -1,
            image_height: -1,
        })
    }

    fn load_version_2_4(mut reader: &mut dyn Read) -> Result<Self, super::Error> {
        let position_u = reader.read_le_i32()?;
        let position_v = reader.read_le_i32()?;
        let spritesheet_cell_index = reader.read_le_i32()?;
        let is_flipped_v = reader.read_le_i32()?;
        let tint = Color {
            red: reader.read_u8()?,
            green: reader.read_u8()?,
            blue: reader.read_u8()?,
            alpha: reader.read_u8()?,
        };
        let scale_u = reader.read_le_f32()?;
        let scale_v = reader.read_le_f32()?;
        let rotation = reader.read_le_i32()?;
        let image_type_id = reader.read_le_i32()?;

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
            image_width: -1,
            image_height: -1,
        })
    }

    fn load_version_2_5(mut reader: &mut dyn Read) -> Result<Self, super::Error> {
        let position_u = reader.read_le_i32()?;
        let position_v = reader.read_le_i32()?;
        let spritesheet_cell_index = reader.read_le_i32()?;
        let is_flipped_v = reader.read_le_i32()?;
        let tint = Color {
            red: reader.read_u8()?,
            green: reader.read_u8()?,
            blue: reader.read_u8()?,
            alpha: reader.read_u8()?,
        };
        let scale_u = reader.read_le_f32()?;
        let scale_v = reader.read_le_f32()?;
        let rotation = reader.read_le_i32()?;
        let image_type_id = reader.read_le_i32()?;
        let image_width = reader.read_le_i32()?;
        let image_height = reader.read_le_i32()?;

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
