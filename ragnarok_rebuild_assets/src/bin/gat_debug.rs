use std::{fmt::Write, io::Cursor, path::Path};

use ragnarok_rebuild_assets::{gat, grf::Grf};

fn main() {
    let Ok(grf) = Grf::new(Path::new("data.grf")).inspect_err(|err| eprintln!("{err}")) else {
        return;
    };

    for gat_filename in grf
        .iter_filenames()
        .filter(|filename| match filename.extension() {
            Some(ext) => {
                matches!(ext.to_str(), Some("gat"))
            }
            None => false,
        })
    {
        let Ok(gat_content) = grf
            .read_file(gat_filename)
            .inspect_err(|err| println!("{gat_filename:?}: {err}"))
        else {
            continue;
        };
        let Ok(rsw) = gat::Gat::from_reader(&mut Cursor::new(gat_content))
            .inspect_err(|err| println!("{gat_filename:?}: {err}"))
        else {
            continue;
        };

        if let Some(rsw_debug) = debug_gat(&rsw) {
            println!("{:?}", gat_filename);
            println!("{}", rsw_debug);
        }
    }
}

fn debug_gat(gat: &gat::Gat) -> Option<String> {
    const INDENT: &str = "\t";
    const AXIS_LIMIT: u32 = 416;
    let header = || format!("{INDENT}{:?}\n", gat.version);
    let mut debug = None;

    if gat.width > AXIS_LIMIT || gat.height > AXIS_LIMIT {
        let debug_ref = debug.get_or_insert_with(header);
        writeln!(
            debug_ref,
            "{INDENT}\thas width or height over {AXIS_LIMIT}. ({}x{})",
            gat.width, gat.height
        )
        .unwrap();
    }

    debug
}
