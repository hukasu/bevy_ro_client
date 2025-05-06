use std::io::Read;

use ragnarok_rebuild_common::{euc_kr::read_euc_kr_string, reader_ext::ReaderExt};

#[derive(Debug)]
pub struct Effect {
    pub name: Box<str>,
    pub position: [f32; 3],
    pub id: i32,
    pub delay: f32,
    pub parameters: [f32; 4],
}

impl Effect {
    pub fn from_reader(mut reader: &mut dyn Read) -> Result<Self, std::io::Error> {
        let name = read_euc_kr_string(reader, 80)?;

        let position = [
            reader.read_le_f32()?,
            reader.read_le_f32()?,
            reader.read_le_f32()?,
        ];

        let id = reader.read_le_i32()?;
        let delay = reader.read_le_f32()?;
        let parameters = [
            reader.read_le_f32()?,
            reader.read_le_f32()?,
            reader.read_le_f32()?,
            reader.read_le_f32()?,
        ];

        Ok(Self {
            name,
            position,
            id,
            delay,
            parameters,
        })
    }
}
