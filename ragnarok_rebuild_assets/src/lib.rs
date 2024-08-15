pub mod common;
pub mod gnd;
pub mod grf;
pub mod rsm;
pub mod rsw;

use std::io::{self, Read};

use ragnarok_rebuild_common::reader_ext::ReaderExt;

#[inline(always)]
fn read_n_euc_kr_strings(
    mut reader: &mut dyn Read,
    count: u32,
    fixed_len: Option<usize>,
) -> Result<Box<[Box<str>]>, io::Error> {
    (0..count)
        .map(|_| -> Result<Box<str>, io::Error> {
            let len = if let Some(len) = fixed_len {
                len
            } else {
                usize::try_from(reader.read_le_u32()?)
                    .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?
            };
            read_euc_kr_string(reader, len)
        })
        .collect::<Result<Box<[Box<str>]>, io::Error>>()
}

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
            Ok(decode.0.as_ref().replace('\\', "/").to_lowercase().into())
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
