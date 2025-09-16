use std::{io::Cursor, path::Path};

use ragnarok_grf::Grf;
use ragnarok_pal::Pal;
use ragnarok_rebuild_common::warning::ReportWarning;

fn main() {
    #[expect(clippy::unwrap_used, reason = "This is a test")]
    let grf = Grf::new(Path::new("data.grf")).unwrap();

    for pal_filename in grf
        .iter_filenames()
        .filter(|filename| match filename.extension() {
            Some(ext) => {
                matches!(ext.to_str(), Some("pal"))
            }
            None => false,
        })
    {
        let Ok(pal_content) = grf
            .read_file(pal_filename)
            .inspect_err(|err| println!("{pal_filename:?}: {err}"))
        else {
            continue;
        };
        let Ok(pal) = Pal::from_reader(&mut Cursor::new(&pal_content))
            .inspect_err(|err| println!("{pal_filename:?}: {err}"))
        else {
            continue;
        };

        let report = pal.report().to_string();
        if !report.is_empty() {
            println!("{:?}", pal_filename);
            println!("{}", report);
        }
    }
}
