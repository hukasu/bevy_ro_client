use std::{io::Cursor, path::Path};

use ragnarok_rebuild_assets::{grf::GRF, rsw::Rsw};

fn main() {
    let Ok(grf) = GRF::new(Path::new("data.grf")).inspect_err(|err| eprintln!("{err}")) else {
        return;
    };

    for rsw_filename in grf
        .iter_filenames()
        .filter(|filename| match filename.extension() {
            Some(ext) => {
                matches!(ext.to_str(), Some("rsw"))
            }
            None => false,
        })
    {
        let Ok(rsm_content) = grf
            .read_file(rsw_filename)
            .inspect_err(|err| println!("{rsw_filename:?}: {err}"))
        else {
            continue;
        };
        let Ok(rsw) = Rsw::from_reader(&mut Cursor::new(rsm_content))
            .inspect_err(|err| println!("{rsw_filename:?}: {err}"))
        else {
            continue;
        };

        if let Some(rsw_debug) = debug_rsw(&rsw) {
            println!("{:?}", rsw_filename);
            println!("{}", rsw_debug);
        }
    }
}

fn debug_rsw(rsw: &Rsw) -> Option<String> {
    let header = || format!("\t{:?}\n", rsw.version);
    let mut debug = None;

    // TODO

    debug
}
