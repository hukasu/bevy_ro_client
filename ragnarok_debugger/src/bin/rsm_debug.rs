use std::{io::Cursor, path::Path};

use ragnarok_rebuild_assets::grf::Grf;
use ragnarok_rebuild_common::warning::ReportWarning;
use ragnarok_rsm::Rsm;

fn main() {
    #[expect(clippy::unwrap_used, reason = "This is a test")]
    let grf = Grf::new(Path::new("data.grf")).unwrap();

    for rsm_filename in grf
        .iter_filenames()
        .filter(|filename| match filename.extension() {
            Some(ext) => {
                matches!(ext.to_str(), Some("rsm") | Some("rsm2"))
            }
            None => false,
        })
    {
        let Ok(rsm_content) = grf
            .read_file(rsm_filename)
            .inspect_err(|err| println!("{rsm_filename:?}: {err}"))
        else {
            continue;
        };
        let Ok(rsm) = Rsm::from_reader(&mut Cursor::new(rsm_content))
            .inspect_err(|err| println!("{rsm_filename:?}: {err}"))
        else {
            continue;
        };

        let report = rsm.report().to_string();
        if !report.is_empty() {
            println!("{:?} {:?}", rsm_filename, rsm.version);
            println!("{}", report);
        }
    }
}
