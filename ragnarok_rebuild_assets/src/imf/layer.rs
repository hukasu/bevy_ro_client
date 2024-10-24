use std::io::Read;

use ragnarok_rebuild_common::reader_ext::ReaderExt;

#[derive(Debug)]
pub struct Layer {
    pub animations: Box<[super::Animation]>,
}

impl Layer {
    pub fn from_reader(mut reader: &mut dyn Read) -> Result<Self, super::Error> {
        let animation_count = reader.read_le_u32()?;

        let animations = (0..animation_count)
            .map(|_| super::Animation::from_reader(reader))
            .collect::<Result<_, _>>()?;

        Ok(Self { animations })
    }
}
