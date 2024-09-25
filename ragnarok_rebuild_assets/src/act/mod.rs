mod animation_clip;
mod animation_event;
mod error;

use std::io::Read;

use animation_event::AnimationEvent;
use ragnarok_rebuild_common::reader_ext::ReaderExt;

use crate::common::Version;

pub use self::{animation_clip::AnimationClip, error::Error};

#[derive(Debug)]
pub struct Act {
    pub signature: [u8; 2],
    pub version: Version,
    pub animation_clips: Box<[AnimationClip]>,
    pub animation_events: Box<[AnimationEvent]>,
    pub frame_times: Box<[f32]>,
}

impl Act {
    pub fn from_reader(reader: &mut dyn Read) -> Result<Self, Error> {
        let signature = Self::read_signature(reader)?;
        let version = Self::read_version(reader)?;

        let actor = match version {
            Version(2, 0, 0) => Self::load_version_2_0(reader, signature, version),
            Version(2, 1, 0) => Self::load_version_2_1(reader, signature, version),
            Version(2, 3, 0) | Version(2, 4, 0) | Version(2, 5, 0) => {
                Self::load_version_2_3(reader, signature, version)
            }
            version => Err(Error::UnsupportedVersion(version)),
        }?;

        let mut remainder = vec![];
        reader.read_to_end(&mut remainder)?;

        if !remainder.is_empty() {
            Err(Error::IncompleteRead(remainder.len()))
        } else {
            Ok(actor)
        }
    }

    fn read_signature(mut reader: &mut dyn Read) -> Result<[u8; 2], Error> {
        let signature = reader.read_array()?;
        if signature.eq(b"AC") {
            Ok(signature)
        } else {
            Err(Error::WrongSignature)
        }
    }

    fn read_version(mut reader: &mut dyn Read) -> Result<Version, Error> {
        let array: [u8; 2] = reader.read_array()?;
        Ok(Version(array[1], array[0], 0))
    }

    fn load_version_2_0(
        mut reader: &mut dyn Read,
        signature: [u8; 2],
        version: Version,
    ) -> Result<Self, Error> {
        let animation_clip_count = reader.read_le_u16()?;
        let _unused: [u8; 10] = reader.read_array()?;

        let animation_clips = (0..animation_clip_count)
            .map(|_| AnimationClip::from_reader(reader, &version))
            .collect::<Result<Box<[_]>, _>>()?;

        Ok(Self {
            signature,
            version,
            animation_clips,
            animation_events: Box::new([]),
            frame_times: Box::new([]),
        })
    }

    fn load_version_2_1(
        mut reader: &mut dyn Read,
        signature: [u8; 2],
        version: Version,
    ) -> Result<Self, Error> {
        let animation_clip_count = reader.read_le_u16()?;
        let _unused: [u8; 10] = reader.read_array()?;

        let animation_clips = (0..animation_clip_count)
            .map(|_| AnimationClip::from_reader(reader, &version))
            .collect::<Result<Box<[_]>, _>>()?;

        let animation_event_count = reader.read_le_u32()?;
        let animation_events = (0..animation_event_count)
            .map(|_| AnimationEvent::from_reader(reader))
            .collect::<Result<Box<[_]>, _>>()?;

        Ok(Self {
            signature,
            version,
            animation_clips,
            animation_events,
            frame_times: Box::new([]),
        })
    }

    fn load_version_2_3(
        mut reader: &mut dyn Read,
        signature: [u8; 2],
        version: Version,
    ) -> Result<Self, Error> {
        let animation_clip_count = reader.read_le_u16()?;
        let _unused: [u8; 10] = reader.read_array()?;

        let animation_clips = (0..animation_clip_count)
            .map(|_| AnimationClip::from_reader(reader, &version))
            .collect::<Result<Box<[_]>, _>>()?;

        let animation_event_count = reader.read_le_u32()?;
        let animation_events = (0..animation_event_count)
            .map(|_| AnimationEvent::from_reader(reader))
            .collect::<Result<Box<[_]>, _>>()?;

        let frame_times = (0..animation_clip_count)
            .map(|_| reader.read_le_f32())
            .collect::<Result<Box<[_]>, _>>()?;

        Ok(Self {
            signature,
            version,
            animation_clips,
            animation_events,
            frame_times,
        })
    }
}
