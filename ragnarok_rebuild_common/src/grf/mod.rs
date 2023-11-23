mod entry;
mod error;
mod header;

use std::{
    fmt::Display,
    fs::File,
    io::{BufReader, Error, ErrorKind, Read, Seek},
    path::Path,
    sync::Mutex,
};

use encoding_rs::EUC_KR;
use flate2::read::ZlibDecoder;

use crate::{
    buf_reader_ext::BufReaderExt,
    grf::{
        entry::Entry,
        header::{Header, Version},
    },
};

use self::{error::GRFError, header::SIZE_OF_HEADER};

const GRF_SIGNATURE: &str = "Master of Magic\0";

pub struct GRF {
    reader: Mutex<BufReader<File>>,
    header: Header,
    file_table: Box<[Entry]>,
}

impl Display for GRF {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "GRF {{")?;
        writeln!(f, "Header {{ {:?} }},", self.header)?;
        writeln!(f, "FileTable {{ {:?} }}", self.file_table)?;
        write!(f, "}}")
    }
}

impl GRF {
    pub fn new(path: &Path) -> Result<Self, error::GRFError> {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);

        let header = Self::read_header(&mut reader)?;
        if header.signature.ne(GRF_SIGNATURE.as_bytes()) {
            Err(error::GRFError::WrongSignature)?;
        }
        if !Self::is_supported_version(&header) {
            Err(error::GRFError::UnsupportedVersion)?;
        }

        reader.seek_relative(header.filetableoffset as i64)?;
        let mut file_table =
            Self::read_file_table(&mut reader, (header.number2 - header.number1 - 7) as usize)?;
        file_table.sort_by_cached_key(|file_entry| file_entry.filename.clone());

        Ok(GRF {
            reader: reader.into(),
            header,
            file_table: file_table.into_boxed_slice(),
        })
    }

    pub fn read_file(&self, path: &str) -> Result<Box<[u8]>, error::GRFError> {
        let entry = self.search_file(path).ok_or(GRFError::FileNotFound)?;

        let data = {
            let mut reader_guard = self.reader.lock()?;
            reader_guard.seek(std::io::SeekFrom::Start(
                SIZE_OF_HEADER as u64 + entry.offset as u64,
            ))?;
            reader_guard.read_vec(entry.compressed_length_aligned as usize)
        }?;

        let decoded_data = match (
            entry.has_mixed_encryption(),
            entry.has_header_only_encryption(),
        ) {
            (true, false) => Ok(crate::des::decode(
                &data,
                entry.compressed_length_aligned as usize,
                entry.compressed_length as usize,
                false,
            )?),
            (false, true) => Ok(crate::des::decode(
                &data,
                entry.compressed_length_aligned as usize,
                entry.compressed_length as usize,
                true,
            )?),
            (false, false) => Ok(data),
            (true, true) => Err(error::GRFError::WrongSignature),
        }?;

        let uncompressed_data = {
            let mut buffer = vec![0; entry.uncompressed_length as usize];

            let mut decompressor = ZlibDecoder::new(decoded_data.as_slice());
            decompressor.read_exact(&mut buffer)?;

            buffer
        };

        Ok(uncompressed_data.into_boxed_slice())
    }

    fn search_file<'a>(&'a self, path: &str) -> Option<&'a Entry> {
        let bin_search = self
            .file_table
            .binary_search_by_key(&path, |entry| &entry.filename);
        match bin_search {
            Ok(position) => Some(&self.file_table[position]),
            Err(_) => None,
        }
    }

    fn read_header(reader: &mut BufReader<File>) -> Result<Header, Error> {
        let signature = reader.read_array()?;
        let allowed_encription = reader.read_array()?;

        let filetableoffset = reader.read_u32()?;
        let number1 = reader.read_u32()?;
        let number2 = reader.read_u32()?;
        let build = reader.read_u8()?;
        let major = reader.read_u8()?;
        let minor = reader.read_u8()?;
        let padding = reader.read_u8()?;

        Ok(Header {
            signature,
            allowed_encription,
            filetableoffset,
            number1,
            number2,
            version: Version {
                padding,
                major,
                minor,
                build,
            },
        })
    }

    fn read_file_table(
        reader: &mut BufReader<File>,
        file_count: usize,
    ) -> Result<Vec<Entry>, Error> {
        let compressed_size = reader.read_u32()?;
        let umcompressed_size = reader.read_u32()?;

        let compressed_table = reader.read_vec(compressed_size as usize)?;
        let uncompressed_table = {
            let mut buffer = vec![0; umcompressed_size as usize];

            let mut decompressor = ZlibDecoder::new(compressed_table.as_slice());
            decompressor.read_exact(&mut buffer)?;

            buffer
        };
        let mut uncompressed_table_reader = BufReader::new(uncompressed_table.as_slice());

        (0..file_count)
            .scan((), |_, _| {
                match Self::read_file_table_entry(&mut uncompressed_table_reader) {
                    Err(err) if err.kind().eq(&ErrorKind::UnexpectedEof) => None,
                    other => Some(other),
                }
            })
            .collect::<Result<Vec<Entry>, Error>>()
    }

    fn read_file_table_entry(table_reader: &mut BufReader<&[u8]>) -> Result<Entry, Error> {
        let cp949_filename = table_reader.read_null_terminated_string()?;

        let (filename, _encoding, chars_replaced) =
            EUC_KR.decode(&cp949_filename[..(cp949_filename.len() - 1)]);
        if chars_replaced {
            Err(Error::new(
                ErrorKind::InvalidInput,
                "String had invalid CP949 characters",
            ))?
        }

        let compressed_length = table_reader.read_u32()?;
        let compressed_length_aligned = table_reader.read_u32()?;
        let uncompressed_length = table_reader.read_u32()?;
        let flags = table_reader.read_u8()?;
        let offset = table_reader.read_u32()?;

        Ok(Entry {
            filename: filename.replace('\\', "/").into_boxed_str(),
            compressed_length,
            compressed_length_aligned,
            uncompressed_length,
            flags,
            offset,
        })
    }

    fn is_supported_version(header: &Header) -> bool {
        static SUPPORTED_VERSION: [Version; 1] = [Version {
            padding: 0,
            major: 2,
            minor: 0,
            build: 0,
        }];

        SUPPORTED_VERSION
            .iter()
            .any(|version| version.eq(&header.version))
    }
}
