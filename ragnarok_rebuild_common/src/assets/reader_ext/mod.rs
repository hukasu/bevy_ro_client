use futures::{AsyncBufReadExt, AsyncReadExt};
use std::io::{BufRead, BufReader, Error, Read};

pub trait ReaderExt: Read {
    fn read_array<const N: usize>(&mut self) -> Result<[u8; N], Error>;
    fn read_vec(&mut self, len: usize) -> Result<Vec<u8>, Error>;
    fn read_le_f64(&mut self) -> Result<f64, Error>;
    fn read_le_f32(&mut self) -> Result<f32, Error>;
    fn read_le_i64(&mut self) -> Result<i64, Error>;
    fn read_le_i32(&mut self) -> Result<i32, Error>;
    fn read_le_i16(&mut self) -> Result<i16, Error>;
    fn read_i8(&mut self) -> Result<i8, Error>;
    fn read_le_u64(&mut self) -> Result<u64, Error>;
    fn read_le_u32(&mut self) -> Result<u32, Error>;
    fn read_le_u16(&mut self) -> Result<u16, Error>;
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

    fn read_le_f64(&mut self) -> Result<f64, Error> {
        let mut tmp = [0u8; 8];
        self.read_exact(&mut tmp)?;
        Ok(f64::from_le_bytes(tmp))
    }

    fn read_le_f32(&mut self) -> Result<f32, Error> {
        let mut tmp = [0u8; 4];
        self.read_exact(&mut tmp)?;
        Ok(f32::from_le_bytes(tmp))
    }

    fn read_le_i64(&mut self) -> Result<i64, Error> {
        let mut tmp = [0u8; 8];
        self.read_exact(&mut tmp)?;
        Ok(i64::from_le_bytes(tmp))
    }

    fn read_le_i32(&mut self) -> Result<i32, Error> {
        let mut tmp = [0u8; 4];
        self.read_exact(&mut tmp)?;
        Ok(i32::from_le_bytes(tmp))
    }

    fn read_le_i16(&mut self) -> Result<i16, Error> {
        let mut tmp = [0u8; 2];
        self.read_exact(&mut tmp)?;
        Ok(i16::from_le_bytes(tmp))
    }

    fn read_i8(&mut self) -> Result<i8, Error> {
        let mut tmp = [0u8; 1];
        self.read_exact(&mut tmp)?;
        Ok(i8::from_le_bytes(tmp))
    }

    fn read_le_u64(&mut self) -> Result<u64, Error> {
        let mut tmp = [0u8; 8];
        self.read_exact(&mut tmp)?;
        Ok(u64::from_le_bytes(tmp))
    }

    fn read_le_u32(&mut self) -> Result<u32, Error> {
        let mut tmp = [0u8; 4];
        self.read_exact(&mut tmp)?;
        Ok(u32::from_le_bytes(tmp))
    }

    fn read_le_u16(&mut self) -> Result<u16, Error> {
        let mut tmp = [0u8; 2];
        self.read_exact(&mut tmp)?;
        Ok(u16::from_le_bytes(tmp))
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
    fn read_le_f64(&mut self) -> AsyncReadPinBox<'_, f64>;
    fn read_le_f32(&mut self) -> AsyncReadPinBox<'_, f32>;
    fn read_le_i64(&mut self) -> AsyncReadPinBox<'_, i64>;
    fn read_le_i32(&mut self) -> AsyncReadPinBox<'_, i32>;
    fn read_le_i16(&mut self) -> AsyncReadPinBox<'_, i16>;
    fn read_i8(&mut self) -> AsyncReadPinBox<'_, i8>;
    fn read_le_u64(&mut self) -> AsyncReadPinBox<'_, u64>;
    fn read_le_u32(&mut self) -> AsyncReadPinBox<'_, u32>;
    fn read_le_u16(&mut self) -> AsyncReadPinBox<'_, u16>;
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

    fn read_le_f64(&mut self) -> AsyncReadPinBox<'_, f64> {
        Box::pin(async {
            let mut tmp = [0u8; 8];
            self.read_exact(&mut tmp).await?;
            Ok(f64::from_le_bytes(tmp))
        })
    }

    fn read_le_f32(&mut self) -> AsyncReadPinBox<'_, f32> {
        Box::pin(async {
            let mut tmp = [0u8; 4];
            self.read_exact(&mut tmp).await?;
            Ok(f32::from_le_bytes(tmp))
        })
    }

    fn read_le_i64(&mut self) -> AsyncReadPinBox<'_, i64> {
        Box::pin(async {
            let mut tmp = [0u8; 8];
            self.read_exact(&mut tmp).await?;
            Ok(i64::from_le_bytes(tmp))
        })
    }

    fn read_le_i32(&mut self) -> AsyncReadPinBox<'_, i32> {
        Box::pin(async {
            let mut tmp = [0u8; 4];
            self.read_exact(&mut tmp).await?;
            Ok(i32::from_le_bytes(tmp))
        })
    }

    fn read_le_i16(&mut self) -> AsyncReadPinBox<'_, i16> {
        Box::pin(async {
            let mut tmp = [0u8; 2];
            self.read_exact(&mut tmp).await?;
            Ok(i16::from_le_bytes(tmp))
        })
    }

    fn read_i8(&mut self) -> AsyncReadPinBox<'_, i8> {
        Box::pin(async {
            let mut tmp = [0u8; 1];
            self.read_exact(&mut tmp).await?;
            Ok(i8::from_le_bytes(tmp))
        })
    }

    fn read_le_u64(&mut self) -> AsyncReadPinBox<'_, u64> {
        Box::pin(async {
            let mut tmp = [0u8; 8];
            self.read_exact(&mut tmp).await?;
            Ok(u64::from_le_bytes(tmp))
        })
    }

    fn read_le_u32(&mut self) -> AsyncReadPinBox<'_, u32> {
        Box::pin(async {
            let mut tmp = [0u8; 4];
            self.read_exact(&mut tmp).await?;
            Ok(u32::from_le_bytes(tmp))
        })
    }

    fn read_le_u16(&mut self) -> AsyncReadPinBox<'_, u16> {
        Box::pin(async {
            let mut tmp = [0u8; 2];
            self.read_exact(&mut tmp).await?;
            Ok(u16::from_le_bytes(tmp))
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
