#![expect(clippy::unwrap_used, reason = "This is a test")]

use std::{io::Cursor, path::Path};

use ragnarok_gat::Gat;
use ragnarok_grf::Grf;
use ragnarok_rebuild_common::warning::ReportWarning;

fn main() {
    let grf = Grf::new(Path::new("data.grf")).unwrap();

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
        let Ok(gat) = Gat::from_reader(&mut Cursor::new(&gat_content))
            .inspect_err(|err| println!("{gat_filename:?}: {err}"))
        else {
            continue;
        };

        let report = gat.report().to_string();
        if !report.is_empty() {
            println!("{:?}", gat_filename);
            println!("{}", report);
        }
    }
}
