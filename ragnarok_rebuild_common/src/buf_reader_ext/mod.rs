use futures::{AsyncBufReadExt, AsyncReadExt};
use std::{
    io::{BufRead, BufReader, Error, Read},
    ops::Shl,
};

pub trait ReaderExt: Read {
    fn read_array<const N: usize>(&mut self) -> Result<[u8; N], Error>;
    fn read_vec(&mut self, len: usize) -> Result<Vec<u8>, Error>;
    fn read_u32(&mut self) -> Result<u32, Error>;
    fn read_u16(&mut self) -> Result<u16, Error>;
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

    fn read_u16(&mut self) -> Result<u16, Error> {
        let mut tmp = [0u8; 2];
        self.read_exact(&mut tmp)?;
        Ok(tmp
            .iter()
            .enumerate()
            .fold(0u16, |sum, (i, b)| sum + (*b as u16).shl(8 * i)))
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

type AsyncReadPinBox<'a, T> =
    std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, Error>> + std::marker::Send + 'a>>;

pub trait AsyncReaderExt: futures::io::AsyncRead {
    fn read_array<const N: usize>(&mut self) -> AsyncReadPinBox<'_, [u8; N]>;
    fn read_vec(&mut self, len: usize) -> AsyncReadPinBox<'_, Vec<u8>>;
    fn read_u32(&mut self) -> AsyncReadPinBox<'_, u32>;
    fn read_u16(&mut self) -> AsyncReadPinBox<'_, u16>;
    fn read_u8(&mut self) -> AsyncReadPinBox<'_, u8>;
}

impl<T: futures::io::AsyncRead + std::marker::Send + std::marker::Unpin> AsyncReaderExt for T {
    fn read_array<const N: usize>(&mut self) -> AsyncReadPinBox<'_, [u8; N]> {
        let a: AsyncReadPinBox<'_, [u8; N]> = Box::pin(async {
            let mut bytes = [0u8; N];
            self.read_exact(&mut bytes).await?;
            Ok(bytes)
        });
        a
    }

    fn read_vec(&mut self, len: usize) -> AsyncReadPinBox<'_, Vec<u8>> {
        let mut bytes = vec![0u8; len];
        Box::pin(async {
            self.read_exact(&mut bytes).await?;
            Ok(bytes)
        })
    }

    fn read_u32(&mut self) -> AsyncReadPinBox<'_, u32> {
        Box::pin(async {
            let mut tmp = [0u8; 4];
            self.read_exact(&mut tmp).await?;
            Ok(tmp
                .iter()
                .enumerate()
                .fold(0u32, |sum, (i, b)| sum + (*b as u32).shl(8 * i)))
        })
    }

    fn read_u16(&mut self) -> AsyncReadPinBox<'_, u16> {
        Box::pin(async {
            let mut tmp = [0u8; 2];
            self.read_exact(&mut tmp).await?;
            Ok(tmp
                .iter()
                .enumerate()
                .fold(0u16, |sum, (i, b)| sum + (*b as u16).shl(8 * i)))
        })
    }

    fn read_u8(&mut self) -> AsyncReadPinBox<'_, u8> {
        Box::pin(async {
            let mut tmp = [0u8; 1];
            self.read_exact(&mut tmp).await?;
            Ok(tmp[0])
        })
    }
}

pub trait AsyncBufReaderExt: futures::io::AsyncRead {
    fn read_null_terminated_string(&mut self) -> AsyncReadPinBox<'_, Vec<u8>>;
}

impl<T: futures::io::AsyncRead + std::marker::Send + std::marker::Unpin> AsyncBufReaderExt
    for futures::io::BufReader<T>
{
    fn read_null_terminated_string(&mut self) -> AsyncReadPinBox<'_, Vec<u8>> {
        Box::pin(async {
            let mut buffer = Vec::with_capacity(256);
            self.read_until(b'\0', &mut buffer).await?;
            Ok(buffer)
        })
    }
}
