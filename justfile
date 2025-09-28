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
rsw_debug:
    cargo run --bin rsw_debug --features="warning ragnarok_grf"

[group("asset_debug")]
spr_debug:
    cargo run --bin spr_debug --features="warning ragnarok_grf"

#[default]
[group("ci")]
default:
    @just format
    @just clippy

[group("ci")]
format:
    cargo fmt
    typos -w
    taplo fmt

[group("ci")]
clippy:
    @just ragnarok_act
    @just ragnarok_grf
    @just ragnarok_pal
    @just ragnarok_rsm
    @just ragnarok_spr

    @just bevy_ragnarok_act
    @just bevy_ragnarok_grf
    @just bevy_ragnarok_pal
    @just bevy_ragnarok_rsm
    @just bevy_ragnarok_spr

[group("clippy")]
ragnarok_act:
    cargo clippy -p ragnarok_act --no-default-features
    cargo clippy -p ragnarok_act --no-default-features --features="warning"
    cargo clippy -p ragnarok_act
    cargo clippy -p ragnarok_act --all-features

[group("clippy")]
ragnarok_grf:
    cargo clippy -p ragnarok_grf --no-default-features
    cargo clippy -p ragnarok_grf
    cargo clippy -p ragnarok_grf --all-features

[group("clippy")]
ragnarok_pal:
    cargo clippy -p ragnarok_pal --no-default-features
    cargo clippy -p ragnarok_pal --no-default-features --features="warning"
    cargo clippy -p ragnarok_pal
    cargo clippy -p ragnarok_pal --all-features

[group("clippy")]
ragnarok_rsm:
    cargo clippy -p ragnarok_rsm --no-default-features
    cargo clippy -p ragnarok_rsm --no-default-features --features="warning"
    cargo clippy -p ragnarok_rsm
    cargo clippy -p ragnarok_rsm --all-features

[group("clippy")]
ragnarok_spr:
    cargo clippy -p ragnarok_spr --no-default-features
    cargo clippy -p ragnarok_spr --no-default-features --features="warning"
    cargo clippy -p ragnarok_spr
    cargo clippy -p ragnarok_spr --all-features

[group("clippy")]
bevy_ragnarok_act:
    cargo clippy -p bevy_ragnarok_act --no-default-features
    cargo clippy -p bevy_ragnarok_act --no-default-features --features="debug"
    cargo clippy -p bevy_ragnarok_act
    cargo clippy -p bevy_ragnarok_act --all-features

[group("clippy")]
bevy_ragnarok_grf:
    cargo clippy -p bevy_ragnarok_grf --no-default-features
    cargo clippy -p bevy_ragnarok_grf
    cargo clippy -p bevy_ragnarok_grf --all-features

[group("clippy")]
bevy_ragnarok_pal:
    cargo clippy -p bevy_ragnarok_pal --no-default-features
    cargo clippy -p bevy_ragnarok_pal --no-default-features --features="debug"
    cargo clippy -p bevy_ragnarok_pal
    cargo clippy -p bevy_ragnarok_pal --all-features

[group("clippy")]
bevy_ragnarok_rsm:
    cargo clippy -p bevy_ragnarok_rsm --no-default-features
    cargo clippy -p bevy_ragnarok_rsm --no-default-features --features="debug"
    cargo clippy -p bevy_ragnarok_rsm
    cargo clippy -p bevy_ragnarok_rsm --all-features

[group("clippy")]
bevy_ragnarok_spr:
    cargo clippy -p bevy_ragnarok_spr --no-default-features
    cargo clippy -p bevy_ragnarok_spr --no-default-features --features="debug"
    cargo clippy -p bevy_ragnarok_spr
    cargo clippy -p bevy_ragnarok_spr --all-features
