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

    if pal.colors[1..]
        .iter()
        .any(|color| color.alpha > 0 && color.alpha < 255)
    {
        let debug_ref = debug.get_or_insert_with(header);
        writeln!(debug_ref, "\t\thas palette color with alpha.").unwrap();
    }

    debug
}
