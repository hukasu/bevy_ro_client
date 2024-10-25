use std::{fmt::Write, io::Cursor, path::Path};

use ragnarok_rebuild_assets::{grf::Grf, rsw};

fn main() {
    let Ok(grf) = Grf::new(Path::new("data.grf")).inspect_err(|err| eprintln!("{err}")) else {
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
        let Ok(rsw) = rsw::Rsw::from_reader(&mut Cursor::new(rsm_content))
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

fn debug_rsw(rsw: &rsw::Rsw) -> Option<String> {
    let header = || format!("\t{:?}\n", rsw.version);
    let mut debug = None;

    for model in rsw.models.iter() {
        if let Some(model_debug) = debug_model(model) {
            let debug_ref = debug.get_or_insert_with(header);
            write!(debug_ref, "{}", model_debug).unwrap();
        }
    }

    debug
}

fn debug_model(model: &rsw::Model) -> Option<String> {
    const INDENT: &str = "\t";
    let header = || format!("{INDENT}Model \"{}\"\n", model.name);
    let mut debug = None;

    if model.flag != 0 {
        let debug_ref = debug.get_or_insert_with(header);
        writeln!(
            debug_ref,
            "{INDENT}\thas a non-zero flag. (\"{}\")",
            model.flag
        )
        .unwrap();
    }

    debug
}
