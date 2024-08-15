use crate::reader_ext::ReaderExt;

#[derive(Debug)]
pub struct AnchorPoint {
    unknown: u32,
    x: u32,
    y: u32,
    attr: u32,
}

impl AnchorPoint {
    pub fn from_bytes(bytes: &mut &[u8]) -> Result<Self, std::io::Error> {
        let unknown = bytes.read_le_u32()?;
        let x = bytes.read_le_u32()?;
        let y = bytes.read_le_u32()?;
        let attr = bytes.read_le_u32()?;

        Ok(Self {
            unknown,
            x,
            y,
            attr,
        })
    }
}
