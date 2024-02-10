use std::io::Read;

use crate::reader_ext::ReaderExt;

#[derive(Debug, PartialEq, PartialOrd)]
pub struct Version(pub u8, pub u8, pub u32);

impl Version {
    pub fn from_reader(mut reader: &mut dyn Read) -> Result<Self, super::Error> {
        let major = reader.read_u8()?;
        let minor = reader.read_u8()?;
        let build = if major == 2 && (2..5).contains(&minor) {
            reader.read_u8()? as u32
        } else if major == 2 && (5..7).contains(&minor) {
            reader.read_le_u32()?
        } else {
            0
        };
        let version = Version(major, minor, build);
        if major > 2 || (major == 2 && minor > 6) || (major == 2 && minor == 6 && build > 162) {
            Err(super::Error::UnknownVersion(version))
        } else {
            Ok(version)
        }
    }
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Version(major, minor, build) = self;
        write!(f, "{major}.{minor}.{build}")
    }
}
