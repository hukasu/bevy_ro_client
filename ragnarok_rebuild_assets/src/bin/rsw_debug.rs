use std::{collections::BTreeSet, fmt::Write, io::Cursor, path::Path};

use ragnarok_rebuild_assets::{grf::Grf, rsw};

fn main() {
    let Ok(grf) = Grf::new(Path::new("data.grf")).inspect_err(|err| println!("{err}")) else {
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

    if let Some(light_debug) = debug_lights(&rsw.lights) {
        let debug_ref = debug.get_or_insert_with(header);
        write!(debug_ref, "{}", light_debug).unwrap();
    }

    debug
}

fn debug_model(model: &rsw::Model) -> Option<String> {
    const INDENT: &str = "\t";
    let header = || format!("{INDENT}Model \"{}\"\n", model.name);
    let mut debug = None;

    if model.filename.is_empty() {
        let debug_ref = debug.get_or_insert_with(header);
        writeln!(debug_ref, "{INDENT}\thas empty filename.",).unwrap();
    }
    if model.filename.len() >= 75 {
        let debug_ref = debug.get_or_insert_with(header);
        writeln!(
            debug_ref,
            "{INDENT}\thas long filename. ({})",
            model.filename.len()
        )
        .unwrap();
    }

    // This error is reported A LOT
    // if model.node_name.is_empty() {
    //     let debug_ref = debug.get_or_insert_with(header);
    //     writeln!(debug_ref, "{INDENT}\thas empty node name.",).unwrap();
    // }
    if model.node_name.len() >= 75 {
        let debug_ref = debug.get_or_insert_with(header);
        writeln!(
            debug_ref,
            "{INDENT}\thas long node name. ({})",
            model.filename.len()
        )
        .unwrap();
    }

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

fn debug_lights(lights: &[rsw::Light]) -> Option<String> {
    const THRESHOLD: f32 = 1.;
    const INDENT: &str = "\t";
    let header = String::new;
    let mut debug = None;

    let mut light_positions: Vec<&rsw::Light> = Vec::new();
    for light in lights.iter() {
        if let Some(repeated) = light_positions.iter().find(|other| {
            ((other.position[0] - light.position[0]).powi(2)
                + (other.position[1] - light.position[1]).powi(2)
                + (other.position[2] - light.position[2]).powi(2))
            .sqrt()
                < THRESHOLD
        }) {
            let debug_ref = debug.get_or_insert_with(header);
            writeln!(
                debug_ref,
                "{INDENT}has a repeated light at {:?}. ({}, {})",
                light.position, light.name, repeated.name
            )
            .unwrap();
        } else {
            light_positions.push(light);
        }
    }

    let mut light_names = BTreeSet::new();
    for light in lights.iter() {
        if light_names.contains(&light.name) {
            let debug_ref = debug.get_or_insert_with(header);
            writeln!(
                debug_ref,
                "{INDENT}has a repeated light name. ({})",
                light.name
            )
            .unwrap();
        } else {
            light_names.insert(&light.name);
        }
    }

    for light in lights.iter() {
        if light
            .color
            .iter()
            .any(|channel| *channel < 0. || *channel > 1.)
        {
            let debug_ref = debug.get_or_insert_with(header);
            writeln!(
                debug_ref,
                "{INDENT}light {} has unnormalized color {:?}.",
                light.name, light.color
            )
            .unwrap();
        }
    }

    debug
}
