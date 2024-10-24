use std::{fmt::Write, io::Cursor, path::Path};

use ragnarok_rebuild_assets::{
    grf::GRF,
    imf::{self, Imf},
};

fn main() {
    let Ok(grf) = GRF::new(Path::new("data.grf")).inspect_err(|err| eprintln!("{err}")) else {
        return;
    };

    for imf_filename in grf
        .iter_filenames()
        .filter(|filename| match filename.extension() {
            Some(ext) => {
                matches!(ext.to_str(), Some("imf"))
            }
            None => false,
        })
    {
        let Ok(imf_content) = grf
            .read_file(imf_filename)
            .inspect_err(|err| println!("{imf_filename:?}: {err}"))
        else {
            continue;
        };
        let Ok(imf) = Imf::from_reader(&mut Cursor::new(imf_content))
            .inspect_err(|err| println!("{imf_filename:?}: {err}"))
        else {
            continue;
        };

        if let Some(imf_debug) = debug_imf(&imf) {
            println!("{:?}", imf_filename);
            println!("{}", imf_debug);
        }
    }
}

fn debug_imf(imf: &Imf) -> Option<String> {
    let header = || format!("\t{:?}\n", imf.version);
    let mut debug = None;

    for (i, layer) in imf.layers.iter().enumerate() {
        if let Some(layer_debug) = debug_layer(layer, imf.max_index) {
            let debug_ref = debug.get_or_insert_with(header);
            writeln!(debug_ref, "\tLayer {}", i).unwrap();
            write!(debug_ref, "{}", layer_debug).unwrap();
        }
    }

    debug
}

fn debug_layer(layer: &imf::Layer, max_index: u32) -> Option<String> {
    let header = String::new;
    let mut debug = None;

    for (i, animation) in layer.animations.iter().enumerate() {
        if let Some(layer_debug) = debug_animation(animation, max_index) {
            let debug_ref = debug.get_or_insert_with(header);
            writeln!(debug_ref, "\t\tAnimation {}", i).unwrap();
            write!(debug_ref, "{}", layer_debug).unwrap();
        }
    }

    debug
}

fn debug_animation(animation: &imf::Animation, max_index: u32) -> Option<String> {
    let header = String::new;
    let mut debug = None;

    for (i, frame) in animation.frames.iter().enumerate() {
        if let Some(layer_debug) = debug_frame(frame, max_index) {
            let debug_ref = debug.get_or_insert_with(header);
            writeln!(debug_ref, "\t\t\tFrame {}", i).unwrap();
            write!(debug_ref, "{}", layer_debug).unwrap();
        }
    }

    debug
}

fn debug_frame(frame: &imf::Frame, max_index: u32) -> Option<String> {
    let header = String::new;
    let mut debug = None;

    if frame.index > max_index {
        let debug_ref = debug.get_or_insert_with(header);
        writeln!(
            debug_ref,
            "\t\t\t\thas index higher than max_index. ({})",
            frame.index
        )
        .unwrap();
    }

    debug
}
