use std::io::Read;

use ragnarok_rebuild_common::reader_ext::ReaderExt;

use crate::common::Version;

const GRF_SIGNATURE: &[u8] = b"Master of Magic";
pub const SIZE_OF_HEADER: usize = 16 + 14 + 4 + 4 + 4 + 4;

#[allow(dead_code)]
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
        let signature = Self::read_signature(reader)?;
        let allowed_encription = reader.read_array()?;

        let filetableoffset = reader.read_le_u32()?;
        let scrambling_seed = reader.read_le_u32()?;
        let scrambled_file_count = reader.read_le_u32()?;

        let version = Self::read_version(reader)?;

        Ok(Header {
            signature,
            allowed_encription,
            filetableoffset,
            scrambling_seed,
            scrambled_file_count,
            version,
        })
    }

    fn read_signature(mut reader: &mut dyn Read) -> Result<[u8; 15], super::Error> {
        let signature = reader.read_array()?;

        if signature.ne(GRF_SIGNATURE) {
            Err(super::Error::WrongSignature)?
        } else {
            Ok(signature)
        }
    }

    fn read_version(mut reader: &mut dyn Read) -> Result<Version, super::Error> {
        let _padding = reader.read_u8()?;

        let major = reader.read_u8()?;
        let minor = reader.read_u8()?;
        let build = u32::from(reader.read_u8()?);

        let version = Version(major, minor, build);

        match version {
            Version(2, 0, 0) => Ok(version),
            version => Err(super::Error::UnsupportedVersion(version)),
        }
    }
}
