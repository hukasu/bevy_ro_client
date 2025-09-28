#![expect(clippy::unwrap_used, reason = "This is a test")]

use std::{io::Cursor, path::Path};

use ragnarok_grf::Grf;
use ragnarok_rebuild_common::warning::ReportWarning;
use ragnarok_rsw::Rsw;

fn main() {
    let grf = Grf::new(Path::new("data.grf")).unwrap();

    for rsw_filename in grf
        .iter_filenames()
        .filter(|filename| match filename.extension() {
            Some(ext) => {
                matches!(ext.to_str(), Some("rsw"))
            }
            None => false,
        })
    {
        let Ok(rsw_content) = grf
            .read_file(rsw_filename)
            .inspect_err(|err| println!("{rsw_filename:?}: {err}"))
        else {
            continue;
        };
        let Ok(rsw) = Rsw::from_reader(&mut Cursor::new(&rsw_content))
            .inspect_err(|err| println!("{rsw_filename:?}: {err}"))
        else {
            continue;
        };

        let report = rsw.report().to_string();
        if !report.is_empty() {
            println!("{:?} {:?}", rsw_filename, rsw.version);
            println!("{}", report);
        }
    }
}
