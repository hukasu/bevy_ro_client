use std::fmt::Display;

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

#[derive(Debug, PartialEq, Eq)]
pub struct Version {
    pub padding: u8,
    pub major: u8,
    pub minor: u8,
    pub build: u8,
}

impl Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.build)
    }
}
