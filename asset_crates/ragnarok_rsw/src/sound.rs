use std::io::Read;

use ragnarok_rebuild_common::{euc_kr::read_euc_kr_string, reader_ext::ReaderExt};

use super::Version;

#[derive(Debug)]
pub struct Sound {
    pub name: Box<str>,
    pub filename: Box<str>,
    pub position: [f32; 3],
    pub volume: f32,
    pub width: i32,
    pub height: i32,
    pub range: f32,
    pub cycle: f32,
}

impl Sound {
    pub fn from_reader(
        mut reader: &mut dyn Read,
        version: &Version,
    ) -> Result<Self, std::io::Error> {
        let name = read_euc_kr_string(reader, 80)?;
        let filename = read_euc_kr_string(reader, 80)?;

        let position = [
            reader.read_le_f32()?,
            reader.read_le_f32()?,
            reader.read_le_f32()?,
        ];

        let volume = reader.read_le_f32()?;
        let width = reader.read_le_i32()?;
        let height = reader.read_le_i32()?;
        let range = reader.read_le_f32()?;

        let cycle = match version {
            Version(2, 0, 0)
            | Version(2, 1, 0)
            | Version(2, 2, _)
            | Version(2, 3, _)
            | Version(2, 4, _)
            | Version(2, 5, _)
            | Version(2, 6, _) => reader.read_le_f32()?,
            _ => 4.,
        };

        Ok(Self {
            name,
            filename,
            position,
            volume,
            width,
            height,
            range,
            cycle,
        })
    }
}
