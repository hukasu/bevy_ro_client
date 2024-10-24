use std::io::Read;

use ragnarok_rebuild_common::reader_ext::ReaderExt;

#[derive(Debug)]
pub struct Animation {
    pub frames_data: Box<[super::Frame]>,
}

impl Animation {
    pub fn from_reader(mut reader: &mut dyn Read) -> Result<Self, super::Error> {
        let frame_data_count = reader.read_le_u32()?;

        let frames_data = (0..frame_data_count)
            .map(|_| super::Frame::from_reader(reader))
            .collect::<Result<_, _>>()?;

        Ok(Self { frames_data })
    }
}
