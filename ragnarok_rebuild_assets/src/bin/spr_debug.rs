use std::{io::Cursor, path::Path};

use ragnarok_rebuild_assets::{grf::GRF, spr::Spr};

fn main() {
    let Ok(grf) = GRF::new(Path::new("data.grf")).inspect_err(|err| eprintln!("{err}")) else {
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
            .inspect_err(|err| eprintln!("{spr_filename:?}: {err}"))
        else {
            continue;
        };
        let Ok(_spr) = Spr::from_reader(&mut Cursor::new(spr_content))
            .inspect_err(|err| eprintln!("{spr_filename:?}: {err}"))
        else {
            continue;
        };
    }
}
