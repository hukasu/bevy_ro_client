mod error;

use std::io::Write;

use crate::reader_ext::ReaderExt;

pub use self::error::PaletteError;

#[derive(Debug)]
pub struct Palette {
    colors: [u8; 1024],
}

impl Palette {
    pub fn from_bytes(mut bytes: &[u8]) -> Result<Self, PaletteError> {
        let mut colors = bytes.read_array()?;
        // colors.reverse();

        if let Ok(mut file) = std::fs::File::create("palette.ppm") {
            writeln!(file, "P3");
            writeln!(file, "16 16");
            writeln!(file, "255");
            colors.windows(4).for_each(|wind| {
                if let [r, g, b, a] = wind {
                    writeln!(
                        file,
                        "{} {} {}",
                        g.saturating_sub(255 - r),
                        b.saturating_sub(255 - r),
                        a.saturating_sub(255 - r)
                    );
                };
            })
        };

        Ok(Self { colors })
    }
}
