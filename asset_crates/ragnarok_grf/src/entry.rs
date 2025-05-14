use std::{fmt::Display, path::PathBuf};

const GRF_FILETYPE_FILE: u8 = 0x01;
const GRF_FILETYPE_ENCRYPT_MIXED: u8 = 0x02;
const GRF_FILETYPE_ENCRYPT_HEADER_ONLY: u8 = 0x04;

#[derive(Debug)]
pub struct Entry {
    pub filename: PathBuf,
    pub compressed_length: u32,
    pub compressed_length_aligned: u32,
    pub uncompressed_length: u32,
    pub flags: u8,
    pub offset: u32,
}

impl Entry {
    pub fn is_file(&self) -> bool {
        self.flags & GRF_FILETYPE_FILE != 0
    }

    pub fn has_mixed_encryption(&self) -> bool {
        self.flags & GRF_FILETYPE_ENCRYPT_MIXED != 0
    }

    pub fn has_header_only_encryption(&self) -> bool {
        self.flags & GRF_FILETYPE_ENCRYPT_HEADER_ONLY != 0
    }
}

impl Display for Entry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Entry {{")?;
        writeln!(f, "filename = {:?}", self.filename)?;
        writeln!(f, "compressed length = {}", self.compressed_length)?;
        writeln!(
            f,
            "compressed length aligned = {}",
            self.compressed_length_aligned
        )?;
        writeln!(f, "uncompressed length = {}", self.uncompressed_length)?;
        writeln!(
            f,
            "flags = {}",
            [
                Some("File").filter(|_| self.is_file()),
                Some("MixedEncryption").filter(|_| self.has_mixed_encryption()),
                Some("HeaderOnlyEncryption").filter(|_| self.has_header_only_encryption())
            ]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>()
            .join(" | ")
        )?;
        writeln!(f, "offset = {}", self.offset)?;
        write!(f, "}}")
    }
}
