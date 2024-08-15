use crate::reader_ext::ReaderExt;

#[derive(Debug)]
pub struct FrameEvent {
    name: Box<str>,
}

impl FrameEvent {
    pub fn from_bytes(bytes: &mut &[u8]) -> Result<Self, std::io::Error> {
        let buffer: [u8; 40] = bytes.read_array()?;
        let trimmed = if let Some(pos) = buffer.iter().position(|c| c.eq(&0)) {
            &buffer[..pos]
        } else {
            &buffer
        };
        let (name, _encoding, malformed) = encoding_rs::EUC_KR.decode(trimmed);
        if malformed {
            Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Could not read EUC_KR event name.",
            ))?
        }

        Ok(Self {
            name: name.into_owned().into_boxed_str(),
        })
    }
}
