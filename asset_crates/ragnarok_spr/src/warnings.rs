use std::{fmt::Display, ops::Deref};

use ragnarok_pal::Pal;
use ragnarok_rebuild_common::warning::ReportWarning;

use crate::Spr;

impl ReportWarning for Spr {
    fn report(&self) -> impl std::fmt::Display {
        SprReport(self)
    }
}

struct SprReport<'a>(&'a Spr);

impl Display for SprReport<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let magenta = magenta_palette(&self.palette).collect::<Vec<_>>();
        if !magenta.is_empty()
            && self
                .bitmap_images
                .iter()
                .any(|sprite| sprite.indexes.iter().any(|index| magenta.contains(index)))
        {
            writeln!(f, "Uses magenta.")?;
        }

        let non_zero_transparency =
            non_zero_transparency_palette(&self.palette).collect::<Vec<_>>();
        if !non_zero_transparency.is_empty()
            && self.bitmap_images.iter().any(|sprite| {
                sprite
                    .indexes
                    .iter()
                    .any(|index| non_zero_transparency.contains(index))
            })
        {
            writeln!(f, "Uses transparency.")?;
        }

        let close_to_key = close_to_key_palette(&self.palette).collect::<Vec<_>>();
        if !close_to_key.is_empty()
            && self.bitmap_images.iter().any(|sprite| {
                sprite
                    .indexes
                    .iter()
                    .any(|index| close_to_key.contains(index))
            })
        {
            writeln!(f, "Uses colors close to key.")?;
        }

        if self.palette.colors[0].alpha != 0 {
            writeln!(
                f,
                "Has palette key with non-zero alpha of {}.",
                self.palette.colors[0].alpha
            )?;
        }

        Ok(())
    }
}

impl Deref for SprReport<'_> {
    type Target = Spr;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

/// Collects all indexes that are not opaque or fully transparent
fn non_zero_transparency_palette(palette: &Pal) -> impl Iterator<Item = u8> + '_ {
    palette.colors[1..]
        .iter()
        .enumerate()
        .filter_map(|(i, color)| {
            if color.alpha > 0 && color.alpha < 255 {
                Some((i + 1) as u8)
            } else {
                None
            }
        })
}

/// Collects all indexes that are not opaque or fully transparent
fn close_to_key_palette(palette: &Pal) -> impl Iterator<Item = u8> + '_ {
    let key = &palette.colors[0];
    palette.colors[1..]
        .iter()
        .enumerate()
        .filter_map(|(i, color)| {
            if color.alpha > 0
                && color.red.abs_diff(key.red) < 4
                && color.green.abs_diff(key.green) < 4
                && color.blue.abs_diff(key.blue) < 4
            {
                Some((i + 1) as u8)
            } else {
                None
            }
        })
}

/// Collects all indexes that are magenta and not fully transparent
fn magenta_palette(palette: &Pal) -> impl Iterator<Item = u8> + '_ {
    palette.colors[1..]
        .iter()
        .enumerate()
        .filter_map(|(i, color)| {
            if color.red == 0xff && color.green == 0x00 && color.blue == 0xff {
                Some((i + 1) as u8)
            } else {
                None
            }
        })
}
