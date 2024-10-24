use std::io::Read;

use ragnarok_rebuild_common::reader_ext::ReaderExt;

use crate::{gnd, rsm};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Version(pub u8, pub u8, pub u32);

impl Version {
    pub fn new(major: u8, minor: u8, patch: u32) -> Self {
        Self(major, minor, patch)
    }

    pub fn major(&self) -> u8 {
        self.0
    }

    pub fn minor(&self) -> u8 {
        self.1
    }

    pub fn patch(&self) -> u32 {
        self.2
    }

    pub fn gnd_version_from_reader(mut reader: &mut dyn Read) -> Result<Self, gnd::Error> {
        let major = reader.read_u8()?;
        let minor = reader.read_u8()?;
        Ok(Version(major, minor, 0))
    }

    pub fn rsm_version_from_reader(mut reader: &mut dyn Read) -> Result<Self, rsm::Error> {
        let major = reader.read_u8()?;
        let minor = reader.read_u8()?;
        Ok(Version(major, minor, 0))
    }
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Version(major, minor, build) = self;
        write!(f, "{major}.{minor}.{build}")
    }
}
