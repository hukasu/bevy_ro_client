use std::{io::Cursor, path::Path};

use ragnarok_grf::Grf;
use ragnarok_rebuild_common::warning::ReportWarning;
use ragnarok_spr::Spr;

fn main() {
    #[expect(clippy::unwrap_used, reason = "This is a test")]
    let grf = Grf::new(Path::new("data.grf")).unwrap();

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
        let Ok(spr) = Spr::from_reader(&mut Cursor::new(&spr_content))
            .inspect_err(|err| println!("{spr_filename:?}: {err}"))
        else {
            continue;
        };

        let report = spr.report().to_string();
        if !report.is_empty() {
            println!("{:?} {:?}", spr_filename, spr.version);
            println!("{}", report);
        }
    }
}
