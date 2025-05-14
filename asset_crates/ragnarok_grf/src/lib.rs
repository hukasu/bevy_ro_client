mod entry;
mod error;
mod header;
#[cfg(feature = "bevy")]
pub mod reader;

use std::{
    fmt::{Display, Formatter},
    fs::File,
    io::{self, BufReader, ErrorKind, Read, Seek},
    path::{Path, PathBuf},
    sync::Mutex,
};

use encoding_rs::EUC_KR;
use flate2::read::ZlibDecoder;

use ragnarok_rebuild_common::{
    des,
    reader_ext::{BufReaderExt, ReaderExt},
};

pub use self::error::Error;
use self::{
    entry::Entry,
    header::{Header, SIZE_OF_HEADER},
};

pub struct Grf {
    reader: Mutex<BufReader<File>>,
    header: Header,
    file_table: Box<[Entry]>,
}

impl Display for Grf {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "GRF {{")?;
        writeln!(f, "Header {{ {:?} }},", self.header)?;
        writeln!(f, "FileTable {{ {:?} }}", self.file_table)?;
        write!(f, "}}")
    }
}

impl Grf {
    pub fn new(path: &Path) -> Result<Self, error::Error> {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);

        let header = Header::from_reader(&mut reader)?;

        reader.seek_relative(header.filetableoffset as i64)?;
        let mut file_table = Self::read_file_table(
            &mut reader,
            (header.scrambled_file_count - header.scrambling_seed - 7) as usize,
        )?;
        file_table.sort_by_cached_key(|file_entry| file_entry.filename.clone());

        Ok(Grf {
            reader: reader.into(),
            header,
            file_table: file_table.into(),
        })
    }

    pub fn iter_filenames(&self) -> impl Iterator<Item = &PathBuf> {
        self.file_table.iter().map(|entry| &entry.filename)
    }

    pub fn read_file(&self, path: &Path) -> Result<Box<[u8]>, error::Error> {
        let entry = self.search_file(path).ok_or(error::Error::FileNotFound)?;

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
            (true, false) => Ok(des::decode(
                &data,
                entry.compressed_length_aligned as usize,
                entry.compressed_length as usize,
                false,
            )?),
            (false, true) => Ok(des::decode(
                &data,
                entry.compressed_length_aligned as usize,
                entry.compressed_length as usize,
                true,
            )?),
            (false, false) => Ok(data),
            (true, true) => Err(error::Error::WrongSignature),
        }?;

        let uncompressed_data = {
            let mut buffer = vec![0; entry.uncompressed_length as usize];

            let mut decompressor = ZlibDecoder::new(decoded_data.as_slice());
            decompressor.read_exact(&mut buffer)?;

            buffer
        };

        Ok(uncompressed_data.into_boxed_slice())
    }

    pub fn is_directory(&self, path: &Path) -> Result<bool, error::Error> {
        self.search_file(path)
            .map(|entry| !entry.is_file())
            .ok_or(error::Error::FileNotFound)
    }

    pub fn read_directory(&self, path: &Path) -> Result<Box<[PathBuf]>, error::Error> {
        let bin_search = self
            .file_table
            .binary_search_by_key(&path, |entry| &entry.filename);
        match bin_search {
            Ok(start) => {
                let end = self
                    .file_table
                    .iter()
                    .position(|entry| !entry.filename.starts_with(path));
                let range = match end {
                    Some(end) => &self.file_table[start..end],
                    None => &self.file_table[start..],
                };
                Ok(range
                    .iter()
                    .filter(|&entry| {
                        entry
                            .filename
                            .parent()
                            .map(|parent| parent.eq(path))
                            .unwrap_or(false)
                    })
                    .map(|entry| entry.filename.clone())
                    .collect())
            }
            Err(_) => Err(error::Error::FileNotFound),
        }
    }

    fn search_file<'a>(&'a self, path: &Path) -> Option<&'a Entry> {
        let bin_search = self
            .file_table
            .binary_search_by_key(&path, |entry| &entry.filename);
        match bin_search {
            Ok(position) => Some(&self.file_table[position]),
            Err(_) => None,
        }
    }

    fn read_file_table(
        reader: &mut BufReader<File>,
        file_count: usize,
    ) -> Result<Vec<Entry>, io::Error> {
        let compressed_size = reader.read_le_u32()?;
        let umcompressed_size = reader.read_le_u32()?;

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
            .collect::<Result<Vec<Entry>, io::Error>>()
    }

    fn read_file_table_entry(table_reader: &mut BufReader<&[u8]>) -> Result<Entry, io::Error> {
        let cp949_filename = table_reader.read_null_terminated_string()?;

        let filename = {
            let (f, _encoding, chars_replaced) =
                EUC_KR.decode(&cp949_filename[..(cp949_filename.len() - 1)]);
            if chars_replaced {
                Err(io::Error::new(
                    ErrorKind::InvalidInput,
                    "String had invalid CP949 characters",
                ))?
            }
            PathBuf::from(f.replace('\\', "/"))
        };

        let compressed_length = table_reader.read_le_u32()?;
        let compressed_length_aligned = table_reader.read_le_u32()?;
        let uncompressed_length = table_reader.read_le_u32()?;
        let flags = table_reader.read_u8()?;
        let offset = table_reader.read_le_u32()?;

        Ok(Entry {
            filename,
            compressed_length,
            compressed_length_aligned,
            uncompressed_length,
            flags,
            offset,
        })
    }
}
