use std::{
    io::{BufRead, BufReader, Error, Read},
    ops::Shl,
};

pub trait ReaderExt: Read {
    fn read_array<const N: usize>(&mut self) -> Result<[u8; N], Error>;
    fn read_vec(&mut self, len: usize) -> Result<Vec<u8>, Error>;
    fn read_u32(&mut self) -> Result<u32, Error>;
    fn read_u8(&mut self) -> Result<u8, Error>;
}

impl<T: Read> ReaderExt for T {
    fn read_array<const N: usize>(&mut self) -> Result<[u8; N], Error> {
        let mut bytes = [0u8; N];
        self.read_exact(&mut bytes)?;
        Ok(bytes)
    }

    fn read_vec(&mut self, len: usize) -> Result<Vec<u8>, Error> {
        let mut bytes = vec![0u8; len];
        self.read_exact(&mut bytes)?;
        Ok(bytes)
    }

    fn read_u32(&mut self) -> Result<u32, Error> {
        let mut tmp = [0u8; 4];
        self.read_exact(&mut tmp)?;
        Ok(tmp
            .iter()
            .enumerate()
            .fold(0u32, |sum, (i, b)| sum + (*b as u32).shl(8 * i)))
    }

    fn read_u8(&mut self) -> Result<u8, Error> {
        let mut tmp = [0u8; 1];
        self.read_exact(&mut tmp)?;
        Ok(tmp[0])
    }
}

pub trait BufReaderExt: Read {
    fn read_null_terminated_string(&mut self) -> Result<Vec<u8>, Error>;
}

impl<T: Read> BufReaderExt for BufReader<T> {
    fn read_null_terminated_string(&mut self) -> Result<Vec<u8>, Error> {
        let mut buffer = Vec::with_capacity(256);
        self.read_until(b'\0', &mut buffer)?;
        Ok(buffer)
    }
}
