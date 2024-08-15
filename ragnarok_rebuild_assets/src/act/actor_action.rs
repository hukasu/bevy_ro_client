use crate::reader_ext::ReaderExt;

#[derive(Debug)]
pub struct ActorAction {
    frame_count: u32,
    frames: Box<[super::ActionFrame]>,
}

impl ActorAction {
    pub fn from_bytes(bytes: &mut &[u8]) -> Result<Self, std::io::Error> {
        let frame_count = bytes.read_le_u32()?;

        let frames = (0..frame_count)
            .map(|_| super::ActionFrame::from_bytes(bytes))
            .collect::<Result<_, _>>()?;

        Ok(Self {
            frame_count,
            frames,
        })
    }
}
