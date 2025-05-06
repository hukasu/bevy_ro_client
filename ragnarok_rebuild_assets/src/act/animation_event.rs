use std::io::Read;

use ragnarok_rebuild_common::euc_kr::read_euc_kr_string;

#[derive(Debug)]
pub struct AnimationEvent {
    pub name: Box<str>,
}

impl AnimationEvent {
    pub fn from_reader(reader: &mut dyn Read) -> Result<Self, super::Error> {
        let name = read_euc_kr_string(reader, 40)?;

        Ok(Self { name })
    }
}
