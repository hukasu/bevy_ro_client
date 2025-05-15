mod animation_clip;
mod animation_event;
#[cfg(feature = "bevy")]
pub mod assets;
#[cfg(feature = "bevy")]
pub mod components;
mod error;
#[cfg(feature = "bevy")]
pub mod events;
#[cfg(feature = "bevy")]
pub mod plugin;
#[cfg(feature = "warning")]
pub mod warnings;

use std::io::Read;

use ragnarok_rebuild_common::{Version, reader_ext::ReaderExt};

pub use self::{
    animation_clip::{AnimationClip, AnimationFrame, SpriteAnchor, SpriteLayer},
    animation_event::AnimationEvent,
    error::Error,
};

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
            Version(2, 0, 0)
            | Version(2, 1, 0)
            | Version(2, 3, 0)
            | Version(2, 4, 0)
            | Version(2, 5, 0) => Self::load_act(reader, signature, version),
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

    fn load_act(
        reader: &mut dyn Read,
        signature: [u8; 2],
        version: Version,
    ) -> Result<Self, Error> {
        let animation_clips = Self::load_animation_clips(reader, &version)?;
        let animation_events = Self::load_animation_events(reader, &version)?;
        let frame_times = Self::load_frame_times(reader, &version, animation_clips.len())?;

        Ok(Self {
            signature,
            version,
            animation_clips,
            animation_events,
            frame_times,
        })
    }

    fn load_animation_clips(
        mut reader: &mut dyn Read,
        version: &Version,
    ) -> Result<Box<[AnimationClip]>, Error> {
        let animation_clip_count = reader.read_le_u16()?;
        let _unused: [u8; 10] = reader.read_array()?;

        (0..animation_clip_count)
            .map(|_| AnimationClip::from_reader(reader, version))
            .collect::<Result<Box<[_]>, _>>()
    }

    fn load_animation_events(
        mut reader: &mut dyn Read,
        version: &Version,
    ) -> Result<Box<[AnimationEvent]>, Error> {
        match version {
            Version(2, 0, 0) => Ok(Box::new([])),
            Version(2, 1, 0) | Version(2, 3, 0) | Version(2, 4, 0) | Version(2, 5, 0) => {
                let animation_event_count = reader.read_le_u32()?;
                (0..animation_event_count)
                    .map(|_| AnimationEvent::from_reader(reader))
                    .collect::<Result<Box<[_]>, _>>()
            }
            version => Err(Error::UnsupportedVersion(*version)),
        }
    }

    fn load_frame_times(
        mut reader: &mut dyn Read,
        version: &Version,
        animation_clip_count: usize,
    ) -> Result<Box<[f32]>, Error> {
        match version {
            Version(2, 0, 0) | Version(2, 1, 0) => (0..animation_clip_count)
                .map(|_| Ok(4.))
                .collect::<Result<Box<[_]>, _>>(),
            Version(2, 3, 0) | Version(2, 4, 0) | Version(2, 5, 0) => (0..animation_clip_count)
                .map(|_| reader.read_le_f32().map_err(Error::from))
                .collect::<Result<Box<[_]>, _>>(),
            version => Err(Error::UnsupportedVersion(*version)),
        }
    }
}
