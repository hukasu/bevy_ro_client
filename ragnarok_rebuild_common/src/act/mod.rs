mod action_frame;
mod actor_action;
mod anchor_point;
mod error;
mod frame_event;
mod frame_layer;

use crate::reader_ext::ReaderExt;

pub use self::{
    action_frame::ActionFrame, actor_action::ActorAction, anchor_point::AnchorPoint,
    error::ActorError, frame_event::FrameEvent, frame_layer::FrameLayer,
};

#[derive(Debug)]
pub struct Actor {
    header: [u8; 2],
    version: [u8; 2],
    action_count: u16,
    padding: [u8; 10],
    actions: Box<[ActorAction]>,
    event_count: u32,
    events: Box<[FrameEvent]>,
    intervals: Box<[f32]>,
}

impl Actor {
    pub fn from_bytes(mut bytes: &[u8]) -> Result<Self, ActorError> {
        let header = bytes.read_array()?;
        if header.ne(&[b'A', b'C']) {
            Err(ActorError::WrongHeader)?
        }
        let version = bytes.read_array()?;
        if version.ne(&[5, 2]) {
            Err(ActorError::UnsupportedVersion)?
        }

        let action_count = bytes.read_le_u16()?;
        let padding = bytes.read_array()?;
        let actions = (0..action_count)
            .map(|_| ActorAction::from_bytes(&mut bytes))
            .collect::<Result<_, _>>()?;

        let event_count = bytes.read_le_u32()?;
        let events = (0..event_count)
            .map(|_| FrameEvent::from_bytes(&mut bytes))
            .collect::<Result<_, _>>()?;

        let intervals = (0..action_count)
            .map(|_| bytes.read_le_f32())
            .collect::<Result<_, _>>()?;

        Ok(Self {
            header,
            version,
            action_count,
            padding,
            actions,
            event_count,
            events,
            intervals,
        })
    }
}
