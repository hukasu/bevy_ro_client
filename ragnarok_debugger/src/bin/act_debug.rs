use std::{io::Cursor, path::Path};

use ragnarok_act::Act;
use ragnarok_grf::Grf;
use ragnarok_rebuild_common::warning::ReportWarning;

fn main() {
    #[expect(clippy::unwrap_used, reason = "This is a test")]
    let grf = Grf::new(Path::new("data.grf")).unwrap();

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
            .inspect_err(|err| println!("{act_filename:?}: {err}"))
        else {
            continue;
        };
        let Ok(act) = Act::from_reader(&mut Cursor::new(&act_content))
            .inspect_err(|err| println!("{act_filename:?}: {err}"))
        else {
            continue;
        };

        let report = act.report().to_string();
        if !report.is_empty() {
            println!("{:?} {:?}", act_filename, act.version);
            println!("{}", report);
        }
    }
}
