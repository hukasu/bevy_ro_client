export RUSTFLAGS := "-Dwarnings"

[group("asset_debug")]
act_debug:
    cargo run --bin act_debug --features="warning ragnarok_grf"

[group("asset_debug")]
pal_debug:
    cargo run --bin pal_debug --features="warning ragnarok_grf"

[group("asset_debug")]
rsm_debug:
    cargo run --bin rsm_debug --features="warning ragnarok_grf"

[group("asset_debug")]
spr_debug:
    cargo run --bin spr_debug --features="warning ragnarok_grf"

[group("ci")]
default:
    @just format
    @just clippy

[group("ci")]
format:
    cargo fmt
    typos -w
    taplo fmt

[group("clippy")]
clippy:
    @just ragnarok_act
    @just ragnarok_spr

[group("clippy")]
ragnarok_act:
    cargo clippy -p ragnarok_act --no-default-features
    cargo clippy -p ragnarok_act --no-default-features --features="warning"
    cargo clippy -p ragnarok_act
    cargo clippy -p ragnarok_act --all-features

[group("clippy")]
ragnarok_spr:
    cargo clippy -p ragnarok_spr --no-default-features
    cargo clippy -p ragnarok_spr --no-default-features --features="warning"
    cargo clippy -p ragnarok_spr
    cargo clippy -p ragnarok_spr --all-features
