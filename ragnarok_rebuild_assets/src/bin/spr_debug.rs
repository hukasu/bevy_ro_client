use std::{fmt::Write, io::Cursor, path::Path};

use ragnarok_rebuild_assets::{grf::Grf, pal, spr::Spr};

fn main() {
    let Ok(grf) = Grf::new(Path::new("data.grf")).inspect_err(|err| eprintln!("{err}")) else {
        return;
    };

    for spr_filename in grf
        .iter_filenames()
        .filter(|filename| match filename.extension() {
            Some(ext) => {
                matches!(ext.to_str(), Some("spr"))
            }
            None => false,
        })
    {
        let Ok(spr_content) = grf
            .read_file(spr_filename)
            .inspect_err(|err| println!("{spr_filename:?}: {err}"))
        else {
            continue;
        };
        let Ok(spr) = Spr::from_reader(&mut Cursor::new(spr_content))
            .inspect_err(|err| println!("{spr_filename:?}: {err}"))
        else {
            continue;
        };

        if let Some(spr_debug) = debug_spr(&spr) {
            println!("{:?}", spr_filename);
            println!("{}", spr_debug);
        }
    }
}

fn debug_spr(spr: &Spr) -> Option<String> {
    let header = || format!("\t{:?}\n", spr.version);
    let mut debug = None;

    if let Some(palette) = &spr.palette {
        let magenta = magenta_palette(palette).collect::<Vec<_>>();
        if !magenta.is_empty()
            && spr
                .bitmap_images
                .iter()
                .any(|sprite| sprite.indexes.iter().any(|index| magenta.contains(index)))
        {
            let debug_ref = debug.get_or_insert_with(header);
            writeln!(debug_ref, "\t\tuses magenta.").unwrap();
        }

        let non_zero_transparency = non_zero_transparency_palette(palette).collect::<Vec<_>>();
        if !non_zero_transparency.is_empty()
            && spr.bitmap_images.iter().any(|sprite| {
                sprite
                    .indexes
                    .iter()
                    .any(|index| non_zero_transparency.contains(index))
            })
        {
            let debug_ref = debug.get_or_insert_with(header);
            writeln!(debug_ref, "\t\tuses transparency.").unwrap();
        }

        let close_to_key = close_to_key_palette(palette).collect::<Vec<_>>();
        if !close_to_key.is_empty()
            && spr.bitmap_images.iter().any(|sprite| {
                sprite
                    .indexes
                    .iter()
                    .any(|index| close_to_key.contains(index))
            })
        {
            let debug_ref = debug.get_or_insert_with(header);
            writeln!(debug_ref, "\t\tuses colors close to key.").unwrap();
        }

        if let Some(spr_debug) = debug_palette(palette) {
            let debug_ref = debug.get_or_insert_with(header);
            write!(debug_ref, "{}", spr_debug).unwrap();
        }
    } else {
        let debug_ref = debug.get_or_insert_with(header);
        writeln!(debug_ref, "\t\thas no palette.").unwrap();
    }

    debug
}

fn debug_palette(pal: &pal::Pal) -> Option<String> {
    let header = String::new;
    let mut debug = None;

    if pal.colors[0].alpha != 0 {
        let debug_ref = debug.get_or_insert_with(header);
        writeln!(
            debug_ref,
            "\t\thas palette key with alpha different from 0."
        )
        .unwrap();
    }

    debug
}

/// Collects all indexes that are not opaque or fully transparent
fn non_zero_transparency_palette(palette: &pal::Pal) -> impl Iterator<Item = u8> + '_ {
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
fn close_to_key_palette(palette: &pal::Pal) -> impl Iterator<Item = u8> + '_ {
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
fn magenta_palette(palette: &pal::Pal) -> impl Iterator<Item = u8> + '_ {
    palette.colors[1..]
        .iter()
        .enumerate()
        .filter_map(|(i, color)| {
            if color.red >= 0xfe && color.green < 0x04 && color.blue >= 0xfe && color.alpha > 0 {
                Some((i + 1) as u8)
            } else {
                None
            }
        })
}
