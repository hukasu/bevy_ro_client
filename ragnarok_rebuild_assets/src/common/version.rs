use std::io::Read;

use ragnarok_rebuild_common::reader_ext::ReaderExt;

use crate::{gnd, grf, rsm, rsw};

#[derive(Debug, PartialEq, PartialOrd)]
pub struct Version(pub u8, pub u8, pub u32);

impl Version {
    pub fn gnd_version_from_reader(mut reader: &mut dyn Read) -> Result<Self, gnd::Error> {
        let major = reader.read_u8()?;
        let minor = reader.read_u8()?;
        Ok(Version(major, minor, 0))
    }

    pub fn grf_version_from_reader(mut reader: &mut dyn Read) -> Result<Self, grf::Error> {
        let _padding = reader.read_u8()?;
        let major = reader.read_u8()?;
        let minor = reader.read_u8()?;
        let build = u32::from(reader.read_u8()?);
        Ok(Version(major, minor, build))
    }

    pub fn rsm_version_from_reader(mut reader: &mut dyn Read) -> Result<Self, rsm::Error> {
        let major = reader.read_u8()?;
        let minor = reader.read_u8()?;
        Ok(Version(major, minor, 0))
    }

    pub fn rsw_version_from_reader(mut reader: &mut dyn Read) -> Result<Self, rsw::Error> {
        let major = reader.read_u8()?;
        let minor = reader.read_u8()?;
        let build = if major == 2 && (2..5).contains(&minor) {
            u32::from(reader.read_u8()?)
        } else if major == 2 && (5..7).contains(&minor) {
            reader.read_le_u32()?
        } else {
            0
        };
        let version = Version(major, minor, build);
        if major > 2 || (major == 2 && minor > 6) || (major == 2 && minor == 6 && build > 162) {
            Err(rsw::Error::UnknownVersion(version))
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
