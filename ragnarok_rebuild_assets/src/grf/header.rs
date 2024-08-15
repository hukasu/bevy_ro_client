use std::io::Read;

use ragnarok_rebuild_common::reader_ext::ReaderExt;

use crate::common::Version;

pub const SIZE_OF_HEADER: usize = 16 + 14 + 4 + 4 + 4 + 4;

#[derive(Debug)]
pub struct Header {
    pub signature: [u8; 15],
    pub allowed_encription: [u8; 15],
    pub filetableoffset: u32,
    pub scrambling_seed: u32,
    pub scrambled_file_count: u32,
    pub version: Version,
}

impl Header {
    pub fn from_reader(mut reader: &mut dyn Read) -> Result<Header, super::Error> {
        let signature = reader.read_array()?;
        let allowed_encription = reader.read_array()?;

        let filetableoffset = reader.read_le_u32()?;
        let scrambling_seed = reader.read_le_u32()?;
        let scrambled_file_count = reader.read_le_u32()?;

        let version = Version::grf_version_from_reader(reader)?;

        Ok(Header {
            signature,
            allowed_encription,
            filetableoffset,
            scrambling_seed,
            scrambled_file_count,
            version,
        })
    }
}
