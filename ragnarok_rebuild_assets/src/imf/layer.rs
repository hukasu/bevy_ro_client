use std::io::Read;

use ragnarok_rebuild_common::reader_ext::ReaderExt;

#[derive(Debug)]
pub struct Layer {
    pub frames: Box<[super::Animation]>,
}

impl Layer {
    pub fn from_reader(mut reader: &mut dyn Read) -> Result<Self, super::Error> {
        let frame_count = reader.read_le_u32()?;

        let frames = (0..frame_count)
            .map(|_| super::Animation::from_reader(reader))
            .collect::<Result<_, _>>()?;

        Ok(Self { frames })
    }
}
