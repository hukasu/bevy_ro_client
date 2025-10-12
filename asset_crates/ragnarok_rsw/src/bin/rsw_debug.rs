#![expect(clippy::unwrap_used, reason = "This is a test")]

use std::{io::Cursor, path::Path};

use ragnarok_gat::Gat;
use ragnarok_grf::Grf;
use ragnarok_rebuild_assets::gnd::Gnd;
use ragnarok_rebuild_common::warning::ReportWarning;
use ragnarok_rsw::{Rsw, quad_tree::Crawler};

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

        let mut header = false;
        let report = rsw.report().to_string();
        if !report.is_empty() {
            println!("{:?} {:?}", rsw_filename, rsw.version);
            println!("{}", report);
            header = true;
        }

        let tile_scale = match grf.read_file(Path::new(&format!("data/{}", rsw.gnd_file))) {
            Ok(gnd_content) => {
                let Ok(gnd) = Gnd::from_reader(&mut Cursor::new(&gnd_content))
                    .inspect_err(|err| eprintln!("data/{:?}: {err}", rsw.gnd_file))
                else {
                    continue;
                };
                gnd.scale / 2.
            }
            Err(err) => {
                if !header {
                    eprintln!("{:?} {:?}", rsw_filename, rsw.version);
                }
                eprintln!("{}: {}", rsw.gnd_file, err);
                continue;
            }
        };

        match grf.read_file(Path::new(&format!("data/{}", rsw.gat_file))) {
            Ok(gat_content) => {
                let Ok(gat) = Gat::from_reader(&mut Cursor::new(&gat_content))
                    .inspect_err(|err| eprintln!("data/{:?}: {err}", rsw.gat_file))
                else {
                    continue;
                };

                let width = gat.width as usize;
                let height = gat.height as usize;
                for (i, tile) in gat.tiles.iter().enumerate() {
                    let x_tile = i % width;
                    let z_tile = i / width;
                    let x = (x_tile as f32 - width as f32 / 2. + 0.5) * tile_scale;
                    let z = (z_tile as f32 - height as f32 / 2. + 0.5) * tile_scale;

                    let max_height = [
                        tile.bottom_left_altitude(),
                        tile.bottom_right_altitude(),
                        tile.top_left_altitude(),
                        tile.top_right_altitude(),
                    ]
                    .into_iter()
                    .max_by(|a, b| a.total_cmp(b))
                    .unwrap();
                    let min_height = [
                        tile.bottom_left_altitude(),
                        tile.bottom_right_altitude(),
                        tile.top_left_altitude(),
                        tile.top_right_altitude(),
                    ]
                    .into_iter()
                    .min_by(|a, b| a.total_cmp(b))
                    .unwrap();

                    let mut current_node = rsw.quad_tree.crawl();
                    if test_node(&current_node, x, z, max_height, min_height) {
                        loop {
                            if current_node.is_leaf() {
                                break;
                            }

                            let bl = current_node.bottom_left().unwrap();
                            if test_node(&bl, x, z, max_height, min_height) {
                                current_node = bl;
                                continue;
                            }

                            let br = current_node.bottom_right().unwrap();
                            if test_node(&br, x, z, max_height, min_height) {
                                current_node = br;
                                continue;
                            }

                            let tl = current_node.top_left().unwrap();
                            if test_node(&tl, x, z, max_height, min_height) {
                                current_node = tl;
                                continue;
                            }

                            let tr = current_node.top_right().unwrap();
                            if test_node(&tr, x, z, max_height, min_height) {
                                current_node = tr;
                                continue;
                            }

                            if !header {
                                eprintln!("{:?} {:?}", rsw_filename, rsw.version);
                                header = true;
                            }
                            eprintln!(
                                "Tile {x_tile}/{z_tile} did not fit on any of the node of depth {}.",
                                current_node.depth() + 1
                            );
                            break;
                        }
                    } else {
                        if !header {
                            eprintln!("{:?} {:?}", rsw_filename, rsw.version);
                        }
                        eprintln!("Tile {x_tile}/{z_tile} did not fit on quadtree's root.");
                        continue;
                    }
                }
                return;
            }
            Err(err) => {
                if !header {
                    eprintln!("{:?} {:?}", rsw_filename, rsw.version);
                }
                eprintln!("{}: {}", rsw.gat_file, err);
            }
        }
    }
}

fn test_node(current_node: &Crawler<'_>, x: f32, z: f32, y_max: f32, y_min: f32) -> bool {
    let top = current_node.top;
    let bottom = current_node.bottom;

    let x_test = bottom[0] <= x && x <= top[0];
    let y_test = bottom[1] <= y_min && y_max <= top[1];
    let z_test = bottom[2] <= z && z <= top[2];

    x_test && y_test && z_test
}
