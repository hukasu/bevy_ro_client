use std::{io::Cursor, path::Path};

use ragnarok_rebuild_assets::{act::Act, grf::GRF};

fn main() {
    let Ok(grf) = GRF::new(Path::new("data.grf")).inspect_err(|err| eprintln!("{err}")) else {
        return;
    };

    for act_filename in grf
        .iter_filenames()
        .filter(|filename| match filename.extension() {
            Some(ext) => {
                matches!(ext.to_str(), Some("act"))
            }
            None => false,
        })
    {
        let Ok(act_content) = grf
            .read_file(act_filename)
            .inspect_err(|err| eprintln!("{act_filename:?}: {err}"))
        else {
            continue;
        };
        let Ok(_act) = Act::from_reader(&mut Cursor::new(act_content))
            .inspect_err(|err| eprintln!("{act_filename:?}: {err}"))
        else {
            continue;
        };
    }
}
