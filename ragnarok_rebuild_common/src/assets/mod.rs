pub mod rsw;
mod water_plane;

use std::io::Read;

use crate::reader_ext::ReaderExt;

fn read_euc_kr_string(
    mut reader: &mut dyn Read,
    length: usize,
) -> Result<Box<str>, std::io::Error> {
    let raw_data = reader.read_vec(length)?;
    let trimmed_data = raw_data
        .into_iter()
        .take_while(|b| b != &0)
        .collect::<Vec<_>>();
    if trimmed_data.is_empty() {
        Ok(Box::default())
    } else {
        let decode = encoding_rs::EUC_KR.decode(&trimmed_data);
        if !decode.2 {
            Ok(decode.0.as_ref().into())
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!(
                    "Could not decode filename from EUC_KR. ('{:?}')",
                    trimmed_data
                ),
            ))
        }
    }
}
