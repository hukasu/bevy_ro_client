use std::{io::Cursor, path::Path};

use ragnarok_gnd::Gnd;
use ragnarok_grf::Grf;
use ragnarok_rebuild_common::warning::ReportWarning;

fn main() {
    #[expect(clippy::unwrap_used, reason = "This is a test")]
    let grf = Grf::new(Path::new("data.grf")).unwrap();

    for gnd_filename in grf
        .iter_filenames()
        .filter(|filename| match filename.extension() {
            Some(ext) => {
                matches!(ext.to_str(), Some("gnd"))
            }
            None => false,
        })
    {
        let Ok(gnd_content) = grf
            .read_file(gnd_filename)
            .inspect_err(|err| println!("{gnd_filename:?}: {err}"))
        else {
            continue;
        };
        let Ok(gnd) = Gnd::from_reader(&mut Cursor::new(gnd_content))
            .inspect_err(|err| println!("{gnd_filename:?}: {err}"))
        else {
            continue;
        };

        let report = gnd.report().to_string();
        if !report.is_empty() {
            println!("{:?} {:?}", gnd_filename, gnd.version);
            println!("{}", report);
        }
    }
}
