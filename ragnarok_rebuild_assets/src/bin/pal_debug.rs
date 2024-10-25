use std::{fmt::Write, io::Cursor, path::Path};

use ragnarok_rebuild_assets::{grf::Grf, pal};

fn main() {
    let Ok(grf) = Grf::new(Path::new("data.grf")).inspect_err(|err| eprintln!("{err}")) else {
        return;
    };

    for pal_filename in grf
        .iter_filenames()
        .filter(|filename| match filename.extension() {
            Some(ext) => {
                matches!(ext.to_str(), Some("pal"))
            }
            None => false,
        })
    {
        let Ok(pal_content) = grf
            .read_file(pal_filename)
            .inspect_err(|err| println!("{pal_filename:?}: {err}"))
        else {
            continue;
        };
        let Ok(_pal) = pal::Pal::from_reader(&mut Cursor::new(pal_content))
            .inspect_err(|err| println!("{pal_filename:?}: {err}"))
        else {
            continue;
        };

        if let Some(pal_debug) = debug_palette(&_pal) {
            println!("{:?}", pal_filename);
            println!("{}", pal_debug);
        }
    }
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

    if pal.colors[1..]
        .iter()
        .any(|color| color.alpha > 0 && color.alpha < 255)
    {
        let debug_ref = debug.get_or_insert_with(header);
        writeln!(debug_ref, "\t\thas palette color with alpha.").unwrap();
    }

    debug
}
